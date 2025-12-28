use super::adapter::Adapter;
use super::decode::{
    DecodeBufReadIntoBufRead, DecodeBufReadIntoRead, DecodeReadIntoBufRead, DecodeReadIntoRead,
};
use std::io::{BufRead, Error, Read};

#[cfg_attr(feature = "bitcode", derive(::bitcode::Encode, ::bitcode::Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Decoder {
    /// No compression (identity) decoding algorithm.
    ///
    /// The option is useful to uniformly handle both compressed and uncompressed data streams.
    Identity(crate::decode::Identity),

    /// DEFLATE compression algorithm
    #[cfg(feature = "decode-deflate")]
    Deflate(crate::decode::Deflate),

    /// GZIP container; Only supports DEFLATE compression.
    #[cfg(feature = "decode-gzip")]
    Gzip(crate::decode::Gzip),

    /// BGZF container; Only supports DEFLATE compression.
    #[cfg(feature = "decode-bgzf")]
    Bgzf(crate::decode::Bgzf),
}

impl Default for Decoder {
    fn default() -> Self {
        Decoder::Identity(Default::default())
    }
}

macro_rules! impl_decompression_trait {
    ($read:ident, $out:ident, $trait:ident, $method:ident) => {
        impl<'a, R: $read + 'a, A: Adapter> $trait<'a, R, A> for Decoder
        where
            crate::decode::Identity: $trait<'a, R, A>,
            crate::decode::Deflate: $trait<'a, R, A>,
            crate::decode::Gzip: $trait<'a, R, A>,
            crate::decode::Bgzf: $trait<'a, R, A>,
        {
            fn $method(&self, reader: R, adapter: A) -> Result<A::$out<'a>, Error> {
                match self {
                    Decoder::Identity(codec) => codec.$method(reader, adapter),
                    #[cfg(feature = "decode-deflate")]
                    Decoder::Deflate(codec) => codec.$method(reader, adapter),
                    #[cfg(feature = "decode-gzip")]
                    Decoder::Gzip(codec) => codec.$method(reader, adapter),
                    #[cfg(feature = "decode-bgzf")]
                    Decoder::Bgzf(codec) => codec.$method(reader, adapter),
                }
            }
        }
    };
}

impl_decompression_trait!(Read, Read, DecodeReadIntoRead, decode_read_into_read);
impl_decompression_trait!(
    Read,
    BufRead,
    DecodeReadIntoBufRead,
    decode_read_into_bufread
);
impl_decompression_trait!(
    BufRead,
    Read,
    DecodeBufReadIntoRead,
    decode_bufread_into_read
);
impl_decompression_trait!(
    BufRead,
    BufRead,
    DecodeBufReadIntoBufRead,
    decode_bufread_into_bufread
);
