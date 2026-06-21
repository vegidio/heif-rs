//! Encode an image to HEIC with default settings.
//!
//! Opens the bundled JPEG, encodes it to HEIC bytes with [`heif::encode`], and writes the result to a file in the
//! `assets/` directory.
//!
//! Run with:
//!
//! ```text
//! cargo run --example encode
//! ```

use std::error::Error;

const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.jpg");

fn main() -> Result<(), Box<dyn Error>> {
    // Load any format the `image` crate understands.
    let img = image::open(SOURCE)?;

    // Encode to HEIC with sensible defaults (x265, quality 60, preset slow).
    let bytes = heif::encode(&img)?;

    let out = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/encode-example.heic");
    std::fs::write(out, &bytes)?;

    println!("encoded {} bytes -> {}", bytes.len(), out);
    Ok(())
}
