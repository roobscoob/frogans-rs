//! The FFI/DLL boundary: install one [`Server`] as the process engine and get the
//! [`Fprt`] table of 179 `extern "C"` trampolines `fprt-exports` turns into C
//! symbols.
//!
//! A `Server` is static-free so many coexist for testing;
//! [`into_process_engine`](Server::into_process_engine) is where exactly one
//! crosses into the process — its [`ServerInner`] moves into the [`ENGINE`] global
//! and every trampoline reads it under a lock. The table is only reachable through
//! that call, so a running trampoline always finds the global set.
//!
//! Trampolines share three bodies: `report::<E>` (an event — decode, run one turn),
//! `pop::<C>` (a command — dequeue, encode), and the 12 bespoke lifecycle calls.
//! All 167 UI exports + 12 lifecycle calls are wired.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use fprt_core::StartInfo;
use fprt_core::component as fc;
use fprt_sys::Fprt;
use fprt_sys::conductor::{get_next_command, start};
use fprt_sys::ctx::Ctx;
use fprt_sys::library::initialize as lib_init;
use fprt_sys::library_version::LibraryVersion;
use fprt_sys::mem::MempoolHandle;
use fprt_sys::start_information::StartInformation;
use fprt_sys::ui::StatusName;
use fprt_sys::ustring::Ustring;

use crate::command::{self, CommandEncode};
use crate::encoder::{self, Context, Phase, SUCCESS};
use crate::event::{self, EventDecode};
use crate::server::{Server, ServerInner};

/// The one installed engine. Written once by [`Server::into_process_engine`], read
/// under its lock by every trampoline. The DLL is inherently one-engine-per-process
/// (raw C fn pointers can't capture state), which is why this is global — but it is
/// the *only* global, so `Server`s remain testable in isolation.
///
/// `ServerInner` is `!Send` — an engine may be thread-affine (e.g. a proxy wrapping
/// the `fprt` client's single-thread `Library`) — so a bare `Mutex<ServerInner>`
/// static won't do (that needs `Send`). This wrapper asserts `Send + Sync` under the
/// FPRT **single-thread-host contract**.
struct ProcessEngine(Mutex<ServerInner>);

// SAFETY: the host drives every export from one thread — FPRT engines are
// thread-affine, exactly like the real `fprt.dll` and the `fprt` client's `Library`.
// The inner `Mutex` serializes access; engine state is never touched from another
// thread. (The cross-thread `Sender` does NOT go through here — it owns the outbox's
// own `Arc<Mutex>`, which stays `Send + Sync` on its own.)
unsafe impl Send for ProcessEngine {}
unsafe impl Sync for ProcessEngine {}

static ENGINE: OnceLock<ProcessEngine> = OnceLock::new();

/// Run `f` against the installed engine under its lock. Panics only if nothing is
/// installed — unreachable, since a trampoline is only callable via the table
/// `into_process_engine` returns after setting [`ENGINE`].
fn with_engine<R>(f: impl FnOnce(&mut ServerInner) -> R) -> R {
    let engine = ENGINE
        .get()
        .expect("fprt-server: no process engine installed");
    // A handler panic is caught inside the trampoline, so the lock is never
    // poisoned by one; recover anyway rather than double-panic across `extern "C"`.
    let mut guard = engine.0.lock().unwrap_or_else(|p| p.into_inner());
    f(&mut guard)
}

impl Server {
    /// Install this server as the process engine and hand back the [`Fprt`] table
    /// of C-ABI trampolines (give it to `fprt_exports::install`). **Mutates process
    /// global state**: it moves the server's state into a process-wide singleton, so
    /// it succeeds at most once. On a second call it does nothing and returns the
    /// *original* server back as `Err`, so the caller can recover it — no panic, no
    /// silent overwrite.
    // The `Err` payload *is* the recovered `Server` (a cold, once-per-process
    // collision path), so the large variant is intentional, not worth boxing.
    #[allow(clippy::result_large_err)]
    pub fn into_process_engine(self) -> Result<Fprt, Server> {
        match ENGINE.set(ProcessEngine(Mutex::new(self.inner))) {
            Ok(()) => Ok(build_table()),
            Err(returned) => Err(Server {
                inner: returned.0.into_inner().unwrap_or_else(|p| p.into_inner()),
            }),
        }
    }
}

// ============================================================================
// The three shared trampoline bodies.
// ============================================================================

