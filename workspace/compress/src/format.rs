use crate::{decode, encode};
use std::path::Path;

const DEFLATE: &[&str] = &[];
const GZIP: &[&str] = &["gz", "gzip"];
const BGZF: &[&str] = &["bgz", "bgzf", "bgzip"];


#[cfg_attr(feature = "bitcode", derive(::bitcode::Encode, ::bitcode::Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Format {
    Uncompressed,
    #[cfg(any(feature = "decode-deflate", feature = "encode-deflate"))]
    Deflate,
    #[cfg(any(feature = "decode-gzip", feature = "encode-gzip"))]
    Gzip,
    #[cfg(any(feature = "decode-bgzf", feature = "encode-bgzf"))]
    Bgzf,
}

impl Format {
    pub fn from_extension(ext: &str, uncompressed_exts: &[&str]) -> Option<Self> {
        match ext {
            // Empty extension can't be used to infer format
            ext if ext.is_empty() => None,

            // Uncompressed files
            ext if uncompressed_exts.contains(&ext) => Some(Self::Uncompressed),

            // DEFLATE compressed files
            #[cfg(any(feature = "decode-deflate", feature = "encode-deflate"))]
            ext if DEFLATE.contains(&ext) => Some(Self::Deflate),

            // GZIP compressed files
            #[cfg(any(feature = "decode-gzip", feature = "encode-gzip"))]
            ext if GZIP.contains(&ext) => Some(Self::Gzip),

            // BGZF compressed files
            #[cfg(any(feature = "decode-bgzf", feature = "encode-bgzf"))]
            ext if BGZF.contains(&ext) => Some(Self::Bgzf),

            _ => None
        }
    }

    pub fn from_path(path: impl AsRef<Path>, uncompressed_exts: &[&str]) -> Option<Self> {
        let path = path.as_ref().extension()?.to_str()?;
        Self::from_extension(path, uncompressed_exts)
    }

    pub fn encoder(&self) -> encode::Encoder {
        match self {
            Self::Uncompressed => encode::Encoder::Identity(Default::default()),
            #[cfg(feature = "encode-deflate")]
            Self::Deflate => encode::Encoder::Deflate(Default::default()),
            #[cfg(feature = "encode-gzip")]
            Self::Gzip => encode::Encoder::Gzip(Default::default()),
            #[cfg(feature = "encode-bgzf")]
            Self::Bgzf => encode::Encoder::Bgzf(Default::default()),
        }
    }

    pub fn decoder(&self) -> decode::Decoder {
        match self {
            Self::Uncompressed => decode::Decoder::Identity(Default::default()),
            #[cfg(feature = "decode-deflate")]
            Self::Deflate => decode::Decoder::Deflate(Default::default()),
            #[cfg(feature = "decode-gzip")]
            Self::Gzip => decode::Decoder::Gzip(Default::default()),
            #[cfg(feature = "decode-bgzf")]
            Self::Bgzf => decode::Decoder::Bgzf(Default::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_format_from_extension() {
        let uncompressed_exts = &["txt", "csv"];
        
        // Uncompressed extensions
        assert_eq!(Format::from_extension("txt", uncompressed_exts), Some(Format::Uncompressed));
        assert_eq!(Format::from_extension(".txt", uncompressed_exts), None);
        
        // Known compressed extensions
        #[cfg(feature = "decode-gzip")]
        assert_eq!(Format::from_extension("gz", uncompressed_exts), Some(Format::Gzip));

        #[cfg(feature = "decode-bgzf")]
        assert_eq!(Format::from_extension("bgzf", uncompressed_exts), Some(Format::Bgzf));
        
        // Unknown or empty extensions
        assert_eq!(Format::from_extension("unknown", uncompressed_exts), None);
        assert_eq!(Format::from_extension("", uncompressed_exts), None);
    }
}