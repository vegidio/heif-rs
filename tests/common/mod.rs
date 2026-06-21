//! Shared helpers for the integration tests. Each test file is its own crate, so they
//! pull this in with `mod common;`. Not every file uses every helper.
#![allow(dead_code)]

use image::DynamicImage;

pub const JPG: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.jpg");
pub const HEIF: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.heic");

/// Loads the sample JPEG used as an encode source.
pub fn source_image() -> DynamicImage {
    image::open(JPG).expect("load assets/image.jpg")
}

/// True when `bytes` begins with an ISO-BMFF `ftyp` box advertising a HEIF/HEIC brand.
pub fn is_heif(bytes: &[u8]) -> bool {
    if bytes.len() < 12 || &bytes[4..8] != b"ftyp" {
        return false;
    }

    // The `ftyp` box lists a major brand followed by compatible brands; scan for any
    // HEIF-family brand within the box.
    const BRANDS: [&[u8; 4]; 6] = [b"heic", b"heix", b"heif", b"mif1", b"msf1", b"hevc"];
    let end = bytes.len().min(64);
    bytes[8..end]
        .windows(4)
        .any(|w| BRANDS.iter().any(|b| w == b.as_slice()))
}