/// An event export (`_report`): reject out-of-state calls (`BASE + offset`), else
/// decode the payload and run the conductor for one event (emitting commands into
/// its outbox). No OUT data → `EMPTY` mempool. Decode + handler run under
/// `catch_unwind` so neither unwinds across `extern "C"`.
unsafe extern "C" fn report<E: EventDecode>(
    ctx: Ctx,
    payload: *const E::Raw,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    with_engine(|inner| {
        let cx = Context {
            op: E::OP,
            ctx: Some(ctx),
        };
        // Framework rejects before the engine runs (error-code DB §5.1: BASE+offset).
        if let Err(reject) = inner.event_ready(ctx) {
            // SAFETY: out-params valid (host contract).
            return unsafe {
                encoder::fail(
                    &mut inner.registry,
                    (E::BASE + reject.offset()) as i32,
                    "event out of conductor state",
                    status3,
                    errbuf16,
                    mempool_out,
                )
            };
        }
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            encoder::phase(Phase::Decode);
            // SAFETY: `payload` is valid for this call (host contract); the borrowed
            // `Event` is consumed before returning.
            let evt = unsafe { E::decode(&*payload) };
            encoder::phase(Phase::Handler);
            inner.run_event(ctx, evt);
        }));
        match outcome {
            Ok(()) => {
                // An event allocates no OUT data → `EMPTY` mempool; emitted commands
                // carry their own pools, registered at their pop.
                // SAFETY: out-params valid (host contract).
                unsafe {
                    *status3 = SUCCESS;
                    *mempool_out = MempoolHandle::EMPTY;
                }
                1
            }
            // SAFETY: out-params valid (host contract).
            Err(p) => unsafe {
                encoder::fail_panic(&mut inner.registry, &cx, &*p, status3, errbuf16, mempool_out)
            },
        }
    })
}

/// A command export (`_pop`): dequeue the next staged command and encode it into the
/// OUT payload; its own pool becomes the mempool the host frees.
unsafe extern "C" fn pop<C: CommandEncode>(
    ctx: Ctx,
    out: *mut C::Raw,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    with_engine(|inner| {
        let cx = Context {
            op: C::OP,
            ctx: Some(ctx),
        };
        match inner.pop_command(ctx) {
            // No such conductor — bad context (`BASE + 0`).
            // SAFETY: out-params valid (host contract).
            None => unsafe {
                encoder::fail(
                    &mut inner.registry,
                    C::BASE as i32,
                    "invalid context",
                    status3,
                    errbuf16,
                    mempool_out,
                )
            },
            // A staged command (or an empty queue → `NONE` seed inside `pop_command`).
            // SAFETY: out-params valid; field 0 of `C::Raw` is a `StatusName` at
            // offset 0 (every command payload is `#[repr(C)]`).
            Some(staged) => unsafe {
                encoder::pop_command(
                    &mut inner.registry,
                    cx,
                    staged,
                    out,
                    status3,
                    errbuf16,
                    mempool_out,
                    C::encode,
                )
            },
        }
    })
}

// ============================================================================
// The 12 lifecycle trampolines (bespoke signatures).
// ============================================================================

unsafe extern "C" fn library_initialize(
    version: *const LibraryVersion,
    status_lib_out: *mut u32,
) -> i32 {
    with_engine(|inner| {
        // SAFETY: `version` is valid for a read when non-null (host contract).
        let result = if version.is_null() {
            Err(lib_init::BAD_VERSION)
        } else {
            inner.initialize(unsafe { *version })
        };
        match result {
            Ok(()) => 1,
            Err(code) => {
                // SAFETY: out-param valid (host contract).
                unsafe {
                    *status_lib_out = code;
                }
                0
            }
        }
    })
}

unsafe extern "C" fn library_finalize(out_status: *mut u32) -> u32 {
    with_engine(|inner| {
        inner.finalize();
        // SAFETY: out-param valid (host contract).
        unsafe {
            *out_status = 0;
        }
        0
    })
}

unsafe extern "C" fn library_is_initialized() -> bool {
    with_engine(|inner| inner.is_initialized())
}

unsafe extern "C" fn library_report_allocated_arguments(
    out_count: *mut u32,
    out_total: *mut u32,
) -> i32 {
    with_engine(|inner| {
        let (count, total) = inner.report_args();
        // SAFETY: out-params valid (host contract).
        unsafe {
            *out_count = count;
            *out_total = total;
        }
        1
    })
}

unsafe extern "C" fn library_free_allocated_arguments(handle: MempoolHandle) -> bool {
    with_engine(|inner| inner.registry.free(handle))
}

