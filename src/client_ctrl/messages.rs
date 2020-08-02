use serde::{Deserialize};

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

// Messages by server