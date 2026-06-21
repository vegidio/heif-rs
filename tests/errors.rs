//! Error-path integration tests: malformed input and buffer mismatches must return
//! `Err` rather than panic.

mod common;

use common::HEIF;

use heif::HeifEncoder;
use image::{ExtendedColorType, ImageEncoder};

#[test]
fn encode_rejects_wrong_length_buffer() {
    // A 1x1 RGBA image needs 4 bytes; hand it 3 so the length check fires.
    let mut out = Vec::new();
    let encoder = HeifEncoder::new(&mut out);
    let result = encoder.write_image(&[0, 0, 0], 1, 1, ExtendedColorType::Rgba8);
    assert!(result.is_err(), "wrong-length buffer should be rejected");
}

#[test]
fn encode_rejects_zero_dimensions() {
    let mut out = Vec::new();
    let encoder = HeifEncoder::new(&mut out);
    let result = encoder.write_image(&[], 0, 0, ExtendedColorType::Rgb8);
    assert!(result.is_err(), "zero dimensions should be rejected");
}

#[test]
fn decode_rejects_empty_input() {
    assert!(heif::decode(&[]).is_err(), "empty input should not decode");
}

#[test]
fn decode_rejects_garbage_input() {
    assert!(
        heif::decode(b"this is definitely not a heic file").is_err(),
        "non-HEIC bytes should not decode"
    );
}

#[test]
fn probe_rejects_truncated_heif() {
    let bytes = std::fs::read(HEIF).expect("read assets/image.heic");
    let truncated = &bytes[..bytes.len() / 4];
    assert!(heif::probe(truncated).is_err(), "truncated HEIC should fail to probe");
}
