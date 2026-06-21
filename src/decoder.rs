//! HEIF decoder, mirroring the `image` crate's per-format decoder convention.
//!
//! [`HeifDecoder`] is generic over a [`Read`] source and implements [`ImageDecoder`],
//! so it slots into `DynamicImage::from_decoder` exactly like the codecs that ship
//! with the `image` crate (e.g. `JpegDecoder`, `PngDecoder`). Decoding uses libde265
//! (HEVC) under the hood.

use std::io::Read;
use std::ptr;

use image::error::{DecodingError, ImageFormatHint};
use image::{ColorType, ImageDecoder, ImageError, ImageResult};

use crate::error::HeifError;
use crate::ffi;
use crate::info::BitDepth;
use crate::sys;

/// Tunable parameters for the HEVC decoder.
#[derive(Default)]
pub struct DecoderConfig {
    /// Worker threads; `None` = auto-detect.
    pub threads: Option<u32>,
}

/// HEIF decoder reading from `R`, using libde265.
///
/// The container header is parsed eagerly in [`new`](HeifDecoder::new) so that
/// [`dimensions`](ImageDecoder::dimensions), [`color_type`](ImageDecoder::color_type),
/// and [`bit_depth`](HeifDecoder::bit_depth) are available before the frame is decoded.
///
/// # Example
/// ```no_run
/// use heif::HeifDecoder;
/// use image::DynamicImage;
/// use std::io::Cursor;
///
/// # let bytes: Vec<u8> = Vec::new();
/// let decoder = HeifDecoder::new(Cursor::new(&bytes))?;
/// let img = DynamicImage::from_decoder(decoder)?;
/// # Ok::<(), image::ImageError>(())
/// ```
pub struct HeifDecoder<R: Read> {
    /// Raw libheif context; freed in `Drop`.
    context: *mut sys::heif_context,
    /// Primary image handle parsed from the container; released in `Drop`.
    handle: *mut sys::heif_image_handle,
    /// Owned compressed bytes. libheif's memory IO references this buffer without
    /// copying, so it must outlive `context`. Never moved out.
    _data: Vec<u8>,
    config: DecoderConfig,
    width: u32,
    height: u32,
    depth: u32,
    alpha_present: bool,
    /// Marker to keep the `R` type parameter; the reader is fully drained in `new`.
    _reader: std::marker::PhantomData<R>,
}

impl<R: Read> HeifDecoder<R> {
    /// Create a decoder from `r`, reading the container header eagerly so that
    /// [`dimensions`](ImageDecoder::dimensions) and [`color_type`](ImageDecoder::color_type)
    /// are available before the frame is decoded.
    pub fn new(mut r: R) -> ImageResult<Self> {
        let mut data = Vec::new();
        r.read_to_end(&mut data).map_err(ImageError::IoError)?;

        ffi::init();

        // SAFETY: pointers are checked; the context/handle are freed on every error path
        // and in `Drop`. `data` outlives `context` (stored alongside it below).
        unsafe {
            let context = sys::heif_context_alloc();
            if context.is_null() {
                return Err(to_image_error(HeifError::DecoderInit(
                    "heif_context_alloc returned null".into(),
                )));
            }

            if let Err(m) = ffi::check(sys::heif_context_read_from_memory_without_copy(
                context,
                data.as_ptr() as *const std::ffi::c_void,
                data.len(),
                ptr::null(),
            )) {
                sys::heif_context_free(context);
                return Err(to_image_error(HeifError::Decode(m)));
            }

            let mut handle: *mut sys::heif_image_handle = ptr::null_mut();
            if let Err(m) = ffi::check(sys::heif_context_get_primary_image_handle(context, &mut handle)) {
                sys::heif_context_free(context);
                return Err(to_image_error(HeifError::Decode(m)));
            }

            let width = sys::heif_image_handle_get_width(handle) as u32;
            let height = sys::heif_image_handle_get_height(handle) as u32;
            let depth = sys::heif_image_handle_get_luma_bits_per_pixel(handle).max(0) as u32;
            let alpha_present = sys::heif_image_handle_has_alpha_channel(handle) != 0;

            Ok(Self {
                context,
                handle,
                width,
                height,
                depth,
                alpha_present,
                _data: data,
                config: DecoderConfig::default(),
                _reader: std::marker::PhantomData,
            })
        }
    }

    /// Set the number of decode worker threads (applied when the frame is decoded).
    pub fn with_threads(mut self, threads: u32) -> Self {
        self.config.threads = Some(threads);
        self
    }

    /// Bit depth of the image — extra information that [`ColorType`] cannot express
    /// (it only distinguishes 8- vs 16-bit).
    pub fn bit_depth(&self) -> BitDepth {
        match self.depth {
            12 => BitDepth::Twelve,
            10 => BitDepth::Ten,
            _ => BitDepth::Eight,
        }
    }

    /// Channels in the decoded interleaved output (3 without alpha, 4 with).
    fn channels(&self) -> usize {
        if self.alpha_present { 4 } else { 3 }
    }

