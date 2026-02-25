use std::fmt::{Display, Formatter};
use rand::prelude::*;
use serbytes::prelude::{
    from_buf, BBReadResult, ReadByteBufferRefMut, SerBytes, WriteByteBufferOwned,
};
use std::time::SystemTime;

pub(super) struct IdGenerator {
    increment: u8,
    rng: ThreadRng,
}

impl IdGenerator {
    pub(super) fn new() -> Self {
        Self {
            increment: 0,
            rng: rand::rng(),
        }
    }

    pub(super) fn new_id(&mut self) -> Id {
        const MAX_TIME: u128 = u16::MAX as u128;

        let header = 0b00000000;
        let increment = self.increment;
        let time = (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            % MAX_TIME) as u16;
        let data = self.rng.random();

        self.increment = self.increment.overflowing_add(1).0;

        Id {
            header,
            increment,
            time,
            data,
        }
    }
}

// 0: Header (optional, first bit always 1 delegates it as a header. remaining 7 bits is header data)
//  With/without header
// (0)/(1): Increment, First bit is always 0 (distinguish from header) remaining 7 bits represents an incrementing value from the builder
// (2-3)/(1-2): Time, nanoseconds since unix epoch mod 2^16. When creating a batch of ids, the time may or may not be the same
// (4-16)/(3-15): Data bytes, should be random

#[derive(Copy, Clone)]
pub struct Id {
    header: u8,
    pub increment: u8,
    pub time: u16,
    pub data: [u8; 12],
}

impl Id {
    pub fn has_header(&self) -> bool {
        is_header(self.header)
    }

    pub fn header(&self) -> Option<u8> {
        if self.has_header() {
            Some(self.header)
        } else {
            None
        }
    }
}

fn is_header(byte: u8) -> bool {
    byte >> 7 == 0b00000001
}

impl SerBytes for Id {
    fn from_buf(buf: &mut ReadByteBufferRefMut) -> BBReadResult<Self>
    where
        Self: Sized,
    {
        let header_maybe = u8::from_buf(buf)?;

        let header;
        let increment;

        if is_header(header_maybe) {
            header = header_maybe;
            increment = u8::from_buf(buf)?;
        } else {
            header = 0b00000000;
            increment = header_maybe;
        }

        let time = from_buf(buf)?;

        let data = buf
            .read_bytes(12)?
            .try_into()
            .expect("Read 12 bytes from the buffer, can't fail");

        Ok(Self {
            header,
            increment,
            time,
            data,
        })
    }

    fn to_buf(&self, buf: &mut WriteByteBufferOwned) {
        if self.has_header() {
            self.header.to_buf(buf);
        }

        self.increment.to_buf(buf);
        self.time.to_buf(buf);
        buf.write_bytes(&self.data);
    }
}

fn get_hex_str(byte:u8) -> [char; 2] {
    let char1 = byte & (!0) >>
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str()
    }
}