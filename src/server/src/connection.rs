use crate::server::Server;
use messaging::{ClientMessage, ServerMessage};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

const TIMEOUT: Duration = Duration::from_secs(30);

#[async_trait::async_trait]
pub(crate) trait ReadStream: Send + 'static {
    async fn read_bytes(&mut self) -> Result<Vec<u8>, String>;
}

#[async_trait::async_trait]
pub(crate) trait WriteStream: Send + 'static {
    async fn write_bytes(&mut self, bytes: Vec<u8>) -> Result<(), String>;
}

async fn write_msg<S: WriteStream>(stream: &mut S, msg: &ServerMessage) -> Result<(), String> {
    let serialized = match msg.serialise() {
        Ok(s) => s,
        Err(e) => return Err(format!("Failed to serialise message: {e}")),
    };

    if let Err(e) = stream.write_bytes(serialized).await {
        return Err(format!("Failed to send message: {e}"));
    }

    Ok(())
}

async fn read_loop<S: ReadStream>(username: String, svr: Arc<Server>, mut stream: S) {
    loop {
        let result = match tokio::time::timeout(TIMEOUT, stream.read_bytes()).await {
            Ok(r) => r,
            Err(_elapsed) => {
                eprintln!("No message received: {username}");
                break;
            }
        };

        let bytes = match result {
            Ok(m) => m,
            Err(e) => {
                eprintln!("No message received {username}: {e}");
                break;
            }
        };

        let msg = match ClientMessage::deserialise(bytes.as_slice()) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to decode msg: {e}");
                continue;
            }
        };

        svr.recv_msg(&username, msg).await;
    }
}

fn read_task<S: ReadStream>(
    username: String,
    svr: Arc<Server>,
    stream: S,
    cancellation_token: CancellationToken,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        tokio::select! {
            _ = cancellation_token.cancelled() => {}
            _ = read_loop(username, svr, stream) => {},
        }
    })
}

async fn write_loop<S: WriteStream>(
    username: String,
    mut write_stream: S,
    mut rx: mpsc::Receiver<ServerMessage>,
) {
    while let Some(msg) = rx.recv().await {
        println!("Sending message to {username}: {msg:?}");
        if let Err(e) = write_msg(&mut write_stream, &msg).await {
            eprintln!("{e}");
        }
    }
}

fn write_task<S: WriteStream>(
    username: String,
    write_stream: S,
    rx: mpsc::Receiver<ServerMessage>,
    cancellation_token: CancellationToken,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        tokio::select! {
            _ = cancellation_token.cancelled() => {}
            _ = write_loop(username, write_stream, rx) => {},
        }
    })
}

async fn login_sequence<S: ReadStream>(
    svr: Arc<Server>,
    tx: &tokio::sync::mpsc::Sender<ServerMessage>,
    read_stream: &mut S,
) -> Result<String, String> {
    let username = match read_stream.read_bytes().await {
        Ok(bytes) => {
            let msg = match ClientMessage::deserialise(bytes.as_ref()) {
                Ok(m) => m,
                Err(e) => {
                    return Err(format!("Failed to decode initial msg: {e}"));
                }
            };

            match msg {
                ClientMessage::LoginRequest { username } => {
                    if let Err(e) = svr.login_user(&username, tx.clone()) {
                        // TODO: Send reason to client
                        return Err(format!("Failed to login user {username}: {e:?}"));
                    }

                    println!("User {username} logged in");
                    username
                }
                _ => {
                    // TODO: Send reason to client
                    return Err(format!("unexpected initial message: {msg:?}"));
                }
            }
        }
        Err(e) => return Err(format!("Failed to recv message frame: {e}")),
    };

    Ok(username)
}

pub(crate) async fn run_connection<R: ReadStream, W: WriteStream>(
    svr: Arc<Server>,
    mut read_stream: R,
    write_stream: W,
) {
    let (tx, rx) = mpsc::channel::<ServerMessage>(1024);

    println!("New JS client connected!");
    let username = match login_sequence(svr.clone(), &tx, &mut read_stream).await {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Failed to decode initial msg: {e}");
            return;
        }
    };

    let cancellation_token = CancellationToken::new();
    let read_handle = read_task(
        username.clone(),
        svr.clone(),
        read_stream,
        cancellation_token.clone(),
    );

    let write_handle = write_task(
        username.clone(),
        write_stream,
        rx,
        cancellation_token.clone(),
    );

    let mut set = tokio::task::JoinSet::new();
    set.spawn(read_handle);
    set.spawn(write_handle);

    let _ = tx.send(ServerMessage::LoginSuccess).await;

    set.join_next().await;

    cancellation_token.cancel();

    set.join_all().await;

    svr.logout_user(&username);
    println!("User {username} logged out");
}
