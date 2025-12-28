use super::adapter::Adapter;
use super::encode::Encode;
use std::io::{Error, Write};
use std::path::Path;

#[cfg_attr(feature = "bitcode", derive(::bitcode::Encode, ::bitcode::Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Encoder {
    /// No compression (identity) encoding algorithm.
    ///
    /// The option is useful to uniformly handle both compressed and uncompressed data streams.
    Identity(crate::encode::Identity),

    /// DEFLATE compression algorithm
    #[cfg(feature = "encode-deflate")]
    Deflate(crate::encode::Deflate),

    /// GZIP container; Only supports DEFLATE compression.
    #[cfg(feature = "encode-gzip")]
    Gzip(crate::encode::Gzip),

    /// BGZF container; Only supports DEFLATE compression.
    #[cfg(feature = "encode-bgzf")]
    Bgzf(crate::encode::Bgzf),
}

impl Default for Encoder {
    fn default() -> Self {
        Encoder::Identity(Default::default())
    }
}

impl<'a, W: Write + 'a, A: Adapter> Encode<'a, W, A> for Encoder
where
    crate::encode::Identity: Encode<'a, W, A>,
    crate::encode::Deflate: Encode<'a, W, A>,
    crate::encode::Gzip: Encode<'a, W, A>,
    crate::encode::Bgzf: Encode<'a, W, A>,
{
    fn encode(&self, writer: W, adapter: A) -> Result<A::Write<'a>, Error> {
        match self {
            Encoder::Identity(codec) => codec.encode(writer, adapter),
            #[cfg(feature = "encode-deflate")]
            Encoder::Deflate(codec) => codec.encode(writer, adapter),
            #[cfg(feature = "encode-gzip")]
            Encoder::Gzip(codec) => codec.encode(writer, adapter),
            #[cfg(feature = "encode-bgzf")]
            Encoder::Bgzf(codec) => codec.encode(writer, adapter),
        }
    }
}

impl Encoder {
    pub fn from_extension(ext: &str, uncompressed_exts: &[&str]) -> std::io::Result<Self> {
        crate::Format::from_extension(ext, uncompressed_exts).map(|x| x.encoder())
    }
    pub fn from_path(path: impl AsRef<Path>, uncompressed_exts: &[&str]) -> std::io::Result<Self> {
        crate::Format::from_path(path, uncompressed_exts).map(|x| x.encoder())
    }
}
