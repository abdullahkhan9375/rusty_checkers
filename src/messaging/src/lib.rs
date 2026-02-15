use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    LoginSuccess,
    LoginFailed { reason: String },
    RecvChatMessage { from: String, message: String },
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

pub fn serialise<T: Serialize>(msg: &T) -> Result<Vec<u8>, String> {
    let s = match serde_json::to_string(msg) {
        Ok(s) => s,
        Err(e) => return Err(e.to_string()),
    };

    Ok(s.into_bytes()) 
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
}

impl ClientMessage {
    pub fn deserialise(frame: &[u8]) -> Result<ClientMessage, String> {
        deserialise(frame)
    }

    pub fn serialise(&self) -> Result<Vec<u8>, String> {
        serialise(self)
    }
}

