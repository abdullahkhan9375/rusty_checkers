use tokio::net::TcpListener;
use futures_util::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[tokio::main]
async fn main() {
    let listener = match TcpListener::bind("0.0.0.0:9000").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind listener: {e}");
            std::process::exit(1);
        },
    };

    loop {
        let (socket, _addr) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to accept: {e}");
                std::process::exit(1);
            }
        };

        tokio::spawn(async move {
            let mut framed = Framed::new(socket, LengthDelimitedCodec::new());

            while let Some(Ok(bytes)) = framed.next().await {
                match messaging::deserialise(bytes.as_ref()) {
                    Ok(msg) => println!("Received message: {msg:#?}"),
                    Err(e) => eprintln!("Failed to decode msg: {e}"),
                }
            }

        });
    }
}
