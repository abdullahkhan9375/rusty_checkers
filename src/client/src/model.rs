use crossterm::event::KeyCode;

#[derive(Clone, Copy)]
pub enum Mode {
    None,
    Chat,
}

pub struct ChatMessage {
    pub username: String,
    pub message: String,
}

pub enum KeyResult {
    None,
    Quit,
    SendChat(ChatMessage),
}

pub struct Model {
    pub username: String,
    pub mode: Mode,
    pub chat_history: Vec<ChatMessage>,
    pub chat_input: Vec<String>,
    pub chat_error: String,
}

impl Model {
    pub fn new(username: &str) -> Self {
        Self {
            username: username.to_string(),
            mode: Mode::None,
            chat_history: vec![],
            chat_input: vec![],
            chat_error: "".to_string(),
        }
    }

    pub fn on_key_press(&mut self, key: KeyCode) -> KeyResult {
        match self.mode {
            Mode::None => self.on_key_press_none(key),
            Mode::Chat => self.on_key_press_chat(key),
        }
    }

    fn on_key_press_none(&mut self, key: KeyCode) -> KeyResult {
        match key {
            KeyCode::Char('q') => KeyResult::Quit,
            KeyCode::Char('c') => {
                self.mode = Mode::Chat;
                KeyResult::None
            }
            _ => KeyResult::None,
        }
    }

    fn take_chat_message(&mut self) -> Option<ChatMessage> {
        let message = self.chat_input.join("\n");
        if message.is_empty() {
            self.chat_error = "".to_string();
            return None;
        }

        if !message.contains('@') {
            self.chat_error =
                "invalid message format, please use <recipient>@<message>".to_string();
            return None;
        }
        if let Some((recipient, content)) = message.split_once('@') {
            if recipient.eq(&self.username) {
                self.chat_error = "Sender can't a message to themselves!".to_string();
                None
            } else {
                self.chat_input.clear();
                Some(ChatMessage {
                    username: recipient.to_string(),
                    message: content.to_string(),
                })
            }
        } else {
            self.chat_error = "message is missing either a recipient or content".to_string();
            None
        }
    }

    fn on_key_press_chat(&mut self, key: KeyCode) -> KeyResult {
        match key {
            KeyCode::Esc => self.mode = Mode::None,
            KeyCode::Char(c) => {
                if self.chat_input.is_empty() {
                    self.chat_input.push(String::new());
                }
                self.chat_input.last_mut().unwrap().push(c)
            }
            KeyCode::Backspace => {
                if let Some(s) = self.chat_input.last_mut() {
                    if s.is_empty() {
                        self.chat_input.pop();
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
            KeyCode::Enter => self.chat_input.push(String::new()),
            _ => {}
        }

        KeyResult::None
    }
}
