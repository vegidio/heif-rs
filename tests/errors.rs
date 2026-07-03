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
fn encode_rejects_unsupported_color_type() {
    // 32-bit float RGB is not a layout libheif's interleaved plane supports, so it must map to
    // the dedicated `ImageError::Unsupported` variant rather than a generic encode failure.
    let mut out = Vec::new();
    let encoder = HeifEncoder::new(&mut out);
    let buf = [0u8; 12]; // 1x1 Rgb32F = 3 channels * 4 bytes
    let result = encoder.write_image(&buf, 1, 1, ExtendedColorType::Rgb32F);
    assert!(
        matches!(result, Err(image::ImageError::Unsupported(_))),
        "unsupported color type should map to ImageError::Unsupported"
    );
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
    // HEIF keeps its metadata boxes (`ftyp`, `meta`) at the front of the file and the pixel
    // data (`mdat`) at the back, so chopping off the tail leaves a perfectly probable header.
    // Truncate to a short prefix that keeps `ftyp` but cuts into the `meta` box, forcing the
    // container parse to fail.
    let bytes = std::fs::read(HEIF).expect("read assets/image.heic");
    let truncated = &bytes[..64.min(bytes.len())];
    assert!(heif::probe(truncated).is_err(), "truncated HEIC should fail to probe");
}
