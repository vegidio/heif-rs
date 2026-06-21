//! Encode an image to HEIC and decode it straight back.
//!
//! Demonstrates the full cycle in one program and confirms the decoded image keeps the original dimensions.
//!
//! Run with:
//!
//! ```text
//! cargo run --example roundtrip
//! ```

use std::error::Error;

use image::GenericImageView;

const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.jpg");

fn main() -> Result<(), Box<dyn Error>> {
    let original = image::open(SOURCE)?;
    let (w, h) = original.dimensions();

    let bytes = heif::encode(&original)?;
    let decoded = heif::decode(&bytes)?;

    assert_eq!(decoded.dimensions(), (w, h), "round-trip changed the dimensions");

    println!(
        "round-trip OK: {}x{} -> {} HEIC bytes -> {}x{}",
        w,
        h,
        bytes.len(),
        decoded.width(),
        decoded.height()
    );
    Ok(())
}