unsafe extern "C" fn conductor_start(
    config: *const StartInformation,
    out_id: *mut Ctx,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    with_engine(|inner| {
        if config.is_null() {
            // SAFETY: out-params valid (host contract).
            return unsafe {
                encoder::fail(
                    &mut inner.registry,
                    start::NULL_OR_EMPTY,
                    "null config",
                    status3,
                    errbuf16,
                    mempool_out,
                )
            };
        }
        // SAFETY: `config` is valid for the call (host contract); `StartInfo` borrows
        // it for the `new_conductor` call only.
        let info = StartInfo::from_raw(unsafe { &*config });
        match inner.start(info) {
            Ok(ctx) => {
                // SAFETY: out-params valid (host contract).
                unsafe {
                    *out_id = ctx;
                    *status3 = SUCCESS;
                    *mempool_out = MempoolHandle::EMPTY;
                }
                1
            }
            Err(e) => {
                let msg = e
                    .message()
                    .map(|m| m.to_string_lossy().into_owned())
                    .unwrap_or_default();
                // SAFETY: out-params valid (host contract).
                unsafe {
                    encoder::fail(&mut inner.registry, e.code(), &msg, status3, errbuf16, mempool_out)
                }
            }
        }
    })
}

unsafe extern "C" fn conductor_stop(
    ctx: Ctx,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    with_engine(|inner| match inner.stop(ctx) {
        Ok(()) => {
            // SAFETY: out-params valid (host contract).
            unsafe {
                *status3 = SUCCESS;
                *mempool_out = MempoolHandle::EMPTY;
            }
            1
        }
        // SAFETY: out-params valid (host contract).
        Err(code) => unsafe {
            encoder::fail(
                &mut inner.registry,
                code,
                "conductor stop rejected",
                status3,
                errbuf16,
                mempool_out,
            )
        },
    })
}

unsafe extern "C" fn conductor_get_next_command(
    ctx: Ctx,
    has_command: *mut u32,
    command_id: *mut u32,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    with_engine(|inner| match inner.next_command_id(ctx) {
        // SAFETY: out-params valid (host contract).
        None => unsafe {
            encoder::fail(
                &mut inner.registry,
                get_next_command::INVALID_CONTEXT,
                "invalid context",
                status3,
                errbuf16,
                mempool_out,
            )
        },
        Some(id_opt) => {
            // SAFETY: out-params valid (host contract). An empty queue is a successful
            // fetch with `has_command == 0` (not an error).
            unsafe {
                *has_command = id_opt.is_some() as u32;
                *command_id = id_opt.unwrap_or(StatusName::NONE).0;
                *status3 = SUCCESS;
                *mempool_out = MempoolHandle::EMPTY;
            }
            1
        }
    })
}

unsafe extern "C" fn conductor_sync_enter(
    ctx: Ctx,
    elapsed_ms: i32,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    // `elapsed_ms` is ≥ -1; clamp the -1 "unknown" to zero.
    let elapsed = Duration::from_millis(elapsed_ms.max(0) as u64);
    with_engine(|inner| match inner.sync_enter(ctx, elapsed) {
        Ok(()) => {
            // SAFETY: out-params valid (host contract).
            unsafe {
                *status3 = SUCCESS;
                *mempool_out = MempoolHandle::EMPTY;
            }
            1
        }
        // SAFETY: out-params valid (host contract).
        Err(code) => unsafe {
            encoder::fail(
                &mut inner.registry,
                code,
                "sync_enter rejected",
                status3,
                errbuf16,
                mempool_out,
            )
        },
    })
}

unsafe extern "C" fn conductor_sync_leave(
    ctx: Ctx,
    nextwake_out: *mut u32,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    with_engine(|inner| match inner.sync_leave(ctx) {
        Ok(wake) => {
            // SAFETY: out-params valid (host contract).
            unsafe {
                *nextwake_out = wake.to_raw();
                *status3 = SUCCESS;
                *mempool_out = MempoolHandle::EMPTY;
            }
            1
        }
        // SAFETY: out-params valid (host contract).
        Err(code) => unsafe {
            encoder::fail(
                &mut inner.registry,
                code,
                "sync_leave rejected",
                status3,
                errbuf16,
                mempool_out,
            )
        },
    })
}

