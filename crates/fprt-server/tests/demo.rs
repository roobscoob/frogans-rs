//! End-to-end demo: build a safe [`Server`] from one engine function, install it
//! as the process engine, and drive the resulting raw [`Fprt`] table exactly as
//! `FrogansPlayer.exe` would — `library_initialize` → `conductor_start` →
//! `sync_enter` → report an event → drain commands via
//! `get_next_command`/`_pop` → `sync_leave` — asserting the bytes the host sees.
//!
//! One `#[test]` only: `into_process_engine` claims a process-global `OnceLock`, so
//! the table is installed once and every check runs against it.

use std::mem::MaybeUninit;
use std::ptr;
use std::time::Duration;

use fprt_core::command::InspectorOpen;
use fprt_core::component::application::{AddClipboardText, ReportStart, UpdateZoom};
use fprt_core::component::inputfa::ReportOk;
use fprt_core::component::inspector::InspectorId;
use fprt_core::{Command, Event, NextWake, StartInfo};
use fprt_server::Server;
use fprt_sys::ctx::Ctx;
use fprt_sys::library_version::LibraryVersion;
use fprt_sys::mem::MempoolHandle;
use fprt_sys::ui::StatusName;
use fprt_sys::ui::application::{
    CMD_ADD_CLIPBOARD_TEXT, CMD_STOP, CMD_UPDATE_ZOOM, add_clipboard_text, update_zoom,
};
use fprt_sys::ui::inspector::{CMD_OPEN as INSPECTOR_OPEN, head};
use fprt_sys::ui::menu::CMD_OPEN;
use fprt_sys::ustring::Ustring;

/// The trailing OUT triple every UI/conductor call writes, freshly zeroed.
fn out_triple() -> (i32, Ustring, MempoolHandle) {
    (
        0,
        Ustring {
            len: 0,
            utf8: ptr::null(),
        },
        MempoolHandle::EMPTY,
    )
}

/// Read a `Ustring` the engine wrote (its bytes live in the registered mempool).
unsafe fn read_ustring(u: Ustring) -> Option<String> {
    if u.utf8.is_null() {
        return None;
    }
    // SAFETY: the engine wrote `len` UTF-8 bytes at `utf8`, kept alive by the
    // mempool the host hasn't freed yet.
    let bytes = unsafe { std::slice::from_raw_parts(u.utf8, u.len as usize) };
    Some(String::from_utf8(bytes.to_vec()).unwrap())
}

