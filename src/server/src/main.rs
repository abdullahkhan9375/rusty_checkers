use tokio::net::TcpListener;
use futures_util::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use std::time::Duration;
use std::sync::Arc;
use messaging::{ServerMessage, ClientMessage};

mod users;
mod server;

const TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() {
    let listener = match TcpListener::bind("0.0.0.0:9000").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind listener: {e}");
            std::process::exit(1);
        },
    };

    let svr = Arc::new(server::Server::new());

    loop {
        let (socket, _addr) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to accept: {e}");
                std::process::exit(1);
            }
        };

        let svr = svr.clone();
        tokio::spawn(async move {
            let mut framed = Framed::new(socket, LengthDelimitedCodec::new());

            let username = match framed.next().await {
                Some(Ok(bytes)) => {
                    let msg = match ClientMessage::deserialise(bytes.as_ref()) {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("Failed to decode initial msg: {e}");
                            return;
                        }
                    };

                    match msg {
                        ClientMessage::LoginRequest { username } => {
                            if !svr.login_user(&username) {
                                // TODO: Send reason to client
                                eprintln!("Failed to login user {username}");
                                return;
                            }

                            username
                        }
                        _ => {
                            // TODO: Send reason to client
                            eprintln!("unexpected initial message: {msg:?}");
                            return;
                        }
                    }
                },
                Some(Err(e)) => {
                    eprintln!("Failed to recv message frame: {e}");
                    return;
                },
                None => {
                    eprintln!("No initial message received");
                    return;
                }
            };

            println!("User {username} logged in");
            loop {
                let result = tokio::time::timeout(TIMEOUT, framed.next()).await;
                let bytes = match result {
                    Ok(Some(Ok(b))) => b,
                    Ok(_) => break,
                    Err(_elapsed) => {
                        println!("Timed out");
                        break;
                    }
                };

                let msg = match ClientMessage::deserialise(bytes.as_ref()) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Failed to decode msg: {e}");
                        continue;
                    }
                };

                println!("{msg:?}");
                svr.recv_msg(msg);
            }

            println!("User {username} logged out");
            svr.logout_user(&username);
        });
    }
}
