use reqwest::{Response, Result};
use serde::{Deserialize, Serialize};
use serde_repr::Serialize_repr;
use std::net::IpAddr;

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

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct GetAllSettingRequest {
    command: String,
}

impl Default for GetAllSettingRequest {
    fn default() -> Self {
        Self {
            command: "Channel/GetAllConf".to_owned(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GetAllSettingsResponse {
    brightness: u8,
    rotation_flag: u8,
    clock_time: u8,
}

#[derive(Serialize_repr)]
#[repr(u8)]
enum Channel {
    Faces,
    Cloud,
    Visualizer,
    Custom,
    BlackScreen,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SelectChannelRequest {
    command: String,
    select_index: Channel,
}

impl SelectChannelRequest {
    fn new(channel: Channel) -> Self {
        Self {
            command: "Channel/SetIndex".to_owned(),
            select_index: channel,
        }
    }
}

async fn get_config(device: &Device) -> Result<GetAllSettingsResponse> {
    let client = reqwest::Client::new();
    let body = GetAllSettingRequest::default();
    client
        .post(device.post_endpoint())
        .json(&body)
        .send()
        .await?
        .json()
        .await
}

async fn select_channel(device: &Device, channel: Channel) -> Result<Response> {
    let client = reqwest::Client::new();
    let body = SelectChannelRequest::new(channel);
    println!("{}", serde_json::to_string(&body).unwrap());
    client.post(device.post_endpoint()).json(&body).send().await
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let device = Device::new("192.168.1.120".parse::<IpAddr>().unwrap());
    let config = get_config(&device).await.unwrap();
    println!("{:#?}", config);
    select_channel(&device, Channel::Faces).await.unwrap();
}
