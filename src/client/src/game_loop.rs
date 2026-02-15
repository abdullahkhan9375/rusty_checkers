use tokio::sync::mpsc;
use messaging::{ServerMessage, ClientMessage};

pub fn run(tx: mpsc::Sender<ClientMessage>, mut rx: mpsc::Receiver<ServerMessage>) {
    loop {
        println!("Please type a message recipient: ");

        let mut recipient = String::new();
        let _ = std::io::stdin().read_line(&mut recipient);
        recipient.truncate(recipient.len() - 1); // trim newline

        println!("Please type a message: ");
        let mut message = String::new();
        let _ = std::io::stdin().read_line(&mut message);
        message.truncate(message.len() - 1); // trim newline

        if let Err(e) = tx.blocking_send(ClientMessage::SendChatMessage { recipient, message }) {
            eprintln!("GameLoop: Failed to post chat message: {e}");
        }

        if let Ok(ServerMessage::RecvChatMessage { from, message }) = rx.try_recv() {
            println!("You received a message from {from}: {message}");
        }
    }
}


