//! The `Ustring` ↔ Rust-string seam shared by every payload codec.
//!
//! A [`Ustring`] is the engine's 16-byte `{len, utf8}` descriptor. These helpers
//! build one from, or read one back into, a Rust string — in whichever memory
//! regime the payload uses:
//!
//!   * **borrowed** (events) — [`ustring`] points a descriptor at a live `&str`,
//!     valid for the call; [`as_str`] reads one back as a borrow.
//!   * **pooled** (commands) — [`ustring_opt`] points a descriptor at the bytes a
//!     [`PooledString`] already owns; the read side is [`Pool::string`].
//!
//! [`Pool::string`]: crate::pool::Pool::string

use fprt_sys::ui::ImageRecord;
use fprt_sys::ui::image_record::{ImageDim, ImageSize};
use fprt_sys::ustring::Ustring;

use crate::pool::{PooledImage, PooledString};

/// A descriptor borrowing `s` for the duration of one call (the event regime).
pub fn ustring(s: &str) -> Ustring {
    Ustring {
        len: s.len() as i32,
        utf8: s.as_ptr(),
    }
}

/// A descriptor over the bytes a [`PooledString`] owns, or a null/empty
/// descriptor for `None` (the command regime).
pub fn ustring_opt(s: Option<&PooledString>) -> Ustring {
    match s {
        Some(ps) => {
            let bytes = ps.as_bytes();
            Ustring {
                len: bytes.len() as i32,
                utf8: bytes.as_ptr(),
            }
        }
        None => Ustring {
            len: 0,
            utf8: core::ptr::null(),
        },
    }
}

/// A descriptor over the bytes a [`PooledImage`] owns, or a null/empty descriptor
/// for `None` (the command regime, image variant).
pub fn image_record(img: Option<&PooledImage>) -> ImageRecord {
    match img {
        Some(pi) => {
            let bytes = pi.bytes();
            ImageRecord {
                dim: ImageDim {
                    size: ImageSize {
                        width: pi.width(),
                        height: pi.height(),
                    },
                },
                byte_len: bytes.len() as u32,
                buffer: bytes.as_ptr() as *mut u8,
            }
        }
        None => ImageRecord {
            dim: ImageDim { fmt_handle: 0 },
            byte_len: 0,
            buffer: core::ptr::null_mut(),
        },
    }
}

/// Read a descriptor back as a borrowed `&str` (the event-receive regime).
/// Null / empty / invalid-UTF-8 all read as `""`.
///
/// # Safety
///
/// `raw` must point at `len` bytes that stay valid for `'a` — e.g. an inbound
/// event payload's field, borrowed for the call.
pub unsafe fn as_str<'a>(raw: Ustring) -> &'a str {
    if raw.utf8.is_null() || raw.len <= 0 {
        return "";
    }
    // SAFETY: the caller guarantees `len` valid bytes at `utf8` for `'a`.
    let bytes = unsafe { core::slice::from_raw_parts(raw.utf8, raw.len as usize) };
    core::str::from_utf8(bytes).unwrap_or("")
}
