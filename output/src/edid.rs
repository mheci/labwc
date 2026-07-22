//! EDID (Extended Display Identification Data) parser.
//!
//! Parses raw 128-byte or 256-byte EDID blocks from /sys/class/drm/card*/edid
//! to extract manufacturer, model, serial, preferred resolution, refresh rate,
//! color depth, HDR10 support, and physical dimensions.

#[derive(Debug, Clone)]
pub struct EdidInfo {
    pub manufacturer: String,
    pub model: String,
    pub serial: String,
    pub physical_width_cm: u8,
    pub physical_height_cm: u8,
    pub gamma: f32,
    pub preferred_mode: Option<EdidMode>,
    pub modes: Vec<EdidMode>,
    pub hdr10_supported: bool,
    pub hdr_st2084: bool,
    pub color_depth: u8,
    pub max_luminance: Option<f32>,
    pub min_luminance: Option<f32>,
    pub vrr_min: Option<u32>,
    pub vrr_max: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdidMode {
    pub width: u32,
    pub height: u32,
    pub refresh_hz: u32,
    pub is_preferred: bool,
    pub is_native: bool,
}

impl EdidMode {
    pub const fn new(w: u32, h: u32, hz: u32) -> Self {
        Self {
            width: w,
            height: h,
            refresh_hz: hz,
            is_preferred: false,
            is_native: false,
        }
    }

    pub fn display_string(&self) -> String {
        format!("{}x{} @ {}Hz", self.width, self.height, self.refresh_hz)
    }
}

impl EdidInfo {
    /// Parse an EDID blob (128 or 256 bytes) from raw bytes.
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 128 {
            return None;
        }

        // Verify EDID magic: 00 FF FF FF FF FF FF 00
        if data[0] != 0x00
            || data[1] != 0xFF
            || data[2] != 0xFF
            || data[3] != 0xFF
            || data[4] != 0xFF
            || data[5] != 0xFF
            || data[6] != 0xFF
            || data[7] != 0x00
        {
            return None;
        }

        let manufacturer = Self::decode_manufacturer(&data[8..10]);
        let product_code = u16::from_le_bytes([data[10], data[11]]);
        let serial = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);

        let model = format!("{}{:04X}", manufacturer, product_code);
        let serial_str = format!("{:08X}", serial);

        let physical_width_cm = data[21];
        let physical_height_cm = data[22];
        let gamma = if data[23] == 0xFF {
            2.2
        } else {
            (data[23] as f32 + 100.0) / 100.0
        };

        let mut modes = Vec::new();
        let mut preferred_mode = None;

        // Parse detailed timing descriptors (DTDs) — 4 blocks at offsets 54, 72, 90, 108
        for block_idx in 0..4 {
            let offset = 54 + block_idx * 18;
            if offset + 18 > data.len() {
                break;
            }
            let block = &data[offset..offset + 18];

            // DTD: pixel clock is non-zero for a timing descriptor
            let pixel_clock_raw = u16::from_le_bytes([block[0], block[1]]);
            if pixel_clock_raw == 0 {
                // This is a monitor descriptor, not a timing descriptor
                // Could be a range limits descriptor (VRR info) or display name
                if block[3] == 0xFD {
                    // Monitor range limits — may contain VRR info
                    let _vrr_max_raw = block[6] as u32;
                    let _vrr_min_raw = block[5] as u32;
                }
                continue;
            }

            let pixel_clock = pixel_clock_raw as f64 * 10_000.0; // in Hz

            let h_active = (block[2] as u32) | (((block[4] as u32) & 0xF0) << 4);
            let h_blank = (block[3] as u32) | (((block[4] as u32) & 0x0F) << 8);
            let v_active = (block[5] as u32) | (((block[7] as u32) & 0xF0) << 4);
            let v_blank = (block[6] as u32) | (((block[7] as u32) & 0x0F) << 8);
            let _h_sync_offset = (block[8] as u32) | (((block[11] as u32) & 0xC0) << 2);
            let _h_sync_pulse = (block[9] as u32) | (((block[11] as u32) & 0x30) << 4);
            let _v_sync_offset = ((block[10] as u32) >> 4) | (((block[11] as u32) & 0x0C) << 2);
            let _v_sync_pulse = (block[10] as u32 & 0x0F) | (((block[11] as u32) & 0x03) << 4);

            let h_total = h_active + h_blank;
            let v_total = v_active + v_blank;

            let refresh_hz = if h_total > 0 && v_total > 0 {
                (pixel_clock / (h_total as f64 * v_total as f64)).round() as u32
            } else {
                60
            };

            let mode = EdidMode {
                width: h_active,
                height: v_active,
                refresh_hz,
                is_preferred: block_idx == 0, // First DTD is the preferred timing
                is_native: block_idx == 0,
            };

            if block_idx == 0 {
                preferred_mode = Some(mode);
            }
            modes.push(mode);
        }

        // Parse standard timings (16 bytes at offset 38)
        for i in 0..8 {
            let offset = 38 + i * 2;
            if offset + 2 > data.len() {
                break;
            }
            let b0 = data[offset];
            let b1 = data[offset + 1];

            if b0 == 0x01 && b1 == 0x01 {
                continue;
            } // Unused slot

            let width = ((b0 as u32) + 31) * 8;
            let aspect_ratio = (b1 >> 6) & 0x03;
            let height = match aspect_ratio {
                0 => (width * 10) / 16,
                1 => (width * 3) / 4,
                2 => (width * 4) / 5,
                3 => (width * 9) / 16,
                _ => (width * 3) / 4,
            };
            let refresh_hz = ((b1 & 0x3F) as u32) + 60;

            modes.push(EdidMode::new(width, height, refresh_hz));
        }

