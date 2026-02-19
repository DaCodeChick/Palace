//! Palace Prop (Avatar Accessory) Support
//!
//! This module provides encoding and decoding for Palace props - small 44x44 pixel
//! images used as avatar accessories and decorations.
//!
//! ## Prop Format
//!
//! Props consist of:
//! - 12-byte fixed header with metadata
//! - Variable-length image data in one of four formats
//!
//! ### Header Structure (12 bytes)
//!
//! ```text
//! Offset  Size  Field           Description
//! ------  ----  -----           -----------
//! 0       2     width           Image width in pixels (typically 44)
//! 2       2     height          Image height in pixels (typically 44)
//! 4       2     h_offset        Horizontal offset for display
//! 6       2     v_offset        Vertical offset for display
//! 8       2     script_offset   Offset to attached script (if any)
//! 10      2     flags           PropFlags bitfield
//! ```
//!
//! ### Image Formats
//!
//! Palace supports four prop image formats:
//!
//! - **8-bit Indexed** (default, 0x0000): Run-length encoded palette-indexed color
//! - **20-bit RGB** (0x0040): Compressed RGB with 1-bit alpha (6+6+6+2 bits/pixel)
//! - **32-bit RGBA** (0x0100): Compressed full RGBA (8+8+8+8 bits/pixel)
//! - **S20-bit** (0x0200): Compressed signed 20-bit (5+5+5+5 bits/pixel)
//!
//! All non-8-bit formats use zlib compression. 8-bit uses custom run-length encoding.

use bytes::{Buf, BufMut};
use std::io::{self, Read, Write};

use crate::messages::flags::{PropFlags, PropFormat};

/// Standard Palace prop dimensions
pub const PROP_WIDTH: usize = 44;
pub const PROP_HEIGHT: usize = 44;
pub const PROP_PIXELS: usize = PROP_WIDTH * PROP_HEIGHT; // 1936

/// RGBA pixel color (alpha, red, green, blue)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Create a new color with ARGB components
    pub const fn new(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { a, r, g, b }
    }

    /// Create a color from a 32-bit ARGB value
    pub const fn from_argb(argb: u32) -> Self {
        Self {
            a: ((argb >> 24) & 0xFF) as u8,
            r: ((argb >> 16) & 0xFF) as u8,
            g: ((argb >> 8) & 0xFF) as u8,
            b: (argb & 0xFF) as u8,
        }
    }

    /// Convert to a 32-bit ARGB value
    pub const fn to_argb(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Fully transparent color
    pub const TRANSPARENT: Color = Color::new(0, 0, 0, 0);
}

/// Palace prop record with metadata and image data
#[derive(Debug, Clone, PartialEq)]
pub struct PropRec {
    /// Image width (typically 44)
    pub width: u16,
    /// Image height (typically 44)
    pub height: u16,
    /// Horizontal display offset
    pub h_offset: i16,
    /// Vertical display offset
    pub v_offset: i16,
    /// Script offset (typically 0)
    pub script_offset: u16,
    /// Prop flags (format, head, ghost, rare, animate, bounce)
    pub flags: PropFlags,
    /// Raw image data (format depends on flags)
    pub image_data: Vec<u8>,
}

impl PropRec {
    /// Create a new prop with the given dimensions and format
    pub fn new(
        width: u16,
        height: u16,
        h_offset: i16,
        v_offset: i16,
        flags: PropFlags,
        image_data: Vec<u8>,
    ) -> Self {
        Self {
            width,
            height,
            h_offset,
            v_offset,
            script_offset: 0,
            flags,
            image_data,
        }
    }

