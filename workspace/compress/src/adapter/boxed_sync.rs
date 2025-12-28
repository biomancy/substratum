use crate::{decode, encode};
use std::io::{BufRead, Error, Read};

/// Adapter that type-erases I/O types into boxed trait objects with `Send + Sync` bounds.
///
/// `BoxedSync` is used by the compression/decompression adapters to wrap concrete
/// `Read`, `BufRead`, and `Write` implementations into `Box<dyn ... + Send + Sync + 'a>`
/// so they can be passed around as thread-safe, type-erased handles in the public API
/// and examples.
pub struct BoxedSync;

impl<'a, In> decode::AdaptRead<'a, In> for BoxedSync
where
    In: Read + Send + Sync + 'a,
{
    fn wrap(self, internal: In) -> Result<Box<dyn Read + Send + Sync + 'a>, Error> {
        Ok(Box::new(internal))
    }
}

impl<'a, In> decode::AdaptBufRead<'a, In> for BoxedSync
where
    In: BufRead + Send + Sync + 'a,
{
    fn wrap(self, internal: In) -> Result<Box<dyn BufRead + Send + Sync + 'a>, Error> {
        Ok(Box::new(internal))
    }
}

impl decode::Adapter for BoxedSync {
    type Read<'a> = Box<dyn Read + Send + Sync + 'a>;
    type BufRead<'a> = Box<dyn BufRead + Send + Sync + 'a>;
}

impl<'a, In> encode::AdaptWrite<'a, In> for BoxedSync
where
    In: std::io::Write + Send + Sync + 'a,
{
    fn wrap(self, internal: In) -> Result<Box<dyn std::io::Write + Send + Sync + 'a>, Error> {
        Ok(Box::new(internal))
    }
}

impl encode::Adapter for BoxedSync {
    type Write<'a> = Box<dyn std::io::Write + Send + Sync + 'a>;
}
