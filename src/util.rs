use rust_lapper::{Interval, Lapper};
use std::io;
use std::io::{Cursor, Read};
use std::ops::{Deref, DerefMut};

type Iv = Interval<u64, ()>;

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
    fn seek_set(&mut self, n: u64);
}

pub struct TrackingCursor<'a> {
    cursor: Cursor<&'a Vec<u8>>,
    intervals: Vec<Iv>,
}

impl<'a> TrackingCursor<'a> {
    pub fn new(data: &'a Vec<u8>) -> Self {
        TrackingCursor {
            cursor: Cursor::new(data),
            intervals: Vec::new(),
        }
    }
    pub fn intervals(&self) -> Lapper<u64, ()> {
        let mut lapper = Lapper::new(self.intervals.clone());
        lapper.merge_overlaps();
        lapper.set_cov();
        lapper
    }
}

impl<'a> Deref for TrackingCursor<'a> {
    type Target = Cursor<&'a Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.cursor
    }
}

impl<'a> DerefMut for TrackingCursor<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cursor
    }
}

impl CursorHelper for TrackingCursor<'_> {
    fn pop(&mut self) -> io::Result<u8> {
        let r = read_n_bytes(self, 1)?;
        self.intervals.push(Iv {
            start: self.position(),
            stop: self.position() + 1,
            val: (),
        });
        Ok(r[0])
    }

    fn read_n(&mut self, n: usize) -> io::Result<Vec<u8>> {
        let r = read_n_bytes(self, n);
        if let Ok(ref _r) = r {
            self.intervals.push(Iv {
                start: self.position(),
                stop: self.position() + (n as u64),
                val: (),
            });
        }
        r
    }

    fn seek(&mut self, n: u64) {
        let pos = self.position();
        self.set_position(pos + n)
    }

    fn seek_set(&mut self, n: u64) {
        self.set_position(n)
    }
}
