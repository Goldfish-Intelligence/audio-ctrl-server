use std::net::{TcpListener, TcpStream};

use serde::Deserialize;
use serde_json::Deserializer;

use crate::client_ctrl::messages::{
    AudioStream, BatteryLevel, DisplayName, Hello, LogMsg, MuteAudio, TransmitAudio,
};
use crate::client_state::{ClientManager, ClientStateChange};
use std::thread;

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
enum MessageToServer {
    Hello(Hello),
    Ping,
    BatteryLevel(BatteryLevel),
    LogMsg(LogMsg),
    DisplayName(DisplayName),
    AudioStream(AudioStream),
    MuteAudio(MuteAudio),
    TransmitAudio(TransmitAudio),
}

pub fn run(port: u16, client_manager: ClientManager) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();

    for stream in listener.incoming() {
        let client_manager = client_manager.clone();
        thread::spawn(move || {
            handle_client(stream.unwrap(), client_manager.clone());
        });
    }
}

fn handle_client(stream: TcpStream, mut client_manager: ClientManager) {
    let session_id = stream.local_addr().unwrap().port();

    client_manager.new_client(session_id);

    let json_stream = Deserializer::from_reader(stream).into_iter::<MessageToServer>();

    for message in json_stream {
        let res = handle_client_message_received(session_id, &mut client_manager, message.unwrap());
        if let Err(e) = res {
            println!("{}, disconnecting client!", e);
        }
    }
    client_manager.rm_client(session_id);
}

fn handle_client_message_received(
    session_id: u16,
    client_manager: &mut ClientManager,
    message: MessageToServer,
) -> Result<(), &'static str> {
    match message {
        MessageToServer::Hello(hello) => client_manager
            .set_client_property(session_id, ClientStateChange::ClientName(hello.client_name)),
        MessageToServer::Ping => {
            println!("Ping from {}", session_id);
            Ok(())
        }
        MessageToServer::BatteryLevel(battery_level) => {
            client_manager.set_client_property(
                session_id,
                ClientStateChange::BatteryLevel(battery_level.level),
            )?;
            client_manager.set_client_property(
                session_id,
                ClientStateChange::IsCharging(battery_level.is_charging),
            )
        }
        MessageToServer::LogMsg(log_msg) => {
            println!("LOG: '{}': {}", session_id, log_msg.message);
            Ok(())
        }
        MessageToServer::DisplayName(display_name) => client_manager.set_client_property(
            session_id,
            ClientStateChange::DisplayName(display_name.display_name),
        ),
        MessageToServer::AudioStream(audio_stream) => {
            client_manager.set_client_property(
                session_id,
                ClientStateChange::RecvAudioPort(audio_stream.recv_audio_port),
            )?;
            client_manager.set_client_property(
                session_id,
                ClientStateChange::RecvRepairPort(audio_stream.recv_repair_port),
            )?;
            client_manager.set_client_property(
                session_id,
                ClientStateChange::SendAudioPort(audio_stream.send_audio_port),
            )?;
            client_manager.set_client_property(
                session_id,
                ClientStateChange::SendRepairPort(audio_stream.send_repair_port),
            )
        }
        MessageToServer::MuteAudio(mute_audio) => {
            client_manager.set_client_property(
                session_id,
                ClientStateChange::SendMute(mute_audio.send_mute),
            )?;
            client_manager.set_client_property(
                session_id,
                ClientStateChange::RecvMute(mute_audio.recv_mute),
            )
        }
        MessageToServer::TransmitAudio(transmit_audio) => {
            client_manager.set_client_property(
                session_id,
                ClientStateChange::SendAudio(transmit_audio.send_audio),
            )?;
            client_manager.set_client_property(
                session_id,
                ClientStateChange::RecvAudio(transmit_audio.recv_audio),
            )
        }
    }
}
