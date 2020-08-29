use serde::{Deserialize, Serialize};

// Messages by client

#[derive(Deserialize, Clone)]
pub struct Hello {
    pub client_name: String,
}

#[derive(Deserialize, Clone)]
pub struct BatteryLevel {
    pub level: f64,
    pub is_charging: bool,
}

#[derive(Deserialize, Clone)]
pub struct LogMsg {
    pub message: String,
}

// Messages by both

#[derive(Deserialize, Serialize, Clone)]
pub struct DisplayName {
    pub display_name: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AudioStream {
    pub recv_audio_port: u16,
    pub recv_repair_port: u16,
    pub send_audio_port: u16,
    pub send_repair_port: u16,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MuteAudio {
    pub send_mute: bool,
    pub recv_mute: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TransmitAudio {
    pub send_audio: bool,
    pub recv_audio: bool,
}

// Messages by server

#[derive(Serialize, Clone)]
pub struct BatLogInterval {
    pub battery_log_interval_secs: Option<u32>,
}