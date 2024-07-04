mod file;
mod format;
mod util;

use crate::file::UnidenFirmware;
use binrw::{BinRead, BinReaderExt};
use std::fs;
use std::fs::File;
use std::io::Cursor;

const TEST_FILE: &str = "./R8_v128.106.116_db240402.bin";

fn main() {
    println!("Hello, world!");

    // let file = File::open(TEST_FILE)?;
    let mut firmware = UnidenFirmware::from(TEST_FILE).unwrap();
    firmware.read_buffer().unwrap();
}
