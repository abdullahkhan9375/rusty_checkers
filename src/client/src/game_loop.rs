use messaging::{ClientMessage, ServerMessage};
use tokio::sync::mpsc;

pub fn run(tx: mpsc::Sender<ClientMessage>, mut rx: mpsc::Receiver<ServerMessage>) {
    loop {
        println!("\nPlease type a message in <user>@<message> format: ");
        let mut message = String::new();
        let _ = std::io::stdin().read_line(&mut message);
        message.truncate(message.len() - 1); // trim newline
        if !message.contains('@') {
            eprint!("invalid message format, please use <recipient>@<message>");
        }
        if let Some((recipient, content)) = message.split_once('@') {
            if let Err(e) = tx.blocking_send(ClientMessage::SendChatMessage {
                recipient: recipient.to_string(),
                message: content.to_string(),
            }) {
                eprintln!("GameLoop: Failed to post chat message: {e}");
            }
        } else {
            eprint!("\nmessage is missing either a recipient or content")
        }

        if let Ok(ServerMessage::RecvChatMessage { from, message }) = rx.try_recv() {
            println!("You received a message from {from}: {message}");
        }
    }
}
