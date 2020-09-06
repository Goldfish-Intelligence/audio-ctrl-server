use crossbeam_channel::{unbounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

#[derive(Default, Clone, Deserialize, Serialize)]
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
    Add,
    Remove(ClientState),

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
    BatteryLogIntervalSecs(Option<u32>),
}

#[derive(Clone)]
pub struct ClientManager {
    // state of currently connected clients
    connected_clients: Arc<RwLock<HashMap<SocketAddr, ClientState>>>,
    // send session_id that was created, modified, deleted
    change_sender: Sender<(SocketAddr, ClientStateChange)>,
    // get session_id that was created, modified, deleted
    pub change_receiver: Receiver<(SocketAddr, ClientStateChange)>,
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

    pub fn new_client(&mut self, session_id: SocketAddr) {
        let mut connected_clients = self.connected_clients.write().unwrap();
        connected_clients.insert(session_id, Default::default());
        self.change_sender
            .send((session_id, ClientStateChange::Add))
            .unwrap();
    }

    pub fn rm_client(&mut self, session_id: SocketAddr) {
        let mut connected_clients = self.connected_clients.write().unwrap();
        let client_state = match connected_clients.get_mut(&session_id) {
            Some(client_state) => client_state.clone(),
            None => return,
        };
        connected_clients.remove(&session_id);
        self.change_sender
            .send((session_id, ClientStateChange::Remove(client_state.clone())))
            .unwrap();
    }

    pub fn set_client_property(
        &mut self,
        session_id: SocketAddr,
        state_change: ClientStateChange,
    ) -> Result<(), &'static str> {
        let mut connected_clients = self.connected_clients.write().unwrap();
        let client_state = match connected_clients.get_mut(&session_id) {
            Some(client_state) => client_state,
            None => Err("No session found")?,
        };

        let has_changed;

