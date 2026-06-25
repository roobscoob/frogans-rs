//! The `SldRect` screen rectangle (`0x10`).

/// FPRT screen coordinates, top-left origin.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SldRect {
    /// Index into the host screen list; out-of-range → primary screen.
    pub screen_index: i32,
    /// Reserved / unused (never read by the host converter).
    pub reserved: i32,
    /// X offset within the screen's frame.
    pub x: i32,
    /// Y offset (FPRT top-left; the host Y-flips to Cocoa).
    pub y: i32,
}
