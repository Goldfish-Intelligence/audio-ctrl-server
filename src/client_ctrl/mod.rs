// set client state in client_state
// used by conf_store

use crate::client_state::ClientManager;

mod messages;
mod tcp_json;

pub fn run(port: u16, client_manager: ClientManager) {
    tcp_json::run(port, client_manager);
}
