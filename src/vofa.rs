use std::sync::atomic::AtomicU8;

pub static ACTIVE_VOFA_MODE: AtomicU8 = AtomicU8::new(0); // 0 = FireWater, 1 = JustFloat, 2 = IndexFloat, 3 = RawData

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VofaMode {
    FireWater = 0,
    JustFloat = 1,
    IndexFloat = 2,
    RawData = 3,
}

impl VofaMode {
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => VofaMode::FireWater,
            1 => VofaMode::JustFloat,
            2 => VofaMode::IndexFloat,
            _ => VofaMode::RawData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VofaImage {
    pub id: usize,
    pub width: usize,
    pub height: usize,
    pub format: u8,
    pub data: Vec<u8>,
}

pub struct VofaParser {
    buffer: Vec<u8>,
    mode: VofaMode,
    index_float_buffer: Vec<f32>,
    pub latest_image: Option<VofaImage>,
}

impl VofaParser {
    pub fn new(mode: VofaMode) -> Self {
        Self {
            buffer: Vec::new(),
            mode,
            index_float_buffer: Vec::new(),
            latest_image: None,
        }
    }

    pub fn take_latest_image(&mut self) -> Option<VofaImage> {
        self.latest_image.take()
    }

    pub fn set_mode(&mut self, mode: VofaMode) {
        if self.mode != mode {
            self.mode = mode;
            self.buffer.clear();
            self.index_float_buffer.clear();
        }
    }

