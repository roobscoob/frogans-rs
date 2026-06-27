//! `update_legal_content` command (engine → host) — the nested document tree.

use fprt_sys::ui::legalinformation::legal_content::UpdateLegalContent as Raw;
use fprt_sys::ui::legalinformation::CMD_UPDATE_LEGAL_CONTENT;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::legalinformation::{
    Document, LegalContentKind, Topic, UpdateLegalContent,
};

impl CommandPayload for UpdateLegalContent {
    const ID: StatusName = CMD_UPDATE_LEGAL_CONTENT;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.legalinformation_update_legal_content
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        // SAFETY: `raw`'s arrays/strings/image were written into `pool` by the pop
        // that produced both.
        Command::LegalinformationUpdateLegalContent(unsafe {
            UpdateLegalContent::from_raw(raw, pool)
        })
    }
}
