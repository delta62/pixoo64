use std::fmt::Debug;

use serde::{Deserialize, Deserializer, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetAllSettingRequest {
    command: &'static str,
}

impl Default for GetAllSettingRequest {
    fn default() -> Self {
        Self {
            command: "Channel/GetAllConf",
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ResetGifIdRequest {
    command: &'static str,
}

impl ResetGifIdRequest {
    pub fn new() -> Self {
        Self {
            command: "Draw/ResetHttpGifId",
        }
    }
}

fn int_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    struct NumberVisitor;

    impl<'de> serde::de::Visitor<'de> for NumberVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("unsigned integer")
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match v {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(E::custom("can only deserialize 0 or 1 into a boolean")),
            }
        }
    }

    deserializer.deserialize_i32(NumberVisitor)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetAllSettingsResponse {
    brightness: u8,
    #[serde(deserialize_with = "int_to_bool")]
    rotation_flag: bool,
    clock_time: u32,
    gallery_time: u32,
    single_galley_time: u32,
    power_on_channel_id: Channel,
    #[serde(deserialize_with = "int_to_bool")]
    gallery_show_time_flag: bool,
    cur_clock_id: u32,
    #[serde(deserialize_with = "int_to_bool")]
    time_24_flag: bool,
    #[serde(deserialize_with = "int_to_bool")]
    temperature_mode: bool,
    gyrate_angle: RotationAngle,
    #[serde(deserialize_with = "int_to_bool")]
    mirror_flag: bool,
    #[serde(deserialize_with = "int_to_bool")]
    light_switch: bool,
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
pub enum RotationAngle {
    Normal,
    Clockwise90,
    Clockwise180,
    Clockwise270,
}

#[derive(Debug, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum Channel {
    Faces,
    Cloud,
    Visualizer,
    Custom,
    BlackScreen,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SelectChannelRequest {
    command: &'static str,
    select_index: Channel,
}

impl SelectChannelRequest {
    pub fn new(channel: Channel) -> Self {
        Self {
            command: "Channel/SetIndex",
            select_index: channel,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetDeviceTimeRequest {
    command: &'static str,
}

impl GetDeviceTimeRequest {
    pub fn new() -> Self {
        Self {
            command: "Device/GetDeviceTime",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetDeviceTimeResponse {
    #[serde(rename = "UTCTime")]
    utc_time: u64,
    local_time: String,
}

#[derive(Debug, Deserialize)]
pub struct DefaultResponse {
    error_code: i32,
}

#[derive(Debug, Serialize_repr)]
#[repr(u8)]
enum PicWidth {
    P16 = 16,
    P32 = 32,
    P64 = 64,
}

impl Default for PicWidth {
    fn default() -> Self {
        PicWidth::P64
    }
}

#[derive(Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendAnimationRequest {
    command: &'static str,
    pic_num: usize,
    pic_width: PicWidth,
    pic_offset: usize,
    pic_id: u8,
    pic_speed: u32,
    pic_data: String,
}

impl SendAnimationRequest {
    pub fn single_frame(frame: &Frame) -> Self {
        Self {
            command: "Draw/SendHttpGif",
            pic_data: serialize_frame(frame),
            pic_num: 1,
            ..Default::default()
        }
    }

    pub fn multi_frame(frames: &Vec<Frame>) -> Vec<Self> {
        frames
            .iter()
            .enumerate()
            .map(|(i, frame)| Self {
                command: "Draw/SendHttpGif",
                pic_data: serialize_frame(frame),
                pic_num: frames.len(),
                pic_width: PicWidth::P64,
                pic_offset: i,
                pic_id: 1,
                pic_speed: 100,
            })
            .collect()
    }
}

fn serialize_frame(frame: &Frame) -> String {
    base64::encode(frame.as_ref())
}

#[derive(Clone, Copy, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct Frame {
    pixels: [u8; 64 * 64 * 3],
}

impl AsRef<[u8]> for Frame {
    fn as_ref(&self) -> &[u8] {
        &self.pixels
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            pixels: [0; 64 * 64 * 3],
        }
    }
}

impl Frame {
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Rgb) {
        debug_assert!(x < 64 || y < 64, "Pixel {},{} out of bounds", x, y);
        let base_addr = y * 64 * 3 + x * 3;

        self.pixels[base_addr + 0] = color.r;
        self.pixels[base_addr + 1] = color.g;
        self.pixels[base_addr + 2] = color.b;
    }

    pub fn fill(&mut self, color: Rgb) {
        for y in 0..64 {
            for x in 0..64 {
                self.set_pixel(x, y, color);
            }
        }
    }

    pub fn fill_with_bytes(&mut self, bytes: &[u8]) {
        self.pixels.copy_from_slice(bytes);
    }
}
