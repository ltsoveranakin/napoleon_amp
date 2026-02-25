use rand::prelude::*;
use serbytes::prelude::{
    from_buf, BBReadResult, ReadByteBufferRefMut, SerBytes, WriteByteBufferOwned,
};
use std::fmt::{Display, Formatter, Write};
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

    pub(super) fn generate_new_id(&mut self) -> Id {
        const MAX_TIME: u128 = u16::MAX as u128;

        let header = 0b00000000;
        let increment = self.increment;
        let time = (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            % MAX_TIME) as u16;
        let data = self.rng.random();

        self.increment += 1;
        self.increment = self.increment % 127;

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

#[derive(Copy, Clone, Debug)]
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

    pub fn headerless_bytes(&self) -> [u8; 15] {
        let mut out = [0u8; 15];

        out[0] = self.increment;

        out[1..2].copy_from_slice(&self.time.to_be_bytes());
        out[3..].copy_from_slice(&self.data);

        out
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

fn to_hex_chars(byte: u8) -> [char; 2] {
    const CHARS: [char; 16] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
    ];

    let char1_index = (byte >> 4) as usize;
    let char2_index = (byte & 0xF) as usize;

    let char1 = CHARS[char1_index];
    let char2 = CHARS[char2_index];

    [char1, char2]
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.has_header() {
            write!(f, "{:?}-", to_hex_chars(self.header))?;
        }

        let time_bytes = self.time.to_be_bytes();

        let incr_chars = to_hex_chars(self.increment);

        let time_chars1 = to_hex_chars(time_bytes[0]);
        let time_chars2 = to_hex_chars(time_bytes[1]);

        write!(
            f,
            "{}{}-{}{}{}{}-",
            incr_chars[0],
            incr_chars[1],
            time_chars1[0],
            time_chars1[1],
            time_chars2[0],
            time_chars2[1]
        )?;

        for byte in self.data {
            let [char1, char2] = to_hex_chars(byte);

            f.write_char(char1)?;
            f.write_char(char2)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::id_generator::IdGenerator;

    #[test]
    fn test_id_gen() {
        let mut idgen = IdGenerator::new();

        for _ in 0..300 {
            println!("{}", idgen.generate_new_id());
        }
    }
}
