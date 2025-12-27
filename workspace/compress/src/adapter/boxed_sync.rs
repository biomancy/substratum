use crate::{decode, encode};
use std::io::{BufRead, Error, Read};

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