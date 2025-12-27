use super::adapter::{AdaptWrite, Adapter};
use super::encode::Encode;
use std::io::{Error, Write};


#[cfg_attr(feature = "bitcode", derive(::bitcode::Encode, ::bitcode::Decode))]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity {}

impl<'a, W: Write + 'a, A: Adapter> Encode<'a, W, A> for Identity
where
    A: AdaptWrite<'a, W>,
{
    fn encode(&self, writer: W, adapter: A) -> Result<A::Write<'a>, Error> {
        adapter.wrap(writer)
    }
}
