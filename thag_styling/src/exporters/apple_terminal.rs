//! Apple Terminal theme exporter
//!
//! Exports thag themes to Apple Terminal's `.terminal` XML plist format.
//!
//! Apple Terminal stores each colour as a `<data>` element whose content is a
//! base64-encoded **binary plist** (`bplist00`) containing an
//! `NSKeyedArchiver`-encoded `NSColor` object (calibrated RGB, `NSColorSpace = 2`).
//! This module encodes that structure entirely in Rust — no external tools or
//! crates are needed.

use crate::{
    exporters::{adjust_color_brightness, ThemeExporter},
    StylingResult, Theme,
};

use std::fmt::Write as _;

/// Apple Terminal theme exporter.
pub struct AppleTerminalExporter;

impl ThemeExporter for AppleTerminalExporter {
    #[allow(clippy::too_many_lines)]
    fn export_theme(theme: &Theme) -> StylingResult<String> {
        let mut output = String::new();

        let bg_color = theme.bg_rgbs.first().copied().unwrap_or([0, 0, 0]);

        // XML plist header
        writeln!(output, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
        writeln!(
            output,
            r#"<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">"#
        )?;
        writeln!(output, r#"<plist version="1.0">"#)?;
        writeln!(output, "<dict>")?;

        // Metadata
        writeln!(output, "\t<key>name</key>")?;
        writeln!(output, "\t<string>{}</string>", theme.name)?;
        writeln!(output, "\t<key>type</key>")?;
        writeln!(output, "\t<string>Window Settings</string>")?;

        // ANSI colours 0–15 (palette role mapping mirrors the iTerm2 exporter)
        write_color_entry(&mut output, "ANSIBlackColor", Some(bg_color))?;
        write_color_entry(&mut output, "ANSIRedColor", theme.palette.emphasis.rgb())?;
        write_color_entry(&mut output, "ANSIGreenColor", theme.palette.success.rgb())?;
        write_color_entry(
            &mut output,
            "ANSIYellowColor",
            theme.palette.commentary.rgb(),
        )?;
        write_color_entry(&mut output, "ANSIBlueColor", theme.palette.info.rgb())?;
        write_color_entry(
            &mut output,
            "ANSIMagentaColor",
            theme.palette.heading1.rgb(),
        )?;
        write_color_entry(&mut output, "ANSICyanColor", theme.palette.code.rgb())?;
        write_color_entry(&mut output, "ANSIWhiteColor", theme.palette.normal.rgb())?;
        write_color_entry(
            &mut output,
            "ANSIBrightBlackColor",
            theme.palette.subtle.rgb(),
        )?;
        write_color_entry(&mut output, "ANSIBrightRedColor", theme.palette.error.rgb())?;
        write_color_entry(
            &mut output,
            "ANSIBrightGreenColor",
            theme.palette.debug.rgb(),
        )?;
        write_color_entry(
            &mut output,
            "ANSIBrightYellowColor",
            theme.palette.warning.rgb(),
        )?;
        write_color_entry(&mut output, "ANSIBrightBlueColor", theme.palette.link.rgb())?;
        write_color_entry(
            &mut output,
            "ANSIBrightMagentaColor",
            theme.palette.heading2.rgb(),
        )?;
        write_color_entry(&mut output, "ANSIBrightCyanColor", theme.palette.hint.rgb())?;
        write_color_entry(
            &mut output,
            "ANSIBrightWhiteColor",
            theme.palette.quote.rgb(),
        )?;

        // Background / foreground
        write_color_entry(&mut output, "BackgroundColor", Some(bg_color))?;
        write_color_entry(&mut output, "TextColor", theme.palette.normal.rgb())?;

        // Bold text
        write_color_entry(
            &mut output,
            "BoldTextColor",
            theme
                .palette
                .emphasis
                .rgb()
                .or_else(|| theme.palette.normal.rgb()),
        )?;

        // Cursor (displayed as a block)
        write_color_entry(
            &mut output,
            "CursorColor",
            theme
                .palette
                .emphasis
                .rgb()
                .or_else(|| theme.palette.normal.rgb()),
        )?;

        // Selection background
        write_color_entry(
            &mut output,
            "SelectionColor",
            theme
                .palette
                .commentary
                .rgb()
                .or_else(|| Some(adjust_color_brightness(bg_color, 1.4))),
        )?;

        writeln!(output, "</dict>")?;
        writeln!(output, "</plist>")?;

        Ok(output)
    }

    fn file_extension() -> &'static str {
        "terminal"
    }

    fn format_name() -> &'static str {
        "Apple Terminal"
    }
}

