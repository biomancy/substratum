use super::adapter::{AdaptBufRead, AdaptRead, Adapter};
use super::decode::{
    DecodeBufReadIntoBufRead, DecodeBufReadIntoRead, DecodeReadIntoBufRead, DecodeReadIntoRead,
};
use std::io;
use std::io::{BufRead, Error, Read};
use std::num::NonZeroUsize;

#[cfg_attr(feature = "bitcode", derive(::bitcode::Encode, ::bitcode::Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Deflate {
    in_bufsize: NonZeroUsize,
    out_bufsize: NonZeroUsize,
}

impl Deflate {
    pub const DEFAULT: Deflate = Deflate {
        in_bufsize: NonZeroUsize::new(32 * 1024).unwrap(),
        out_bufsize: NonZeroUsize::new(8 * 1024).unwrap(),
    };

    pub fn new(deflate_bufsize: NonZeroUsize, out_bufsize: NonZeroUsize) -> Self {
        Self {
            in_bufsize: deflate_bufsize,
            out_bufsize,
        }
    }

    pub fn in_bufsize(&self) -> usize {
        self.in_bufsize.get()
    }

    pub fn out_bufsize(&self) -> usize {
        self.out_bufsize.get()
    }
}

impl Default for Deflate {
    fn default() -> Self {
        Deflate::DEFAULT
    }
}

impl<'a, R: Read + 'a, A: Adapter> DecodeReadIntoRead<'a, R, A> for Deflate
where
    A: AdaptRead<'a, flate2::read::DeflateDecoder<R>>,
{
    fn decode_read_into_read(&self, reader: R, adapter: A) -> Result<A::Read<'a>, Error> {
        adapter.wrap(flate2::read::DeflateDecoder::new(reader))
    }
}

impl<'a, R: Read + 'a, A: Adapter> DecodeReadIntoBufRead<'a, R, A> for Deflate
where
    A: AdaptBufRead<'a, io::BufReader<flate2::read::DeflateDecoder<R>>>,
{
    fn decode_read_into_bufread(&self, reader: R, adapter: A) -> Result<A::BufRead<'a>, Error> {
        let reader = flate2::read::DeflateDecoder::new(reader);
        let bufreader = io::BufReader::with_capacity(self.out_bufsize.get(), reader);
        adapter.wrap(bufreader)
    }
}

impl<'a, R: BufRead + 'a, A: Adapter> DecodeBufReadIntoRead<'a, R, A> for Deflate
where
    A: AdaptRead<'a, flate2::bufread::DeflateDecoder<R>>,
{
    fn decode_bufread_into_read(&self, reader: R, adapter: A) -> Result<A::Read<'a>, Error> {
        let reader = flate2::bufread::DeflateDecoder::new(reader);
        adapter.wrap(reader)
    }
}

impl<'a, R: BufRead + 'a, A: Adapter> DecodeBufReadIntoBufRead<'a, R, A> for Deflate
where
    A: AdaptBufRead<'a, io::BufReader<flate2::bufread::DeflateDecoder<R>>>,
{
    fn decode_bufread_into_bufread(&self, reader: R, adapter: A) -> Result<A::BufRead<'a>, Error> {
        let reader = flate2::bufread::DeflateDecoder::new(reader);
        let bufreader = io::BufReader::with_capacity(self.out_bufsize.get(), reader);
        adapter.wrap(bufreader)
    }
}
