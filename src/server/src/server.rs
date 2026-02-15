use crate::users::{self, Users};
use messaging::{ServerMessage, ClientMessage};
use std::sync::Mutex;

use tokio::net::{tcp::{OwnedReadHalf, OwnedWriteHalf}, TcpListener};
use tokio::sync::mpsc;
use tokio_util::{codec::LengthDelimitedCodec, sync::CancellationToken};
use tokio::task::JoinHandle;

use futures_util::{SinkExt, StreamExt};

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(30);

type ReadFramed = tokio_util::codec::Framed<OwnedReadHalf, LengthDelimitedCodec>;
type WriteFramed = tokio_util::codec::Framed<OwnedWriteHalf, LengthDelimitedCodec>;

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

pub async fn read_loop(svr: Arc<Server>, mut framed: ReadFramed) {
    loop {
        let result = tokio::time::timeout(TIMEOUT, framed.next()).await;
        let bytes = match result {
            Ok(Some(Ok(b))) => b,
            Ok(None) => break,
            Ok(Some(Err(e))) => {
                eprintln!("Read loop error: {e:?}");
                break;
            },
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
}

pub fn read_task(svr: Arc<Server>, framed: ReadFramed, cancellation_token: CancellationToken) -> JoinHandle<()> {
    tokio::spawn(async move {
        tokio::select! {
            _ = cancellation_token.cancelled() => {}
            _ = read_loop(svr, framed) => {},
        }
    })
}

pub async fn write_loop(write_stream: OwnedWriteHalf, mut rx: mpsc::Receiver<ServerMessage>) {
    let mut framed = WriteFramed::new(write_stream, LengthDelimitedCodec::new());
    while let Some(msg) = rx.recv().await {
        let serialized = match msg.serialise() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to serialise message: {e}");
                std::process::exit(1);
            },
        };

        match framed.send(serialized.clone().into()).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to send login message: {e}");
                std::process::exit(1);
            },
        }
    }
}

pub fn write_task(write_stream: OwnedWriteHalf, rx: mpsc::Receiver<ServerMessage>, cancellation_token: CancellationToken) -> JoinHandle<()> {
    tokio::spawn(async move {
        tokio::select! {
            _ = cancellation_token.cancelled() => {}
            _ = write_loop(write_stream, rx) => {},
        }
    })
}

pub async fn run(addr: SocketAddr) {
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind listener: {e}");
            std::process::exit(1);
        },
    };

    let svr = Arc::new(Server::new());

    loop {
        let (stream, _addr) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to accept: {e}");
                std::process::exit(1);
            }
        };

        let (read_stream, write_stream) = stream.into_split();
        let svr = svr.clone();
        tokio::spawn(async move {
            let mut framed = ReadFramed::new(read_stream, LengthDelimitedCodec::new());

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

            let (tx, rx) = mpsc::channel::<ServerMessage>(1024);
            let cancellation_token = CancellationToken::new();
            let read_handle = read_task(svr.clone(), framed, cancellation_token.clone());
            let write_handle = write_task(write_stream, rx, cancellation_token.clone());

            let mut set = tokio::task::JoinSet::new();
            set.spawn(read_handle);
            set.spawn(write_handle);

            let _ = tx.send(ServerMessage::LoginSuccess).await;

            set.join_next().await;

            cancellation_token.cancel();

            set.join_all().await;

            svr.logout_user(&username);
            println!("User {username} logged out");
        });
    }
}
