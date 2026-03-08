use crate::model::{ChatMessage, Model};
use crossterm::event::KeyCode;
use plugins::PluginType;

#[derive(Clone, Copy)]
pub enum Mode {
    None,
    Chat,
    GameListing { selected_game_id: u64 },
    PlayGame,
    NewGame(PluginType),
}

pub enum KeyResult {
    None,
    Quit,
    SendChat(ChatMessage),
    SendGameMsg { game_id: u64, msg: String },
    NewGame(PluginType),
    EnterGame { game_id: u64 },
}

pub enum GameInput {
    None,
    TickTackToe { selected_cell: usize },
}

pub struct Input {
    pub mode: Mode,
    pub game_input_state: GameInput,
    pub model: Model,
}

impl Input {
    pub fn new(username: &str) -> Self {
        Self {
            mode: Mode::None,
            game_input_state: GameInput::None,
            model: Model::new(username),
        }
    }

    pub fn on_key_press(&mut self, key: KeyCode) -> KeyResult {
        if key == KeyCode::Esc {
            self.mode = Mode::None;
            return KeyResult::None;
        }

        match self.mode {
            Mode::None => self.on_key_press_none(key),
            Mode::Chat => self.on_key_press_chat(key),
            Mode::GameListing { selected_game_id } => {
                self.on_key_press_game_listing(selected_game_id, key)
            }
            Mode::PlayGame => self.on_key_press_play_game(key),
            Mode::NewGame(selected_plugin_type) => {
                self.on_key_press_new_game(selected_plugin_type, key)
            }
        }
    }

    fn on_key_press_none(&mut self, key: KeyCode) -> KeyResult {
        match key {
            KeyCode::Char('q') => KeyResult::Quit,
            KeyCode::Char('c') => {
                self.mode = Mode::Chat;
                KeyResult::None
            }
            KeyCode::Char('p') => {
                self.mode = Mode::PlayGame;
                KeyResult::None
            }
            KeyCode::Char('l') => {
                let selected_game_id = if self.model.active_games.is_empty() {
                    0
                } else {
                    self.model.active_games[0].0
                };

                self.mode = Mode::GameListing { selected_game_id };
                KeyResult::None
            }
            KeyCode::Char('n') => {
                self.mode = Mode::NewGame(PluginType::TickTackToe);
                KeyResult::None
            }
            _ => KeyResult::None,
        }
    }

    fn take_chat_message(&mut self) -> Option<ChatMessage> {
        let message = self.model.chat_input.join("\n");
        if message.is_empty() {
            self.model.chat_error = "".to_string();
            return None;
        }

        if !message.contains('@') {
            self.model.chat_error =
                "invalid message format, please use <recipient>@<message>".to_string();
            return None;
        }
        if let Some((recipient, content)) = message.split_once('@') {
            if recipient.eq(&self.model.username) {
                self.model.chat_error = "Sender can't a message to themselves!".to_string();
                None
            } else {
                self.model.chat_input.clear();
                Some(ChatMessage {
                    username: recipient.to_string(),
                    message: content.to_string(),
                })
            }
        } else {
            self.model.chat_error = "message is missing either a recipient or content".to_string();
            None
        }
    }

    fn on_key_press_chat(&mut self, key: KeyCode) -> KeyResult {
        match key {
            KeyCode::Char(c) => {
                if self.model.chat_input.is_empty() {
                    self.model.chat_input.push(String::new());
                }
                self.model.chat_input.last_mut().unwrap().push(c)
            }
            KeyCode::Backspace => {
                if let Some(s) = self.model.chat_input.last_mut() {
                    if s.is_empty() {
                        self.model.chat_input.pop();
                    } else {
                        s.pop();
                    }
                }
            }
            KeyCode::Tab => {
                if let Some(msg) = self.take_chat_message() {
                    return KeyResult::SendChat(msg);
                }
            }
            KeyCode::Enter => self.model.chat_input.push(String::new()),
            _ => {}
        }

        KeyResult::None
    }

    fn on_key_press_game_listing(&mut self, selected_game_id: u64, key: KeyCode) -> KeyResult {
        if self.model.active_games.is_empty() {
            return KeyResult::None;
        }

        let idx = self
            .model
            .active_games
            .iter()
            .position(|(id, _)| selected_game_id == *id)
            .unwrap_or(0);

        let game_count = self.model.active_games.len();

        match key {
            KeyCode::Up => {
                self.mode = Mode::GameListing {
                    selected_game_id: self.model.active_games[(idx + 1) % game_count].0,
                }
            }
            KeyCode::Down => {
                let prev_idx = if idx > 0 { idx - 1 } else { game_count - 1 };
                self.mode = Mode::GameListing {
                    selected_game_id: self.model.active_games[prev_idx].0,
                };
            }
            KeyCode::Enter => {
                self.mode = Mode::None;
                return KeyResult::EnterGame {
                    game_id: selected_game_id,
                };
            }
            _ => {}
        }

        KeyResult::None
    }

    fn on_key_press_play_game(&mut self, key: KeyCode) -> KeyResult {
        let Some((game_id, game_state)) = &self.model.current_game else {
            return KeyResult::None;
        };
        match &mut self.game_input_state {
            GameInput::None => KeyResult::None,
            GameInput::TickTackToe { selected_cell } => {
                use plugins::tick_tack_toe::CELL_COUNT;
                use plugins::tick_tack_toe::GRID_WIDTH;
                let plugins::ClientState::TickTackToe(ttt) = game_state else {
                    return KeyResult::None;
                };
                if ttt.last_turn() == Some(self.model.username.as_str()) {
                    eprintln!("it's not your turn!");
                    // print something.
                    return KeyResult::None;
                }
                match key {
                    KeyCode::Up => {
                        if *selected_cell < GRID_WIDTH {
                            *selected_cell += CELL_COUNT - GRID_WIDTH;
                        } else {
                            *selected_cell -= GRID_WIDTH;
                        }
                    }
                    KeyCode::Down => *selected_cell = (*selected_cell + GRID_WIDTH) % CELL_COUNT,
                    KeyCode::Left => {
                        let column = *selected_cell % GRID_WIDTH;
                        let row_base = (*selected_cell / GRID_WIDTH) * GRID_WIDTH;
                        let new_column = if column > 0 {
                            column - 1
                        } else {
                            GRID_WIDTH - 1
                        };
                        *selected_cell = row_base + new_column;
                    }
                    KeyCode::Right => {
                        let column = *selected_cell % GRID_WIDTH;
                        let row_base = (*selected_cell / GRID_WIDTH) * GRID_WIDTH;
                        let new_column = (column + 1) % GRID_WIDTH;
                        *selected_cell = row_base + new_column;
                    }
                    KeyCode::Enter => {
                        return KeyResult::SendGameMsg {
                            game_id: *game_id,
                            msg: ttt.request_move_msg(*selected_cell, &self.model.username),
                        };
                    }
                    _ => {}
                }
                KeyResult::None
            }
        }
    }

    fn on_key_press_new_game(&mut self, plugin_type: PluginType, key: KeyCode) -> KeyResult {
        match key {
            KeyCode::Up => self.mode = Mode::NewGame(plugin_type.prev()),
            KeyCode::Down => self.mode = Mode::NewGame(plugin_type.next()),
            KeyCode::Enter => {
                self.mode = Mode::None;
                return KeyResult::NewGame(plugin_type);
            }
            _ => {}
        }

        KeyResult::None
    }
}