        match state_change.clone() {
            ClientStateChange::Add | ClientStateChange::Remove(_) => {
                return Err("Remove and Add not supported");
            }
            ClientStateChange::ClientName(client_name) => {
                has_changed = client_state.client_name.as_ref() == Some(&client_name);
                client_state.client_name = Some(client_name);
            }
            ClientStateChange::BatteryLevel(battery_level) => {
                has_changed = client_state.battery_level.as_ref() == Some(&battery_level);
                client_state.battery_level = Some(battery_level)
            }
            ClientStateChange::IsCharging(is_charging) => {
                has_changed = client_state.is_charging.as_ref() == Some(&is_charging);
                client_state.is_charging = Some(is_charging)
            }
            ClientStateChange::DisplayName(display_name) => {
                has_changed = client_state.display_name.as_ref() == Some(&display_name);
                client_state.display_name = Some(display_name)
            }
            ClientStateChange::RecvAudioPort(recv_audio_port) => {
                has_changed = client_state.recv_audio_port.as_ref() == Some(&recv_audio_port);
                client_state.recv_audio_port = Some(recv_audio_port)
            }
            ClientStateChange::RecvRepairPort(recv_repair_port) => {
                has_changed = client_state.recv_repair_port.as_ref() == Some(&recv_repair_port);
                client_state.recv_repair_port = Some(recv_repair_port)
            }
            ClientStateChange::SendAudioPort(send_audio_port) => {
                has_changed = client_state.send_audio_port.as_ref() == Some(&send_audio_port);
                client_state.send_audio_port = Some(send_audio_port)
            }
            ClientStateChange::SendRepairPort(send_repair_port) => {
                has_changed = client_state.send_repair_port.as_ref() == Some(&send_repair_port);
                client_state.send_repair_port = Some(send_repair_port)
            }
            ClientStateChange::SendMute(send_mute) => {
                has_changed = client_state.send_mute.as_ref() == Some(&send_mute);
                client_state.send_mute = Some(send_mute)
            }
            ClientStateChange::RecvMute(recv_mute) => {
                has_changed = client_state.recv_mute.as_ref() == Some(&recv_mute);
                client_state.recv_mute = Some(recv_mute)
            }
            ClientStateChange::SendAudio(send_audio) => {
                has_changed = client_state.send_audio.as_ref() == Some(&send_audio);
                client_state.send_audio = Some(send_audio)
            }
            ClientStateChange::RecvAudio(recv_audio) => {
                has_changed = client_state.recv_audio.as_ref() == Some(&recv_audio);
                client_state.recv_audio = Some(recv_audio)
            }
            ClientStateChange::BatteryLogIntervalSecs(battery_log_interval_secs) => {
                has_changed = client_state.battery_log_interval_secs == battery_log_interval_secs;
                client_state.battery_log_interval_secs = battery_log_interval_secs
            }
        }
        if has_changed {
            self.change_sender.send((session_id, state_change)).unwrap();
        }
        Ok(())
    }

    pub fn update_client(
        &mut self,
        session_id: SocketAddr,
        changed: ClientState,
    ) -> Result<(), String> {
        let mut connected_clients = self.connected_clients.write().unwrap();
        let client_state = match connected_clients.get_mut(&session_id) {
            Some(client_state) => client_state.clone(),
            None => Err(String::from("No session found"))?,
        };

        connected_clients.insert(session_id, changed.clone());

        if let Some(client_name) = changed.client_name {
            if client_state.client_name.as_deref() != Some(&client_name) {
                self.change_sender
                    .send((session_id, ClientStateChange::ClientName(client_name)))
                    .unwrap();
            }
        }
        if let Some(battery_level) = changed.battery_level {
            if client_state.battery_level != changed.battery_level {
                self.change_sender
                    .send((session_id, ClientStateChange::BatteryLevel(battery_level)))
                    .unwrap();
            }
        }
        if let Some(is_charging) = changed.is_charging {
            if client_state.is_charging != changed.is_charging {
                self.change_sender
                    .send((session_id, ClientStateChange::IsCharging(is_charging)))
                    .unwrap();
            }
        }
        if let Some(display_name) = changed.display_name {
            if client_state.display_name.as_deref() != Some(&display_name) {
                self.change_sender
                    .send((session_id, ClientStateChange::DisplayName(display_name)))
                    .unwrap();
            }
        }
        if let Some(recv_audio_port) = changed.recv_audio_port {
            if client_state.recv_audio_port != changed.recv_audio_port {
                self.change_sender
                    .send((
                        session_id,
                        ClientStateChange::RecvAudioPort(recv_audio_port),
                    ))
                    .unwrap();
            }
        }
        if let Some(recv_repair_port) = changed.recv_repair_port {
            if client_state.recv_repair_port != changed.recv_repair_port {
                self.change_sender
                    .send((
                        session_id,
                        ClientStateChange::RecvRepairPort(recv_repair_port),
                    ))
                    .unwrap();
            }
        }
        if let Some(send_audio_port) = changed.send_audio_port {
            if client_state.send_audio_port != changed.send_audio_port {
                self.change_sender
                    .send((
                        session_id,
                        ClientStateChange::SendAudioPort(send_audio_port),
                    ))
                    .unwrap();
            }
        }
        if let Some(send_repair_port) = changed.send_repair_port {
            if client_state.send_repair_port != changed.send_repair_port {
                self.change_sender
                    .send((
                        session_id,
                        ClientStateChange::SendRepairPort(send_repair_port),
                    ))
                    .unwrap();
            }
        }
        if let Some(send_mute) = changed.send_mute {
            if client_state.send_mute != changed.send_mute {
                self.change_sender
                    .send((session_id, ClientStateChange::SendMute(send_mute)))
                    .unwrap();
            }
        }
        if let Some(recv_mute) = changed.recv_mute {
            if client_state.recv_mute != changed.recv_mute {
                self.change_sender
                    .send((session_id, ClientStateChange::RecvMute(recv_mute)))
                    .unwrap();
            }
        }
        if let Some(send_audio) = changed.send_audio {
            if client_state.send_audio != changed.send_audio {
                self.change_sender
                    .send((session_id, ClientStateChange::SendAudio(send_audio)))
                    .unwrap();
            }
        }
        if let Some(recv_audio) = changed.recv_audio {
            if client_state.recv_audio != changed.recv_audio {
                self.change_sender
                    .send((session_id, ClientStateChange::RecvAudio(recv_audio)))
                    .unwrap();
            }
        }

        if client_state.battery_log_interval_secs != changed.battery_log_interval_secs {
            self.change_sender
                .send((
                    session_id,
                    ClientStateChange::BatteryLogIntervalSecs(changed.battery_log_interval_secs),
                ))
                .unwrap();
        }

        Ok(())
    }

    pub fn get_client(&mut self, session_id: SocketAddr) -> Result<ClientState, &'static str> {
        let connected_clients = self.connected_clients.read().unwrap();
        match connected_clients.get(&session_id) {
            Some(client_state) => Ok((*client_state).clone()),
            None => Err("No session found")?,
        }
    }

    pub fn get_session_id(&self, client_name: &str) -> Option<SocketAddr> {
        let connected_clients = self.connected_clients.read().unwrap();
        for (session_id, state) in connected_clients.iter() {
            if let Some(connected_client_name) = state.client_name.clone() {
                if connected_client_name == client_name {
                    return Some(session_id.clone());
                }
            }
        }

        None
    }

    pub fn get_all_clients(&self) -> Vec<ClientState> {
        let connected_clients = self.connected_clients.read().unwrap();
        connected_clients.values().cloned().collect()
    }
}
