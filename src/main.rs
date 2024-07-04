mod file;
mod util;
mod format;

use binrw::{BinRead, BinReaderExt};
use std::fs;
use std::fs::File;
use std::io::Cursor;
use crate::file::UnidenFirmware;

const TEST_FILE: &str = "./R8_v128.106.116_db240402.bin";


fn main() {
    println!("Hello, world!");

    // let file = File::open(TEST_FILE)?;
    let mut firmware = UnidenFirmware::from(TEST_FILE).unwrap();
    firmware.read_buffer().unwrap();

}