    pub fn feed(&mut self, data: &[u8]) -> Vec<Vec<f32>> {
        self.buffer.extend_from_slice(data);
        let mut frames = Vec::new();

        loop {
            match self.mode {
                VofaMode::FireWater => {
                    if let Some(pos) = self.buffer.iter().position(|&b| b == b'\n') {
                        let line_bytes = &self.buffer[..pos];
                        if let Ok(line_str) = std::str::from_utf8(line_bytes) {
                            let line_str_owned = line_str.to_string();
                            let trimmed = line_str_owned.trim();
                            let segments: Vec<&str> = trimmed.split(':').collect();
                            if segments.len() >= 2 {
                                let name = segments[segments.len() - 2].trim();
                                let data_part = segments[segments.len() - 1].trim();
                                if name == "image" {
                                    let datas: Vec<usize> = data_part
                                        .split(',')
                                        .filter_map(|s| s.trim().parse::<usize>().ok())
                                        .collect();
                                    if datas.len() == 5 {
                                        let image_id = datas[0];
                                        let image_size = datas[1];
                                        let image_width = datas[2];
                                        let image_height = datas[3];
                                        let image_format = datas[4];

                                        if self.buffer.len() < pos + 1 + image_size {
                                            break; // Wait for full image data
                                        }
                                        self.buffer.drain(..=pos);
                                        let image_data =
                                            self.buffer.drain(..image_size).collect::<Vec<u8>>();
                                        self.latest_image = Some(VofaImage {
                                            id: image_id,
                                            width: image_width,
                                            height: image_height,
                                            format: image_format as u8,
                                            data: image_data,
                                        });
                                    } else {
                                        // Invalid image parameters, skip the line
                                        self.buffer.drain(..=pos);
                                    }
                                } else {
                                    // CSV data frame with a prefix name
                                    let data_part_owned = data_part.to_string();
                                    self.buffer.drain(..=pos);
                                    let mut parsed = Vec::new();
                                    let mut ok = true;
                                    for s in data_part_owned.split(',') {
                                        let s = s.trim();
                                        if s.is_empty() {
                                            continue;
                                        }
                                        if let Ok(v) = s.parse::<f32>() {
                                            parsed.push(v);
                                        } else {
                                            ok = false;
                                            break;
                                        }
                                    }
                                    if ok && !parsed.is_empty() {
                                        frames.push(parsed);
                                    }
                                }
                            } else {
                                // Pure CSV frame without prefix
                                let trimmed_owned = trimmed.to_string();
                                self.buffer.drain(..=pos);
                                let mut parsed = Vec::new();
                                let mut ok = true;
                                for s in trimmed_owned.split(',') {
                                    let s = s.trim();
                                    if s.is_empty() {
                                        continue;
                                    }
                                    if let Ok(v) = s.parse::<f32>() {
                                        parsed.push(v);
                                    } else {
                                        ok = false;
                                        break;
                                    }
                                }
                                if ok && !parsed.is_empty() {
                                    frames.push(parsed);
                                }
                            }
                        } else {
                            // Invalid UTF-8 bytes, drain and skip
                            self.buffer.drain(..=pos);
                        }
                    } else {
                        break;
                    }
                }
                VofaMode::JustFloat | VofaMode::IndexFloat => {
                    let tail = [0x00, 0x00, 0x80, 0x7F];
                    if let Some(pos) = self.buffer.windows(4).position(|w| w == tail) {
                        // Check if it is a potential image header.
                        // Image header is exactly 28 bytes: 5 * i32 + 2 * NaN.
                        // So the first NaN is at pos >= 20.
                        let is_image = if pos >= 20 {
                            if self.buffer.len() >= pos + 8 {
                                self.buffer[pos + 4..pos + 8] == tail
                            } else {
                                // Heuristics on metadata to see if we should wait for the second NaN
                                let size = i32::from_le_bytes(
                                    self.buffer[pos - 16..pos - 12].try_into().unwrap_or([0; 4]),
                                );
                                let width = i32::from_le_bytes(
                                    self.buffer[pos - 12..pos - 8].try_into().unwrap_or([0; 4]),
                                );
                                let height = i32::from_le_bytes(
                                    self.buffer[pos - 8..pos - 4].try_into().unwrap_or([0; 4]),
                                );
                                let format = i32::from_le_bytes(
                                    self.buffer[pos - 4..pos].try_into().unwrap_or([0; 4]),
                                );

                                let looks_like_image_header = size > 0
                                    && (width > 0 || width == -1)
                                    && (height > 0 || height == -1)
                                    && (format >= 0 && format <= 34);

                                if looks_like_image_header {
                                    break; // wait for the second NaN to arrive
                                }
                                false
                            }
                        } else {
                            false
                        };

                        if is_image {
                            let image_id = i32::from_le_bytes(
                                self.buffer[pos - 20..pos - 16].try_into().unwrap_or([0; 4]),
                            ) as usize;
                            let image_size = i32::from_le_bytes(
                                self.buffer[pos - 16..pos - 12].try_into().unwrap_or([0; 4]),
                            ) as usize;
                            let image_width = i32::from_le_bytes(
                                self.buffer[pos - 12..pos - 8].try_into().unwrap_or([0; 4]),
                            ) as usize;
                            let image_height = i32::from_le_bytes(
                                self.buffer[pos - 8..pos - 4].try_into().unwrap_or([0; 4]),
                            ) as usize;
                            let image_format = i32::from_le_bytes(
                                self.buffer[pos - 4..pos].try_into().unwrap_or([0; 4]),
                            ) as u8;

                            if self.buffer.len() < pos + 8 + image_size {
                                break; // Wait for full image payload
                            }

                            // Drain any trash/misaligned bytes before the image header.
                            self.buffer.drain(..pos - 20);
                            // Drain the 28-byte header.
                            self.buffer.drain(..28);
                            // Drain the image data.
                            let image_data = self.buffer.drain(..image_size).collect::<Vec<u8>>();
                            self.latest_image = Some(VofaImage {
                                id: image_id,
                                width: image_width,
                                height: image_height,
                                format: image_format,
                                data: image_data,
                            });
                        } else {
                            // Process normal float frame
                            let frame_bytes = self.buffer.drain(..pos + 4).collect::<Vec<u8>>();
                            if pos % 4 == 0 {
                                let val_bytes = &frame_bytes[..pos];
                                if self.mode == VofaMode::JustFloat {
                                    let mut vals = Vec::new();
                                    for chunk in val_bytes.chunks_exact(4) {
                                        let val = f32::from_le_bytes(chunk.try_into().unwrap());
                                        vals.push(val);
                                    }
                                    if !vals.is_empty() {
                                        frames.push(vals);
                                    }
                                } else {
                                    // IndexFloat
                                    if val_bytes.len() >= 4 {
                                        let start_index_val =
                                            f32::from_le_bytes(val_bytes[0..4].try_into().unwrap());
                                        let start_index = start_index_val as usize;
                                        let data_bytes = &val_bytes[4..];
                                        let data_count = data_bytes.len() / 4;
                                        // Guard against massive index inputs to prevent OOM
                                        if start_index + data_count < 2000 {
                                            if self.index_float_buffer.len()
                                                < start_index + data_count
                                            {
                                                self.index_float_buffer
                                                    .resize(start_index + data_count, 0.0);
                                            }
                                            for (idx, chunk) in
                                                data_bytes.chunks_exact(4).enumerate()
                                            {
                                                let val =
                                                    f32::from_le_bytes(chunk.try_into().unwrap());
                                                self.index_float_buffer[start_index + idx] = val;
                                            }
                                            frames.push(self.index_float_buffer.clone());
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        break;
                    }
                }
                VofaMode::RawData => {
                    // Consume/discard buffer for waveform parsing
                    self.buffer.clear();
                    break;
                }
            }
        }
        frames
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.index_float_buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firewater_parser() {
        let mut parser = VofaParser::new(VofaMode::FireWater);

        // Test normal CSV
        let res = parser.feed(b"1.2,3.4\n");
        assert_eq!(res, vec![vec![1.2, 3.4]]);

        // Test CSV with label prefix
        let res = parser.feed(b"d:5.6,7.8\n");
        assert_eq!(res, vec![vec![5.6, 7.8]]);

        // Test CSV with spaces and CRLF
        let res = parser.feed(b"  my_label :  10.0 , 20.0 \r\n");
        assert_eq!(res, vec![vec![10.0, 20.0]]);
    }

    #[test]
    fn test_firewater_image_parser() {
        let mut parser = VofaParser::new(VofaMode::FireWater);

        // Feed partial image data header and verify no image yet
        let res = parser.feed(b"image:0,6,2,3,0\nabcd");
        assert!(res.is_empty());
        assert!(parser.take_latest_image().is_none());

        // Feed the rest of the image data
        let res = parser.feed(b"ef");
        assert!(res.is_empty());

        let img = parser.take_latest_image().expect("Should parse image");
        assert_eq!(img.id, 0);
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 3);
        assert_eq!(img.format, 0);
        assert_eq!(img.data, b"abcdef");
    }

    #[test]
    fn test_justfloat_image_parser() {
        let mut parser = VofaParser::new(VofaMode::JustFloat);

        // Pre-frame variables
        let img_id: i32 = 1;
        let img_size: i32 = 6;
        let img_width: i32 = 2;
        let img_height: i32 = 3;
        let img_format: i32 = 24; // Grayscale8

        let mut buf = Vec::new();
        buf.extend_from_slice(&img_id.to_le_bytes());
        buf.extend_from_slice(&img_size.to_le_bytes());
        buf.extend_from_slice(&img_width.to_le_bytes());
        buf.extend_from_slice(&img_height.to_le_bytes());
        buf.extend_from_slice(&img_format.to_le_bytes());
        buf.extend_from_slice(&[0x00, 0x00, 0x80, 0x7F]); // First NaN
        buf.extend_from_slice(&[0x00, 0x00, 0x80, 0x7F]); // Second NaN

        // Feed partial pre-frame data and verify no image yet
        let res = parser.feed(&buf[..buf.len() - 4]); // omit last 4 bytes of NaN
        assert!(res.is_empty());
        assert!(parser.take_latest_image().is_none());

        // Feed rest of preframe and partial payload (4 bytes of b"abcdef")
        let mut part2 = Vec::new();
        part2.extend_from_slice(&[0x00, 0x00, 0x80, 0x7F]); // complete header
        part2.extend_from_slice(b"abcd");
        let res = parser.feed(&part2);
        assert!(res.is_empty());
        assert!(parser.take_latest_image().is_none());

        // Feed final 2 bytes of payload
        let res = parser.feed(b"ef");
        assert!(res.is_empty());

        let img = parser.take_latest_image().expect("Should parse image");
        assert_eq!(img.id, 1);
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 3);
        assert_eq!(img.format, 24);
        assert_eq!(img.data, b"abcdef");
    }

    #[test]
    fn test_indexfloat_image_parser() {
        let mut parser = VofaParser::new(VofaMode::IndexFloat);

        // IndexFloat should support image parsing identically to JustFloat
        let img_id: i32 = 2;
        let img_size: i32 = 4;
        let img_width: i32 = 2;
        let img_height: i32 = 2;
        let img_format: i32 = 24;

        let mut buf = Vec::new();
        buf.extend_from_slice(&img_id.to_le_bytes());
        buf.extend_from_slice(&img_size.to_le_bytes());
        buf.extend_from_slice(&img_width.to_le_bytes());
        buf.extend_from_slice(&img_height.to_le_bytes());
        buf.extend_from_slice(&img_format.to_le_bytes());
        buf.extend_from_slice(&[0x00, 0x00, 0x80, 0x7F]); // First NaN
        buf.extend_from_slice(&[0x00, 0x00, 0x80, 0x7F]); // Second NaN
        buf.extend_from_slice(b"test");

        let res = parser.feed(&buf);
        assert!(res.is_empty());
        let img = parser
            .take_latest_image()
            .expect("Should parse image in IndexFloat mode");
        assert_eq!(img.id, 2);
        assert_eq!(img.data, b"test");
    }

    #[test]
    fn test_justfloat_resync_trash() {
        let mut parser = VofaParser::new(VofaMode::JustFloat);
        // Send 2 trash bytes, then a valid float frame (1.0f32 + NaN)
        let mut buf = vec![0x11, 0x22];
        buf.extend_from_slice(&1.0f32.to_le_bytes());
        buf.extend_from_slice(&[0x00, 0x00, 0x80, 0x7F]);

        let res = parser.feed(&buf);
        // Since it had 2 trash bytes, the frame is misaligned (pos = 6, 6 % 4 != 0)
        // It should be drained and discarded (no frame added)
        assert!(res.is_empty());

        // Send a properly aligned frame next
        let mut buf2 = Vec::new();
        buf2.extend_from_slice(&2.5f32.to_le_bytes());
        buf2.extend_from_slice(&[0x00, 0x00, 0x80, 0x7F]);
        let res2 = parser.feed(&buf2);
        assert_eq!(res2, vec![vec![2.5]]);
    }
}
