use crate::client_state::ClientManager;
use std::sync::mpsc::{channel, Sender};
use notify::{watcher, Watcher, RecursiveMode};
use std::time::Duration;
use std::{thread, fs};
use std::path::PathBuf;
use notify::DebouncedEvent;

enum ConfChange {
    FileChanged,
}

pub fn run (client_manager: ClientManager) {
    let (file_tx, file_rx) = channel();
    let mut watcher = watcher(file_tx, Duration::from_secs(1)).unwrap();
    watcher.watch("./client_config/", RecursiveMode::NonRecursive).unwrap();

    let (conf_tx, conf_rx) = channel();

    let file_conf_tx = conf_tx.clone();
    thread::spawn(move || {
        loop {
            match file_rx.recv().unwrap() {
                DebouncedEvent::NoticeWrite(path) |
                DebouncedEvent::Create(path) |
                DebouncedEvent::Write(path) |
                DebouncedEvent::Rename(_, path) => {
                    file_conf_tx.send(path).unwrap()
                },
                DebouncedEvent::Rescan => scan_dir(&file_conf_tx),
                _ => {}
            }
        }
    });

    thread::spawn(move || {
       loop {
           let path = conf_rx.recv().unwrap();

           match path.extension() {
               None => continue,
               Some(ext) => if ext != "json" {
                   continue
               }
           }

           let client_name = path.file_stem().unwrap().to_str().unwrap().into_string();
           
       }
    });

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