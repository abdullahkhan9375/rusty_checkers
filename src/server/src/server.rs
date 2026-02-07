use crate::users::{self, Users};
use messaging::ClientMessage;
use std::sync::Mutex;

pub struct Server {
    users: Mutex<Users>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            users: Mutex::new(Users::new()),
        }
    }

    pub fn login_user(&self, username: &str) -> bool {
        let mut lock = self.users.lock().unwrap();
        matches!(lock.login_user(username), users::LoginResult::Success)
    }

    pub fn logout_user(&self, username: &str) -> bool {
        let mut lock = self.users.lock().unwrap();
        matches!(lock.logout_user(username), users::LogoutResult::Success)
    }

    pub fn recv_msg(&self, msg: ClientMessage) {
        match msg {
            ClientMessage::Ping => {
                println!("Recv ping from client");
            }

            ClientMessage::LoginRequest { .. } => {
                eprintln!("Unexpected LoginRequest message");
            },
        }
    }
}