    /// Read prop from bytes (with endianness detection)
    pub fn from_bytes(buf: &mut impl Buf) -> io::Result<Self> {
        if buf.remaining() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes for prop header",
            ));
        }

        // Read first two bytes to detect endianness
        let first_byte = buf.get_u8();
        let second_byte = buf.get_u8();

        let (width, is_little_endian) = if second_byte == 0 {
            // Little endian: second byte is 0
            ((first_byte as u16) | ((second_byte as u16) << 8), true)
        } else {
            // Big endian: first byte is likely smaller
            ((second_byte as u16) | ((first_byte as u16) << 8), false)
        };

        // Read remaining header fields based on endianness
        let (height, h_offset, v_offset, script_offset, flags_raw) = if is_little_endian {
            (
                buf.get_u16_le(),
                buf.get_i16_le(),
                buf.get_i16_le(),
                buf.get_u16_le(),
                buf.get_u16_le(),
            )
        } else {
            (
                buf.get_u16(),
                buf.get_i16(),
                buf.get_i16(),
                buf.get_u16(),
                buf.get_u16(),
            )
        };

        let flags = PropFlags::from_bits_truncate(flags_raw);

        // Read remaining image data
        let mut image_data = vec![0u8; buf.remaining()];
        buf.copy_to_slice(&mut image_data);

        Ok(Self {
            width,
            height,
            h_offset,
            v_offset,
            script_offset,
            flags,
            image_data,
        })
    }

    /// Write prop to bytes (big endian)
    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_u16(self.width);
        buf.put_u16(self.height);
        buf.put_i16(self.h_offset);
        buf.put_i16(self.v_offset);
        buf.put_u16(self.script_offset);
        buf.put_u16(self.flags.bits());
        buf.put_slice(&self.image_data);
    }

    /// Get the prop's image format
    pub fn format(&self) -> PropFormat {
        self.flags.format()
    }

    /// Decode the prop's image data to RGBA pixels
    ///
    /// Returns a vector of RGBA pixels in row-major order.
    pub fn decode(&self) -> io::Result<Vec<Color>> {
        match self.format() {
            PropFormat::Indexed8 => decode_8bit(&self.image_data, self.width, self.height),
            PropFormat::Rgb20 => decode_20bit(&self.image_data, self.width, self.height),
            PropFormat::Rgb32 => decode_32bit(&self.image_data, self.width, self.height),
            PropFormat::S20Bit => decode_s20bit(&self.image_data, self.width, self.height),
        }
    }

    /// Encode RGBA pixels to the prop's format
    ///
    /// The input must be exactly width * height pixels in row-major order.
    pub fn encode(
        pixels: &[Color],
        width: u16,
        height: u16,
        h_offset: i16,
        v_offset: i16,
        flags: PropFlags,
    ) -> io::Result<Self> {
        let expected_len = (width as usize) * (height as usize);
        if pixels.len() != expected_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Expected {} pixels, got {}", expected_len, pixels.len()),
            ));
        }

        let format = flags.format();
        let image_data = match format {
            PropFormat::S20Bit => encode_s20bit(pixels, width, height)?,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("Encoding for {:?} format not implemented", format),
                ))
            }
        };

        Ok(Self::new(
            width, height, h_offset, v_offset, flags, image_data,
        ))
    }
}

/// Decode 8-bit indexed color prop (run-length encoded)
fn decode_8bit(data: &[u8], width: u16, height: u16) -> io::Result<Vec<Color>> {
    let total_pixels = (width as usize) * (height as usize);
    let mut pixels = vec![Color::TRANSPARENT; total_pixels];

    let mut data_idx = 0;
    let mut pixel_idx = width as usize; // Start after first row (Palace quirk)
    let mut counter = 0;

    // Process from bottom to top
    for _y in (0..height).rev() {
        let mut x = width;

        while x > 0 {
            if data_idx >= data.len() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Unexpected end of 8-bit prop data",
                ));
            }

            let cb = data[data_idx] as usize;
            data_idx += 1;

            let skip_count = cb >> 4; // High nibble: transparent pixels to skip
            let pixel_count = cb & 0x0F; // Low nibble: palette pixels to copy

            let total_count = (skip_count + pixel_count) as u16;
            if total_count > x {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "8-bit prop run exceeded row width",
                ));
            }
            x -= total_count;

            // Safety check for infinite loops
            counter += 1;
            if counter > 6000 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "8-bit prop decode runaway (max iterations exceeded)",
                ));
            }

            // Skip transparent pixels
            pixel_idx += skip_count;

            // Copy palette pixels
            for _ in 0..pixel_count {
                if data_idx >= data.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Unexpected end of 8-bit prop pixel data",
                    ));
                }

                let palette_idx = data[data_idx] as usize;
                data_idx += 1;

                if pixel_idx < pixels.len() {
                    pixels[pixel_idx] = palette_lookup(palette_idx);
                    pixel_idx += 1;
                }
            }
        }
    }

    Ok(pixels)
}

