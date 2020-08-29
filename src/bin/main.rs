extern crate gecko_audio_ctrl;
use gecko_audio_ctrl::{client_ctrl, ui};
use gecko_audio_ctrl::zeroconf;
use gecko_audio_ctrl::client_state::{ClientManager, ClientStateChange};
use std::thread;

fn main() {
    let mut client_manager = ClientManager::new();
    zeroconf::start(9000);

    let tcp_client_manager = client_manager.clone();
    thread::spawn(move || {
        client_ctrl::run(9000, tcp_client_manager);
    });

    let ui_client_manager = client_manager.clone();
    ui::run(ui_client_manager);
}
