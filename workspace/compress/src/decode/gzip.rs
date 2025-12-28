use super::adapter::{AdaptBufRead, AdaptRead, Adapter};
use super::decode::{
    DecodeBufReadIntoBufRead, DecodeBufReadIntoRead, DecodeReadIntoBufRead, DecodeReadIntoRead,
};
use std::io::{BufRead, BufReader, Error, Read};
use std::num::NonZeroUsize;

#[cfg_attr(feature = "bitcode", derive(::bitcode::Encode, ::bitcode::Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Gzip {
    out_bufsize: NonZeroUsize,
    read_multi: bool,
}

impl Gzip {
    pub const DEFAULT: Gzip = Gzip {
        out_bufsize: NonZeroUsize::new(8 * 1024).unwrap(),
        read_multi: true,
    };

    pub fn new(out_bufsize: NonZeroUsize, read_multi: bool) -> Self {
        Self {
            out_bufsize,
            read_multi,
        }
    }

    pub fn out_bufsize(&self) -> NonZeroUsize {
        self.out_bufsize
    }

    pub fn read_multi(&self) -> bool {
        self.read_multi
    }
}

impl Default for Gzip {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl<'a, R: Read + 'a, A: Adapter> DecodeReadIntoRead<'a, R, A> for Gzip
where
    A: AdaptRead<'a, flate2::read::GzDecoder<R>>,
    A: AdaptRead<'a, flate2::read::MultiGzDecoder<R>>,
{
    fn decode_read_into_read(&self, reader: R, adapter: A) -> Result<A::Read<'a>, Error> {
        if self.read_multi {
            let reader = flate2::read::MultiGzDecoder::new(reader);
            adapter.wrap(reader)
        } else {
            let reader = flate2::read::GzDecoder::new(reader);
            adapter.wrap(reader)
        }
    }
}

impl<'a, R: Read + 'a, A: Adapter> DecodeReadIntoBufRead<'a, R, A> for Gzip
where
    A: AdaptBufRead<'a, BufReader<flate2::read::GzDecoder<R>>>,
    A: AdaptBufRead<'a, BufReader<flate2::read::MultiGzDecoder<R>>>,
{
    fn decode_read_into_bufread(&self, reader: R, adapter: A) -> Result<A::BufRead<'a>, Error> {
        if self.read_multi {
            let reader = flate2::read::MultiGzDecoder::new(reader);
            let bufreader = BufReader::with_capacity(self.out_bufsize.get(), reader);
            adapter.wrap(bufreader)
        } else {
            let reader = flate2::read::GzDecoder::new(reader);
            let bufreader = BufReader::with_capacity(self.out_bufsize.get(), reader);
            adapter.wrap(bufreader)
        }
    }
}

impl<'a, R: BufRead + 'a, A: Adapter> DecodeBufReadIntoRead<'a, R, A> for Gzip
where
    A: AdaptRead<'a, flate2::bufread::GzDecoder<R>>,
    A: AdaptRead<'a, flate2::bufread::MultiGzDecoder<R>>,
{
    fn decode_bufread_into_read(&self, reader: R, adapter: A) -> Result<A::Read<'a>, Error> {
        if self.read_multi {
            let reader = flate2::bufread::MultiGzDecoder::new(reader);
            adapter.wrap(reader)
        } else {
            let reader = flate2::bufread::GzDecoder::new(reader);
            adapter.wrap(reader)
        }
    }
}

impl<'a, R: BufRead + 'a, A: Adapter> DecodeBufReadIntoBufRead<'a, R, A> for Gzip
where
    A: AdaptBufRead<'a, BufReader<flate2::bufread::GzDecoder<R>>>,
    A: AdaptBufRead<'a, BufReader<flate2::bufread::MultiGzDecoder<R>>>,
{
    fn decode_bufread_into_bufread(&self, reader: R, adapter: A) -> Result<A::BufRead<'a>, Error> {
        if self.read_multi {
            let reader = flate2::bufread::MultiGzDecoder::new(reader);
            let bufreader = BufReader::with_capacity(self.out_bufsize.get(), reader);
            adapter.wrap(bufreader)
        } else {
            let reader = flate2::bufread::GzDecoder::new(reader);
            let bufreader = BufReader::with_capacity(self.out_bufsize.get(), reader);
            adapter.wrap(bufreader)
        }
    }
}
