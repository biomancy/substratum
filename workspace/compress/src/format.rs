use crate::{decode, encode};
use std::io::{Error, ErrorKind, Result};
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
    pub fn from_extension(ext: &str, uncompressed_exts: &[&str]) -> Result<Self> {
        match ext {
            // Empty extension can't be used to infer format
            ext if ext.is_empty() => Err(Error::new(ErrorKind::InvalidInput, "Can't infer format from empty extension")),

            // Uncompressed files
            ext if uncompressed_exts.contains(&ext) => Ok(Self::Uncompressed),

            // DEFLATE compressed files
            #[cfg(any(feature = "decode-deflate", feature = "encode-deflate"))]
            ext if DEFLATE.contains(&ext) => Ok(Self::Deflate),

            // GZIP compressed files
            #[cfg(any(feature = "decode-gzip", feature = "encode-gzip"))]
            ext if GZIP.contains(&ext) => Ok(Self::Gzip),

            // BGZF compressed files
            #[cfg(any(feature = "decode-bgzf", feature = "encode-bgzf"))]
            ext if BGZF.contains(&ext) => Ok(Self::Bgzf),

            _ => Err(Error::new(
                ErrorKind::InvalidInput, format!("Unknown format '{}'", ext),
            )),
        }
    }

    pub fn from_path(path: impl AsRef<Path>, uncompressed_exts: &[&str]) -> Result<Self> {
        let path = path.as_ref().extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
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
        assert_eq!(
            Format::from_extension("txt", uncompressed_exts).unwrap(),
            Format::Uncompressed
        );

        // Known compressed extensions
        #[cfg(feature = "decode-gzip")]
        assert_eq!(
            Format::from_extension("gz", uncompressed_exts).unwrap(),
            Format::Gzip
        );

        #[cfg(feature = "decode-bgzf")]
        assert_eq!(
            Format::from_extension("bgzf", uncompressed_exts).unwrap(),
            Format::Bgzf
        );

        // Unknown or empty extensions
        assert!(Format::from_extension("unknown", uncompressed_exts).is_err());
        assert!(Format::from_extension("", uncompressed_exts).is_err());
    }
}
