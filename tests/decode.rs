//! Decoder- and probe-focused integration tests against the bundled HEIC asset.

mod common;

use common::HEIF;

use std::io::Cursor;

use heif::HeifDecoder;
use image::{DynamicImage, GenericImageView, ImageDecoder};

#[test]
fn probe_reads_header_only() {
    let bytes = std::fs::read(HEIF).expect("read assets/image.heic");
    let info = heif::probe(&bytes).expect("probe");

    assert!(info.width > 0 && info.height > 0);
    // The sample asset is an 8-bit HEIC.
    assert_eq!(info.bit_depth, heif::BitDepth::Eight);
}

#[test]
fn decodes_bundled_heic_asset() {
    let bytes = std::fs::read(HEIF).expect("read assets/image.heic");
    let img = heif::decode(&bytes).expect("decode bundled asset");
    assert!(img.width() > 0 && img.height() > 0);
}

/// The `HeifDecoder` builder exposes explicit decode-thread control; setting it must still yield a
/// correct image (exercises the threaded read path libheif takes when a thread count is supplied).
#[test]
fn decodes_with_explicit_threads() {
    let bytes = std::fs::read(HEIF).expect("read assets/image.heic");
    let decoder = HeifDecoder::new(Cursor::new(bytes))
        .expect("construct decoder")
        .with_threads(2);
    let (w, h) = decoder.dimensions();
    let img = DynamicImage::from_decoder(decoder).expect("decode with explicit threads");
    assert_eq!(img.dimensions(), (w, h));
    assert!(img.width() > 0 && img.height() > 0);
}
