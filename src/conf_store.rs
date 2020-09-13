use crate::client_state::{ClientManager, ClientState, ClientStateChange};
use notify::DebouncedEvent;
use notify::{watcher, RecursiveMode, Watcher};
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;
use std::{fs, thread};

pub fn run(client_manager: ClientManager) {
    let (file_tx, file_rx) = channel();
    let mut watcher = watcher(file_tx, Duration::from_secs(1)).unwrap();
    watcher
        .watch("./client_config/", RecursiveMode::NonRecursive)
        .unwrap();

    let (conf_tx, conf_rx) = channel();

    let file_conf_tx = conf_tx.clone();
    thread::spawn(move || loop {
        match file_rx.recv().unwrap() {
            DebouncedEvent::NoticeWrite(path)
            | DebouncedEvent::Create(path)
            | DebouncedEvent::Write(path)
            | DebouncedEvent::Rename(_, path) => file_conf_tx.send(path).unwrap(),
            DebouncedEvent::Rescan => scan_dir(&file_conf_tx),
            _ => {}
        }
    });

    let mut client_manager_f_change = client_manager.clone();

    thread::spawn(move || {
        loop {
            let path = conf_rx.recv().unwrap();

            let client_config = match read_config_file(path) {
                Ok(conf) => conf,
                Err(_e) => {
                    //log(e)
                    continue;
                }
            };

            let client_name = match client_config.client_name.clone() {
                Some(client_name) => client_name,
                None => {
                    // log("no client name set in config, ignoring")
                    continue;
                }
            };

            let session_id = match client_manager_f_change.get_session_id(&client_name) {
                Some(session_id) => session_id,
                None => continue, // client not connected maybe log as trace
            };

            if let Err(_e) = client_manager_f_change.update_client(session_id, client_config) {
                //log(e)
            }
        }
    });

    let mut client_manager_state_change = client_manager.clone();
    let client_state_change_receiver = client_manager.get_change_receiver();

    loop {
        let (session_id, state_change) = client_state_change_receiver.recv().unwrap();
        if let Err(_e) =
            handle_client_state_change(session_id, state_change, &mut client_manager_state_change)
        {

            //log(e)
        }
    }
}

fn read_config_file(file_path: PathBuf) -> Result<ClientState, String> {
    match file_path.extension() {
        None => Err("no file extension")?,
        Some(ext) => {
            if ext != "json" {
                Err("No json file")?
            }
        }
    }

    let file = OpenOptions::new()
        .read(true)
        .create(true)
        .write(true)
        .open(file_path);

    let file = match file {
        Ok(res) => res,
        Err(e) => Err(e.to_string())?,
    };
    let reader = BufReader::new(file);

    match serde_json::from_reader(reader) {
        Ok(res) => Ok(res),
        Err(e) => Err(e.to_string()),
    }
}

fn handle_client_state_change(
    session_id: SocketAddr,
    state_change: ClientStateChange,
    client_manager: &mut ClientManager,
) -> Result<(), String> {
    match state_change {
        ClientStateChange::Remove(_) | ClientStateChange::Add => Ok(()), // we do not hold any state regarding connected clients
        ClientStateChange::ClientName(client_name) => {
            let mut conf_path = PathBuf::from("./client_config/");
            let mut filename = client_name.clone();
            filename.push_str(".json");
            conf_path.push(filename);
            let client_config = read_config_file(conf_path)?;
            client_manager.update_client(session_id, client_config)
        }
        _ => {
            let client_config = client_manager.get_client(session_id)?;
            let client_name = match client_config.client_name.clone() {
                Some(client_name) => client_name,
                None => Err("Client name not set")?,
            };
            let mut conf_path = PathBuf::from("./client_config/");
            let mut filename = client_name.clone();
            filename.push_str(".json");
            conf_path.push(filename);
            let file = match File::create(conf_path) {
                Ok(res) => res,
                Err(e) => Err(e.to_string())?,
            };

            match serde_json::to_writer_pretty(file, &client_config) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        }
    }
}

fn scan_dir(file_tx: &Sender<PathBuf>) {
    for entry in fs::read_dir("./client_config/").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            continue;
        } else {
            file_tx.send(path).unwrap();
        }
    }
}
