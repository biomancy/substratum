use super::adapter::Adapter;
use std::io::{BufRead, Error, Read};

pub trait DecodeReadIntoRead<'a, R: Read + 'a, A: Adapter> {
    fn decode_read_into_read(&self, reader: R, adapter: A) -> Result<A::Read<'a>, Error>;
}

pub trait DecodeReadIntoBufRead<'a, R: Read + 'a, A: Adapter> {
    fn decode_read_into_bufread(&self, reader: R, adapter: A) -> Result<A::BufRead<'a>, Error>;
}

pub trait DecodeBufReadIntoRead<'a, R: BufRead + 'a, A: Adapter> {
    fn decode_bufread_into_read(&self, reader: R, adapter: A) -> Result<A::Read<'a>, Error>;
}

pub trait DecodeBufReadIntoBufRead<'a, R: BufRead + 'a, A: Adapter> {
    fn decode_bufread_into_bufread(&self, reader: R, adapter: A) -> Result<A::BufRead<'a>, Error>;
}
