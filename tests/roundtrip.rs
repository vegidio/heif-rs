//! Encode→decode round-trip tests across pixel layouts.

mod common;

use common::source_image;
use image::{DynamicImage, GenericImageView};

#[test]
fn roundtrip_preserves_dimensions() {
    let img = source_image();
    let (w, h) = (img.width(), img.height());

    let bytes = heif::encode(&img).expect("encode");
    let decoded = heif::decode(&bytes).expect("decode");

    assert_eq!(decoded.dimensions(), (w, h));
}

/// Encode an image with explicit alpha so the RGBA path is exercised.
#[test]
fn roundtrip_rgba() {
    let img = DynamicImage::ImageRgba8(source_image().to_rgba8());
    let bytes = heif::encode(&img).expect("encode rgba");
    let decoded = heif::decode(&bytes).expect("decode rgba");
    assert_eq!(decoded.dimensions(), img.dimensions());
}

/// Grayscale input is expanded to RGB before encoding.
#[test]
fn roundtrip_grayscale() {
    let img = DynamicImage::ImageLuma8(source_image().to_luma8());
    let bytes = heif::encode(&img).expect("encode grayscale");
    let decoded = heif::decode(&bytes).expect("decode grayscale");
    assert_eq!(decoded.dimensions(), img.dimensions());
}

/// 16-bit input must be accepted and down-scaled to the bundled 8-bit HEVC build (exercises the
/// `Rgb16` layout and the sample down-shift path); dimensions must survive the round-trip.
#[test]
fn roundtrip_rgb16_input() {
    let img = DynamicImage::ImageRgb16(source_image().to_rgb16());
    let bytes = heif::encode(&img).expect("encode rgb16");
    let decoded = heif::decode(&bytes).expect("decode rgb16");
    assert_eq!(decoded.dimensions(), img.dimensions());
}
