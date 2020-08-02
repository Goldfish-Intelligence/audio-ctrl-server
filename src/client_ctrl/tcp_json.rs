use std::net::{TcpListener, TcpStream};

use serde::{Deserialize};
use serde_json::{Deserializer};

use super::messages::{Hello, BatteryLevel, LogMsg};

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
enum MessageToServer {
    Hello(Hello),
    Ping,
    BatteryLevel(BatteryLevel),
    LogMsg(LogMsg),
}

pub fn start() {
    let listener = TcpListener::bind("127.0.0.1:9000").unwrap();

    for stream in listener.incoming() {
        handle_client(stream.unwrap());
    }
}

fn handle_client(stream: TcpStream) {
    let json_stream = Deserializer::from_reader(stream).into_iter::<MessageToServer>();
    let mut client_name = String::from("UNKNOWN");

    for message in json_stream {
        match message.unwrap() {
            MessageToServer::Hello(hello) => {
                println!("Hello from: '{}'", hello.client_name);
                client_name = hello.client_name;
            },
            MessageToServer::Ping => {
                println!("Ping!");
            },
            MessageToServer::BatteryLevel(battery_level) => {
                println!("Charge: '{}', Is charging: '{}'", battery_level.level, battery_level.is_charging);
            },
            MessageToServer::LogMsg(log_msg) => {
                println!("LOG: '{}': {}", client_name, log_msg.message);
            },
        }
    }
}