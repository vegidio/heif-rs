//! Decoder- and probe-focused integration tests against the bundled HEIC asset.

mod common;

use common::HEIF;

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
