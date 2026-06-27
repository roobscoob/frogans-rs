//! `fprt.dll` (`.so` / `.dylib`) — the shippable engine library (the root
//! package's `cdylib`).
//!
//! Three crates, one artifact: the [`fprt-impl`] engine, wrapped by the
//! [`fprt-server`] harness into an [`Fprt`] table, surfaced by [`fprt-exports`] as
//! the 179 `#[no_mangle]` C symbols the Frogans Player host imports by name. This
//! file is just the glue + the load-time install.
//!
//! Install must happen before the host calls any export, so it runs from a
//! **load-time constructor** ([`#[ctor]`](ctor::ctor)) — the dynamic library is
//! mapped, the constructor runs, *then* the host calls exports. `ctor` picks the
//! right platform hook (Windows CRT init / ELF `.init_array` / Mach-O
//! `__mod_init_func`), so this stays one portable annotation. The work is light —
//! build the engine, claim the global, hand over the table — so it's safe under the
//! loader lock; heavy startup belongs in the engine's `library_initialize`
//! handling, which the host calls later.
//!
//! [`Fprt`]: fprt_exports::Fprt
//! [`fprt-impl`]: https://docs.rs/fprt-impl
//! [`fprt-server`]: https://docs.rs/fprt-server
//! [`fprt-exports`]: https://docs.rs/fprt-exports

use fprt_server::Server;

/// Build the engine, claim the process engine slot, and publish its table to the
/// export shell — at library load, before the host calls any export. Idempotent in
/// effect: a second attempt (or a slot already taken) is a silent no-op rather than
/// an overwrite.
#[ctor::ctor]
fn install_engine() {
    attach_debug_console();
    let server = Server::new::<fprt_impl::FrogansEngine>();
    if let Ok(table) = server.into_process_engine() {
        let _ = fprt_exports::install(table);
    }
}

/// In **debug builds on Windows**, attach a console at load so the engine's
/// `println!` / panic output is visible. A DLL loaded by a GUI host
/// (`FrogansPlayer.exe`) inherits no console, so stdout/stderr otherwise go nowhere.
/// Release builds (and other platforms, which keep their console) do nothing.
#[cfg(all(windows, debug_assertions))]
fn attach_debug_console() {
    // kernel32 `AllocConsole` — gives this process a console window if it has none
    // (no-op if launched from a terminal). Running it in the ctor means the new
    // stdout/stderr handles are in place before any `println!`.
    unsafe extern "system" {
        fn AllocConsole() -> i32;
    }
    // SAFETY: `AllocConsole` takes no arguments and is always safe to call — it just
    // returns 0 if a console already exists.
    unsafe {
        AllocConsole();
    }
    eprintln!("[fprt.dll] debug console attached");
}

#[cfg(not(all(windows, debug_assertions)))]
fn attach_debug_console() {}