// ── XML helpers ──────────────────────────────────────────────────────────────

/// Write a single `<key>`/`<data>` colour pair to the plist XML output.
fn write_color_entry(
    output: &mut String,
    key: &str,
    rgb_opt: Option<[u8; 3]>,
) -> Result<(), std::fmt::Error> {
    let [r, g, b] = rgb_opt.unwrap_or([128, 128, 128]);

    let plist_bytes = encode_nscolor_bplist(r, g, b);
    let b64 = base64_encode(&plist_bytes);

    writeln!(output, "\t<key>{key}</key>")?;
    writeln!(output, "\t<data>")?;

    // Wrap base64 at 68 chars per line (matches Apple's plist serialisation)
    let mut pos = 0;
    while pos < b64.len() {
        let end = (pos + 68).min(b64.len());
        writeln!(output, "\t{}", &b64[pos..end])?;
        pos = end;
    }

    writeln!(output, "\t</data>")?;
    Ok(())
}

// ── Binary plist (bplist00) encoder for NSColor ──────────────────────────────
//
// Apple Terminal stores each colour as an NSKeyedArchiver binary plist whose
// object graph looks like this (indices are into the bplist offset table):
//
//  0  dict  root NSKeyedArchiver wrapper (4 pairs: 1→2, 3→4, 8→9, 10→11)
//  1  str   "$archiver"
//  2  str   "NSKeyedArchiver"
//  3  str   "$objects"
//  4  arr   $objects array  [5, 6, 7]
//  5  str   "$null"
//  6  dict  NSColor object  (3 pairs: 12→13, 14→15, 16→17)
//  7  dict  NSColor class   (2 pairs: 18→19, 20→21)
//  8  str   "$top"
//  9  dict  top reference   (1 pair: 22→23)
// 10  str   "$version"
// 11  int   100000
// 12  str   "$class"
// 13  uid   UID(2)   ← index of class dict in $objects
// 14  str   "NSColorSpace"
// 15  int   2        ← NSCalibratedRGBColorSpace
// 16  str   "NSRGB"
// 17  data  ASCII "r g b " (space-separated f64 components, trailing space)
// 18  str   "$classes"
// 19  arr   class names  [24, 25]
// 20  str   "$classname"
// 21  str   "NSColor"   (classname value)
// 22  str   "root"
// 23  uid   UID(1)   ← index of NSColor dict in $objects
// 24  str   "NSColor"   (element of $classes array)
// 25  str   "NSObject"
//
// Total: 26 objects.  All byte offsets fit in one byte (verified below).

