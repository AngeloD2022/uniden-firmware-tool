use std::fs::File;

const TEST_FILE: &str = "./R8_v128.106.116_db240402.bin";

fn main() {
    println!("Hello, world!");

    let file = File::open(TEST_FILE)?;


}
