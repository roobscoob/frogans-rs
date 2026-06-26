//! Per-component types — the safe command payloads and the report enums — one
//! module per UI component, mirroring `fprt_sys::ui::*`. [`visual`] is the one
//! non-component module: the safe X-types shared by `menu` and `sitehandler`.

pub mod application;
pub mod blocked;
pub mod devtools;
pub mod favorites;
pub mod inputfa;
pub mod inspector;
pub mod language;
pub mod leaptofrogans;
pub mod legalinformation;
pub mod menu;
pub mod pad;
pub mod recentlyvisited;
pub mod recovery;
pub mod sitehandler;
pub mod update;
pub mod visual;
pub mod zoom;
