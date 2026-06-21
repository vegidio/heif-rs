//! Encode several images concurrently.
//!
//! Each thread runs an independent libheif encode. x265 manages its own internal thread pool per encode, so concurrent
//! encodes are safe — unlike some AV1 encoders, there is no shared per-encode global state to corrupt.
//!
//! Run with:
//!
//! ```text
//! cargo run --example parallel_encode
//! ```

use std::error::Error;
use std::thread;

const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.jpg");

fn main() -> Result<(), Box<dyn Error>> {
    let img = image::open(SOURCE)?;

    // Spawn several encodes at once.
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let img = img.clone();
            thread::spawn(move || (i, heif::encode(&img).expect("encode")))
        })
        .collect();

    for handle in handles {
        let (i, bytes) = handle.join().expect("thread panicked");
        println!("thread {i}: encoded {} bytes", bytes.len());
    }

    Ok(())
}
