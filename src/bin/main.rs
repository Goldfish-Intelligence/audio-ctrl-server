extern crate gecko_audio_ctrl;
use gecko_audio_ctrl::client_state::ClientManager;
use gecko_audio_ctrl::tcp_json;
use gecko_audio_ctrl::ui;
use gecko_audio_ctrl::zeroconf;
use std::thread;

fn main() {
    let client_manager = ClientManager::new();
    zeroconf::start(9000);

    let tcp_client_manager = client_manager.clone();
    thread::spawn(move || {
        tcp_json::run(9000, tcp_client_manager);
    });

    let ui_client_manager = client_manager.clone();
    ui::run(ui_client_manager);
}
