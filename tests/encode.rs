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