    /// Bytes per output sample (1 for 8-bit, 2 for >8-bit).
    fn sample_bytes(&self) -> usize {
        if self.depth > 8 { 2 } else { 1 }
    }

    /// libheif interleaved chroma format matching this image's depth and alpha.
    fn decode_chroma(&self) -> sys::heif_chroma {
        match (self.depth > 8, self.alpha_present) {
            (false, false) => sys::heif_chroma_heif_chroma_interleaved_RGB,
            (false, true) => sys::heif_chroma_heif_chroma_interleaved_RGBA,
            (true, false) => sys::heif_chroma_heif_chroma_interleaved_RRGGBB_LE,
            (true, true) => sys::heif_chroma_heif_chroma_interleaved_RRGGBBAA_LE,
        }
    }
}

impl<R: Read> ImageDecoder for HeifDecoder<R> {
    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn color_type(&self) -> ColorType {
        match (self.depth > 8, self.alpha_present) {
            (false, false) => ColorType::Rgb8,
            (false, true) => ColorType::Rgba8,
            (true, false) => ColorType::Rgb16,
            (true, true) => ColorType::Rgba16,
        }
    }

    fn read_image(self, buf: &mut [u8]) -> ImageResult<()> {
        let expected = self.width as usize * self.height as usize * self.channels() * self.sample_bytes();
        if buf.len() != expected {
            return Err(to_image_error(HeifError::Decode(format!(
                "output buffer length {} does not match expected {expected}",
                buf.len()
            ))));
        }

        // SAFETY: `self.context`/`self.handle` are valid handles created and parsed in `new`.
        unsafe {
            if let Some(threads) = self.config.threads {
                sys::heif_context_set_max_decoding_threads(self.context, threads as i32);
            }

            let mut image: *mut sys::heif_image = ptr::null_mut();
            ffi::check(sys::heif_decode_image(
                self.handle,
                &mut image,
                sys::heif_colorspace_heif_colorspace_RGB,
                self.decode_chroma(),
                ptr::null(),
            ))
            .map_err(|m| to_image_error(HeifError::Decode(m)))?;

            let result = self.copy_pixels(image, buf);
            sys::heif_image_release(image);
            result
        }
    }

    fn read_image_boxed(self: Box<Self>, buf: &mut [u8]) -> ImageResult<()> {
        self.read_image(buf)
    }
}

impl<R: Read> HeifDecoder<R> {
    /// Copies the decoded interleaved plane into `buf` row by row (honoring libheif's
    /// stride), scaling >8-bit samples up to `image`'s full-range 16-bit layout.
    ///
    /// # Safety
    /// `image` must be a valid decoded `heif_image` owned by the caller.
    unsafe fn copy_pixels(&self, image: *mut sys::heif_image, buf: &mut [u8]) -> ImageResult<()> {
        let channels = self.channels();
        let sample_bytes = self.sample_bytes();
        let out_row_bytes = self.width as usize * channels * sample_bytes;
        // Decoded samples occupy `depth` bits; shift up to fill 16 bits for the `image` crate.
        let up_shift = if self.depth > 8 { 16 - self.depth } else { 0 };

        // SAFETY: `image` is valid per contract; the plane spans `width`×`height`.
        unsafe {
            let mut stride: i32 = 0;
            let plane =
                sys::heif_image_get_plane_readonly(image, sys::heif_channel_heif_channel_interleaved, &mut stride);
            if plane.is_null() {
                return Err(to_image_error(HeifError::Decode(
                    "heif_image_get_plane_readonly returned null".into(),
                )));
            }
            let stride = stride as usize;

            for y in 0..self.height as usize {
                let src_row = plane.add(y * stride);
                let dst_row = y * out_row_bytes;

                if sample_bytes == 1 {
                    // 8-bit: a straight row copy.
                    ptr::copy_nonoverlapping(src_row, buf[dst_row..].as_mut_ptr(), out_row_bytes);
                } else {
                    // >8-bit: read native-endian samples, scale up to 16-bit, write LE.
                    for i in 0..(self.width as usize * channels) {
                        let s = src_row.add(i * 2);
                        let value = u16::from_ne_bytes([*s, *s.add(1)]) as u32;
                        let scaled = (value << up_shift) as u16;
                        let bytes = scaled.to_le_bytes();
                        let off = dst_row + i * 2;
                        buf[off] = bytes[0];
                        buf[off + 1] = bytes[1];
                    }
                }
            }
        }

        Ok(())
    }
}

impl<R: Read> Drop for HeifDecoder<R> {
    fn drop(&mut self) {
        // SAFETY: `handle`/`context` were created in `new` and are not freed elsewhere.
        unsafe {
            if !self.handle.is_null() {
                sys::heif_image_handle_release(self.handle);
            }
            if !self.context.is_null() {
                sys::heif_context_free(self.context);
            }
        }
    }
}

/// Wraps a [`HeifError`] as an `image` decoding error.
fn to_image_error(err: HeifError) -> ImageError {
    ImageError::Decoding(DecodingError::new(ImageFormatHint::Name("HEIF".into()), err))
}