/// Encode `(r, g, b)` as an `NSKeyedArchiver` binary plist for `NSColor`.
///
/// The result is suitable for base64-encoding into a `.terminal` plist `<data>` element.
fn encode_nscolor_bplist(r: u8, g: u8, b: u8) -> Vec<u8> {
    // Build the NSRGB ASCII string: "r g b " (space-separated f64, trailing space)
    let rf = f64::from(r) / 255.0;
    let gf = f64::from(g) / 255.0;
    let bf = f64::from(b) / 255.0;
    let nsrgb = format!("{rf} {gf} {bf} ");
    let nsrgb_bytes = nsrgb.as_bytes();

    let mut buf: Vec<u8> = Vec::with_capacity(300);
    let mut offsets: Vec<usize> = Vec::with_capacity(26);

    // Magic header
    buf.extend_from_slice(b"bplist00");

    macro_rules! rec {
        ($write:expr) => {{
            offsets.push(buf.len());
            $write;
        }};
    }

    rec!(bplist_dict(&mut buf, &[1, 3, 8, 10], &[2, 4, 9, 11])); //  0 root dict
    rec!(bplist_string(&mut buf, "$archiver")); //  1
    rec!(bplist_string(&mut buf, "NSKeyedArchiver")); //  2
    rec!(bplist_string(&mut buf, "$objects")); //  3
    rec!(bplist_array(&mut buf, &[5, 6, 7])); //  4 $objects array
    rec!(bplist_string(&mut buf, "$null")); //  5
    rec!(bplist_dict(&mut buf, &[12, 14, 16], &[13, 15, 17])); //  6 NSColor object
    rec!(bplist_dict(&mut buf, &[18, 20], &[19, 21])); //  7 NSColor class
    rec!(bplist_string(&mut buf, "$top")); //  8
    rec!(bplist_dict(&mut buf, &[22], &[23])); //  9 top reference
    rec!(bplist_string(&mut buf, "$version")); // 10
    rec!(bplist_int(&mut buf, 100_000)); // 11
    rec!(bplist_string(&mut buf, "$class")); // 12
    rec!(bplist_uid(&mut buf, 2)); // 13 UID(2)
    rec!(bplist_string(&mut buf, "NSColorSpace")); // 14
    rec!(bplist_int(&mut buf, 2)); // 15  NSCalibratedRGBColorSpace
    rec!(bplist_string(&mut buf, "NSRGB")); // 16
    rec!(bplist_data(&mut buf, nsrgb_bytes)); // 17 variable-length colour data
    rec!(bplist_string(&mut buf, "$classes")); // 18
    rec!(bplist_array(&mut buf, &[24, 25])); // 19 class names
    rec!(bplist_string(&mut buf, "$classname")); // 20
    rec!(bplist_string(&mut buf, "NSColor")); // 21 classname value
    rec!(bplist_string(&mut buf, "root")); // 22
    rec!(bplist_uid(&mut buf, 1)); // 23 UID(1)
    rec!(bplist_string(&mut buf, "NSColor")); // 24 element of $classes
    rec!(bplist_string(&mut buf, "NSObject")); // 25

    // ── Offset table ─────────────────────────────────────────────────────────
    let offset_table_start = buf.len();
    let num_objects = offsets.len() as u64; // 26

    // With 26 objects and NSRGB ≤ ~60 bytes, all offsets fit in one byte
    // (worst-case obj-25 offset ≈ 235 < 256).  Use 2-byte entries as a
    // safe fallback should that ever change.
    let max_offset = offsets.iter().copied().max().unwrap_or(0);
    let offset_size: u8 = if max_offset < 256 { 1 } else { 2 };

    for &off in &offsets {
        if offset_size == 1 {
            #[allow(clippy::cast_possible_truncation)]
            buf.push(off as u8);
        } else {
            #[allow(clippy::cast_possible_truncation)]
            buf.extend_from_slice(&(off as u16).to_be_bytes());
        }
    }

    // ── Trailer (32 bytes) ────────────────────────────────────────────────────
    buf.extend_from_slice(&[0u8; 5]); // unused padding
    buf.push(0u8); // sort version (unused)
    buf.push(offset_size); // bytes per offset table entry
    buf.push(1u8); // bytes per object reference (26 < 256)
    buf.extend_from_slice(&num_objects.to_be_bytes()); // object count
    buf.extend_from_slice(&0u64.to_be_bytes()); // top object index = 0
    buf.extend_from_slice(&(offset_table_start as u64).to_be_bytes()); // offset table offset

    buf
}

// ── bplist primitive writers ──────────────────────────────────────────────────

/// Write an ASCII string object (`0x5_` type family).
fn bplist_string(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    bplist_typed_length(buf, 0x50, bytes.len());
    buf.extend_from_slice(bytes);
}

/// Write a binary data object (`0x4_` type family).
fn bplist_data(buf: &mut Vec<u8>, data: &[u8]) {
    bplist_typed_length(buf, 0x40, data.len());
    buf.extend_from_slice(data);
}

/// Write an array object (`0xA_` type family); `refs` are 1-byte object indices.
fn bplist_array(buf: &mut Vec<u8>, refs: &[u8]) {
    bplist_typed_length(buf, 0xA0, refs.len());
    buf.extend_from_slice(refs);
}

/// Write a dict object (`0xD_` type family).
///
/// bplist dicts store **all key refs first**, then **all value refs** (not interleaved).
fn bplist_dict(buf: &mut Vec<u8>, keys: &[u8], vals: &[u8]) {
    debug_assert_eq!(keys.len(), vals.len(), "dict key/value count mismatch");
    bplist_typed_length(buf, 0xD0, keys.len());
    buf.extend_from_slice(keys);
    buf.extend_from_slice(vals);
}

/// Write a non-negative integer object (`0x1_` type family).
#[allow(clippy::cast_possible_truncation)]
fn bplist_int(buf: &mut Vec<u8>, value: u64) {
    if value <= 0xFF {
        buf.push(0x10);
        buf.push(value as u8);
    } else if value <= 0xFFFF {
        buf.push(0x11);
        buf.extend_from_slice(&(value as u16).to_be_bytes());
    } else if value <= 0xFFFF_FFFF {
        buf.push(0x12);
        buf.extend_from_slice(&(value as u32).to_be_bytes());
    } else {
        buf.push(0x13);
        buf.extend_from_slice(&value.to_be_bytes());
    }
}