/// Decode 20-bit RGB prop (6+6+6+2 bits per pixel, compressed)
fn decode_20bit(compressed_data: &[u8], width: u16, height: u16) -> io::Result<Vec<Color>> {
    // Decompress using zlib
    let mut decoder = flate2::read::ZlibDecoder::new(compressed_data);
    let mut data = Vec::new();
    decoder.read_to_end(&mut data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to decompress 20-bit prop: {}", e),
        )
    })?;

    let total_pixels = (width as usize) * (height as usize);
    let mut pixels = Vec::with_capacity(total_pixels);

    // 20-bit format: 2 pixels per 5 bytes (40 bits)
    // Pixel 1: RRRRRR GGGGGG BBBBBB AA (6+6+6+2 = 20 bits)
    // Pixel 2: RRRRRR GGGGGG BBBBBB AA (6+6+6+2 = 20 bits)
    const DITHER_20BIT: f32 = 255.0 / 63.0; // Scale 6-bit to 8-bit

    let mut pos = 0;
    for _ in 0..(total_pixels / 2) {
        if pos + 5 > data.len() {
            break;
        }

        // Pixel 1
        let r1 = (((data[pos] >> 2) & 63) as f32 * DITHER_20BIT) as u8;
        let c1 = ((data[pos] as u16) << 8) | (data[pos + 1] as u16);
        let g1 = ((((c1 >> 4) & 63) as f32) * DITHER_20BIT) as u8;
        let c2 = ((data[pos + 1] as u16) << 8) | (data[pos + 2] as u16);
        let b1 = ((((c2 >> 6) & 63) as f32) * DITHER_20BIT) as u8;
        let a1 = (((c2 >> 4) & 3) * 85) as u8; // Scale 2-bit to 8-bit

        pixels.push(Color::new(a1, r1, g1, b1));

        // Pixel 2
        let c3 = ((data[pos + 2] as u16) << 8) | (data[pos + 3] as u16);
        let r2 = ((((c3 >> 6) & 63) as f32) * DITHER_20BIT) as u8;
        let g2 = (((c3 & 63) as f32) * DITHER_20BIT) as u8;
        let b2 = (((data[pos + 4] >> 2) & 63) as f32 * DITHER_20BIT) as u8;
        let a2 = (data[pos + 4] & 3) * 85; // Scale 2-bit to 8-bit

        pixels.push(Color::new(a2, r2, g2, b2));

        pos += 5;
    }

    // Pad if needed
    while pixels.len() < total_pixels {
        pixels.push(Color::TRANSPARENT);
    }

    Ok(pixels)
}

/// Decode 32-bit RGBA prop (8+8+8+8 bits per pixel, compressed)
fn decode_32bit(compressed_data: &[u8], width: u16, height: u16) -> io::Result<Vec<Color>> {
    // Decompress using zlib
    let mut decoder = flate2::read::ZlibDecoder::new(compressed_data);
    let mut data = Vec::new();
    decoder.read_to_end(&mut data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to decompress 32-bit prop: {}", e),
        )
    })?;

    let total_pixels = (width as usize) * (height as usize);
    let mut pixels = Vec::with_capacity(total_pixels);

    // 32-bit format: 4 bytes per pixel (RGBA)
    for chunk in data.chunks(4) {
        if chunk.len() == 4 {
            let r = chunk[0];
            let g = chunk[1];
            let b = chunk[2];
            let a = chunk[3];
            pixels.push(Color::new(a, r, g, b));
        }

        if pixels.len() >= total_pixels {
            break;
        }
    }

    // Pad if needed
    while pixels.len() < total_pixels {
        pixels.push(Color::TRANSPARENT);
    }

    Ok(pixels)
}

