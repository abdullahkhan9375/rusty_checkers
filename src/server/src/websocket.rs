use crate::connection::run_connection;
use crate::connection::{ReadStream, WriteStream};
use crate::server::Server;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

type ReadWebsocket = SplitStream<WebSocketStream<tokio::net::TcpStream>>;
type WriteWebsocket =
    SplitSink<WebSocketStream<tokio::net::TcpStream>, tokio_tungstenite::tungstenite::Message>;

#[async_trait::async_trait]
impl ReadStream for ReadWebsocket {
    async fn read_bytes(&mut self) -> Result<Vec<u8>, String> {
        let Some(result) = self.next().await else {
            return Err("".to_string());
        };

        match result {
            Ok(Message::Text(t)) => {
                let b: tokio_util::bytes::Bytes = t.into();
                Ok(b.into())
            }
            Ok(r) => Err(format!("Websocket unexpected message type: {r:?}")),
            Err(e) => Err(format!("{e:?}")),
        }
    }
}

#[async_trait::async_trait]
impl WriteStream for WriteWebsocket {
    async fn write_bytes(&mut self, bytes: Vec<u8>) -> Result<(), String> {
        let utf8_bytes: tokio_tungstenite::tungstenite::Utf8Bytes = match bytes.try_into() {
            Ok(b) => b,
            Err(e) => return Err(format!("Websocket failed to convert bytes to utf8: {e:?}")),
        };

        self.send(Message::Text(utf8_bytes))
            .await
            .map_err(|e| format!("Websocket failed to send message: {e:?}"))
    }
}

pub(crate) async fn run(addr: SocketAddr, svr: Arc<Server>) {
    let websocket_listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind listener: {e}");
            return;
        }
    };

    loop {
        match websocket_listener.accept().await {
            Ok((stream, _addr)) => {
                let svr_clone = svr.clone();
                tokio::spawn(async move {
                    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Websocket failed handshake: {e:?}");
                            return;
                        }
                    };
                    let (write_stream, read_stream) = ws_stream.split();
                    run_connection(svr_clone.clone(), read_stream, write_stream).await;
                });
            }
            Err(e) => {
                eprintln!("Failed to accept: {e}");
                return;
            }
        }
    }
}
