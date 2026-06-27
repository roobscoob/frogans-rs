//! The payload types, one module per UI component.
//!
//! Each component holds its command (pooled) and event (borrowed) payloads, each
//! a self-contained codec. Visual X-types shared by `menu`/`sitehandler`/`pad`
//! live in [`visual`].

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
pub mod selection;
pub mod sitehandler;
pub mod update;
pub mod visual;
pub mod zoom;
