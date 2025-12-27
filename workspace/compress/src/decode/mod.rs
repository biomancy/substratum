mod adapter;
pub use adapter::{AdaptBufRead, AdaptRead, Adapter};

#[allow(clippy::module_inception)]
mod decode;
pub use decode::{DecodeBufReadIntoBufRead, DecodeBufReadIntoRead, DecodeReadIntoBufRead, DecodeReadIntoRead};

mod identity;
pub use identity::Identity;

#[cfg(feature = "decode-deflate")]
mod deflate;
#[cfg(feature = "decode-deflate")]
pub use deflate::Deflate;

#[cfg(feature = "decode-gzip")]
mod gzip;
#[cfg(feature = "decode-gzip")]
pub use gzip::Gzip;

#[cfg(feature = "decode-bgzf")]
mod bgzf;
#[cfg(feature = "decode-bgzf")]
pub use bgzf::Bgzf;

mod decoder;
pub use decoder::Decoder;
