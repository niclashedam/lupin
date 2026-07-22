// Copyright 2025 Niclas Hedam
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Matroska/WebM (MKV) steganography engine using a Void element
//!
//! # How It Works
//!
//! Matroska (and its subset WebM) files are [EBML](https://www.rfc-editor.org/rfc/rfc8794)
//! documents: a tree of elements, each `[element ID][size][data]`, where both the ID and the
//! size are variable-length integers (VINTs). The top level of the file is an `EBML Header`
//! element followed by one or more `Segment` elements; the Segment holds all of the media
//! (Tracks, Clusters, Cues, ...).
//!
//! This engine hides data in a **`Void` element** (ID `0xEC`) appended as the last child
//! *inside* the Segment. `Void` is the EBML primitive for reserved/padding space: the
//! specification requires every reader to skip it without interpreting its contents, so a
//! `Void` carrying our payload is ignored by every conformant player (VLC, ffmpeg,
//! mkvtoolnix, ...). The image/audio/video streams are untouched and playback is unaffected.
//!
//! ## Why this is safe for every file
//!
//! Two properties make this robust where a naive "append after the file" approach is not:
//!
//! * **Offsets are preserved.** Matroska's `SeekHead` and `Cues` index elements by their
//!   *Segment-relative* position. Because we insert the `Void` *after* all existing children
//!   (at the very end of the Segment), no existing element moves relative to the start of the
//!   Segment data, so every stored position stays correct. (Even when we must widen the
//!   Segment's own size field, all children shift together and their relative positions are
//!   unchanged.)
//! * **Both Segment size forms are handled.** A Segment may declare an explicit size or use
//!   the reserved "unknown size" form (all VINT value bits set), which runs to the next
//!   top-level element or EOF. For a known-size Segment we rewrite the size field to include
//!   the new `Void`; for an unknown-size Segment the appended `Void` naturally falls inside
//!   the still-open Segment and no size needs changing.
//!
//! ## Storage Format
//!
//! The `Void` element we write is:
//!
//! ```text
//! [1 byte: 0xEC][VINT: size][6 bytes: "Lupin\0"][N bytes: Raw Payload]
//! ```
//!
//! The `Lupin\0` signature (matching the JPEG engine) distinguishes our `Void` from ordinary
//! padding `Void` elements written by other muxers, so those are never mistaken for hidden
//! data. The payload is stored raw — EBML element data is binary-safe, so no Base64 is needed.

use crate::error::{LupinError, Result};
use crate::{EmbedMode, SteganographyEngine};

/// Matroska/WebM steganography engine.
///
/// Hides data in a `Void` element appended inside the Segment. See the module documentation
/// for the storage format and the reasoning behind why this leaves playback and the file's
/// internal seek indexes intact.
pub struct MkvEngine;

/// Parsed location of the top-level `Segment` element.
struct SegmentInfo {
    /// Offset of the first byte of the Segment's size VINT.
    size_field_pos: usize,
    /// Width, in bytes, of the Segment's size VINT.
    size_width: usize,
    /// Offset of the first byte of the Segment's data (first child element).
    data_start: usize,
    /// Offset one past the last byte of the Segment's data.
    data_end: usize,
    /// Whether the Segment uses the reserved "unknown size" form.
    unknown_size: bool,
}

impl MkvEngine {
    /// Creates a new MKV engine.
    pub fn new() -> Self {
        Self
    }

    /// Signature distinguishing Lupin's `Void` from ordinary padding `Void` elements.
    const SIGNATURE: &'static [u8] = b"Lupin\0";

    /// EBML Header element ID.
    const ID_EBML: &'static [u8] = &[0x1A, 0x45, 0xDF, 0xA3];

    /// Segment element ID.
    const ID_SEGMENT: &'static [u8] = &[0x18, 0x53, 0x80, 0x67];

    /// `Void` element ID (a single-byte VINT).
    const VOID_ID: u8 = 0xEC;

