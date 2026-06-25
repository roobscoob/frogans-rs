//! The `Ctx` conductor handle.

/// A packed conductor context handle — **not a pointer**, a 32-bit value.
///
/// Produced by `fprt_conductor_start` and consumed by every other conductor (and
/// later UI) call. The low 24 bits are a slot index XOR a per-run key; the high
/// 8 bits are an anti-stale generation tag, re-checked against the slot per call.
///
/// Modelled as `u32`: `start` writes it as a confirmed `uint32_t` and the engine
/// treats it as a packed 32-bit value. (Some decompiled call sites type the
/// parameter `void*`, but that is a register-width artifact, not a pointer.)
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Ctx(pub u32);
