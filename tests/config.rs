//! Encoder configuration tests: non-default builder settings must still produce a
//! valid HEIC that decodes back to the original dimensions.

mod common;

use common::{is_heif, source_image};

use heif::{Chroma, HeifEncoder, Preset};
use image::{DynamicImage, GenericImageView};

#[test]
fn encodes_with_custom_config() {
    let img = source_image();
    let (w, h) = (img.width(), img.height());

    // Custom quality + preset.
    let mut buf = Vec::new();
    img.write_with_encoder(
        HeifEncoder::new(&mut buf)
            .with_quality(20)
            .with_preset(Preset::Ultrafast),
    )
    .expect("encode with custom quality/preset");
    assert!(
        is_heif(&buf),
        "custom quality/preset output should be a valid HEIC stream"
    );
    assert_eq!(heif::decode(&buf).expect("decode").dimensions(), (w, h));

    // 4:4:4 chroma.
    let mut buf = Vec::new();
    img.write_with_encoder(
        HeifEncoder::new(&mut buf)
            .with_chroma(Chroma::Yuv444)
            .with_preset(Preset::Ultrafast),
    )
    .expect("encode 4:4:4");
    assert!(is_heif(&buf), "4:4:4 output should be a valid HEIC stream");

    // RGBA (exercises the separate alpha-plane encoder).
    let rgba = DynamicImage::ImageRgba8(source_image().to_rgba8());
    let mut buf = Vec::new();
    rgba.write_with_encoder(HeifEncoder::new(&mut buf).with_preset(Preset::Ultrafast))
        .expect("encode rgba");
    assert!(is_heif(&buf), "rgba output should be a valid HEIC stream");
    assert_eq!(heif::decode(&buf).expect("decode rgba").dimensions(), (w, h));
}
