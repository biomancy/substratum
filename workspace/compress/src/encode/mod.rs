mod adapter;
pub use adapter::{AdaptWrite, Adapter};

#[allow(clippy::module_inception)]
mod encode;
pub use encode::Encode;

mod identity;
pub use identity::Identity;

#[cfg(feature = "encode-deflate")]
mod deflate;
#[cfg(feature = "encode-deflate")]
pub use deflate::Deflate;

#[cfg(feature = "encode-gzip")]
mod gzip;
#[cfg(feature = "encode-gzip")]
pub use gzip::Gzip;

#[cfg(feature = "encode-bgzf")]
mod bgzf;
#[cfg(feature = "encode-bgzf")]
pub use bgzf::Bgzf;

mod encoder;
pub use encoder::Encoder;
