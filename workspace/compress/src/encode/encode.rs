use super::adapter::Adapter;
use std::io::{Error, Write};

pub trait Encode<'a, W: Write + 'a, A: Adapter> {
    fn encode(&self, writer: W, adapter: A) -> Result<A::Write<'a>, Error>;
}
