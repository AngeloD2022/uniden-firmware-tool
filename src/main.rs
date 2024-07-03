mod file;

use std::fs;
use std::fs::File;
use std::io::{Cursor};
use binrw::{BinRead, BinReaderExt};

const TEST_FILE: &str = "./R8_v128.106.116_db240402.bin";


#[inline(always)]
fn change(val: u32) -> u32 {
    (val & 0xfffffe00) + 0x200
}


fn main() {
    println!("Hello, world!");

    // let file = File::open(TEST_FILE)?;
    let data = fs::read(TEST_FILE);

    let mut cursor = Cursor::new(data.unwrap());

    let ui_len = cursor.read_le::<u32>().unwrap();
    let ui_len = cursor.read_le::<u32>().unwrap();
    let ui_len = cursor.read_le::<u32>().unwrap();


}
