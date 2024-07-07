mod file;
mod format;
mod util;
use std::{fs, path};

use clap::{Parser, Subcommand};

use crate::file::UnidenFirmware;

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(name = "Uniden R-Series Firmware BLOB Parser")]
#[command(author = "@angelod2022 and @jevinskie")]
#[command(
    help_template = "{name}\nBy {author}\nVersion: {version}\n\n{usage-heading} {usage}\n{all-args} {tab}"
)]
struct Args {
    #[command(subcommand)]
    pub subcmd: SubCmd,
}

#[derive(Subcommand, Debug)]
pub enum SubCmd {
    Extract(ExtractSubcommand),
    Parse(ParseSubcommand),
}

/// Extract the contents of a firmware BLOB
#[derive(Parser, Debug)]
struct ExtractSubcommand {
    /// Input firmware BLOB
    firmware: path::PathBuf,

    /// Output directory
    out_dir: Option<path::PathBuf>,

    /// Print read intervals
    #[arg(short, long)]
    verbose: bool,
}

/// View the contents of a firmware BLOB
#[derive(Parser, Debug)]
struct ParseSubcommand {
    /// Input firmware BLOB
    firmware: path::PathBuf,

    /// Print read intervals
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cmd = Args::parse();

    match cmd.subcmd {
        SubCmd::Extract(args) => {
            let mut firmware = UnidenFirmware::from(&args.firmware).unwrap();
            firmware.read_buffer().unwrap();

            print_fw_contents(&firmware, args.verbose);

            if let Some(dir) = args.out_dir.as_ref().cloned() {
                fs::create_dir_all(dir.as_path()).unwrap_or_else(|_| {
                    panic!("Couldn't create output directory: {}", dir.display())
                })
            }
            if let Some(out_dir) = args.out_dir.as_ref().cloned() {
                firmware.extract_to(out_dir.as_path());
            }
        }
        SubCmd::Parse(args) => {
            let mut firmware: UnidenFirmware = UnidenFirmware::from(&args.firmware).unwrap();
            firmware.read_buffer().unwrap();
            print_fw_contents(&firmware, args.verbose);
        }
    }
}

fn print_fw_contents(firmware: &UnidenFirmware, intervals: bool) {
    let metadata = firmware.metadata.as_ref().unwrap();
    println!("BLOB format version: {}", metadata.format_version);
    println!("Model: Uniden {}", metadata.model.to_name());
    println!("Embedded files: ");
    for file in &firmware.files {
        let name = file.kind.to_file_name();
        println!("   - {}", name);
    }
    if intervals {
        firmware.print_intervals();
    }
}