    /// Returns the encoded byte-width (1..=8) of a VINT or element ID from its first byte.
    ///
    /// The width is signalled by the position of the most significant set bit: `0x80` marks a
    /// 1-byte value, `0x40` a 2-byte value, and so on down to `0x01` for 8 bytes. A first byte
    /// of `0x00` is invalid (it would encode a width greater than 8).
    fn vint_width(first: u8) -> Option<usize> {
        if first == 0 {
            return None;
        }
        Some(first.leading_zeros() as usize + 1)
    }

    /// Reads the element ID starting at `pos`, returning the raw ID bytes and their width.
    fn read_id(data: &[u8], pos: usize) -> Result<(&[u8], usize)> {
        let first = *data
            .get(pos)
            .ok_or_else(|| Self::malformed("unexpected end of file"))?;
        let width = Self::vint_width(first).ok_or_else(|| Self::malformed("invalid element ID"))?;
        if pos + width > data.len() {
            return Err(Self::malformed("truncated element ID"));
        }
        Ok((&data[pos..pos + width], width))
    }

    /// Reads the size VINT starting at `pos`, returning `(value, width, is_unknown)`.
    ///
    /// `is_unknown` is true for the reserved all-ones encoding that marks an unknown-size
    /// element (one that extends to the next element or EOF).
    fn read_size(data: &[u8], pos: usize) -> Result<(u64, usize, bool)> {
        let first = *data
            .get(pos)
            .ok_or_else(|| Self::malformed("unexpected end of file"))?;
        let width = Self::vint_width(first).ok_or_else(|| Self::malformed("invalid size VINT"))?;
        if pos + width > data.len() {
            return Err(Self::malformed("truncated size VINT"));
        }

        // Strip the width-marker bit from the first byte, then fold in the remaining bytes.
        // A width of 8 leaves no value bits in the first byte; compute the mask in u16 so the
        // `>> 8` shift can't overflow a u8.
        let mut value = (first & (0xFFu16 >> width) as u8) as u64;
        for &byte in &data[pos + 1..pos + width] {
            value = (value << 8) | byte as u64;
        }

        // The reserved "unknown size" value is all value bits set to 1 (7 bits per byte).
        let all_ones = (1u64 << (7 * width)) - 1;
        Ok((value, width, value == all_ones))
    }

    /// Encodes `value` as a size VINT of exactly `width` bytes.
    fn encode_size(value: u64, width: usize) -> Vec<u8> {
        // The width-marker bit sits just above the value bits (7 per byte).
        let marked = (1u64 << (7 * width)) | value;
        let mut out = vec![0u8; width];
        for (i, slot) in out.iter_mut().rev().enumerate() {
            *slot = ((marked >> (8 * i)) & 0xFF) as u8;
        }
        out
    }

    /// Returns the smallest VINT width that can represent `value` without colliding with the
    /// reserved all-ones ("unknown size") encoding.
    fn min_size_width(value: u64) -> usize {
        for width in 1..=8 {
            // Largest representable value at this width, minus the reserved all-ones value.
            if value <= (1u64 << (7 * width)) - 2 {
                return width;
            }
        }
        8
    }

    /// Builds a `MkvInvalidFormat` error with the given reason.
    fn malformed(reason: &str) -> LupinError {
        LupinError::MkvInvalidFormat {
            reason: reason.to_string(),
        }
    }

