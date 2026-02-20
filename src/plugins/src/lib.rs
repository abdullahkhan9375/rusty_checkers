use serde::{Deserialize, Serialize};
use strum::VariantArray;

pub mod tick_tack_toe;

pub enum EvaluateResult {
    Running,
    WinningEnd { username: String },
    TieEnd,
}

// It seems to make more sense for the server to use dynamic dispatch
// since it treat each game the same.
// But on the client side, the renderer will need to understand which
// game it is drawing; so an enum is used.
pub trait ServerPlugin: Send + Sync {
    fn has_user(&self, username: &str) -> bool;
    fn recv_msg(&mut self, msg: &str) -> Result<String, String>;
    fn get_state_msg_for_user(&mut self, username: &str) -> Result<String, String>;
    fn evaluate_state(&self) -> EvaluateResult;
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, VariantArray, PartialEq, Eq)]
pub enum PluginType {
    TickTackToe,
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::TickTackToe => "TickTackToe",
        };

        write!(f, "{}", s)
    }
}

impl PluginType {
    pub fn new_server(self) -> Box<dyn ServerPlugin> {
        match self {
            Self::TickTackToe => tick_tack_toe::new_server(),
        }
    }

    pub fn new_client(self) -> ClientState {
        match self {
            Self::TickTackToe => ClientState::TickTackToe(tick_tack_toe::ClientState::new(
                tick_tack_toe::Cell::Empty,
            )),
        }
    }

    pub fn to_string(self) -> &'static str {
        match self {
            PluginType::TickTackToe => "TickTackToe",
        }
    }

    pub fn get_all() -> &'static [Self] {
        Self::VARIANTS
    }

    pub fn next(self) -> Self {
        let idx = Self::VARIANTS.iter().position(|p| *p == self).unwrap();
        Self::VARIANTS[(idx + 1) % Self::VARIANTS.len()]
    }

    pub fn prev(self) -> Self {
        let idx = Self::VARIANTS.iter().position(|p| *p == self).unwrap();
        if idx > 0 {
            Self::VARIANTS[idx - 1]
        } else {
            Self::VARIANTS[Self::VARIANTS.len() - 1]
        }
    }
}

pub enum ClientState {
    TickTackToe(tick_tack_toe::ClientState),
}

impl ClientState {
    pub fn recv_msg(&mut self, msg_str: &str) {
        match self {
            Self::TickTackToe(ttt) => {
                ttt.recv_msg(msg_str);
            }
        }
    }
}
