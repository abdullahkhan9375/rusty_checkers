use tokio::net::{tcp::{OwnedReadHalf, OwnedWriteHalf}, TcpStream};
use tokio_util::{codec::LengthDelimitedCodec, sync::CancellationToken};
use tokio::task::JoinHandle;
use tokio::sync::mpsc;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use messaging::{ServerMessage, ClientMessage};
use crate::game_loop;

const HEARTBEAT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(10);

type ReadFramed = tokio_util::codec::Framed<OwnedReadHalf, LengthDelimitedCodec>;
type WriteFramed = tokio_util::codec::Framed<OwnedWriteHalf, LengthDelimitedCodec>;

type ClientMsgSender = mpsc::Sender<ClientMessage>;

type ServerMsgSender = mpsc::Sender<ServerMessage>;

async fn read_msg(framed: &mut ReadFramed) -> Result<ServerMessage, String> {
    let result = framed.next().await;
    let bytes = match result {
        Some(Ok(b)) => b,
        None => return Err("No frame received".to_string()),
        Some(Err(e)) => return Err(e.to_string()),
    };

    match ServerMessage::deserialise(bytes.as_ref()) {
        Ok(m) => Ok(m),
        Err(e) => Err(e),
    }
}

async fn read_loop(mut framed: ReadFramed, tx: ServerMsgSender) {
    loop {
        let msg = match read_msg(&mut framed).await {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to read message: {e}");
                break;
            },
        };

        println!("Received server message: {msg:?}");
        if let Err(e) = tx.send(msg).await {
            eprintln!("Failed to post server message: {e:?}");
        }
    }
}

fn read_task(framed: ReadFramed, tx: ServerMsgSender, cancellation_token: CancellationToken) -> JoinHandle<()> {
    tokio::spawn(async move {
        tokio::select! {
            _ = cancellation_token.cancelled() => {},
            _ = read_loop(framed, tx) => {},
        }
    })
}

async fn write_loop(mut write_framed: WriteFramed, mut rx: mpsc::Receiver<ClientMessage>) {
    while let Some(msg) = rx.recv().await {
        let serialized = match msg.serialise() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to serialise message: {e}");
                std::process::exit(1);
            },
        };

        match write_framed.send(serialized.into()).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to send message: {e}");
                std::process::exit(1);
            },
        }
        println!("Sent message: {msg:?}");
    }
    println!("Write loop finished");
}

fn write_task(write_framed: WriteFramed, rx: mpsc::Receiver<ClientMessage>, cancellation_token: CancellationToken) -> JoinHandle<()> {
    tokio::spawn(async move {
        tokio::select! {
            _ = cancellation_token.cancelled() => {},
            _ = write_loop(write_framed, rx) => {},
        }
    })
}

async fn heartbeat_loop(tx: ClientMsgSender) {
    loop {
        if let Err(e) = tx.send(ClientMessage::HeartBeat).await {
            eprintln!("Failed to post heartbeat: {e}");
        }
        tokio::time::sleep(HEARTBEAT_INTERVAL).await;
    }
}

fn heartbeat_task(tx: ClientMsgSender, cancellation_token: CancellationToken) -> JoinHandle<()> {
    tokio::spawn(async move {
        tokio::select! {
            _ = cancellation_token.cancelled() => {},
            _ = heartbeat_loop(tx) => {},
        }
    })
}

pub async fn run(username: &str, addr: SocketAddr) {
    let stream = match TcpStream::connect(addr).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to connect: {e}");
            std::process::exit(1);
        },
    };

    let (read_stream, write_stream) = stream.into_split();

    let mut read_framed = ReadFramed::new(read_stream, LengthDelimitedCodec::new());
    let mut write_framed = WriteFramed::new(write_stream, LengthDelimitedCodec::new());

    {
        let login_msg = ClientMessage::LoginRequest { username: username.to_string() };

        // Serialize and send
        let serialized = match login_msg.serialise() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to serialise message: {e}");
                std::process::exit(1);
            },
        };

        match write_framed.send(serialized.clone().into()).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to send login message: {e}");
                std::process::exit(1);
            },
        }
    }

    match read_msg(&mut read_framed).await {
        Ok(ServerMessage::LoginSuccess) => {
            println!("Recieved login success message");
        },
        Ok(m) => {
            eprintln!("Received message, but not login success message: {m:?}");
            std::process::exit(1);
        },
        Err(e) => {
            eprintln!("Login success message not received: {e}");
            std::process::exit(1);
        },
    }

    let (server_tx, server_rx) = mpsc::channel(1024);
    let (client_tx, client_rx) = mpsc::channel(1024);
    let cancellation_token = CancellationToken::new();

    let mut set = tokio::task::JoinSet::new();
    set.spawn(read_task(read_framed, server_tx, cancellation_token.clone()));
    set.spawn(write_task(write_framed, client_rx, cancellation_token.clone()));
    set.spawn(heartbeat_task(client_tx.clone(), cancellation_token.clone()));

    let cancellation_token_clone = cancellation_token.clone();
    tokio::task::spawn_blocking(move || {
        game_loop::run(client_tx, server_rx);
        cancellation_token_clone.cancel();
    });
    set.join_next().await;
    cancellation_token.cancel();

    set.join_all().await;
}