    /// Locates the top-level `Segment`, skipping the EBML Header (and any top-level `Void`/
    /// `CRC-32` padding that may precede the Segment).
    fn find_segment(data: &[u8]) -> Result<SegmentInfo> {
        // The EBML Header must come first and always has a known size.
        let (id, id_width) = Self::read_id(data, 0)?;
        if id != Self::ID_EBML {
            return Err(Self::malformed("missing EBML header"));
        }
        let (header_size, size_width, unknown) = Self::read_size(data, id_width)?;
        if unknown {
            return Err(Self::malformed("EBML header has unknown size"));
        }
        let mut pos = id_width + size_width + header_size as usize;

        // Walk top-level elements until we reach the Segment.
        loop {
            if pos >= data.len() {
                return Err(Self::malformed("no Segment element found"));
            }
            let (id, id_width) = Self::read_id(data, pos)?;
            let size_field_pos = pos + id_width;
            let (size, size_width, unknown_size) = Self::read_size(data, size_field_pos)?;
            let data_start = size_field_pos + size_width;

            if id == Self::ID_SEGMENT {
                let data_end = if unknown_size {
                    data.len()
                } else {
                    let end = data_start + size as usize;
                    if end > data.len() {
                        return Err(Self::malformed("Segment size exceeds file length"));
                    }
                    end
                };
                return Ok(SegmentInfo {
                    size_field_pos,
                    size_width,
                    data_start,
                    data_end,
                    unknown_size,
                });
            }

            // Some other top-level element (e.g. padding Void/CRC-32); it must be known-size.
            if unknown_size {
                return Err(Self::malformed("unknown-size element before Segment"));
            }
            pos = data_start + size as usize;
        }
    }

    /// Searches `data[start..end]` for a Lupin `Void` carrier and returns the byte range of its
    /// payload, or `None` if no valid carrier is present.
    ///
    /// A match is only accepted when the signature is immediately preceded by a well-formed
    /// `Void` header (`0xEC` + a size VINT) whose declared size is consistent with the
    /// signature-plus-payload it frames. That framing makes a stray `Lupin\0` occurring inside
    /// ordinary media data extremely unlikely to be mistaken for a carrier.
    fn find_carrier(data: &[u8], start: usize, end: usize) -> Option<(usize, usize)> {
        let sig = Self::SIGNATURE;
        if end < start + sig.len() {
            return None;
        }

        let mut i = start;
        while i + sig.len() <= end {
            if &data[i..i + sig.len()] == sig {
                // The signature is the first byte of the Void data, so the size VINT ends at
                // `i`. Try each possible VINT width and validate the surrounding framing.
                for width in 1..=8 {
                    if i < width + 1 {
                        continue;
                    }
                    let size_field_pos = i - width;
                    let void_id_pos = size_field_pos - 1;
                    if void_id_pos < start || data[void_id_pos] != Self::VOID_ID {
                        continue;
                    }
                    if Self::vint_width(data[size_field_pos]) != Some(width) {
                        continue;
                    }
                    if let Ok((size, _, false)) = Self::read_size(data, size_field_pos) {
                        // The Void data begins at `i`; its declared size covers signature +
                        // payload and must not run past the Segment.
                        let payload_end = i + size as usize;
                        if size as usize >= sig.len() && payload_end <= end {
                            return Some((i + sig.len(), payload_end));
                        }
                    }
                }
            }
            i += 1;
        }
        None
    }

    /// Builds the `Void` carrier element for `payload`.
    fn build_carrier(payload: &[u8]) -> Vec<u8> {
        let content_len = Self::SIGNATURE.len() + payload.len();
        let size_width = Self::min_size_width(content_len as u64);

        let mut carrier = Vec::with_capacity(1 + size_width + content_len);
        carrier.push(Self::VOID_ID);
        carrier.extend_from_slice(&Self::encode_size(content_len as u64, size_width));
        carrier.extend_from_slice(Self::SIGNATURE);
        carrier.extend_from_slice(payload);
        carrier
    }
}

impl Default for MkvEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SteganographyEngine for MkvEngine {
    fn magic_bytes(&self) -> &[u8] {
        // The EBML Header ID; shared by Matroska and its WebM subset.
        Self::ID_EBML
    }

    fn format_name(&self) -> &str {
        "MKV"
    }

    fn format_ext(&self) -> &str {
        ".mkv"
    }

