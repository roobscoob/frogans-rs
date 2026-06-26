//! `pad` — the floating Frogans pad / launcher window. Commands only, no events.

use crate::conductor::command::marker_command;

mod cmd_update_layout;

pub use cmd_update_layout::UpdateLayout;

marker_command!(Open, fprt_sys::ui::pad::CMD_OPEN, pad_open, PadOpen);
marker_command!(Show, fprt_sys::ui::pad::CMD_SHOW, pad_show, PadShow);
marker_command!(Hide, fprt_sys::ui::pad::CMD_HIDE, pad_hide, PadHide);
marker_command!(Close, fprt_sys::ui::pad::CMD_CLOSE, pad_close, PadClose);
marker_command!(
    BeginAnimation,
    fprt_sys::ui::pad::CMD_BEGIN_ANIMATION,
    pad_begin_animation,
    PadBeginAnimation
);
marker_command!(
    EndAnimation,
    fprt_sys::ui::pad::CMD_END_ANIMATION,
    pad_end_animation,
    PadEndAnimation
);
