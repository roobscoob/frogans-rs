//! `launch_way_out` command (engine → host) — client transport for the core codec.

use fprt_sys::Fprt;
use fprt_sys::ui::application::CMD_LAUNCH_WAY_OUT;
use fprt_sys::ui::application::launch_way_out::LaunchWayOut as Raw;
use fprt_sys::ui::{Pop, StatusName};

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::application::{LaunchWayOut, UriScheme};

impl CommandPayload for LaunchWayOut {
    const ID: StatusName = CMD_LAUNCH_WAY_OUT;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_launch_way_out
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::ApplicationLaunchWayOut(LaunchWayOut::from_raw(raw, pool))
    }
}
