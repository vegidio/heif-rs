//! Encoder-focused integration tests.

mod common;

use common::{is_heif, source_image};

#[test]
fn encode_produces_valid_heif() {
    let img = source_image();
    let bytes = heif::encode(&img).expect("encode");
    assert!(!bytes.is_empty(), "encoded output should not be empty");
    assert!(is_heif(&bytes), "output should be a valid HEIC stream");
}

/// The typed `encode_buffer` facade must produce a valid HEIC from a concrete `RgbImage`,
/// bypassing the `DynamicImage` dispatch that `encode` uses.
#[test]
fn encode_buffer_produces_valid_heif() {
    let img = source_image().to_rgb8();
    let bytes = heif::encode_buffer(&img).expect("encode_buffer");
    assert!(!bytes.is_empty(), "encoded output should not be empty");
    assert!(is_heif(&bytes), "encode_buffer output should be a valid HEIC stream");
}