    fn embed(&self, source_data: &[u8], payload: &[u8], mode: EmbedMode) -> Result<Vec<u8>> {
        // Reject empty payloads so the embed contract is uniform across engines.
        if payload.is_empty() {
            return Err(LupinError::EmptyPayload);
        }

        // Exhaustive so a future EmbedMode variant is a compile error here rather than
        // silently falling through to the capacity implementation below.
        match mode {
            EmbedMode::Capacity => {}
            EmbedMode::Stealth => return Err(LupinError::StealthNotSupported { format: "MKV" }),
        }

        let segment = Self::find_segment(source_data)?;

        // Refuse to embed into an MKV that already carries a Lupin Void; a second carrier
        // would be appended and silently lost on extract (which returns the first match).
        if Self::find_carrier(source_data, segment.data_start, segment.data_end).is_some() {
            return Err(LupinError::EmbedCollision {
                source: std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "MKV already contains a Lupin Void element",
                ),
            });
        }

        let carrier = Self::build_carrier(payload);

        // Insert the carrier as the Segment's last child (at `data_end`), preserving every
        // existing element's Segment-relative offset.
        let mut output = Vec::with_capacity(source_data.len() + carrier.len() + 1);

        if segment.unknown_size {
            // The Segment runs to EOF, so the carrier falls inside it with no size to rewrite.
            output.extend_from_slice(&source_data[..segment.data_end]);
            output.extend_from_slice(&carrier);
            output.extend_from_slice(&source_data[segment.data_end..]);
        } else {
            // Grow the Segment's declared size to include the carrier. Keep the original size
            // width when the new value still fits it; otherwise widen (still offset-safe, as
            // all children shift together relative to the Segment data start).
            let old_content_len = segment.data_end - segment.data_start;
            let new_content_len = (old_content_len + carrier.len()) as u64;
            let new_width = if new_content_len <= (1u64 << (7 * segment.size_width)) - 2 {
                segment.size_width
            } else {
                Self::min_size_width(new_content_len)
            };

            output.extend_from_slice(&source_data[..segment.size_field_pos]);
            output.extend_from_slice(&Self::encode_size(new_content_len, new_width));
            output.extend_from_slice(&source_data[segment.data_start..segment.data_end]);
            output.extend_from_slice(&carrier);
            output.extend_from_slice(&source_data[segment.data_end..]);
        }