/// Decode S20-bit prop (5+5+5+5 bits per pixel, compressed)
fn decode_s20bit(compressed_data: &[u8], width: u16, height: u16) -> io::Result<Vec<Color>> {
    // Decompress using zlib
    let mut decoder = flate2::read::ZlibDecoder::new(compressed_data);
    let mut data = Vec::new();
    decoder.read_to_end(&mut data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to decompress S20-bit prop: {}", e),
        )
    })?;

    let total_pixels = (width as usize) * (height as usize);
    let mut pixels = Vec::with_capacity(total_pixels);

    // S20-bit format: 2 pixels per 5 bytes (40 bits)
    // Pixel 1: RRRRR GGGGG BBBBB AAAAA (5+5+5+5 = 20 bits)
    // Pixel 2: RRRRR GGGGG BBBBB AAAAA (5+5+5+5 = 20 bits)
    const DITHER_S20BIT: f32 = 255.0 / 31.0; // Scale 5-bit to 8-bit

    let mut pos = 0;
    for _ in 0..(total_pixels / 2).max(1) {
        if pos + 5 > data.len() {
            break;
        }

        // Pixel 1
        let r1 = (((data[pos] >> 3) & 31) as f32 * DITHER_S20BIT) as u8;
        let c1 = ((data[pos] as u16) << 8) | (data[pos + 1] as u16);
        let g1 = (((c1 >> 6) & 31) as f32 * DITHER_S20BIT) as u8;
        let b1 = (((c1 >> 1) & 31) as f32 * DITHER_S20BIT) as u8;
        let c2 = ((data[pos + 1] as u16) << 8) | (data[pos + 2] as u16);
        let a1 = (((c2 >> 4) & 31) as f32 * DITHER_S20BIT) as u8;

        pixels.push(Color::new(a1, r1, g1, b1));

        // Pixel 2
        let c3 = ((data[pos + 2] as u16) << 8) | (data[pos + 3] as u16);
        let r2 = (((c3 >> 7) & 31) as f32 * DITHER_S20BIT) as u8;
        let g2 = (((c3 >> 2) & 31) as f32 * DITHER_S20BIT) as u8;
        let c4 = ((data[pos + 3] as u16) << 8) | (data[pos + 4] as u16);
        let b2 = (((c4 >> 5) & 31) as f32 * DITHER_S20BIT) as u8;
        let a2 = ((c4 & 31) as f32 * DITHER_S20BIT) as u8;

        pixels.push(Color::new(a2, r2, g2, b2));

        pos += 5;
    }

    // Pad if needed
    while pixels.len() < total_pixels {
        pixels.push(Color::TRANSPARENT);
    }

    Ok(pixels)
}