unsafe extern "C" fn conductor_sleep_enter(
    ctx: Ctx,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    with_engine(|inner| match inner.sleep_enter(ctx) {
        Ok(()) => {
            // SAFETY: out-params valid (host contract).
            unsafe {
                *status3 = SUCCESS;
                *mempool_out = MempoolHandle::EMPTY;
            }
            1
        }
        // SAFETY: out-params valid (host contract).
        Err(code) => unsafe {
            encoder::fail(
                &mut inner.registry,
                code,
                "sleep_enter rejected",
                status3,
                errbuf16,
                mempool_out,
            )
        },
    })
}

unsafe extern "C" fn conductor_sleep_leave(
    ctx: Ctx,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    with_engine(|inner| match inner.sleep_leave(ctx) {
        Ok(()) => {
            // SAFETY: out-params valid (host contract).
            unsafe {
                *status3 = SUCCESS;
                *mempool_out = MempoolHandle::EMPTY;
            }
            1
        }
        // SAFETY: out-params valid (host contract).
        Err(code) => unsafe {
            encoder::fail(
                &mut inner.registry,
                code,
                "sleep_leave rejected",
                status3,
                errbuf16,
                mempool_out,
            )
        },
    })
}

// ============================================================================
// The 179-field table. Modeled exports name their `pop::<C>` / `report::<E>`;
// the rest ride the generic `unimpl_*` stub (its raw type inferred per field) so
// the table type-checks today and fills in incrementally.
// ============================================================================

