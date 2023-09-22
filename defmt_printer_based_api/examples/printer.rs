/**
 * this is just the defmt-printer rewritten to use the crate
 */
use std::{
    env,
    io::{self, Read},
    path::{PathBuf},
};

use clap::Parser;
use defmt_decoder::DecodeError;
use anyhow::Context;
extern crate defmt_printer_based_api as dpba;

/// Prints defmt-encoded logs to stdout
#[derive(Parser)]
#[command(name = "defmt-print")]
struct Opts {
    #[arg(short, required = true, conflicts_with("version"))]
    elf: Option<PathBuf>,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    show_skipped_frames: bool,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short = 'V', long)]
    version: bool,
}

const READ_BUFFER_SIZE: usize = 1024;

fn main() -> anyhow::Result<()> {
    let Opts {
        elf,
        json,
        show_skipped_frames,
        verbose,
        version,
    } = Opts::parse();

    if version {
        return print_version();
    }

    defmt_decoder::log::init_logger(verbose, json, move |metadata| match verbose {
        false => defmt_decoder::log::is_defmt_frame(metadata), // We display *all* defmt frames, but nothing else.
        true => true,                                          // We display *all* frames.
    });
    
    let mut helper = dpba::DefmtPrintHelper::new(elf.context("missing or invalid elf path")?)?;

    let mut buf = [0; READ_BUFFER_SIZE];

    let mut stdin = io::stdin().lock();

    loop {
        // read from stdin and push it to the decoder
        let n = stdin.read(&mut buf)?;
        // if 0 bytes where read, we reached EOF, so quit
        if n == 0 {
            break;
        }
        match helper.handle_frame(&buf[..n]) {
            Ok(_) => {
                // nothing to do because the helper already forwarded the frame to the logger
            },
            Err(DecodeError::UnexpectedEof) => {break;},

            Err(DecodeError::Malformed) => match helper.table().encoding().can_recover() {
                // if recovery is impossible, abort
                false => return Err(DecodeError::Malformed.into()),
                // if recovery is possible, skip the current frame and continue with new data
                true => {
                    if show_skipped_frames || verbose {
                        println!("(HOST) malformed frame skipped");
                        println!("└─ {} @ {}:{}", env!("CARGO_PKG_NAME"), file!(), line!());
                    }
                    continue;
                }
            },
        };
    }
    Ok(())
}

/// Report version from Cargo.toml _(e.g. "0.1.4")_ and supported `defmt`-versions.
///
/// Used by `--version` flag.
#[allow(clippy::unnecessary_wraps)]
fn print_version() -> anyhow::Result<()> {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("supported defmt version: {}", defmt_decoder::DEFMT_VERSION);
    Ok(())
}