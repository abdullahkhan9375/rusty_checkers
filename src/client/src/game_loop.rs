use crate::{
    input::{self, Input, KeyResult},
    model::ChatMessage,
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
    let mut input = Input::new(username);

    loop {
        terminal.draw(|frame| {
            view::view(&input, frame);
        })?;

        if crossterm::event::poll(POLL_INTERVAL)? {
            let event = crossterm::event::read()?;
            if let Some(key_event) = event.as_key_event() {
                match input.on_key_press(key_event.code) {
                    KeyResult::None => {}
                    KeyResult::SendChat(msg) => {
                        if let Err(e) = tx.blocking_send(ClientMessage::SendChatMessage {
                            recipient: msg.username,
                            message: msg.message,
                        }) {
                            input.model.chat_error = format!("Failed to post chat message: {e}");
                        }
                    }
                    KeyResult::Quit => break Ok(()),
                    KeyResult::NewGame(plugin_type) => {
                        if let Err(e) = tx.blocking_send(ClientMessage::RequestNewGame(plugin_type))
                        {
                            input.model.chat_error =
                                format!("Failed to post new game request: {e}");
                        }
                    }
                    KeyResult::EnterGame { game_id } => {
                        if let Err(e) =
                            tx.blocking_send(ClientMessage::RequestEnterGame { game_id })
                        {
                            input.model.chat_error =
                                format!("Failed to post new game request: {e}");
                        }
                    }
                    KeyResult::SendGameMsg { game_id, msg } => {
                        if let Err(e) =
                            tx.blocking_send(ClientMessage::RequestGameUpdate { game_id, msg })
                        {
                            input.model.chat_error =
                                format!("Failed to post send game update: {e}");
                        }
                    }
                }
            }
        }

        let mut processed_count = 0;
        while processed_count < 128
            && let Ok(msg) = rx.try_recv()
        {
            match msg {
                ServerMessage::LoginSuccess => {}
                ServerMessage::LoginFailed { .. } => {}
                ServerMessage::NewGameCreated {
                    game_id,
                    plugin_type,
                } => {
                    input.model.active_games.retain(|(id, _)| *id != game_id);
                    input.model.active_games.push((game_id, plugin_type));
                }
                ServerMessage::RecvChatMessage { from, message } => {
                    input.model.chat_history.push(ChatMessage {
                        username: from,
                        message,
                    });
                }
                ServerMessage::GameUpdate { game_id, msg } => {
                    if let Some((id, state)) = &mut input.model.current_game
                        && *id == game_id
                    {
                        state.recv_msg(&msg);
                    }
                }
                ServerMessage::GameEntered {
                    game_id,
                    plugin_type,
                    game_state_msg,
                } => {
                    let mut client = plugin_type.new_client();
                    client.recv_msg(&game_state_msg);

                    input.model.current_game = Some((game_id, client));
                    input.game_input_state = input::GameInput::TickTackToe { selected_cell: 0 };
                }
            }
            processed_count += 1;
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
