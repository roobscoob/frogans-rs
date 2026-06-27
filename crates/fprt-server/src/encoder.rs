//! Writing the OUT triple — the inverse of the client's `call.rs` — and the
//! panic→error helpers that stop a panicking handler from unwinding across
//! `extern "C"`.
//!
//! A command pops through [`pop_command`] (its data lives in the staged
//! [`CallPool`], registered as the handle); a framework reject writes through
//! [`fail`]; a caught panic through [`fail_panic`]. The pool is **lazy**: a no-data
//! command never materializes it, so its handle is [`MempoolHandle::EMPTY`].
//!
//! Errors are *not* a special memory case: an error message is a `PooledString` in
//! a registered pool, exactly like the engine's own data — the host frees it the
//! same way.
#![allow(dead_code)]

use std::any::Any;
use std::cell::{Cell, OnceCell};
use std::panic::{AssertUnwindSafe, catch_unwind};

use fprt_core::pool::{OwnedPool, PooledImage, PooledString};
use fprt_core::wire::ustring_opt;
use fprt_core::Command;
use fprt_sys::ctx::Ctx;
use fprt_sys::mem::MempoolHandle;
use fprt_sys::ui::StatusName;
use fprt_sys::ustring::Ustring;

use crate::registry::Registry;
use crate::session::Staged;

/// `status3` success sentinel (mirrors the client's `SUCCESS`).
pub const SUCCESS: i32 = 100;

/// `status3` reported when a handler panics — caught at the FFI boundary and
/// surfaced to the host instead of unwinding across `extern "C"` (UB). Arbitrary
/// and recognizable; matches no `fprt_sys` call code.
pub const ENGINE_PANIC: i32 = 777_999_777;

/// The call's argument pool, created **lazily** on first allocation. A body that
/// never allocates (a no-data event success) leaves it empty, so the call hands
/// back [`MempoolHandle::EMPTY`] and allocates nothing.
pub struct CallPool {
    pool: OnceCell<OwnedPool>,
}

impl CallPool {
    pub(crate) fn new() -> Self {
        CallPool {
            pool: OnceCell::new(),
        }
    }

    /// The underlying pool, materialized on first call — what a command's `new`
    /// allocates into.
    pub(crate) fn arena(&self) -> &OwnedPool {
        self.pool.get_or_init(OwnedPool::new)
    }

    /// Allocate a string into the call pool.
    pub fn alloc_str(&self, s: &str) -> PooledString {
        self.arena().alloc_str(s)
    }

    /// Allocate an encoded image into the call pool.
    pub fn alloc_image(&self, bytes: &[u8], width: u32, height: u32) -> PooledImage {
        self.arena().alloc_image(bytes, width, height)
    }

    /// Allocate a descriptor array (`[Ustring]` / `[ImageRecord]`) into the call
    /// pool — the slice a list/array payload points at.
    pub fn alloc_slice<T: Copy>(&self, src: &[T]) -> *const [T] {
        self.arena().alloc_slice(src)
    }

    /// Register the pool (if it was ever touched) and hand back its handle, or
    /// [`MempoolHandle::EMPTY`] if the call allocated nothing.
    pub(crate) fn into_handle(self, registry: &mut Registry) -> MempoolHandle {
        self.pool
            .into_inner()
            .map_or(MempoolHandle::EMPTY, |pool| registry.register(pool))
    }
}

/// Which phase of a trampoline was running — the diagnostically important part of
/// a caught panic: *whose* bug it is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// Decoding the inbound payload — a codec / framework concern (our bug).
    Decode,
    /// Running the implementor's engine handler — their concern.
    Handler,
    /// Encoding a command into the OUT payload — a codec / framework concern.
    Encode,
}

impl Phase {
    fn label(self) -> &'static str {
        match self {
            Phase::Decode => "decode",
            Phase::Handler => "handler",
            Phase::Encode => "encode",
        }
    }
}

thread_local! {
    /// Where the current trampoline is; read by [`fail_panic`] when it reports a
    /// caught panic. The trampoline advances it as it descends.
    static PHASE: Cell<Phase> = const { Cell::new(Phase::Handler) };
}

/// Mark the phase the trampoline is entering. A panic after this point is
/// attributed to `p`.
pub fn phase(p: Phase) {
    PHASE.with(|c| c.set(p));
}

fn current_phase() -> Phase {
    PHASE.with(Cell::get)
}

/// Which export (and session) a trampoline is serving — stamped onto a caught
/// panic so the failure says *what it was processing*.
pub struct Context {
    /// Human operation name, e.g. `"ui.inputfa.ok event"`, `"conductor.start"`.
    pub op: &'static str,
    /// The session the call targets, when it has one (library calls don't).
    pub ctx: Option<Ctx>,
}

/// Pull a human message out of a caught panic payload (`&str` / `String`).
fn panic_text(payload: &(dyn Any + Send)) -> &str {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("<non-string panic payload>")
}

/// Prefix `detail` with the call's breadcrumb: `op (ctx N) [phase]: detail`.
fn decorate(cx: &Context, phase: Phase, detail: &str) -> String {
    match cx.ctx {
        Some(ctx) => format!("{} (ctx {}) [{}]: {detail}", cx.op, ctx.0, phase.label()),
        None => format!("{} [{}]: {detail}", cx.op, phase.label()),
    }
}

