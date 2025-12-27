use super::adapter::{AdaptWrite, Adapter};
use super::encode::Encode;
use super::deflate::Deflate;
use flate2::Compression;
use std::io::{Error, Write};


#[cfg_attr(feature = "bitcode", derive(::bitcode::Encode, ::bitcode::Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Gzip {
    deflate: Deflate,
}

impl Gzip {
    pub const DEFAULT: Gzip = Gzip {
        deflate: Deflate::DEFAULT
    };

    pub fn new(deflate: Deflate) -> Self {
        Gzip { deflate }
    }

    pub fn deflate(&self) -> &Deflate {
        &self.deflate
    }
}

impl Default for Gzip {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl<'a, W: Write + 'a, A: Adapter> Encode<'a, W, A> for Gzip
where
    A: AdaptWrite<'a, flate2::write::GzEncoder<W>>,
{
    fn encode(&self, writer: W, adapter: A) -> Result<A::Write<'a>, Error> {
        adapter.wrap(
            flate2::write::GzEncoder::new(writer, Compression::new(self.deflate.level() as u32))
        )
    }
}
