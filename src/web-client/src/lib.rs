use messaging::{ClientMessage, ServerMessage};
use wasm_bindgen::prelude::*;
use js_sys::Function;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout(callback: Function, delay: i32) -> f64;
}

fn console_log(s: &str) {
    log(&format!("wasm: {s}"));
}

#[derive(Debug)]
enum LoginStatus {
    NotLoggedOn,
    AwaitingResponse { on_success: Function, on_failed: Function },
    LoggedOn,
    LoginFailed,
}

#[wasm_bindgen]
struct Engine {
    send_bytes: Function,
    login_status: LoginStatus,
    on_recv_chat_msg: std::cell::RefCell<Option<Function>>,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new(send_bytes: Function) -> Self {
        Self {
            send_bytes,
            login_status: LoginStatus::NotLoggedOn,
            on_recv_chat_msg: std::cell::RefCell::new(None),
        }
    }

    #[wasm_bindgen]
    pub fn set_on_recv_chat_msg(&self, on_recv_chat_msg: Function) {
        self.on_recv_chat_msg.replace(Some(on_recv_chat_msg));
    }

    fn send_message(&self, message: ClientMessage) -> Result<(), String> {
        let result = message.serialise_string();
        let msg = match result {
            Ok(m) => m,
            Err(e) => return Err(format!("Failed to serialise message {message:?}: {e}")),
        };

        console_log(&format!("Sending message: {msg}"));
        let _x = self.send_bytes.call1(&JsValue::null(), &JsValue::from(&msg));
        Ok(())
    }

    #[wasm_bindgen]
    pub fn send_login_req(&mut self, username: &str, on_success: Function, on_failed: Function) -> Result<(), String> {
        if !matches!(self.login_status, LoginStatus::NotLoggedOn) {
            return Err(format!("Invalid login status"));
        }

        self.login_status = LoginStatus::AwaitingResponse {
            on_success,
            on_failed,
        };
        self.send_message(ClientMessage::LoginRequest { username: username.to_string() })
    }

    #[wasm_bindgen]
    pub fn send_chat_msg(&self, recipient: &str, msg: &str) -> Result<(), String> {
        console_log(&format!("CHAT: {recipient}: {msg}"));
        self.send_message(ClientMessage::SendChatMessage {
            recipient: recipient.to_string(),
            message: msg.to_string(),
        })
    }

    #[wasm_bindgen]
    pub fn send_heartbeat(&self) -> Result<(), String> {
        self.send_message(ClientMessage::HeartBeat)
    }

    #[wasm_bindgen]
    pub fn recv_server_msg(&mut self, msg_str: &str) -> Result<(), String> {
        let msg = ServerMessage::deserialise(msg_str.as_bytes())?;

        match msg {
            ServerMessage::LoginSuccess => {
                console_log("Login successful");
                let status = std::mem::replace(&mut self.login_status, LoginStatus::NotLoggedOn);
                if let LoginStatus::AwaitingResponse { on_success, .. } = status {
                    console_log("Login successful");
                    // This callback is probably going to call back into wasm.
                    // Calling it directly here will not work
                    set_timeout(on_success, 1);
                } else {
                    console_log("Invalid login status: ");
                }
            }
            ServerMessage::LoginFailed { .. } => {
                console_log("Login failed");
                let status = std::mem::replace(&mut self.login_status, LoginStatus::NotLoggedOn);
                if let LoginStatus::AwaitingResponse { on_failed, .. } = status {
                    console_log("Login failed");
                    // This callback is probably going to call back into wasm.
                    // Calling it directly here will not work
                    set_timeout(on_failed, 1);
                } else {
                    console_log("Invalid login status: ");
                }
            }
            ServerMessage::RecvChatMessage { from, message } => {
                console_log(&format!("RecvChatMessage {from}: {message}"));
                if let Some(on_recv_chat_msg) = &*self.on_recv_chat_msg.borrow() {
                    console_log("RecvChatMessage: Call");
                    let _x = on_recv_chat_msg.call2(&JsValue::null(), &JsValue::from(&from), &JsValue::from(&message));
                }
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
}