fn build_table() -> Fprt {
    Fprt {
        library_initialize,
        library_finalize,
        library_is_initialized,
        library_report_allocated_arguments,
        library_free_allocated_arguments,
        conductor_start,
        conductor_stop,
        conductor_get_next_command,
        conductor_sync_enter,
        conductor_sync_leave,
        conductor_sleep_enter,
        conductor_sleep_leave,

        // ui::application
        application_update_images: pop::<fc::application::UpdateImages>,
        application_update_zoom: pop::<fc::application::UpdateZoom>,
        application_update_layout: pop::<fc::application::UpdateLayout>,
        application_update_directionality: pop::<fc::application::UpdateDirectionality>,
        application_add_clipboard_text: pop::<fc::application::AddClipboardText>,
        application_add_clipboard_image: pop::<fc::application::AddClipboardImage>,
        application_open_directory: pop::<fc::application::OpenDirectory>,
        application_reinitialize_developers_directory: pop::<
            command::ApplicationReinitializeDevelopersDirectory,
        >,
        application_launch_way_out: pop::<fc::application::LaunchWayOut>,
        application_stop: pop::<command::ApplicationStop>,
        application_start: report::<event::ApplicationStart>,
        application_timeout: report::<event::ApplicationTimeout>,
        application_menu_access_wanted: report::<event::ApplicationMenuAccessWanted>,
        application_menu_access_unwanted: report::<event::ApplicationMenuAccessUnwanted>,
        application_leaptofrogans: report::<event::ApplicationLeaptofrogans>,
        application_quit: report::<event::ApplicationQuit>,
        application_change_layout: report::<event::ApplicationChangeLayout>,

        // ui::sitehandler
        sitehandler_open: pop::<command::SitehandlerOpen>,
        sitehandler_close: pop::<command::SitehandlerClose>,
        sitehandler_show: pop::<command::SitehandlerShow>,
        sitehandler_hide: pop::<command::SitehandlerHide>,
        sitehandler_begin_animation_inprogress: pop::<command::SitehandlerBeginAnimationInprogress>,
        sitehandler_end_animation_inprogress: pop::<command::SitehandlerEndAnimationInprogress>,
        sitehandler_push: pop::<command::SitehandlerPush>,
        sitehandler_update_layout: pop::<fc::sitehandler::UpdateLayout>,
        sitehandler_update_visual: pop::<fc::sitehandler::UpdateVisual>,
        sitehandler_button_triggered: report::<event::SitehandlerButtonTriggered>,
        sitehandler_force_close: report::<event::SitehandlerForceClose>,

        // ui::menu
        menu_open: pop::<command::MenuOpen>,
        menu_show: pop::<command::MenuShow>,
        menu_push: pop::<command::MenuPush>,
        menu_hide: pop::<command::MenuHide>,
        menu_close: pop::<command::MenuClose>,
        menu_update_visual: pop::<fc::menu::UpdateVisual>,
        menu_update_layout: pop::<fc::menu::UpdateLayout>,
        menu_button_triggered: report::<event::MenuButtonTriggered>,

        // ui::favorites
        favorites_open: pop::<command::FavoritesOpen>,
        favorites_show: pop::<command::FavoritesShow>,
        favorites_push: pop::<command::FavoritesPush>,
        favorites_hide: pop::<command::FavoritesHide>,
        favorites_close: pop::<command::FavoritesClose>,
        favorites_update_labels: pop::<fc::favorites::UpdateLabels>,
        favorites_update_addresses: pop::<fc::favorites::UpdateAddresses>,
        favorites_open_event: report::<event::FavoritesOpen>,
        favorites_remove: report::<event::FavoritesRemove>,
        favorites_remove_all: report::<event::FavoritesRemoveAll>,
        favorites_cancel: report::<event::FavoritesCancel>,

        // ui::recentlyvisited
        recentlyvisited_open: pop::<command::RecentlyvisitedOpen>,
        recentlyvisited_show: pop::<command::RecentlyvisitedShow>,
        recentlyvisited_push: pop::<command::RecentlyvisitedPush>,
        recentlyvisited_hide: pop::<command::RecentlyvisitedHide>,
        recentlyvisited_close: pop::<command::RecentlyvisitedClose>,
        recentlyvisited_update_labels: pop::<fc::recentlyvisited::UpdateLabels>,
        recentlyvisited_update_addresses: pop::<fc::recentlyvisited::UpdateAddresses>,
        recentlyvisited_open_event: report::<event::RecentlyvisitedOpen>,
        recentlyvisited_delete: report::<event::RecentlyvisitedDelete>,
        recentlyvisited_delete_all: report::<event::RecentlyvisitedDeleteAll>,
        recentlyvisited_cancel: report::<event::RecentlyvisitedCancel>,

        // ui::inputfa
        inputfa_open: pop::<command::InputfaOpen>,
        inputfa_show: pop::<command::InputfaShow>,
        inputfa_push: pop::<command::InputfaPush>,
        inputfa_hide: pop::<command::InputfaHide>,
        inputfa_close: pop::<command::InputfaClose>,
        inputfa_update_error_clear: pop::<command::InputfaUpdateErrorClear>,
        inputfa_update_address: pop::<fc::inputfa::UpdateAddress>,
        inputfa_update_error_raise: pop::<fc::inputfa::UpdateErrorRaise>,
        inputfa_update_labels: pop::<fc::inputfa::UpdateLabels>,
        inputfa_change: report::<event::InputfaChange>,
        inputfa_ok: report::<event::InputfaOk>,
        inputfa_cancel: report::<event::InputfaCancel>,

        // ui::blocked
        blocked_open: pop::<command::BlockedOpen>,
        blocked_show: pop::<command::BlockedShow>,
        blocked_push: pop::<command::BlockedPush>,
        blocked_hide: pop::<command::BlockedHide>,
        blocked_close: pop::<command::BlockedClose>,
        blocked_update_labels: pop::<fc::blocked::UpdateLabels>,
        blocked_update_addresses: pop::<fc::blocked::UpdateAddresses>,
        blocked_remove: report::<event::BlockedRemove>,
        blocked_remove_all: report::<event::BlockedRemoveAll>,
        blocked_cancel: report::<event::BlockedCancel>,

        // ui::devtools
        devtools_open: pop::<command::DevtoolsOpen>,
        devtools_show: pop::<command::DevtoolsShow>,
        devtools_push: pop::<command::DevtoolsPush>,
        devtools_hide: pop::<command::DevtoolsHide>,
        devtools_close: pop::<command::DevtoolsClose>,
        devtools_update_labels: pop::<fc::devtools::UpdateLabels>,
        devtools_update_addresses: pop::<fc::devtools::UpdateAddresses>,
        devtools_inspect: report::<event::DevtoolsInspect>,
        devtools_cancel: report::<event::DevtoolsCancel>,

        // ui::recovery
        recovery_open: pop::<command::RecoveryOpen>,
        recovery_show: pop::<command::RecoveryShow>,
        recovery_hide: pop::<command::RecoveryHide>,
        recovery_close: pop::<command::RecoveryClose>,
        recovery_update_labels: pop::<fc::recovery::UpdateLabels>,
        recovery_update_addresses: pop::<fc::recovery::UpdateAddresses>,
        recovery_open_event: report::<event::RecoveryOpen>,
        recovery_cancel: report::<event::RecoveryCancel>,

        // ui::zoom
        zoom_open: pop::<command::ZoomOpen>,
        zoom_show: pop::<command::ZoomShow>,
        zoom_push: pop::<command::ZoomPush>,
        zoom_hide: pop::<command::ZoomHide>,
        zoom_close: pop::<command::ZoomClose>,
        zoom_update_labels: pop::<fc::zoom::UpdateLabels>,
        zoom_ok: report::<event::ZoomOk>,
        zoom_cancel: report::<event::ZoomCancel>,

        // ui::update
        update_open: pop::<command::UpdateOpen>,
        update_show: pop::<command::UpdateShow>,
        update_push: pop::<command::UpdatePush>,
        update_hide: pop::<command::UpdateHide>,
        update_close: pop::<command::UpdateClose>,
        update_update_labels: pop::<fc::update::UpdateLabels>,
        update_update_data: pop::<fc::update::UpdateData>,
        update_cancel: report::<event::UpdateCancel>,

        // ui::pad
        pad_open: pop::<command::PadOpen>,
        pad_show: pop::<command::PadShow>,
        pad_hide: pop::<command::PadHide>,
        pad_close: pop::<command::PadClose>,
        pad_begin_animation: pop::<command::PadBeginAnimation>,
        pad_end_animation: pop::<command::PadEndAnimation>,
        pad_update_layout: pop::<fc::pad::UpdateLayout>,

        // ui::language
        language_open: pop::<command::LanguageOpen>,
        language_show: pop::<command::LanguageShow>,
        language_push: pop::<command::LanguagePush>,
        language_hide: pop::<command::LanguageHide>,
        language_close: pop::<command::LanguageClose>,
        language_update_labels: pop::<fc::language::UpdateLabels>,
        language_update_list: pop::<fc::language::UpdateList>,
        language_ok: report::<event::LanguageOk>,
        language_cancel: report::<event::LanguageCancel>,

        // ui::leaptofrogans
        leaptofrogans_open: pop::<command::LeaptofrogansOpen>,
        leaptofrogans_show: pop::<command::LeaptofrogansShow>,
        leaptofrogans_push: pop::<command::LeaptofrogansPush>,
        leaptofrogans_hide: pop::<command::LeaptofrogansHide>,
        leaptofrogans_close: pop::<command::LeaptofrogansClose>,
        leaptofrogans_update_labels: pop::<fc::leaptofrogans::UpdateLabels>,
        leaptofrogans_update_address: pop::<fc::leaptofrogans::UpdateAddress>,
        leaptofrogans_confirm: report::<event::LeaptofrogansConfirm>,
        leaptofrogans_cancel: report::<event::LeaptofrogansCancel>,
        leaptofrogans_block: report::<event::LeaptofrogansBlock>,
        leaptofrogans_purge: report::<event::LeaptofrogansPurge>,
        leaptofrogans_close_event: report::<event::LeaptofrogansClose>,

        // ui::legalinformation
        legalinformation_open: pop::<command::LegalinformationOpen>,
        legalinformation_show: pop::<command::LegalinformationShow>,
        legalinformation_push: pop::<command::LegalinformationPush>,
        legalinformation_hide: pop::<command::LegalinformationHide>,
        legalinformation_close: pop::<command::LegalinformationClose>,
        legalinformation_update_labels: pop::<fc::legalinformation::UpdateLabels>,
        legalinformation_update_legal_content: pop::<fc::legalinformation::UpdateLegalContent>,
        legalinformation_close_event: report::<event::LegalinformationClose>,

        // ui::inspector (lifecycle = id-carriers)
        inspector_open: pop::<command::InspectorOpen>,
        inspector_close: pop::<command::InspectorClose>,
        inspector_show: pop::<command::InspectorShow>,
        inspector_hide: pop::<command::InspectorHide>,
        inspector_push: pop::<command::InspectorPush>,
        inspector_update_address: pop::<fc::inspector::UpdateAddress>,
        inspector_update_status: pop::<fc::inspector::UpdateStatus>,
        inspector_update_labels: pop::<fc::inspector::UpdateLabels>,
        inspector_update_steps_labels: pop::<fc::inspector::UpdateStepsLabels>,
        inspector_update_content_labels: pop::<fc::inspector::UpdateContentLabels>,
        inspector_update_content_viewer: pop::<fc::inspector::UpdateContentViewer>,
        inspector_update_sync: pop::<fc::inspector::UpdateSync>,
        inspector_step_selected: report::<event::InspectorStepSelected>,
        inspector_content_selected: report::<event::InspectorContentSelected>,
        inspector_synchronize: report::<event::InspectorSynchronize>,
        inspector_change_autosync: report::<event::InspectorChangeAutosync>,
        inspector_rerun: report::<event::InspectorRerun>,
        inspector_close_event: report::<event::InspectorClose>,
    }
}
