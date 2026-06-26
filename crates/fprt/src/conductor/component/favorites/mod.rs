//! `favorites` — the favorites-manager dialog.
//!
//! Five lifecycle markers + `update_labels` / `update_addresses` (engine → host);
//! `open` / `remove` carry the selected addresses, `remove_all` / `cancel` are
//! bare (host → engine).

use crate::conductor::command::marker_command;
use crate::conductor::report::{address_selection_event, marker_event};

mod cmd_update_addresses;
mod cmd_update_labels;

pub use cmd_update_addresses::UpdateAddresses;
pub use cmd_update_labels::UpdateLabels;

// --- lifecycle commands (engine → host), no payload ---
marker_command!(Open, fprt_sys::ui::favorites::CMD_OPEN, favorites_open, FavoritesOpen);
marker_command!(Show, fprt_sys::ui::favorites::CMD_SHOW, favorites_show, FavoritesShow);
marker_command!(Push, fprt_sys::ui::favorites::CMD_PUSH, favorites_push, FavoritesPush);
marker_command!(Hide, fprt_sys::ui::favorites::CMD_HIDE, favorites_hide, FavoritesHide);
marker_command!(
    Close,
    fprt_sys::ui::favorites::CMD_CLOSE,
    favorites_close,
    FavoritesClose
);

// --- events (host → engine) ---
address_selection_event!(
    ReportOpen,
    fprt_sys::ui::favorites::EVT_OPEN,
    favorites_open_event
);
address_selection_event!(
    ReportRemove,
    fprt_sys::ui::favorites::EVT_REMOVE,
    favorites_remove
);
marker_event!(
    ReportRemoveAll,
    fprt_sys::ui::favorites::EVT_REMOVE_ALL,
    favorites_remove_all
);
marker_event!(
    ReportCancel,
    fprt_sys::ui::favorites::EVT_CANCEL,
    favorites_cancel
);
