use std::io;
use std::io::{Cursor, Read};

fn read_n_bytes(cursor: &mut Cursor<&Vec<u8>>, n: usize) -> io::Result<Vec<u8>> {
    let mut buffer = vec![0; n];
    cursor.read(&mut buffer)?;
    Ok(buffer)
}

#[inline(always)]
pub(crate) fn alter_length(length: i32) -> i32 {
    // can also be written as (length & 0xfffffe00) + 512
    if length != 0 {
        (length / 512 + 1) * 512
    } else {
        length
    }
}

pub trait CursorHelper {
    fn pop(&mut self) -> io::Result<u8>;
    fn read_n(&mut self, n: usize) -> io::Result<Vec<u8>>;
    fn seek(&mut self, n: u64);
}

impl CursorHelper for Cursor<&Vec<u8>> {
    fn pop(&mut self) -> io::Result<u8> {
        let r = read_n_bytes(self, 1)?;
        Ok(r[0])
    }

    fn read_n(&mut self, n: usize) -> io::Result<Vec<u8>> {
        read_n_bytes(self, 1)
    }

    fn seek(&mut self, n: u64) {
        self.set_position(self.position() + n)
    }
}
