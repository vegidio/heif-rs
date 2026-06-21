//! Encode a typed [`image::ImageBuffer`] directly with [`heif::encode_buffer`].
//!
//! Prefer this over [`heif::encode`] when you already know the pixel type at compile time: it skips the runtime
//! dispatch of [`image::DynamicImage`] and works straight off a concrete buffer such as `RgbaImage`.
//!
//! Run with:
//!
//! ```text
//! cargo run --example encode_buffer
//! ```

use std::error::Error;

use image::RgbaImage;

const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.jpg");

fn main() -> Result<(), Box<dyn Error>> {
    // A concrete, compile-time-known pixel type — no DynamicImage in sight.
    let img: RgbaImage = image::open(SOURCE)?.into_rgba8();

    let bytes = heif::encode_buffer(&img)?;

    let out = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/encode-buffer-example.heic");
    std::fs::write(out, &bytes)?;

    println!("encoded {} bytes from an RgbaImage -> {}", bytes.len(), out);
    Ok(())
}
