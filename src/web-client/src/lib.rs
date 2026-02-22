use messaging::ServerMessage;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn onRecvChatMessage(from: &str, msg: &str);
}

#[wasm_bindgen]
pub fn recv_server_msg(msg_str: &str) -> Result<(), String> {
    let msg = ServerMessage::deserialise(msg_str.as_bytes())?;

    match msg {
        ServerMessage::LoginSuccess | ServerMessage::LoginFailed { .. } => {
            return Err(format!("Unexpected server msg: {msg:?}"));
        }
        ServerMessage::RecvChatMessage { from, message } => {
            onRecvChatMessage(&from, &message);
        }
        ServerMessage::NewGameCreated {
            game_id,
            plugin_type,
        } => {}
        ServerMessage::GameEntered {
            game_id,
            plugin_type,
            game_state_msg,
        } => {}
        ServerMessage::GameUpdate { game_id, msg } => {}
    }
    Ok(())
}