        // CEA-861 extension block for HDR metadata (offset 128+)
        let mut hdr10 = false;
        let mut hdr_st2084 = false;
        let max_luminance = None;
        let min_luminance = None;

        if data.len() >= 256 {
            let ext_tag = data[128];
            if ext_tag == 0x02 {
                // CEA-861 extension
                let dtd_start = data[130] as usize;
                let _underscan = (data[131] & 0x80) != 0;
                let _audio = (data[131] & 0x40) != 0;

                // Parse CEA video data blocks for detailed modes
                let _dtd_offset = 132 + dtd_start;

                // Parse CEA extension for HDR static metadata block (tag 0x06)
                let mut offset = 132;
                while offset < 255 && offset + 1 < data.len() {
                    let tag = (data[offset] >> 5) & 0x07;
                    let length = data[offset] as usize & 0x1F;
                    if length == 0 {
                        offset += 1;
                        continue;
                    }

                    if tag == 0x07 && offset + 1 + length <= data.len() {
                        // Extended tag: check for HDR static metadata (ext tag 0x06)
                        if length >= 3 && data[offset + 1] == 0x06 {
                            // HDR Static Metadata Data Block
                            let eotf = data[offset + 2];
                            hdr_st2084 = (eotf & 0x02) != 0;
                            hdr10 = (eotf & 0x04) != 0;
                            // Static metadata descriptor
                            if length >= 5 {
                                let sdo = data[offset + 3];
                                if sdo & 0x01 != 0 { // SDR luminance present
                                     // bytes follow
                                }
                            }
                        }
                    }

                    offset += 1 + length;
                }
            }

            // Check for DisplayID extension blocks for VRR
            if data.len() >= 384 {
                // Block at offset 256 — DisplayID VRR?
                for blk in [256usize, 384].iter().filter(|&&b| b + 128 <= data.len()) {
                    let blk_off = *blk;
                    if blk_off + 8 <= data.len() && data[blk_off] == 0x70 {
                        // DisplayID data block — could contain adaptive sync
                    }
                }
            }
        }

        // Deduplicate modes by resolution + refresh
        modes.sort_by(|a, b| {
            b.width
                .cmp(&a.width)
                .then(b.height.cmp(&a.height))
                .then(b.refresh_hz.cmp(&a.refresh_hz))
        });
        modes.dedup_by(|a, b| {
            a.width == b.width && a.height == b.height && a.refresh_hz == b.refresh_hz
        });

        Some(Self {
            manufacturer,
            model,
            serial: serial_str,
            physical_width_cm,
            physical_height_cm,
            gamma,
            preferred_mode,
            modes,
            hdr10_supported: hdr10,
            hdr_st2084,
            color_depth: if hdr10 { 10 } else { 8 },
            max_luminance,
            min_luminance,
            vrr_min: None,
            vrr_max: None,
        })
    }

    fn decode_manufacturer(data: &[u8]) -> String {
        // PNP ID: 3 characters packed into 2 bytes (5 bits each, big-endian)
        let packed = u16::from_be_bytes([data[0], data[1]]);
        let c1 = ((packed >> 10) & 0x1F) as u8 + b'A' - 1;
        let c2 = ((packed >> 5) & 0x1F) as u8 + b'A' - 1;
        let c3 = (packed & 0x1F) as u8 + b'A' - 1;
        format!("{}{}{}", c1 as char, c2 as char, c3 as char)
    }

    pub fn diagonal_inches(&self) -> f32 {
        let w = self.physical_width_cm as f32;
        let h = self.physical_height_cm as f32;
        if w > 0.0 && h > 0.0 {
            ((w * w + h * h).sqrt() / 2.54 * 10.0).round() / 10.0
        } else {
            0.0
        }
    }

    pub fn best_mode(&self) -> Option<EdidMode> {
        self.preferred_mode.or_else(|| self.modes.last().copied())
    }

    pub fn max_refresh(&self) -> u32 {
        self.modes.iter().map(|m| m.refresh_hz).max().unwrap_or(60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edid_magic_rejection() {
        assert!(EdidInfo::parse(&[0u8; 128]).is_none());
        assert!(EdidInfo::parse(&[]).is_none());
    }

    #[test]
    fn test_manufacturer_decode() {
        // Dell = 0x10AC in PNP ID encoding
        let data = [0x10u8, 0xAC];
        let name = EdidInfo::decode_manufacturer(&data);
        assert_eq!(name.len(), 3);
    }

    #[test]
    fn test_edid_mode_display() {
        let m = EdidMode::new(3840, 2160, 144);
        assert_eq!(m.display_string(), "3840x2160 @ 144Hz");
    }

    #[test]
    #[allow(clippy::useless_vec)]
    fn test_edid_mode_sorting() {
        let mut modes = vec![
            EdidMode::new(1920, 1080, 60),
            EdidMode::new(3840, 2160, 60),
            EdidMode::new(1920, 1080, 144),
        ];
        modes.sort_by(|a, b| {
            b.width
                .cmp(&a.width)
                .then(b.height.cmp(&a.height))
                .then(b.refresh_hz.cmp(&a.refresh_hz))
        });
        assert_eq!(modes[0].width, 3840);
        assert_eq!(modes[1].refresh_hz, 144);
    }
}
