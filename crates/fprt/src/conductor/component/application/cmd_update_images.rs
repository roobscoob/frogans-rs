//! `update_images` command (engine → host) — client transport for the core codec.

use fprt_sys::Fprt;
use fprt_sys::ui::application::CMD_UPDATE_IMAGES;
use fprt_sys::ui::application::update_images::UpdateImages as Raw;
use fprt_sys::ui::{Pop, StatusName};

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::application::{Animation, UpdateImages};

impl CommandPayload for UpdateImages {
    const ID: StatusName = CMD_UPDATE_IMAGES;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_update_images
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::ApplicationUpdateImages(UpdateImages::from_raw(raw, pool))
    }
}
