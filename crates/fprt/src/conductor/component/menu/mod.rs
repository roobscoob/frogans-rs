//! `menu` — application / context menus, engine-rendered SLD content.

use crate::conductor::command::marker_command;

mod cmd_update_layout;
mod cmd_update_visual;
mod evt_button_triggered;

pub use cmd_update_layout::UpdateLayout;
pub use cmd_update_visual::{MenuVariant, UpdateVisual};
pub use evt_button_triggered::ReportButtonTriggered;

marker_command!(Open, fprt_sys::ui::menu::CMD_OPEN, menu_open, MenuOpen);
marker_command!(Show, fprt_sys::ui::menu::CMD_SHOW, menu_show, MenuShow);
marker_command!(Push, fprt_sys::ui::menu::CMD_PUSH, menu_push, MenuPush);
marker_command!(Hide, fprt_sys::ui::menu::CMD_HIDE, menu_hide, MenuHide);
marker_command!(Close, fprt_sys::ui::menu::CMD_CLOSE, menu_close, MenuClose);
