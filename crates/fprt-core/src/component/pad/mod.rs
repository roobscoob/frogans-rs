//! `pad` — the floating Frogans pad / launcher window. Commands only, no events.
//! Only `update_layout` carries data; the lifecycle commands are bare markers
//! handled by the transport layer.

mod cmd_update_layout;

pub use cmd_update_layout::UpdateLayout;
