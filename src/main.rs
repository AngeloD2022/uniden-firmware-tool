mod file;
mod format;
mod util;
use std::{fs, path};

use clap::Parser;

use crate::file::UnidenFirmware;

const TEST_FILE: &str = "./R8_v128.106.116_db240402.bin";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input firmware binary
    #[arg(short, long)]
    firmware: path::PathBuf,

    /// Output directory
    #[arg(short, long)]
    out_dir: Option<path::PathBuf>,
}

fn main() {
    let args = Args::parse();
    println!("Uniden Firmware BLOB Parser");
    let mut firmware = UnidenFirmware::from(&args.firmware).unwrap();
    firmware.read_buffer().unwrap();
    if let Some(dir) = args.out_dir.as_ref().cloned() {
        fs::create_dir_all(dir.as_path())
            .unwrap_or_else(|_| panic!("Couldn't create output directory: {}", dir.display()))
    }

    let metadata = firmware.metadata.unwrap();

    println!("Firmware bundle version: {}", metadata.format_version);
    println!("Model: Uniden {}", metadata.model.to_name());
    println!("Embedded files: ");
    for file in firmware.files {
        let name = file.to_file_name();
        println!("   - {}", name);
        if let Some(ref mut dir) = args.out_dir.as_ref().cloned() {
            dir.push(name);
        }
    }
}
