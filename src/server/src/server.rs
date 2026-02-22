use crate::game_states::GameStates;
use crate::users::{self, Users};
use messaging::{ClientMessage, ServerMessage};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;

pub struct ServerConfig {
    pub tcp_addr: SocketAddr,
    pub websocket_addr: SocketAddr,
}

pub(crate) struct Server {
    users: Mutex<Users>,
    game_states: Mutex<GameStates>,
}

impl Server {
    fn new() -> Self {
        Self {
            users: Mutex::new(Users::new()),
            game_states: Mutex::new(GameStates::new()),
        }
    }

    pub fn login_user(
        &self,
        username: &str,
        tx: mpsc::Sender<ServerMessage>,
    ) -> Result<(), users::LoginResult> {
        let mut lock = self.users.lock().unwrap();
        let result = lock.login_user(username, tx);
        if matches!(result, users::LoginResult::Success) {
            Ok(())
        } else {
            Err(result)
        }
    }

    pub fn logout_user(&self, username: &str) -> bool {
        let mut lock = self.users.lock().unwrap();
        matches!(lock.logout_user(username), users::LogoutResult::Success)
    }

    pub async fn recv_msg(&self, username: &str, msg: ClientMessage) {
        println!("Recived msg from {username}: {msg:?}");
        match msg {
            ClientMessage::HeartBeat => {}
            ClientMessage::LoginRequest { .. } => {
                eprintln!("Unexpected LoginRequest message");
            }
            ClientMessage::RequestNewGame(plugin_type) => {
                let new_game_id = {
                    let mut lock = self.game_states.lock().unwrap();
                    lock.new_game(plugin_type)
                };

                println!("Created new game ({new_game_id}): {plugin_type:?}");
                let users_tx = {
                    let lock = self.users.lock().unwrap();
                    lock.get_all_users_tx()
                };

                let msg = ServerMessage::NewGameCreated {
                    game_id: new_game_id,
                    plugin_type,
                };
                for (username, tx) in users_tx {
                    if let Err(e) = tx.send(msg.clone()).await {
                        eprintln!("Failed to post broadcast msg to {username}: {msg:?}. {e:?}");
                    }
                }
            }
            ClientMessage::RequestEnterGame { game_id } => {
                let state_msg = {
                    let mut lock = self.game_states.lock().unwrap();
                    lock.get_game_mut(game_id).map(|(plugin_type, game)| {
                        (plugin_type, game.get_state_msg_for_user(username))
                    })
                };

                if let Some((plugin_type, state_msg)) = state_msg
                    && let Ok(state_msg) = state_msg
                {
                    let tx = {
                        let lock = self.users.lock().unwrap();
                        lock.get_user_tx(username).clone()
                    };

                    if let Some(tx) = tx {
                        let _ = tx
                            .send(ServerMessage::GameEntered {
                                game_id,
                                plugin_type,
                                game_state_msg: state_msg,
                            })
                            .await;
                    }
                }
            }
            ClientMessage::RequestGameUpdate { game_id, msg } => {
                let result = {
                    let mut lock = self.game_states.lock().unwrap();
                    lock.get_game_mut(game_id)
                        .map(|(_, game)| game.recv_msg(&msg))
                };

                if let Some(result) = result {
                    match result {
                        Ok(_) => {
                            let channels = {
                                let lock = self.users.lock().unwrap();
                                lock.get_all_users_tx()
                            };

                            Users::broadcast_msg(
                                ServerMessage::GameUpdate { game_id, msg },
                                &channels,
                            )
                            .await;
                        }
                        Err(e) => eprintln!("Failed to update game {game_id}: {e}"),
                    }
                }
            }
            ClientMessage::SendChatMessage { recipient, message } => {
                let txs = {
                    let lock = self.users.lock().unwrap();
                    [
                        (recipient.as_str(), lock.get_user_tx(&recipient)),
                        (username, lock.get_user_tx(username)),
                    ]
                };

                for (to, tx) in txs {
                    if let Some(tx) = tx {
                        let msg = ServerMessage::RecvChatMessage {
                            from: username.to_string(),
                            message: message.clone(),
                        };
                        if let Err(e) = tx.send(msg).await {
                            eprintln!(
                                "Failed to post received client message from {username} to {to}: {e}"
                            );
                        }
                    } else {
                        // TODO: Send error to user
                        eprintln!("Unable to find user tx: {to}");
                    }
                }
            }
        }
    }
}

pub async fn run(config: ServerConfig) {
    let svr = Arc::new(Server::new());

    let websocket_handle = crate::websocket::run(config.websocket_addr, svr.clone());
    let tcp_handle = crate::tcp::run(config.tcp_addr, svr.clone());

    tokio::join!(websocket_handle, tcp_handle);
}
