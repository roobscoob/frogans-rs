//! `menu` — application / context menus, engine-rendered SLD content.
//!
//! `update_visual` is COMPLEX (embeds visual `Representation` + `Button`):
//! decode-only for now. `update_layout` and the `button_triggered` event are
//! full codecs. The lifecycle commands are bare markers handled by transport.

mod cmd_update_layout;
mod cmd_update_visual;
mod evt_button_triggered;

pub use cmd_update_layout::UpdateLayout;
pub use cmd_update_visual::{MenuVariant, UpdateVisual};
pub use evt_button_triggered::ReportButtonTriggered;
