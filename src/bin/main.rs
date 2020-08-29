extern crate gecko_audio_ctrl;
use gecko_audio_ctrl::client_ctrl;
use gecko_audio_ctrl::zeroconf;
use gecko_audio_ctrl::client_state::ClientManager;

fn main() {
    let client_manager = ClientManager::new();
    zeroconf::start(9000);
    client_ctrl::run(9000, client_manager);
}
