use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use futures_util::SinkExt;

#[tokio::main]
async fn main() {
    let stream = match TcpStream::connect("127.0.0.1:10000").await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to connect: {e}");
            std::process::exit(1);
        },
    };

    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

    let args = std::env::args().collect::<Vec<_>>();
    let Some(username) = args.get(1) else {
        eprintln!("Must specify username");
        std::process::exit(1);
    };

    {
        let login_msg = messaging::ClientMessage::LoginRequest { username: username.to_string() };

        // Serialize and send
        let serialized = match login_msg.serialise() {
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

    let ping_msg = messaging::ClientMessage::Ping;

    // Serialize and send
    let serialized = match ping_msg.serialise() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to serialise message: {e}");
            std::process::exit(1);
        },
    };

    for _ in 0..5 {
        match framed.send(serialized.clone().into()).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to send message: {e}");
                std::process::exit(1);
            },
        }
        println!("Sent ping");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