/// Write a UID object (`0x8_` type family).  Handles UIDs 0–65 535.
fn bplist_uid(buf: &mut Vec<u8>, uid: u16) {
    if uid <= 0xFF {
        buf.push(0x80); // 1-byte UID
        #[allow(clippy::cast_possible_truncation)]
        buf.push(uid as u8);
    } else {
        buf.push(0x81); // 2-byte UID
        buf.extend_from_slice(&uid.to_be_bytes());
    }
}

/// Emit the type+length byte(s) for strings, data, arrays, and dicts.
///
/// When `count < 15` the count is packed into the low nibble of the type byte.
/// Otherwise `0x_F` signals an extended count encoded as a separate bplist int.
#[allow(clippy::cast_possible_truncation)]
fn bplist_typed_length(buf: &mut Vec<u8>, type_base: u8, count: usize) {
    if count < 15 {
        buf.push(type_base | count as u8);
    } else {
        buf.push(type_base | 0x0F);
        bplist_int(buf, count as u64);
    }
}

// ── Base-64 encoder (RFC 4648, no line wrapping) ──────────────────────────────

const B64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encode `input` as standard base64 (RFC 4648).  Line-wrapping is left to the caller.
fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(((input.len() + 2) / 3) * 4);

    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);

        out.push(B64_CHARS[(b0 >> 2) as usize] as char);
        out.push(B64_CHARS[(((b0 & 0x3) << 4) | (b1 >> 4)) as usize] as char);
        out.push(if chunk.len() > 1 {
            B64_CHARS[(((b1 & 0xF) << 2) | (b2 >> 6)) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64_CHARS[(b2 & 0x3F) as usize] as char
        } else {
            '='
        });
    }

    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exporters::create_test_theme;

    #[test]
    fn test_apple_terminal_export_structure() {
        let theme = create_test_theme();
        let result = AppleTerminalExporter::export_theme(&theme);

        assert!(result.is_ok(), "Apple Terminal export failed");
        let content = result.unwrap();

        // Top-level plist structure
        assert!(content.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(content.contains("<!DOCTYPE plist"));
        assert!(content.contains(r#"<plist version="1.0">"#));
        assert!(content.contains("<dict>"));
        assert!(content.contains("</dict>"));
        assert!(content.contains("</plist>"));

        // Required colour keys
        assert!(content.contains("<key>BackgroundColor</key>"));
        assert!(content.contains("<key>TextColor</key>"));
        assert!(content.contains("<key>ANSIBlackColor</key>"));
        assert!(content.contains("<key>ANSIBrightWhiteColor</key>"));
        assert!(content.contains("<key>CursorColor</key>"));
        assert!(content.contains("<key>SelectionColor</key>"));

        // Colours encoded as base64 data
        assert!(content.contains("<data>"));
        assert!(content.contains("</data>"));
    }

    #[test]
    fn test_encode_nscolor_bplist_header_and_trailer() {
        let data = encode_nscolor_bplist(128, 64, 32);

        // bplist00 magic
        assert_eq!(&data[..8], b"bplist00", "Missing bplist00 magic");

        // Sanity: at least header + objects + trailer
        assert!(data.len() > 40);

        // Decode trailer (last 32 bytes)
        let t = &data[data.len() - 32..];
        let offset_size = t[6];
        let obj_ref_size = t[7];
        let num_objects = u64::from_be_bytes(t[8..16].try_into().unwrap());
        let top_object = u64::from_be_bytes(t[16..24].try_into().unwrap());

        assert_eq!(offset_size, 1, "Expected 1-byte offsets for small plist");
        assert_eq!(obj_ref_size, 1, "Expected 1-byte object refs (26 < 256)");
        assert_eq!(num_objects, 26, "Expected exactly 26 objects");
        assert_eq!(top_object, 0, "Root dict should be at object index 0");
    }

    #[test]
    fn test_encode_nscolor_bplist_various_colours() {
        for &(r, g, b) in &[
            (255u8, 255, 255), // white
            (0, 0, 0),         // black
            (128, 128, 128),   // mid-grey
            (100, 150, 200),   // arbitrary
        ] {
            let data = encode_nscolor_bplist(r, g, b);
            assert_eq!(&data[..8], b"bplist00", "Bad magic for ({r},{g},{b})");
            let t = &data[data.len() - 32..];
            let num_objects = u64::from_be_bytes(t[8..16].try_into().unwrap());
            assert_eq!(num_objects, 26, "Wrong object count for ({r},{g},{b})");
        }
    }

    #[test]
    fn test_base64_encode_rfc4648_vectors() {
        // Standard RFC 4648 §10 test vectors
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }
}