#[test]
fn demo_drives_the_raw_fprt_table() {
    // ── the entire engine: one (stateless) handler ──────────────────────────
    let server = Server::from_event_fn(|_elapsed, event, out| match event {
        Event::ApplicationStart(start) => {
            assert_eq!(start.locale, "en_US");
            out.command(fprt_core::command::MenuOpen); // marker
            out.command(UpdateZoom::new(150)); // scalar
            out.command_pooled(|p| AddClipboardText::new(p, "réseau").into()); // pooled
            NextWake::In(Duration::from_millis(16))
        }
        Event::InputfaOk(ok) => {
            // typed-event decode: the borrowed field text crossed the FFI seam.
            assert_eq!(ok.text, "frogans*confirmed");
            // id-carrier emit: the InspectorId rides through to the Head payload.
            out.command(InspectorOpen(InspectorId(7)));
            NextWake::Idle
        }
        Event::ApplicationQuit => {
            out.command(fprt_core::command::ApplicationStop);
            NextWake::Idle
        }
        _ => NextWake::Idle,
    });

    // ── install it as the process engine → the raw C-ABI table ──────────────
    let table = server
        .into_process_engine()
        .unwrap_or_else(|_| panic!("first install succeeds"));

    // A second install collides and is rejected, handing the server back.
    let other = Server::from_event_fn(|_, _, _| NextWake::Idle);
    assert!(
        other.into_process_engine().is_err(),
        "the process engine slot is taken"
    );

    // ── library lifecycle: init with the required version (the engine is built
    //    here) ─────────────────────────────────────────────────────────────────
    assert!(!unsafe { (table.library_is_initialized)() });
    let version = LibraryVersion::REQUIRED;
    let mut status_lib = 7u32; // success leaves this untouched
    assert_eq!(
        unsafe { (table.library_initialize)(&version, &mut status_lib) },
        1
    );
    assert_eq!(status_lib, 7, "success leaves status_lib_out untouched");
    assert!(unsafe { (table.library_is_initialized)() });

    // ── boot a conductor from a (valid) config ──────────────────────────────
    let config = StartInfo::EMPTY.to_raw();
    let mut ctx = Ctx(0);
    let (mut s3, mut err, mut mp) = out_triple();
    assert_eq!(
        unsafe { (table.conductor_start)(&config, &mut ctx, &mut s3, &mut err, &mut mp) },
        1
    );
    assert_eq!(s3, 100);
    assert_eq!(mp, MempoolHandle::EMPTY);

    // ── open a turn, report `application_start` ─────────────────────────────
    let (mut s3, mut err, mut mp) = out_triple();
    assert_eq!(
        unsafe { (table.conductor_sync_enter)(ctx, 16, &mut s3, &mut err, &mut mp) },
        1
    );
    assert_eq!(s3, 100);

    let start_raw = ReportStart::new("en_US").to_raw();
    let (mut s3, mut err, mut mp) = out_triple();
    assert_eq!(
        unsafe { (table.application_start)(ctx, &start_raw, &mut s3, &mut err, &mut mp) },
        1
    );
    assert_eq!(s3, 100);
    assert_eq!(mp, MempoolHandle::EMPTY, "an event allocates no mempool");

    // ── drain the emitted commands, FIFO ────────────────────────────────────
    let mut seen = Vec::new();
    loop {
        let (mut has, mut id) = (0u32, 0u32);
        let (mut s3, mut err, mut mp) = out_triple();
        let r = unsafe {
            (table.conductor_get_next_command)(
                ctx, &mut has, &mut id, &mut s3, &mut err, &mut mp,
            )
        };
        assert_eq!(r, 1);
        assert_eq!(s3, 100);
        if has == 0 {
            break;
        }

        match StatusName(id) {
            CMD_OPEN => {
                let mut raw = MaybeUninit::<StatusName>::uninit();
                let (mut s3, mut err, mut mp) = out_triple();
                let r = unsafe {
                    (table.menu_open)(ctx, raw.as_mut_ptr(), &mut s3, &mut err, &mut mp)
                };
                assert_eq!(r, 1);
                assert_eq!(s3, 100);
                assert_eq!(unsafe { raw.assume_init() }, CMD_OPEN); // field 0 = the id
                assert_eq!(mp, MempoolHandle::EMPTY, "marker carries no data");
                seen.push(Command::MenuOpen);
            }
            CMD_UPDATE_ZOOM => {
                let mut raw = MaybeUninit::<update_zoom::UpdateZoom>::uninit();
                let (mut s3, mut err, mut mp) = out_triple();
                let r = unsafe {
                    (table.application_update_zoom)(
                        ctx,
                        raw.as_mut_ptr(),
                        &mut s3,
                        &mut err,
                        &mut mp,
                    )
                };
                assert_eq!(r, 1);
                assert_eq!(s3, 100);
                let raw = unsafe { raw.assume_init() };
                assert_eq!(raw.status_id, CMD_UPDATE_ZOOM);
                assert_eq!(raw.zoom_level_percent, 150);
                assert_eq!(mp, MempoolHandle::EMPTY, "scalar carries no mempool");
                seen.push(Command::ApplicationUpdateZoom(UpdateZoom::from_raw(raw)));
            }
            CMD_ADD_CLIPBOARD_TEXT => {
                let mut raw = MaybeUninit::<add_clipboard_text::AddClipboardText>::uninit();
                let (mut s3, mut err, mut mp) = out_triple();
                let r = unsafe {
                    (table.application_add_clipboard_text)(
                        ctx,
                        raw.as_mut_ptr(),
                        &mut s3,
                        &mut err,
                        &mut mp,
                    )
                };
                assert_eq!(r, 1);
                assert_eq!(s3, 100);
                let raw = unsafe { raw.assume_init() };
                assert_eq!(raw.status_id, CMD_ADD_CLIPBOARD_TEXT);
                let text = unsafe { read_ustring(raw.text) };
                assert_eq!(text.as_deref(), Some("réseau"));
                assert_ne!(mp, MempoolHandle::EMPTY, "pooled data → a real mempool");
                // The host frees the argument pool it was handed.
                assert!(unsafe { (table.library_free_allocated_arguments)(mp) });
                seen.push(Command::ApplicationAddClipboardText(AddClipboardText {
                    text: None,
                }));
            }
            other => panic!("unexpected command id {other:?}"),
        }
    }

    assert_eq!(seen.len(), 3, "MenuOpen, UpdateZoom, AddClipboardText");
    assert!(matches!(seen[0], Command::MenuOpen));
    assert!(matches!(seen[1], Command::ApplicationUpdateZoom(z) if z.percent == 150));
    assert!(matches!(seen[2], Command::ApplicationAddClipboardText(_)));

    // ── close the turn: the engine's next-wake is reported ──────────────────
    let mut nextwake = 0u32;
    let (mut s3, mut err, mut mp) = out_triple();
    assert_eq!(
        unsafe { (table.conductor_sync_leave)(ctx, &mut nextwake, &mut s3, &mut err, &mut mp) },
        1
    );
    assert_eq!(s3, 100);
    assert_eq!(NextWake::from_raw(nextwake), NextWake::In(Duration::from_millis(16)));

    // ── a typed-event turn: inputfa.ok decodes, engine opens an inspector ───
    let (mut s3, mut err, mut mp) = out_triple();
    assert_eq!(
        unsafe { (table.conductor_sync_enter)(ctx, 8, &mut s3, &mut err, &mut mp) },
        1
    );
    let ok_raw = ReportOk::new("frogans*confirmed").to_raw();
    let (mut s3, mut err, mut mp) = out_triple();
    assert_eq!(
        unsafe { (table.inputfa_ok)(ctx, &ok_raw, &mut s3, &mut err, &mut mp) },
        1
    );
    assert_eq!(s3, 100);

    let (mut has, mut id) = (0u32, 0u32);
    let (mut s3, mut err, mut mp) = out_triple();
    unsafe { (table.conductor_get_next_command)(ctx, &mut has, &mut id, &mut s3, &mut err, &mut mp) };
    assert_eq!(has, 1);
    assert_eq!(StatusName(id), INSPECTOR_OPEN);

    let mut raw = MaybeUninit::<head::Head>::uninit();
    let (mut s3, mut err, mut mp) = out_triple();
    assert_eq!(
        unsafe { (table.inspector_open)(ctx, raw.as_mut_ptr(), &mut s3, &mut err, &mut mp) },
        1
    );
    let raw = unsafe { raw.assume_init() };
    assert_eq!(raw.status_id, INSPECTOR_OPEN);
    assert_eq!(raw.reference, 7, "the InspectorId carried through id-carrier encode");

    let mut nextwake = 0u32;
    let (mut s3, mut err, mut mp) = out_triple();
    unsafe { (table.conductor_sync_leave)(ctx, &mut nextwake, &mut s3, &mut err, &mut mp) };
    assert_eq!(NextWake::from_raw(nextwake), NextWake::Idle);

    // ── a second turn: a quit event makes the engine ask the host to stop ───
    let (mut s3, mut err, mut mp) = out_triple();
    assert_eq!(
        unsafe { (table.conductor_sync_enter)(ctx, 16, &mut s3, &mut err, &mut mp) },
        1
    );
    let quit = fprt_sys::ui::EventTag(0); // the marker's tag is ignored on decode
    let (mut s3, mut err, mut mp) = out_triple();
    assert_eq!(
        unsafe { (table.application_quit)(ctx, &quit, &mut s3, &mut err, &mut mp) },
        1
    );

    let (mut has, mut id) = (0u32, 0u32);
    let (mut s3, mut err, mut mp) = out_triple();
    unsafe { (table.conductor_get_next_command)(ctx, &mut has, &mut id, &mut s3, &mut err, &mut mp) };
    assert_eq!(has, 1);
    assert_eq!(StatusName(id), CMD_STOP);

    let mut raw = MaybeUninit::<StatusName>::uninit();
    let (mut s3, mut err, mut mp) = out_triple();
    assert_eq!(
        unsafe { (table.application_stop)(ctx, raw.as_mut_ptr(), &mut s3, &mut err, &mut mp) },
        1
    );
    assert_eq!(unsafe { raw.assume_init() }, CMD_STOP);

    // ── tear down ───────────────────────────────────────────────────────────
    let (mut s3, mut err, mut mp) = out_triple();
    assert_eq!(
        unsafe { (table.conductor_stop)(ctx, &mut s3, &mut err, &mut mp) },
        1
    );
    // A stopped ctx is gone: a further call on it is rejected, not a panic.
    let (mut s3, mut err, mut mp) = out_triple();
    assert_eq!(
        unsafe { (table.conductor_sync_enter)(ctx, 0, &mut s3, &mut err, &mut mp) },
        0
    );
    assert_eq!(s3, fprt_sys::conductor::sync_enter::INVALID_CONTEXT);
}
