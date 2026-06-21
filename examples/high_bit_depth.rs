//! Encode at 10-bit depth using [`heif::EncoderConfig`].
//!
//! Bit depth is the one encoder setting without a builder method, so it's set by constructing a
//! [`heif::EncoderConfig`] and passing it to [`heif::HeifEncoder::new_with_config`]. After encoding we probe the output
//! to confirm the stored depth is 10-bit.
//!
//! NOTE: libheif supports 10/12-bit natively, but the prebuilt x265 in the bundled binaries is an 8-bit build, so this
//! example may fail with an encoder error. Decoding 10/12-bit HEIC works regardless.
//!
//! Run with:
//!
//! ```text
//! cargo run --example high_bit_depth
//! ```

use std::error::Error;

use heif::{BitDepth, EncoderConfig, HeifEncoder};

const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.jpg");

fn main() -> Result<(), Box<dyn Error>> {
    let img = image::open(SOURCE)?;

    let config = EncoderConfig {
        bit_depth: BitDepth::Ten,
        ..Default::default()
    };

    let mut bytes = Vec::new();
    match img.write_with_encoder(HeifEncoder::new_with_config(&mut bytes, config)) {
        Ok(()) => {
            // Confirm the encoded stream really is 10-bit.
            let info = heif::probe(&bytes)?;
            println!("encoded {} bytes at {:?}", bytes.len(), info.bit_depth);

            let out = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/high-bit-depth-example.heic");
            std::fs::write(out, &bytes)?;
            println!("saved -> {}", out);
        }
        Err(e) => {
            println!("10-bit encode not supported by the bundled x265 binary: {e}");
        }
    }
    Ok(())
}
