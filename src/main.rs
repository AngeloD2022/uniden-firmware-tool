mod file;
mod format;
mod util;

use crate::file::UnidenFirmware;
use binrw::{BinRead, BinReaderExt};

const TEST_FILE: &str = "./R8_v128.106.116_db240402.bin";

fn main() {
    println!("Uniden Firmware BLOB Parser");

    // let file = File::open(TEST_FILE)?;
    let mut firmware = UnidenFirmware::from(TEST_FILE).unwrap();
    firmware.read_buffer().unwrap();

    let metadata = firmware.metadata.unwrap();

    println!("Firmware bundle version: {}", metadata.format_version);
    println!("Model: Uniden {}", metadata.model.to_name());
    println!("Embedded files: ");
    for file in firmware.files {
        let name = file.to_file_name();
        println!("   - {}", name);
    }
}
