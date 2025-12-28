//! # Compress
//!
//! A decoupled compression facade that separates I/O stream handling from the
//! underlying compression algorithms.
//!
//! ## Quick Start: Roundtrip Example
//!
//! This example demonstrates how to infer a format from a file extension,
//! compress data into a memory buffer, and then decompress it back to verify integrity.
//!
//! ```rust
//! use substratum_compress::{adapter::BoxedSync, decode::DecodeReadIntoRead, encode::Encode, Format};
//! use std::io::{Read, Write};
//!
//! fn main() -> std::io::Result<()> {
//!     let data = b"Hello, Beautiful Compression World!";
//!
//!     // ----------------------------------------------------------------------
//!     // 1. Setup & Inference
//!     // ----------------------------------------------------------------------
//!     // Detect format from extension (e.g., "gz" -> Format::Gzip)
//!     let format = Format::from_extension("gz", &[]).unwrap();
//!     assert_eq!(format, Format::Gzip, "Should infer Gzip format");
//!
//!     // ----------------------------------------------------------------------
//!     // 2. Encode (Write)
//!     // ----------------------------------------------------------------------
//!     // Get a generic encoder facade based on the inferred format
//!     let encoder = format.encoder();
//!
//!     // Encode the data into a memory buffer
//!     let mut buffer = Vec::new();
//!     {
//!         // We use the `BoxedSync` adapter to type-erase the specific encoder.
//!         // This allows `encoder_algo` to return a consistent `Box<dyn Write + Send + Sync>`
//!         // regardless of whether it's Gzip, Deflate, or another format.
//!         let mut writer = encoder.encode(&mut buffer, BoxedSync)?;
//!         writer.write_all(data)?;
//!
//!         // The compression finishes when the writer is dropped or flushed/finished.
//!         // In this scope-based block, `writer` is dropped here.
//!     }
//!
//!     // Verify something was written (Gzip header + body + footer)
//!     assert!(!buffer.is_empty());
//!
//!     // ----------------------------------------------------------------------
//!     // 3. Decode (Read)
//!     // ----------------------------------------------------------------------
//!     // Get a generic decoder facade based on the inferred format
//!     let decoder = format.decoder();
//!
//!     // We explicitly ask for a `Read` interface and supply a `Read` source.
//!     // Again, we use `BoxedSync` to type-erase the specific decoder.
//!     let mut reader = decoder.decode_read_into_read(&buffer[..], BoxedSync)?;
//!     let mut decompressed = Vec::new();
//!     reader.read_to_end(&mut decompressed)?;
//!
//!     // ----------------------------------------------------------------------
//!     // 4. Verify
//!     // ----------------------------------------------------------------------
//!     assert_eq!(decompressed, data);
//!
//!     Ok(())
//! }
//! ```

pub mod adapter;
pub mod decode;
pub mod encode;
mod format;

pub use format::Format;
