//! Encode with custom settings via the [`heif::HeifEncoder`] builder.
//!
//! Instead of the one-line [`heif::encode`] facade, use `HeifEncoder` directly (through `image`'s `ImageEncoder` trait)
//! to tune quality, preset, and chroma. Builder methods: `with_quality` (0–100, higher = better), `with_lossless`,
//! `with_preset` (x265 speed preset), `with_chroma` (4:2:0 / 4:2:2 / 4:4:4), `with_bit_depth`.
//!
//! Run with:
//!
//! ```text
//! cargo run --example custom_encoder
//! ```

use std::error::Error;

use heif::{Chroma, HeifEncoder, Preset};

const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.jpg");

fn main() -> Result<(), Box<dyn Error>> {
    let img = image::open(SOURCE)?;

    let mut bytes = Vec::new();
    img.write_with_encoder(
        HeifEncoder::new(&mut bytes)
            .with_quality(80) // 0–100, higher = better quality
            .with_preset(Preset::Fast) // faster = quicker encode, less compression
            .with_chroma(Chroma::Yuv444), // no chroma subsampling
    )?;

    let out = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/custom-encoder-example.heic");
    std::fs::write(out, &bytes)?;

    println!(
        "encoded {} bytes (quality 80, preset fast, 4:4:4) -> {}",
        bytes.len(),
        out
    );
    Ok(())
}