        Ok(output)
    }

    fn extract(&self, source_data: &[u8]) -> Result<Vec<u8>> {
        let segment = Self::find_segment(source_data)?;
        match Self::find_carrier(source_data, segment.data_start, segment.data_end) {
            Some((start, end)) => Ok(source_data[start..end].to_vec()),
            None => Err(LupinError::MkvNoHiddenData),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds an EBML element `[id][size][data]` with a minimally-encoded size.
    fn build_element(id: &[u8], data: &[u8]) -> Vec<u8> {
        let width = MkvEngine::min_size_width(data.len() as u64);
        let mut out = id.to_vec();
        out.extend_from_slice(&MkvEngine::encode_size(data.len() as u64, width));
        out.extend_from_slice(data);
        out
    }

    /// Builds a minimal known-size MKV: an EBML header plus a Segment carrying `content` bytes
    /// of opaque children.
    fn build_mkv(content: &[u8]) -> Vec<u8> {
        let mut mkv = build_element(MkvEngine::ID_EBML, &[0x42, 0x82, 0x01, 0x02]); // header w/ data
        mkv.extend_from_slice(&build_element(MkvEngine::ID_SEGMENT, content));
        mkv
    }

    fn create_minimal_mkv() -> Vec<u8> {
        build_mkv(&[0x11u8; 64])
    }

    /// Builds an MKV whose Segment uses the reserved "unknown size" form (runs to EOF).
    fn build_unknown_size_mkv(content: &[u8]) -> Vec<u8> {
        let mut mkv = build_element(MkvEngine::ID_EBML, &[0x42, 0x82, 0x01, 0x02]);
        mkv.extend_from_slice(MkvEngine::ID_SEGMENT);
        mkv.push(0xFF); // 1-byte unknown-size VINT
        mkv.extend_from_slice(content);
        mkv
    }

    #[test]
    fn test_magic_bytes() {
        let engine = MkvEngine::new();
        assert_eq!(engine.magic_bytes(), &[0x1A, 0x45, 0xDF, 0xA3]);
    }

    #[test]
    fn test_format_name() {
        assert_eq!(MkvEngine::new().format_name(), "MKV");
    }

    #[test]
    fn test_format_ext() {
        assert_eq!(MkvEngine::new().format_ext(), ".mkv");
    }

    #[test]
    fn test_vint_round_trip() {
        // Encoding then decoding a size must recover the original value across widths.
        for value in [0u64, 1, 126, 127, 16382, 16383, 1_000_000, 1u64 << 40] {
            let width = MkvEngine::min_size_width(value);
            let encoded = MkvEngine::encode_size(value, width);
            let (decoded, decoded_width, unknown) = MkvEngine::read_size(&encoded, 0).unwrap();
            assert_eq!(decoded, value, "value {value} should round-trip");
            assert_eq!(decoded_width, width);
            assert!(
                !unknown,
                "minimally-encoded sizes are never the reserved value"
            );
        }
    }

    #[test]
    fn test_read_size_eight_byte_vint() {
        // Real muxers (ffmpeg, mkvmerge) commonly encode the Segment size as a full 8-byte
        // VINT, which leaves no value bits in the first byte. Reading it must not overflow.
        let encoded = MkvEngine::encode_size(1_000_000, 8);
        assert_eq!(encoded.len(), 8);
        let (value, width, unknown) = MkvEngine::read_size(&encoded, 0).unwrap();
        assert_eq!(value, 1_000_000);
        assert_eq!(width, 8);
        assert!(!unknown);
    }

    #[test]
    fn test_eight_byte_segment_size_round_trip() {
        // A Segment whose size is stored as an 8-byte VINT must embed and extract cleanly.
        let engine = MkvEngine::new();
        let content = vec![0x44u8; 40];
        let mut source = build_element(MkvEngine::ID_EBML, &[0x42, 0x82, 0x01, 0x02]);
        source.extend_from_slice(MkvEngine::ID_SEGMENT);
        source.extend_from_slice(&MkvEngine::encode_size(content.len() as u64, 8)); // force width 8
        source.extend_from_slice(&content);

        let embedded = engine
            .embed(&source, b"eight byte size", EmbedMode::Capacity)
            .unwrap();

        assert_eq!(engine.extract(&embedded).unwrap(), b"eight byte size");
    }

    #[test]
    fn test_find_segment_known_size() {
        let mkv = create_minimal_mkv();

        let segment = MkvEngine::find_segment(&mkv).unwrap();

        assert!(!segment.unknown_size);
        // The Segment data is the 64 opaque bytes we put in.
        assert_eq!(segment.data_end - segment.data_start, 64);
        assert_eq!(segment.data_end, mkv.len());
    }

    #[test]
    fn test_find_segment_unknown_size() {
        let mkv = build_unknown_size_mkv(&[0x22u8; 30]);

        let segment = MkvEngine::find_segment(&mkv).unwrap();

        assert!(segment.unknown_size);
        // An unknown-size Segment extends to EOF.
        assert_eq!(segment.data_end, mkv.len());
    }

    #[test]
    fn test_embed_success() {
        let engine = MkvEngine::new();
        let source = create_minimal_mkv();
        let payload = b"Hello, MKV steganography!";

        let embedded = engine.embed(&source, payload, EmbedMode::Capacity).unwrap();

        assert!(embedded.len() > source.len());
        assert!(embedded.starts_with(&[0x1A, 0x45, 0xDF, 0xA3])); // still a valid EBML doc
                                                                  // The Segment must still parse and now enclose the carrier.
        let segment = MkvEngine::find_segment(&embedded).unwrap();
        assert_eq!(segment.data_end, embedded.len());
    }

    #[test]
    fn test_embed_and_extract_round_trip() {
        let engine = MkvEngine::new();
        let source = create_minimal_mkv();
        let payload = b"Secret message hidden in MKV!";

        let embedded = engine
            .embed(&source, payload, EmbedMode::Capacity)
            .expect("Embed should succeed");
        let extracted = engine.extract(&embedded).expect("Extract should succeed");

        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_unknown_size_segment_round_trip() {
        // The "safe everywhere" case: an unknown-size Segment must embed and extract without
        // touching the size field.
        let engine = MkvEngine::new();
        let source = build_unknown_size_mkv(&[0x22u8; 30]);
        let payload = b"payload in an open-ended segment";

        let embedded = engine.embed(&source, payload, EmbedMode::Capacity).unwrap();

        // The unknown-size marker (0xFF) must be preserved, and the payload recoverable.
        assert!(MkvEngine::find_segment(&embedded).unwrap().unknown_size);
        assert_eq!(engine.extract(&embedded).unwrap(), payload);
    }

    #[test]
    fn test_segment_size_widened_and_offsets_preserved() {
        // A Segment whose size VINT must widen to hold the carrier: original content of 120
        // bytes (fits a 1-byte size, max 126) plus the carrier crosses into a 2-byte size.
        let engine = MkvEngine::new();
        let content = vec![0x33u8; 120];
        let source = build_mkv(&content);

        let embedded = engine
            .embed(&source, b"widen me", EmbedMode::Capacity)
            .unwrap();

        let segment = MkvEngine::find_segment(&embedded).unwrap();
        assert_eq!(segment.size_width, 2, "size field should have widened");
        // The original 120 content bytes must be byte-for-byte intact and contiguous at the
        // start of the Segment data (offsets preserved).
        assert_eq!(
            &embedded[segment.data_start..segment.data_start + 120],
            &content[..]
        );
        assert_eq!(engine.extract(&embedded).unwrap(), b"widen me");
    }

    #[test]
    fn test_embed_collision() {
        let engine = MkvEngine::new();
        let source = create_minimal_mkv();

        let embedded_once = engine
            .embed(&source, b"first payload", EmbedMode::Capacity)
            .unwrap();
        let result = engine.embed(&embedded_once, b"second payload", EmbedMode::Capacity);

        assert!(matches!(result, Err(LupinError::EmbedCollision { .. })));
        // The already-embedded payload must remain intact and extractable.
        assert_eq!(engine.extract(&embedded_once).unwrap(), b"first payload");
    }

    #[test]
    fn test_extract_no_hidden_data() {
        let engine = MkvEngine::new();
        let source = create_minimal_mkv();

        let result = engine.extract(&source);

        assert!(matches!(result, Err(LupinError::MkvNoHiddenData)));
    }

    #[test]
    fn test_round_trip_with_binary_data() {
        let engine = MkvEngine::new();
        let source = create_minimal_mkv();
        let payload: Vec<u8> = (0..=255).cycle().take(1000).collect();

        let embedded = engine
            .embed(&source, &payload, EmbedMode::Capacity)
            .expect("Embed should succeed");
        let extracted = engine.extract(&embedded).expect("Extract should succeed");

        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_embed_empty_payload_rejected() {
        let engine = MkvEngine::new();
        let source = create_minimal_mkv();

        let result = engine.embed(&source, b"", EmbedMode::Capacity);

        assert!(matches!(result, Err(LupinError::EmptyPayload)));
    }

    #[test]
    fn test_stealth_mode_not_supported() {
        let engine = MkvEngine::new();
        let source = create_minimal_mkv();

        let result = engine.embed(&source, b"payload", EmbedMode::Stealth);

        assert!(matches!(
            result,
            Err(LupinError::StealthNotSupported { format: "MKV" })
        ));
    }

    #[test]
    fn test_extract_invalid_format() {
        // EBML magic but no Segment: extraction must fail cleanly rather than panic.
        let engine = MkvEngine::new();
        let header_only = build_element(MkvEngine::ID_EBML, &[0x42, 0x82]);

        let result = engine.extract(&header_only);

        assert!(matches!(result, Err(LupinError::MkvInvalidFormat { .. })));
    }
}
