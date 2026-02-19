use crate::{
    model::{ChatMessage, KeyResult, Model},
    view,
};
use messaging::{ClientMessage, ServerMessage};
use ratatui::DefaultTerminal;
use tokio::sync::mpsc;

const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis((1000. / 60.) as u64); // 60fps

fn app(
    username: &str,
    tx: mpsc::Sender<ClientMessage>,
    mut rx: mpsc::Receiver<ServerMessage>,
    terminal: &mut DefaultTerminal,
) -> std::io::Result<()> {
    let mut model = Model::new(username);

    loop {
        terminal.draw(|frame| {
            view::view(&model, frame);
        })?;

        if crossterm::event::poll(POLL_INTERVAL)? {
            let event = crossterm::event::read()?;
            if let Some(key_event) = event.as_key_event() {
                match model.on_key_press(key_event.code) {
                    KeyResult::None => {}
                    KeyResult::SendChat(msg) => {
                        if let Err(e) = tx.blocking_send(ClientMessage::SendChatMessage {
                            recipient: msg.username,
                            message: msg.message,
                        }) {
                            model.chat_error = format!("Failed to post chat message: {e}");
                        }
                    }
                    KeyResult::Quit => break Ok(()),
                }
            }
        }

        if let Ok(ServerMessage::RecvChatMessage { from, message }) = rx.try_recv() {
            model.chat_history.push(ChatMessage {
                username: from,
                message,
            });
        }
    }
}

pub fn run(username: &str, tx: mpsc::Sender<ClientMessage>, rx: mpsc::Receiver<ServerMessage>) {
    if let Err(e) = color_eyre::install() {
        eprintln!("Failed to init color_eyre: {e}");
        std::process::exit(1);
    }

    if let Err(e) = ratatui::run(|terminal| app(username, tx, rx, terminal)) {
        eprintln!("Failed to run ratatui app: {e}");
        std::process::exit(1);
    }
}
