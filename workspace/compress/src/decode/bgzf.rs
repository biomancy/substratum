use super::adapter::{AdaptBufRead, AdaptRead, Adapter};
use super::decode::{DecodeBufReadIntoBufRead, DecodeBufReadIntoRead, DecodeReadIntoBufRead, DecodeReadIntoRead};
use std::io::{BufRead, BufReader, Error, Read};
use std::num::NonZeroUsize;

#[cfg_attr(feature = "bitcode", derive(::bitcode::Encode, ::bitcode::Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bgzf {
    out_bufsize: NonZeroUsize,
}

impl Bgzf {
    pub const DEFAULT: Bgzf = Bgzf {
        out_bufsize: NonZeroUsize::new(8 * 1024).unwrap(),
    };

    pub fn new(out_bufsize: NonZeroUsize) -> Self {
        Self { out_bufsize }
    }

    pub fn out_bufsize(&self) -> NonZeroUsize {
        self.out_bufsize
    }
}

impl Default for Bgzf {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl<'a, R: Read + 'a, A: Adapter> DecodeReadIntoRead<'a, R, A> for Bgzf
where
    A: AdaptRead<'a, flate2::read::MultiGzDecoder<R>>,
{
    fn decode_read_into_read(&self, reader: R, adapter: A) -> Result<A::Read<'a>, Error> {
        let reader = flate2::read::MultiGzDecoder::new(reader);
        adapter.wrap(reader)
    }
}

impl<'a, R: Read + 'a, A: Adapter> DecodeReadIntoBufRead<'a, R, A> for Bgzf
where
    A: AdaptBufRead<'a, BufReader<flate2::read::MultiGzDecoder<R>>>,
{
    fn decode_read_into_bufread(&self, reader: R, adapter: A) -> Result<A::BufRead<'a>, Error> {
        let reader = flate2::read::MultiGzDecoder::new(reader);
        let bufreader = BufReader::with_capacity(self.out_bufsize.get(), reader);
        adapter.wrap(bufreader)
    }
}

impl<'a, R: BufRead + 'a, A: Adapter> DecodeBufReadIntoRead<'a, R, A> for Bgzf
where
    A: AdaptRead<'a, flate2::bufread::MultiGzDecoder<R>>,
{
    fn decode_bufread_into_read(&self, reader: R, adapter: A) -> Result<A::Read<'a>, Error> {
        let reader = flate2::bufread::MultiGzDecoder::new(reader);
        adapter.wrap(reader)
    }
}

impl<'a, R: BufRead + 'a, A: Adapter> DecodeBufReadIntoBufRead<'a, R, A> for Bgzf
where
    A: AdaptBufRead<'a, BufReader<flate2::bufread::MultiGzDecoder<R>>>,
{
    fn decode_bufread_into_bufread(&self, reader: R, adapter: A) -> Result<A::BufRead<'a>, Error> {
        let reader = flate2::bufread::MultiGzDecoder::new(reader);
        let bufreader = BufReader::with_capacity(self.out_bufsize.get(), reader);
        adapter.wrap(bufreader)
    }
}