/// Encode RGBA pixels to S20-bit format (compressed)
fn encode_s20bit(pixels: &[Color], width: u16, height: u16) -> io::Result<Vec<u8>> {
    // S20-bit format: 2 pixels per 5 bytes (40 bits)
    // Each component is 5 bits, scaled from 8-bit
    const SCALE_S20BIT: f32 = 31.0 / 255.0;

    let mut data = Vec::new();

    // Process pixels in pairs
    for y in 0..height {
        for x in (0..width).step_by(2) {
            let idx1 = (y as usize * width as usize) + x as usize;
            let idx2 = idx1 + 1;

            let color1 = pixels.get(idx1).copied().unwrap_or(Color::TRANSPARENT);
            let color2 = pixels.get(idx2).copied().unwrap_or(Color::TRANSPARENT);

            // Scale 8-bit components to 5-bit
            let r1 = ((color1.r as f32 * SCALE_S20BIT).round() as u32) & 31;
            let g1 = ((color1.g as f32 * SCALE_S20BIT).round() as u32) & 31;
            let b1 = ((color1.b as f32 * SCALE_S20BIT).round() as u32) & 31;
            let a1 = ((color1.a as f32 * SCALE_S20BIT).round() as u32) & 31;

            let r2 = ((color2.r as f32 * SCALE_S20BIT).round() as u32) & 31;
            let g2 = ((color2.g as f32 * SCALE_S20BIT).round() as u32) & 31;
            let b2 = ((color2.b as f32 * SCALE_S20BIT).round() as u32) & 31;
            let a2 = ((color2.a as f32 * SCALE_S20BIT).round() as u32) & 31;

            // Pack pixel 1: RRRRR GGGGG BBBBB AAAAA (20 bits)
            let mut int_comp = (r1 << 19) | (g1 << 14) | (b1 << 9) | (a1 << 4);

            data.push(((int_comp >> 16) & 0xFF) as u8);
            data.push(((int_comp >> 8) & 0xFF) as u8);

            // Pack pixel 2 into remaining 4 bits + next 16 bits
            int_comp = ((int_comp & 0xF0) << 16) | (r2 << 15) | (g2 << 10) | (b2 << 5) | a2;

            data.push(((int_comp >> 16) & 0xFF) as u8);
            data.push(((int_comp >> 8) & 0xFF) as u8);
            data.push((int_comp & 0xFF) as u8);
        }
    }

    // Compress using zlib
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(&data)
        .map_err(|e| io::Error::other(format!("Failed to compress S20-bit prop: {}", e)))?;

    encoder
        .finish()
        .map_err(|e| io::Error::other(format!("Failed to finish S20-bit compression: {}", e)))
}

/// Look up a palette color by index
///
/// This is a simplified Palace color palette. A full implementation would
/// use the exact Palace CLUT (Color Look-Up Table).
fn palette_lookup(index: usize) -> Color {
    // TODO: Use actual Palace palette (256 colors)
    // For now, use a simple grayscale mapping
    let gray = (index & 0xFF) as u8;
    Color::new(255, gray, gray, gray)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_argb_conversion() {
        let color = Color::new(255, 128, 64, 32);
        assert_eq!(color.to_argb(), 0xFF804020);

        let color2 = Color::from_argb(0xFF804020);
        assert_eq!(color2, color);
    }

    #[test]
    fn test_prop_header_big_endian() {
        let mut buf = vec![];
        buf.put_u16(44); // width
        buf.put_u16(44); // height
        buf.put_i16(0); // h_offset
        buf.put_i16(0); // v_offset
        buf.put_u16(0); // script_offset
        buf.put_u16(PropFlags::FORMAT_S20BIT.bits()); // flags

        let prop = PropRec::from_bytes(&mut buf.as_slice()).unwrap();
        assert_eq!(prop.width, 44);
        assert_eq!(prop.height, 44);
        assert_eq!(prop.format(), PropFormat::S20Bit);
    }

    #[test]
    fn test_s20bit_encode_decode_roundtrip() {
        // Create a simple test pattern
        let mut pixels = vec![Color::TRANSPARENT; PROP_PIXELS];

        // Set some colored pixels
        pixels[0] = Color::new(255, 255, 0, 0); // Red
        pixels[1] = Color::new(255, 0, 255, 0); // Green
        pixels[2] = Color::new(255, 0, 0, 255); // Blue
        pixels[3] = Color::new(128, 128, 128, 128); // Gray

        // Encode
        let flags = PropFlags::FORMAT_S20BIT;
        let prop = PropRec::encode(&pixels, PROP_WIDTH as u16, PROP_HEIGHT as u16, 0, 0, flags)
            .expect("Failed to encode");

        // Decode
        let decoded = prop.decode().expect("Failed to decode");
        assert_eq!(decoded.len(), PROP_PIXELS);

        // Check colors are approximately correct (some precision loss expected)
        // S20-bit uses 5 bits per channel, so expect rounding
        assert!((decoded[0].r as i16 - 255).abs() <= 8);
        assert!((decoded[1].g as i16 - 255).abs() <= 8);
        assert!((decoded[2].b as i16 - 255).abs() <= 8);
    }
}
