mod pixoo_64;

use clap::Parser;
use pixoo_64::{DefaultResponse, Frame, ResetGifIdRequest, Rgb, SendAnimationRequest};
use png::{BitDepth, ColorType};
use reqwest::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::{net::IpAddr, path::Path};

struct Device {
    address: IpAddr,
}

impl Device {
    pub fn new(address: IpAddr) -> Self {
        Self { address }
    }

    pub fn post_endpoint(&self) -> String {
        format!("http://{}/post", self.address)
    }
}

async fn send_request<B, R>(device: &Device, body: &B) -> Result<R>
where
    B: Serialize,
    R: DeserializeOwned,
{
    let client = reqwest::Client::new();
    client
        .post(device.post_endpoint())
        .json(body)
        .send()
        .await?
        .json()
        .await
}

#[derive(Debug, Parser)]
struct Args {
    /// The IP address of the device
    #[clap(long)]
    pub ip: String,

    /// The frames of the animation
    pub frames: Vec<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    let args = Args::parse();
    let ip = args.ip.parse::<IpAddr>().expect("Invalid IP address");
    let device = Device::new(ip);

    log::info!("connecting to {:?}", ip);

    let req = ResetGifIdRequest::new();
    let _: DefaultResponse = send_request(&device, &req).await.unwrap();

    let mut animation_builder = AnimationBuilder::new(args.frames.len());

    for frame_path in args.frames {
        log::debug!("Loaded frame");
        let (bytes, len) = load_png(frame_path).unwrap();
        let bytes = &bytes[..len];
        let mut frame = Frame::default();
        frame.fill_with_bytes(bytes);
        animation_builder.add_frame(frame);
    }

    let reqs = SendAnimationRequest::multi_frame(animation_builder.frames());
    for req in reqs {
        let _: DefaultResponse = send_request(&device, &req).await.unwrap();
    }
}

struct AnimationBuilder {
    frames: Vec<Frame>,
}

impl AnimationBuilder {
    fn new(frame_count: usize) -> Self {
        let frames = Vec::with_capacity(frame_count);
        Self { frames }
    }

    fn add_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    fn frames(&self) -> &Vec<Frame> {
        &self.frames
    }
}

fn load_png<P: AsRef<Path>>(path: P) -> std::io::Result<(Vec<u8>, usize)> {
    let decoder = png::Decoder::new(std::fs::File::open(path)?);
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let len = info.buffer_size();

    debug_assert!(info.width == 64);
    debug_assert!(info.height == 64);
    debug_assert!(info.bit_depth == BitDepth::Eight);
    debug_assert!(info.color_type == ColorType::Rgb);

    Ok((buf, len))
}

fn color_test(frame: &mut Frame) {
    for y in 0..64 {
        for x in 0..64 {
            let r = x * 4;
            let g = y * 4;
            let b = u8::min(x, y) * 4;
            let color = Rgb { r, g, b };
            frame.set_pixel(x as usize, y as usize, color);
        }
    }
}
