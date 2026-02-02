use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use futures_util::SinkExt;

#[tokio::main]
async fn main() {
    let stream = match TcpStream::connect("127.0.0.1:9000").await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to connect: {e}");
            std::process::exit(1);
        },
    };

    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

    let args = std::env::args().collect::<Vec<_>>();
    // Create a structured message
    let msg = messaging::Message::Hello {
        msg: args.get(1).map(String::as_str).unwrap_or_else(|| "Hello server!").to_string(),
    };

    // Serialize and send
    let serialized = match messaging::serialise(&msg) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to serialise message: {e}");
            std::process::exit(1);
        },
    };

    match framed.send(serialized.into()).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to send message: {e}");
            std::process::exit(1);
        },
    };
}
