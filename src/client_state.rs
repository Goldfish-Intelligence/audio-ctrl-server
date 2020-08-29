use std::sync::{Arc, RwLock};
use crossbeam_channel::{Sender, Receiver, unbounded};
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct ClientState {
    pub client_name: Option<String>,
    pub battery_level: Option<f64>,
    pub is_charging: Option<bool>,
    pub display_name: Option<String>,
    pub recv_audio_port: Option<u16>,
    pub recv_repair_port: Option<u16>,
    pub send_audio_port: Option<u16>,
    pub send_repair_port: Option<u16>,
    pub send_mute: Option<bool>,
    pub recv_mute: Option<bool>,
    pub send_audio: Option<bool>,
    pub recv_audio: Option<bool>,
    pub battery_log_interval_secs: Option<u32>,
}

#[derive(Clone)]
pub enum ClientStateChange {
    ClientName(String),
    BatteryLevel(f64),
    IsCharging(bool),
    DisplayName(String),
    RecvAudioPort(u16),
    RecvRepairPort(u16),
    SendAudioPort(u16),
    SendRepairPort(u16),
    SendMute(bool),
    RecvMute(bool),
    SendAudio(bool),
    RecvAudio(bool),
    BatteryLogIntervalSecs(u32),
}

#[derive(Clone)]
pub struct ClientManager {
    // state of currently connected clients
    connected_clients: Arc<RwLock<HashMap<u16, ClientState>>>,
    // send session_id that was created, modified, deleted
    change_sender: Sender<(u16, Option<ClientStateChange>)>,
    // get session_id that was created, modified, deleted
    pub change_receiver: Receiver<(u16, Option<ClientStateChange>)>,
}

impl ClientManager {
    pub fn new() -> ClientManager {
        let (sender, receiver) = unbounded();

        ClientManager {
            connected_clients: Arc::new(Default::default()),
            change_sender: sender,
            change_receiver: receiver,
        }
    }

    pub fn new_client(&mut self, session_id: u16) {
        let mut connected_clients = self.connected_clients.write().unwrap();
        connected_clients.insert(session_id, Default::default());
        self.change_sender.send((session_id, None)).unwrap();
    }

    pub fn rm_client(&mut self, session_id: u16) {
        let mut connected_clients = self.connected_clients.write().unwrap();
        connected_clients.remove(&session_id);
        self.change_sender.send((session_id, None)).unwrap();
    }

    pub fn set_client_property(&mut self, session_id: u16, changed: ClientStateChange) -> Result<(), &'static str> {
        let mut connected_clients = self.connected_clients.write().unwrap();
        let client_state = match connected_clients.get_mut(&session_id) {
            Some(client_state) => client_state,
            None => Err("No session found")?,
        };

        let has_changed;

        match changed.clone() {
            ClientStateChange::ClientName(client_name) => {
                has_changed = client_state.client_name.as_ref() == Some(&client_name);
                client_state.client_name = Some(client_name);
            },
            ClientStateChange::BatteryLevel(battery_level) => {
                has_changed = client_state.battery_level.as_ref() == Some(&battery_level);
                client_state.battery_level = Some(battery_level)
            },
            ClientStateChange::IsCharging(is_charging) => {
                has_changed = client_state.is_charging.as_ref() == Some(&is_charging);
                client_state.is_charging = Some(is_charging)
            },
            ClientStateChange::DisplayName(display_name) => {
                has_changed = client_state.display_name.as_ref() == Some(&display_name);
                client_state.display_name = Some(display_name)
            },
            ClientStateChange::RecvAudioPort(recv_audio_port) => {
                has_changed = client_state.recv_audio_port.as_ref() == Some(&recv_audio_port);
                client_state.recv_audio_port = Some(recv_audio_port)
            },
            ClientStateChange::RecvRepairPort(recv_repair_port) => {
                has_changed = client_state.recv_repair_port.as_ref() == Some(&recv_repair_port);
                client_state.recv_repair_port = Some(recv_repair_port)
            },
            ClientStateChange::SendAudioPort(send_audio_port) => {
                has_changed = client_state.send_audio_port.as_ref() == Some(&send_audio_port);
                client_state.send_audio_port = Some(send_audio_port)
            },
            ClientStateChange::SendRepairPort(send_repair_port) => {
                has_changed = client_state.send_repair_port.as_ref() == Some(&send_repair_port);
                client_state.send_repair_port = Some(send_repair_port)
            },
            ClientStateChange::SendMute(send_mute) => {
                has_changed = client_state.send_mute.as_ref() == Some(&send_mute);
                client_state.send_mute = Some(send_mute)
            },
            ClientStateChange::RecvMute(recv_mute) => {
                has_changed = client_state.recv_mute.as_ref() == Some(&recv_mute);
                client_state.recv_mute = Some(recv_mute)
            },
            ClientStateChange::SendAudio(send_audio) => {
                has_changed = client_state.send_audio.as_ref() == Some(&send_audio);
                client_state.send_audio = Some(send_audio)
            },
            ClientStateChange::RecvAudio(recv_audio) => {
                has_changed = client_state.recv_audio.as_ref() == Some(&recv_audio);
                client_state.recv_audio = Some(recv_audio)
            },
            ClientStateChange::BatteryLogIntervalSecs(battery_log_interval_secs) => {
                has_changed = client_state.battery_log_interval_secs.as_ref() == Some(&battery_log_interval_secs);
                client_state.battery_log_interval_secs = Some(battery_log_interval_secs)
            },
        }
        if has_changed {
            self.change_sender.send((session_id, Some(changed))).unwrap();
        }
        Ok(())
    }

    pub fn get_client(&mut self, session_id: u16) -> Result<ClientState, &'static str> {
        let connected_clients = self.connected_clients.read().unwrap();
        match connected_clients.get(&session_id) {
            Some(client_state) => Ok((*client_state).clone()),
            None => Err("No session found")?,
        }
    }
}