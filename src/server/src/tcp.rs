use crate::connection::run_connection;
use crate::connection::{ReadStream, WriteStream};
use crate::server::Server;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::task::JoinHandle;
use tokio_util::codec::Framed;
use tokio_util::codec::LengthDelimitedCodec;

type ReadFramed = Framed<OwnedReadHalf, LengthDelimitedCodec>;
type WriteFramed = Framed<OwnedWriteHalf, LengthDelimitedCodec>;

#[async_trait::async_trait]
impl ReadStream for ReadFramed {
    async fn read_bytes(&mut self) -> Result<Vec<u8>, String> {
        let Some(result) = self.next().await else {
            return Err("".to_string());
        };

        match result {
            Ok(r) => Ok(r.freeze().to_vec()),
            Err(e) => Err(format!("{e:?}")),
        }
    }
}

#[async_trait::async_trait]
impl WriteStream for WriteFramed {
    async fn write_bytes(&mut self, bytes: Vec<u8>) -> Result<(), String> {
        if let Err(e) = self.send(bytes.into()).await {
            Err(format!("{e:?}"))
        } else {
            Ok(())
        }
    }
}

pub(crate) async fn run(addr: SocketAddr, svr: Arc<Server>) {
    let tcp_listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind listener: {e}");
            return;
        }
    };

    loop {
        let (stream, _addr) = match tcp_listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to accept: {e}");
                return;
            }
        };

        let (read_stream, write_stream) = stream.into_split();
        let svr = svr.clone();
        tokio::spawn(async move {
            let write_framed = WriteFramed::new(write_stream, LengthDelimitedCodec::new());
            let read_framed = ReadFramed::new(read_stream, LengthDelimitedCodec::new());
            run_connection(svr.clone(), read_framed, write_framed).await;
        });
    }
}
