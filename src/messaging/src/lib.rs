use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Hello { msg: String },
}

pub fn deserialise(frame: &[u8]) -> Result<Message, String> {
    let s = match str::from_utf8(frame) {
        Ok(s) => s,
        Err(e) => return Err(e.to_string()),
    };

    let msg: Message = match serde_json::from_str(s) {
        Ok(msg) => msg,
        Err(e) => return Err(e.to_string()),
    };

    Ok(msg)
}

pub fn serialise(msg: &Message) -> Result<Vec<u8>, String> {
    let s = match serde_json::to_string(msg) {
        Ok(s) => s,
        Err(e) => return Err(e.to_string()),
    };

    Ok(s.into_bytes()) 
}

