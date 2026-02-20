use plugins::{ClientState, PluginType};

pub struct ChatMessage {
    pub username: String,
    pub message: String,
}

pub struct Model {
    pub username: String,

    pub active_games: Vec<(u64, PluginType)>,
    pub current_game: Option<(u64, ClientState)>,

    pub chat_history: Vec<ChatMessage>,
    pub chat_input: Vec<String>,
    pub chat_error: String,
}

impl Model {
    pub fn new(username: &str) -> Self {
        Self {
            username: username.to_string(),

            active_games: Default::default(),
            current_game: Default::default(),

            chat_history: vec![],
            chat_input: vec![],
            chat_error: "".to_string(),
        }
    }
}
