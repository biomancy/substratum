use super::adapter::{AdaptWrite, Adapter};
use super::encode::Encode;
use noodles_bgzf as bgzf;
use std::io;
use std::io::{Error, Write};
use std::num::NonZeroUsize;

use super::deflate::Deflate;


#[cfg_attr(feature = "bitcode", derive(::bitcode::Encode, ::bitcode::Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bgzf {
    deflate: Deflate,
    threads: Option<NonZeroUsize>,
}

impl Bgzf {
    pub const DEFAULT: Bgzf = Bgzf {
        deflate: Deflate::DEFAULT,
        threads: None,
    };

    pub fn new(deflate: Deflate, threads: Option<NonZeroUsize>) -> io::Result<Self> {
        Ok(Self { deflate, threads })
    }

    pub fn deflate(&self) -> &Deflate {
        &self.deflate
    }

    pub fn threads(&self) -> Option<NonZeroUsize> {
        self.threads
    }
}

impl Default for Bgzf {
    fn default() -> Self {
        Bgzf::DEFAULT
    }
}

impl<'a, W: Write + 'a, A: Adapter> Encode<'a, W, A> for Bgzf
where
    A: AdaptWrite<'a, bgzf::io::Writer<W>>,
{
    fn encode(&self, writer: W, adapter: A) -> Result<A::Write<'a>, Error> {
        let level =
            bgzf::io::writer::CompressionLevel::new(self.deflate().level()).unwrap();
        if let Some(_threads) = self.threads() {
            Err(Error::other("Multithreaded BGZF is not supported"))
            // let writer = bgzf::io::multithreaded_writer::Builder::default()
            //     .set_compression_level(level)
            //     .set_worker_count(threads)
            //     .build_from_writer(writer);
            // adapter.wrap(writer)
        } else {
            let writer = bgzf::io::writer::Builder::default()
                .set_compression_level(level)
                .build_from_writer(writer);
            adapter.wrap(writer)
        }
    }
}
