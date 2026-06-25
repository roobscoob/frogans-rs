//! `fprt_conductor_start` — boot a conductor from a config.
//!
//! Export VA `0x6d6025f0`. The odd one out: it takes a [`StartInformation`]
//! pointer (not a [`Ctx`]) and *produces* the `Ctx` (`out_id`) plus a mempool
//! handle (`mempool_out`).

use crate::ctx::Ctx;
use crate::mem::MempoolHandle;
use crate::start_information::StartInformation;
use crate::ustring::Ustring;

/// `status3`: a config enum field (`imgfmt_a/b`, `reserved_flag`,
/// `deployment_mode`, `devtools_support`) was out of its legal range.
pub const BAD_ENUM: i32 = 0x0bfb_0829;
/// `status3`: a required field was null/empty (e.g. a `path` data pointer).
pub const NULL_OR_EMPTY: i32 = 0x0bfb_082a;
/// `status3`: object / mempool creation failed.
pub const CREATE_FAILED: i32 = 0x0bfb_082b;
/// `status3`: a config ustring failed to convert.
pub const BAD_STRING: i32 = 0x0bfb_082c;
/// `status3`: engine fleet not cruising (library not initialized).
pub const NOT_CRUISING: i32 = 0x0bfb_082d;
/// `status3`: generic internal failure.
pub const INTERNAL_FAILURE: i32 = 0x0bfb_083c;
// `0x0bfb_082e..=0x0bfb_0839` encode late inner-validation failures (font
// integrity, path emptiness/writability, manifest-string emptiness); see
// `work/notes/conductorconfig-reimpl.md` for the full `0xf90cx → 0x0bfb08xx` map.

/// `int fprt_conductor_start(const StartInformation *config, Ctx *out_id, int32_t *status3, Ustring *errbuf16, MempoolHandle *mempool_out);`
///
/// Boots the conductor from `config`, writing the live context handle to
/// `out_id` and a mempool token to `mempool_out`. Returns `1` on success (with
/// `status3` left at `100`), `0` on failure (with a `0x0bfb08xx` code in
/// `status3` and a detail ustring in `errbuf16`).
pub type FprtConductorStart = unsafe extern "C" fn(
    config: *const StartInformation,
    out_id: *mut Ctx,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32;