/// Resolve a `_pop` export's OUT triple from the next staged command — the
/// engine→host direction.
///
/// Unlike `guard`, the success `mempool_out` is the **staged command's own pool**
/// (which holds its string/image data), registered here; and an empty queue is
/// not an error but the engine's "no command" seed — [`StatusName::NONE`] written
/// into the payload's field 0 with `status3 == 100` and no mempool, exactly the
/// engine's pre-pop behaviour.
///
/// `encode` turns the dequeued [`Command`] into its raw payload, allocating any
/// descriptor arrays into the command's pool; a panic in it is caught and surfaced
/// as an [`ENGINE_PANIC`] error in a fresh pool (the half-built staged pool is
/// dropped), never unwinding across the C boundary.
///
/// # Safety
/// `out` must be valid for writing one `R` (field 0 a [`StatusName`]); `status3`,
/// `errbuf16`, `mempool_out` must each be valid for writes.
// The arg list mirrors the C `_pop` export's own (`out` + the OUT triple) plus the
// registry/breadcrumb/encoder it needs — bundling the OUT triple into a struct would
// just hide the 1:1 mapping to the ABI signature.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn pop_command<R>(
    registry: &mut Registry,
    cx: Context,
    staged: Option<Staged>,
    out: *mut R,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
    encode: impl FnOnce(Command, &CallPool) -> R,
) -> i32 {
    let Some(Staged { command, pool }) = staged else {
        // Nothing queued for this pop. Seed the engine's "no command" tag into
        // field 0 and succeed with no mempool — the engine's pre-pop seed. (A
        // well-behaved host never reaches here: it pops only what
        // `get_next_command` announced.)
        // SAFETY: `out` is valid for a write and field 0 is a `StatusName` at
        // offset 0 (every command payload is `#[repr(C)]` with the id first).
        unsafe {
            out.cast::<StatusName>().write(StatusName::NONE);
            *status3 = SUCCESS;
            *mempool_out = MempoolHandle::EMPTY;
        }
        return 1;
    };

    phase(Phase::Encode);
    // Pass the *lazy* pool: a scalar/marker `encode` ignores it (it stays empty →
    // `EMPTY` handle), a list payload materializes it only to alloc its descriptors.
    match catch_unwind(AssertUnwindSafe(|| encode(command, &pool))) {
        Ok(raw) => {
            // The staged pool carries the command's data → it is the mempool the
            // host frees. (Empty for scalar/marker commands → `EMPTY` handle.)
            let handle = pool.into_handle(registry);
            // SAFETY: `raw` is a fully-initialized `R`; out-params valid (caller).
            unsafe {
                out.write(raw);
                *status3 = SUCCESS;
                *mempool_out = handle;
            }
            1
        }
        Err(payload) => {
            // Encoding panicked — a codec/framework bug. Drop the half-built staged
            // pool and report through a fresh one, mirroring `guard`'s error path.
            drop(pool);
            let detail = format!("encoding a command panicked: {}", panic_text(&*payload));
            let message = decorate(&cx, Phase::Encode, &detail);
            let errpool = OwnedPool::new();
            let pooled = errpool.alloc_str(&message);
            let descriptor = ustring_opt(Some(&pooled));
            let handle = registry.register(errpool);
            // SAFETY: out-params valid (caller); the descriptor points into the
            // registered error pool, kept alive until the host frees `handle`.
            unsafe {
                *status3 = ENGINE_PANIC;
                *errbuf16 = descriptor;
                *mempool_out = handle;
            }
            0
        }
    }
}

/// Write an error OUT triple, `message` pooled in a fresh registered pool. Returns
/// `0` (the export's failure int). For framework-level rejects that arise before
/// any command pool exists — bad context, library not initialized.
///
/// # Safety
/// `status3`, `errbuf16`, `mempool_out` must each be valid for writes.
pub(crate) unsafe fn fail(
    registry: &mut Registry,
    code: i32,
    message: &str,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    let pool = OwnedPool::new();
    let pooled = pool.alloc_str(message);
    let descriptor = ustring_opt(Some(&pooled));
    let handle = registry.register(pool);
    // SAFETY: out-params valid (caller); descriptor points into the registered
    // pool, alive until the host frees `handle`.
    unsafe {
        *status3 = code;
        *errbuf16 = descriptor;
        *mempool_out = handle;
    }
    0
}

