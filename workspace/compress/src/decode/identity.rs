use super::adapter::{AdaptBufRead, AdaptRead, Adapter};
use super::decode::{
    DecodeBufReadIntoBufRead, DecodeBufReadIntoRead, DecodeReadIntoBufRead, DecodeReadIntoRead,
};
use std::io::{BufRead, BufReader, Error, Read};
use std::num::NonZeroUsize;

#[cfg_attr(feature = "bitcode", derive(::bitcode::Encode, ::bitcode::Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity {
    out_bufsize: NonZeroUsize,
}

impl Identity {
    pub const DEFAULT: Identity = Identity {
        out_bufsize: NonZeroUsize::new(8 * 1024).unwrap(),
    };

    pub fn new(out_bufsize: NonZeroUsize) -> Self {
        Self { out_bufsize }
    }

    pub fn out_bufsize(&self) -> NonZeroUsize {
        self.out_bufsize
    }
}
impl Default for Identity {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl<'a, R: Read + 'a, A: Adapter> DecodeReadIntoRead<'a, R, A> for Identity
where
    A: AdaptRead<'a, R>,
{
    fn decode_read_into_read(&self, reader: R, adapter: A) -> Result<A::Read<'a>, Error> {
        adapter.wrap(reader)
    }
}

impl<'a, R: Read + 'a, A: Adapter> DecodeReadIntoBufRead<'a, R, A> for Identity
where
    A: AdaptBufRead<'a, BufReader<R>>,
{
    fn decode_read_into_bufread(&self, reader: R, adapter: A) -> Result<A::BufRead<'a>, Error> {
        adapter.wrap(BufReader::with_capacity(self.out_bufsize.get(), reader))
    }
}

impl<'a, R: BufRead + 'a, A: Adapter> DecodeBufReadIntoRead<'a, R, A> for Identity
where
    A: AdaptRead<'a, R>,
{
    fn decode_bufread_into_read(&self, reader: R, adapter: A) -> Result<A::Read<'a>, Error> {
        adapter.wrap(reader)
    }
}

impl<'a, R: BufRead + 'a, A: Adapter> DecodeBufReadIntoBufRead<'a, R, A> for Identity
where
    A: AdaptBufRead<'a, R>,
{
    fn decode_bufread_into_bufread(&self, reader: R, adapter: A) -> Result<A::BufRead<'a>, Error> {
        adapter.wrap(reader)
    }
}
