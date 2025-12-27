use super::adapter::{AdaptWrite, Adapter};
use super::encode::Encode;
use std::io;
use std::io::{Error, Write};

#[cfg_attr(feature = "bitcode", derive(::bitcode::Encode, ::bitcode::Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Deflate {
    level: u8,
}

impl Deflate {
    pub const NONE: Deflate = Deflate { level: 0 };
    pub const FAST: Deflate = Deflate { level: 1 };
    pub const DEFAULT: Deflate = Deflate { level: 6 };
    pub const BEST: Deflate = Deflate { level: 9 };

    pub fn new(level: u8) -> io::Result<Self> {
        if level > 9 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid DEFLATE compress level: {}", level),
            ));
        }
        Ok(Self { level })
    }

    pub fn level(&self) -> u8 {
        self.level
    }
}

impl Default for Deflate {
    fn default() -> Self {
        Deflate::DEFAULT
    }
}

impl<'a, W: Write + 'a, A: Adapter> Encode<'a, W, A> for Deflate
where
    A: AdaptWrite<'a, flate2::write::DeflateEncoder<W>>,
{
    fn encode(&self, writer: W, adapter: A) -> Result<A::Write<'a>, Error> {
        adapter.wrap(flate2::write::DeflateEncoder::new(
            writer,
            flate2::Compression::new(self.level as u32),
        ))
    }
}
