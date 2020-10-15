use crate::client_messages::{
    AudioStream, BatLogInterval, BatteryLevel, DisplayName, Hello, LogMsg, MuteAudio, TransmitAudio,
};
use crate::client_state::{ClientManager, ClientStateChange};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};
use std::{io, thread};
use std::io::Write;

#[derive(Deserialize, Clone, Debug)]
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

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type")]
enum MessagesFromServer {
    //Ping,
    DisplayName(DisplayName),
    AudioStream(AudioStream),
    MuteAudio(MuteAudio),
    TransmitAudio(TransmitAudio),
    BatLogInterval(BatLogInterval),
}

pub fn run(port: u16, client_manager: ClientManager) {
    //let listener_v4 = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
    let listener_v6 = TcpListener::bind(format!("[::]:{}", port)).unwrap();

    let send_streams: Arc<RwLock<HashMap<SocketAddr, TcpStream>>> = Default::default();

    let state_change_send_streams = send_streams.clone();
    let state_change_client_manager = client_manager.clone();
    thread::spawn(move || {
        handle_client_state_change(state_change_send_streams, state_change_client_manager);
    });

    let (tx, rx) = channel();

    //let v4_tx = tx.clone();
    //thread::spawn(move || {
    //    for stream in listener_v4.incoming() {
    //        v4_tx.send(stream).unwrap();
    //    }
    //});

    let v6_tx = tx.clone();
    thread::spawn(move || {
        for stream in listener_v6.incoming() {
            v6_tx.send(stream).unwrap();
        }
    });

    loop {
        let stream = rx.recv().unwrap();
        let client_manager = client_manager.clone();
        let send_streams = send_streams.clone();
        thread::spawn(move || {
            if let Err(_e) = handle_client(send_streams, stream, client_manager.clone()) {
                // log e
            }
        });
    }
}

fn handle_client(
    send_streams: Arc<RwLock<HashMap<SocketAddr, TcpStream>>>,
    stream: io::Result<TcpStream>,
    mut client_manager: ClientManager,
) -> Result<(), String> {
    let session_id = match stream.as_ref() {
        Ok(stream) => match stream.peer_addr() {
            Ok(addr) => addr.clone(),
            Err(e) => Err(e.to_string())?,
        },
        Err(e) => Err(e.to_string())?,
    };

    let receive_stream = match stream {
        Ok(stream) => stream,
        Err(e) => Err(e.to_string())?,
    };

    let send_stream = match receive_stream.try_clone() {
        Ok(stream) => stream,
        Err(e) => Err(e.to_string())?,
    };

    {
        // new scope so, that write lock drops after insertion
        let mut send_streams = send_streams.write().unwrap();
        send_streams.insert(session_id, send_stream);
    }

    client_manager.new_client(session_id);

    let json_stream = Deserializer::from_reader(receive_stream).into_iter::<MessageToServer>();

    for message in json_stream {
        let mes = match message {
            Ok(mes) => mes,
            Err(_e) => {
                //log e
                break;
            }
        };
        let res = handle_client_message_received(session_id, &mut client_manager, mes);
        if let Err(e) = res {
            println!("{}, disconnecting client!", e);
            break;
        }
    }
    client_manager.rm_client(session_id);
    Ok(())
}

fn handle_client_message_received(
    session_id: SocketAddr,
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

fn handle_client_state_change(
    send_streams: Arc<RwLock<HashMap<SocketAddr, TcpStream>>>,
    mut client_manager: ClientManager,
) {
    let client_state_change_receiver = client_manager.get_change_receiver();
    loop {
        let (session_id, event) = client_state_change_receiver.recv().unwrap();
        let state = match client_manager.get_client(session_id) {
            Ok(state) => state,
            Err(_) => continue, // probably disconnected
        };

        let msg = match event {
            ClientStateChange::Remove(_) => {
                send_streams.write().unwrap().remove(&session_id);
                None
            }
            ClientStateChange::Add => None,

            ClientStateChange::ClientName(_) => None,
            ClientStateChange::BatteryLevel(_) => None,
            ClientStateChange::IsCharging(_) => None,
            ClientStateChange::DisplayName(display_name) => {
                Some(MessagesFromServer::DisplayName(DisplayName {
                    display_name,
                }))
            }
            ClientStateChange::RecvAudioPort(_)
            | ClientStateChange::RecvRepairPort(_)
            | ClientStateChange::SendAudioPort(_)
            | ClientStateChange::SendRepairPort(_) => {
                let recv_audio_port = match state.recv_audio_port {
                    Some(port) => port,
                    None => continue,
                };
                let recv_repair_port = match state.recv_repair_port {
                    Some(port) => port,
                    None => continue,
                };
                let send_audio_port = match state.send_audio_port {
                    Some(port) => port,
                    None => continue,
                };
                let send_repair_port = match state.send_repair_port {
                    Some(port) => port,
                    None => continue,
                };

                Some(MessagesFromServer::AudioStream(AudioStream {
                    recv_audio_port,
                    recv_repair_port,
                    send_audio_port,
                    send_repair_port,
                }))
            }
            ClientStateChange::SendMute(_) | ClientStateChange::RecvMute(_) => {
                let send_mute = match state.send_mute {
                    Some(mute) => mute,
                    None => continue,
                };
                let recv_mute = match state.recv_mute {
                    Some(mute) => mute,
                    None => continue,
                };

                Some(MessagesFromServer::MuteAudio(MuteAudio {
                    send_mute,
                    recv_mute,
                }))
            }
            ClientStateChange::SendAudio(_) | ClientStateChange::RecvAudio(_) => {
                let send_audio = match state.send_audio {
                    Some(send) => send,
                    None => continue,
                };
                let recv_audio = match state.recv_audio {
                    Some(send) => send,
                    None => continue,
                };

                Some(MessagesFromServer::TransmitAudio(TransmitAudio {
                    send_audio,
                    recv_audio,
                }))
            }
            ClientStateChange::BatteryLogIntervalSecs(battery_log_interval_secs) => {
                Some(MessagesFromServer::BatLogInterval(BatLogInterval {
                    battery_log_interval_secs,
                }))
            }
        };

        if let Some(msg) = msg {
            let mut send_streams = send_streams.write().unwrap();
            {
                let send_stream = send_streams.get_mut(&session_id).unwrap();

                if let Err(_e) = serde_json::to_writer(send_stream, &msg) {
                    // log e
                };
            }
            {
                send_streams.get_mut(&session_id).unwrap().write(b"\n").unwrap();
            }
        }
    }
}
