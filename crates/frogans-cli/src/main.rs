//! `frogans` — a headless command-line front-end over the `fprt` runtime.
//!
//! Unlike the `rose-demo` example, the engine module is not baked in next to a
//! manifest: `--fprt` points at the `fprt.dll` to drive, and `--data` (default
//! `<dll_dir>/data`) is the writable root the engine's four directories live
//! under. Today there is one command — [`render`](Command::Render), which leaps
//! to a Frogans address and emits the resulting slide as SVG — but the session
//! plumbing ([`session`]) is built to carry the rest later.

mod session;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand, ValueEnum};

use session::Slide;

#[derive(Parser)]
#[command(name = "frogans", version, about = "Drive the Frogans Player Runtime from the command line")]
struct Cli {
    /// Path to the engine module to load (`fprt.dll`).
    #[arg(long, value_name = "DLL")]
    fprt: PathBuf,

    /// Writable root for the engine's directories (created if missing).
    /// Defaults to `<dll_dir>/data`.
    #[arg(long, value_name = "DIR")]
    data: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Leap to a Frogans address and emit its rendered slide as SVG.
    Render(RenderArgs),
}

#[derive(Args)]
struct RenderArgs {
    /// A Frogans address, e.g. `frogans*demo`.
    address: String,

    /// Write the SVG here instead of standard output.
    #[arg(short, long, value_name = "FILE")]
    out: Option<PathBuf>,

    /// Which slide of the site to render.
    #[arg(long, value_enum, default_value_t = SlideArg::Lead)]
    slide: SlideArg,

    /// Locale reported to the engine at start.
    #[arg(long, default_value = "en_US.UTF-8")]
    locale: String,
}

/// CLI mirror of [`session::Slide`] (so the engine type needs no clap derives).
#[derive(Clone, Copy, ValueEnum)]
enum SlideArg {
    /// The expanded lead slide (carries the interactive rollovers).
    Lead,
    /// The collapsed vignette slide.
    Vignette,
}

impl From<SlideArg> for Slide {
    fn from(s: SlideArg) -> Self {
        match s {
            SlideArg::Lead => Slide::Lead,
            SlideArg::Vignette => Slide::Vignette,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("frogans: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // The engine needs its four directories to exist and be writable; derive the
    // root from the DLL's location unless overridden, and create it eagerly.
    let data = cli.data.unwrap_or_else(|| {
        cli.fprt
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("data")
    });

    match cli.command {
        Command::Render(args) => {
            session::ensure_directories(&data)?;

            let visual = session::render_address(&cli.fprt, &data, &args.address, &args.locale)?;
            let representation = match args.slide.into() {
                Slide::Lead => &visual.lead,
                Slide::Vignette => &visual.vignette,
            };
            let document = frogans_svg::representation_to_svg(representation);

            match args.out {
                Some(path) => std::fs::write(&path, document)?,
                None => print!("{document}"),
            }
            Ok(())
        }
    }
}
