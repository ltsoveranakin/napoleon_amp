use rand::prelude::*;
use serbytes::prelude::{
    from_buf, BBReadResult, ReadByteBufferRefMut, SerBytes, WriteByteBufferOwned,
};
use std::fmt::{Display, Formatter, Write};
use std::hash::{Hash, Hasher};
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

#[derive(Copy, Clone, Eq, Debug)]
pub struct Id {
    header: u8,
    pub increment: u8,
    pub time: u16,
    pub data: [u8; 12],
}

impl Hash for Id {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(self.increment);
        state.write_u16(self.time);
        state.write(&self.data);
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        self.increment == other.increment && self.time == other.time && self.data == other.data
    }
}

impl Id {
    const BYTE_LEN_WITHOUT_HEADER: usize = 15;
    const BYTE_LEN_WITH_HEADER: usize = Self::BYTE_LEN_WITHOUT_HEADER + 1;

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

    // TODO: make function to combine this and serbytes impl?
    pub fn try_from_str(s: &str) -> Option<Self> {
        let no_hyphens: Vec<char> = s.chars().filter(|c| *c != '-').collect();

        if no_hyphens.len() != Self::BYTE_LEN_WITHOUT_HEADER * 2
            && no_hyphens.len() != Self::BYTE_LEN_WITH_HEADER * 2
        {
            return None;
        }

        let mut chunks = no_hyphens.as_chunks::<2>().0.into_iter();

        let header;
        let increment;

        let maybe_header = from_hex_chars(*chunks.next()?)?;

        if is_header(maybe_header) {
            header = maybe_header;
            increment = from_hex_chars(*chunks.next()?)?;
        } else {
            header = 0b00000000;
            increment = maybe_header;
        }

        let time_hi = from_hex_chars(*chunks.next()?)? as u16;
        let time_low = from_hex_chars(*chunks.next()?)? as u16;

        let time = (time_hi << 8) | time_low;

        let mut data = [0; 12];

        for (i, &chunk) in chunks.enumerate() {
            data[i] = from_hex_chars(chunk)?;
        }

        Some(Self {
            header,
            increment,
            time,
            data,
        })
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
        let maybe_header = u8::from_buf(buf)?;

        let header;
        let increment;

        if is_header(maybe_header) {
            header = maybe_header;
            increment = u8::from_buf(buf)?;
        } else {
            header = 0b00000000;
            increment = maybe_header;
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

const HEX_CHARS: [u8; 16] = *b"0123456789ABCDEF";

fn to_hex_chars(byte: u8) -> [char; 2] {
    let char1_index = (byte >> 4) as usize;
    let char2_index = (byte & 0xF) as usize;

    let char1 = HEX_CHARS[char1_index];
    let char2 = HEX_CHARS[char2_index];

    [char1 as char, char2 as char]
}

fn from_hex_char(c: char) -> Option<u8> {
    c.to_digit(16).map(|u| u as u8)
}

fn from_hex_chars([c1, c2]: [char; 2]) -> Option<u8> {
    Some((from_hex_char(c1)? << 4) | from_hex_char(c2)?)
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
    use crate::id_generator::{Id, IdGenerator};

    #[test]
    fn test_id_gen() {
        let mut idgen = IdGenerator::new();

        for _ in 0..300 {
            println!("{}", idgen.generate_new_id());
        }
    }

    #[test]
    fn test_string() {
        let mut idgen = IdGenerator::new();

        let id = idgen.generate_new_id();

        let id_str = id.to_string();

        println!("id string: {}", id_str);

        let id_from_str = Id::try_from_str(&id_str).expect("parse from str");

        println!("from str: {}", id_from_str);

        assert_eq!(id, id_from_str);
    }
}
