use plugins::PluginType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    LoginSuccess,
    LoginFailed {
        reason: String,
    },
    RecvChatMessage {
        from: String,
        message: String,
    },

    NewGameCreated {
        game_id: u64,
        plugin_type: PluginType,
    },
    GameEntered {
        game_id: u64,
        plugin_type: PluginType,
        game_state_msg: String,
    },
    GameUpdate {
        game_id: u64,
        msg: String,
    },
}

pub fn deserialise<'de, T: Deserialize<'de>>(frame: &'de [u8]) -> Result<T, String> {
    let s = match str::from_utf8(frame) {
        Ok(s) => s,
        Err(e) => return Err(e.to_string()),
    };

    let msg: T = match serde_json::from_str(s) {
        Ok(msg) => msg,
        Err(e) => return Err(e.to_string()),
    };

    Ok(msg)
}

pub fn serialise_string<T: Serialize>(msg: &T) -> Result<String, String> {
    serde_json::to_string(msg).map_err(|e| e.to_string())
}

pub fn serialise<T: Serialize>(msg: &T) -> Result<Vec<u8>, String> {
    Ok(serialise_string(msg)?.into_bytes())
}

impl ServerMessage {
    pub fn deserialise(frame: &[u8]) -> Result<ServerMessage, String> {
        deserialise(frame)
    }

    pub fn serialise(&self) -> Result<Vec<u8>, String> {
        serialise(&self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    HeartBeat,

    LoginRequest { username: String },
    SendChatMessage { recipient: String, message: String },

    RequestNewGame(PluginType),
    RequestEnterGame { game_id: u64 },
    RequestGameUpdate { game_id: u64, msg: String },
}

impl ClientMessage {
    pub fn deserialise(frame: &[u8]) -> Result<ClientMessage, String> {
        deserialise(frame)
    }

    pub fn serialise(&self) -> Result<Vec<u8>, String> {
        serialise(self)
    }

    pub fn serialise_string(&self) -> Result<String, String> {
        serialise_string(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn xxxx() {
        println!(
            "{}",
            String::from_utf8(
                ServerMessage::RecvChatMessage {
                    from: "Ben".to_string(),
                    message: "Hidy ho".to_string(),
                }
                .serialise()
                .unwrap()
            )
            .unwrap()
        );
    }
}
