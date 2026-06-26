//! [`EngineError`] — the one error type for any failed engine call, from
//! bringing the library up to driving a conductor.

use core::fmt;

use crate::pool::PooledString;

/// A failed engine call.
///
/// Failure is reported the same way everywhere: a numeric status code and
/// (usually) a human-readable message the engine wrote into the call's argument
/// pool. We keep both, and keep the message *engine-owned* — a [`PooledString`]
/// borrowing the pool — rather than copying it out. That pins the engine alive
/// for as long as the error lives (so the message stays readable) and keeps
/// reading lazy; utility methods to snapshot into a Rust-owned string can come
/// later. Some calls carry no message at all — `library_initialize` reports only
/// a status word — so [`message`](EngineError::message) is always optional.
///
/// We deliberately do **not** enumerate the status codes as variants: most are
/// either impossible by our construction (null out-args) or prevented by design,
/// and the few a caller branches on are already `pub const`s in `fprt_sys`. So
/// `EngineError` stays one type across every call — init through teardown;
/// compare [`code`] against those constants (e.g.
/// `fprt_sys::library::initialize::BAD_VERSION`) when you need to.
///
/// [`code`]: EngineError::code
pub struct EngineError {
    code: i32,
    message: Option<PooledString>,
}

impl EngineError {
    pub(crate) fn new(code: i32, message: Option<PooledString>) -> Self {
        EngineError { code, message }
    }

    /// The raw status code. Compare against the `fprt_sys` per-call constants
    /// (e.g. `fprt_sys::conductor::stop::INVALID_CONTEXT`).
    pub fn code(&self) -> i32 {
        self.code
    }

    /// The engine's failure message, if it wrote one.
    pub fn message(&self) -> Option<&PooledString> {
        self.message.as_ref()
    }
}

/// Format a status code in decimal with a comma every three digits
/// (e.g. `1,000,000,000`).
fn grouped(code: i32) -> String {
    let mut out = String::new();
    if code < 0 {
        out.push('-');
    }
    let digits = code.unsigned_abs().to_string();
    let len = digits.len();
    for (i, ch) in digits.char_indices() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(m) => write!(
                f,
                "FPRT call failed (status {}): {}",
                grouped(self.code),
                m.to_string_lossy()
            ),
            None => write!(f, "FPRT call failed (status {})", grouped(self.code)),
        }
    }
}

impl fmt::Debug for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = grouped(self.code);
        f.debug_struct("EngineError")
            .field("code", &format_args!("{code}"))
            .field("message", &self.message.as_ref().map(|m| m.to_string_lossy()))
            .finish()
    }
}

impl std::error::Error for EngineError {}
