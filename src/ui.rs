use crate::client_state::ClientManager;
use crate::ui::Redraw::ClientState;
use std::sync::mpsc;
use std::time::Duration;
use std::{io, thread};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Layout};
use tui::widgets::{Block, Borders, Row, Table};
use tui::Terminal;

enum Redraw {
    Key(Key),
    ClientState,
}

pub fn run(client_manager: ClientManager) {
    let stdout = io::stdout().into_raw_mode().unwrap();
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let (tx, rx) = mpsc::channel();

    let key_tx = tx.clone();
    thread::spawn(move || {
        let stdin = io::stdin();
        for c in stdin.keys() {
            key_tx.send(Redraw::Key(c.unwrap())).unwrap();
        }
    });

    let client_tx = tx.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(2));
        client_tx.send(ClientState).unwrap();
    });

    let mut rows = vec![vec![]];

    loop {
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(f.size());

                let table = Table::new(
                    [
                        "Name",
                        "Power (Last update)",
                        "Audio In",
                        "Audio Out",
                        "Port In (Repair)",
                        "Port Out (Repair)",
                        "Last ping",
                    ]
                    .iter(),
                    rows.iter().map(|i| Row::Data(i.iter())),
                )
                .block(
                    Block::default()
                        .title("Connected Clients")
                        .borders(Borders::ALL),
                )
                .widths(&[
                    Constraint::Percentage(14),
                    Constraint::Percentage(14),
                    Constraint::Percentage(14),
                    Constraint::Percentage(14),
                    Constraint::Percentage(14),
                    Constraint::Percentage(14),
                    Constraint::Percentage(14),
                ]);
                f.render_widget(table, chunks[0]);
            })
            .unwrap();

        if let Ok(input) = rx.recv() {
            match input {
                Redraw::Key(Key::Char('q')) => {
                    break;
                }
                Redraw::ClientState => {
                    rows.clear();
                    rows.extend(client_manager.get_all_clients().iter().map(|i| {
                        vec![
                            // Name
                            match &i.display_name {
                                Some(display_name) => display_name.to_string(),
                                None => "Not reported".to_string(),
                            },
                            // Power (Last update)
                            match i.is_charging {
                                Some(true) => "External (-)".to_string(),
                                Some(false) => match i.battery_level {
                                    Some(level) => format!("{:1}% (-)", level * 100.0),
                                    None => "Internal (-)".to_string(),
                                },
                                None => "Not reported".to_string(),
                            },
                            // Audio In
                            match i.send_audio {
                                Some(true) => match i.send_mute {
                                    Some(true) => "YES (muted)".to_string(),
                                    Some(false) => "YES (not muted)".to_string(),
                                    None => "YES (muted?)".to_string(),
                                },
                                Some(false) => match i.send_mute {
                                    Some(true) => "NO (muted)".to_string(),
                                    Some(false) => "NO (not muted)".to_string(),
                                    None => "NO (muted?)".to_string(),
                                },
                                None => match i.send_mute {
                                    Some(true) => "? (muted)".to_string(),
                                    Some(false) => "? (not muted)".to_string(),
                                    None => "? (muted?)".to_string(),
                                },
                            },
                            // Audio Out
                            match i.recv_audio {
                                Some(true) => match i.recv_mute {
                                    Some(true) => "YES (muted)".to_string(),
                                    Some(false) => "YES (not muted)".to_string(),
                                    None => "YES (muted?)".to_string(),
                                },
                                Some(false) => match i.recv_mute {
                                    Some(true) => "NO (muted)".to_string(),
                                    Some(false) => "NO (not muted)".to_string(),
                                    None => "NO (muted?)".to_string(),
                                },
                                None => match i.recv_mute {
                                    Some(true) => "? (muted)".to_string(),
                                    Some(false) => "? (not muted)".to_string(),
                                    None => "? (muted?)".to_string(),
                                },
                            },
                            // Port In (Repair)
                            format!(
                                "{} ({})",
                                i.send_audio_port
                                    .map(|f| f.to_string())
                                    .unwrap_or("?".to_string()),
                                i.send_repair_port
                                    .map(|f| f.to_string())
                                    .unwrap_or("?".to_string())
                            ),
                            // Port Out (Repair)
                            format!(
                                "{} ({})",
                                i.recv_audio_port
                                    .map(|f| f.to_string())
                                    .unwrap_or("?".to_string()),
                                i.recv_repair_port
                                    .map(|f| f.to_string())
                                    .unwrap_or("?".to_string())
                            ),
                            // Last Ping
                            "-".to_string(),
                        ]
                    }));
                }
                _ => {}
            }
        }
    }
}