/// Surface a caught panic as an [`ENGINE_PANIC`] error, decorated with `cx` and the
/// phase the panic happened in (the thread-local [`Phase`] breadcrumb). Returns `0`.
///
/// # Safety
/// As [`fail`].
pub(crate) unsafe fn fail_panic(
    registry: &mut Registry,
    cx: &Context,
    payload: &(dyn Any + Send),
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    let detail = format!("handler panicked: {}", panic_text(payload));
    let message = decorate(cx, current_phase(), &detail);
    // SAFETY: forwarded to `fail`, whose contract the caller upholds.
    unsafe { fail(registry, ENGINE_PANIC, &message, status3, errbuf16, mempool_out) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fprt_core::pool::Pool;

    fn out_params() -> (i32, Ustring, MempoolHandle) {
        (
            0,
            Ustring {
                len: 0,
                utf8: core::ptr::null(),
            },
            MempoolHandle::EMPTY,
        )
    }

    fn read_errbuf(reg: &Registry, err: Ustring, mp: MempoolHandle) -> String {
        let pool: Pool = reg.pool(mp).expect("error pool registered");
        // SAFETY: `err` was written into `mp`'s pool, which `pool` reads from.
        let s = unsafe { pool.string(err) }.expect("non-empty message");
        s.as_str().unwrap().to_string()
    }

    #[test]
    fn fail_pools_the_message_and_registers_one_handle() {
        let mut r = Registry::new();
        let (mut s3, mut err, mut mp) = out_params();
        let ret =
            unsafe { fail(&mut r, 0x0bfb_082a, "developers path empty", &mut s3, &mut err, &mut mp) };
        assert_eq!(ret, 0);
        assert_eq!(s3, 0x0bfb_082a);
        assert_ne!(mp, MempoolHandle::EMPTY);
        assert_eq!(r.live(), 1);
        assert_eq!(read_errbuf(&r, err, mp), "developers path empty");
    }

    #[test]
    fn fail_panic_is_decorated_with_op_ctx_and_phase() {
        let mut r = Registry::new();
        let (mut s3, mut err, mut mp) = out_params();
        phase(Phase::Decode);
        let context = Context {
            op: "ui.inputfa.ok event",
            ctx: Some(Ctx(9)),
        };
        // A caught-panic payload, as `catch_unwind` would hand it back.
        let payload: Box<dyn Any + Send> = Box::new("bad address frogans*foo");
        let ret = unsafe { fail_panic(&mut r, &context, &*payload, &mut s3, &mut err, &mut mp) };
        assert_eq!(ret, 0);
        assert_eq!(s3, ENGINE_PANIC);
        let msg = read_errbuf(&r, err, mp);
        assert!(msg.contains("ui.inputfa.ok event (ctx 9) [decode]"), "{msg}");
        assert!(
            msg.contains("handler panicked: bad address frogans*foo"),
            "{msg}"
        );
    }

    #[test]
    fn pop_no_command_seeds_none_with_empty_handle() {
        let mut r = Registry::new();
        let (mut s3, mut err, mut mp) = out_params();
        let mut out = core::mem::MaybeUninit::<StatusName>::uninit();
        let ret = unsafe {
            pop_command(&mut r, cx(), None, out.as_mut_ptr(), &mut s3, &mut err, &mut mp, |_c, _p| {
                unreachable!("no staged command, so encode is never called")
            })
        };
        assert_eq!(ret, 1);
        assert_eq!(s3, SUCCESS);
        assert_eq!(unsafe { out.assume_init() }, StatusName::NONE);
        assert_eq!(mp, MempoolHandle::EMPTY);
        assert_eq!(r.live(), 0);
    }

    #[test]
    fn pop_marker_command_is_empty_handle() {
        let mut r = Registry::new();
        let (mut s3, mut err, mut mp) = out_params();
        let mut out = core::mem::MaybeUninit::<StatusName>::uninit();
        // A marker command: encode allocates nothing → its pool stays empty.
        let staged = Some(Staged {
            command: Command::MenuOpen,
            pool: CallPool::new(),
        });
        let ret = unsafe {
            pop_command(&mut r, cx(), staged, out.as_mut_ptr(), &mut s3, &mut err, &mut mp, |_c, _p| {
                StatusName(0x2195bc) // menu CMD_OPEN
            })
        };
        assert_eq!(ret, 1);
        assert_eq!(s3, SUCCESS);
        assert_eq!(unsafe { out.assume_init() }, StatusName(0x2195bc));
        assert_eq!(mp, MempoolHandle::EMPTY, "marker carries no data");
        assert_eq!(r.live(), 0);
    }

    #[test]
    fn pop_pooled_command_registers_its_staged_pool() {
        let mut r = Registry::new();
        let (mut s3, mut err, mut mp) = out_params();
        let mut out = core::mem::MaybeUninit::<StatusName>::uninit();
        // A command whose data was allocated into its pool at emit time.
        let pool = CallPool::new();
        let _ = pool.alloc_str("réseau");
        let staged = Some(Staged {
            command: Command::MenuOpen,
            pool,
        });
        let ret = unsafe {
            pop_command(&mut r, cx(), staged, out.as_mut_ptr(), &mut s3, &mut err, &mut mp, |_c, _p| {
                StatusName(0x2195bc)
            })
        };
        assert_eq!(ret, 1);
        assert_ne!(mp, MempoolHandle::EMPTY, "pooled data → a real handle");
        assert_eq!(r.live(), 1);
    }

    #[test]
    fn engine_panic_code_is_the_agreed_sentinel() {
        assert_eq!(ENGINE_PANIC, 777_999_777);
    }

    fn cx() -> Context {
        Context {
            op: "test.op",
            ctx: None,
        }
    }
}
