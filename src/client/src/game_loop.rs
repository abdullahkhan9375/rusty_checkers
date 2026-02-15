use messaging::{ClientMessage, ServerMessage};
use tokio::sync::mpsc;

pub fn run(tx: mpsc::Sender<ClientMessage>, mut rx: mpsc::Receiver<ServerMessage>) {
    loop {
        println!("Please type a message in <user>@<message> format: ");
        let mut message = String::new();
        let _ = std::io::stdin().read_line(&mut message);
        message.truncate(message.len() - 1); // trim newline
        if !message.contains('@') {
            eprint!("invalid message format, please use <recipient>@<message>");
        }
        let v: Vec<&str> = message.split('@').collect();
        if v.len() < 2 {
            eprint!("message is missing either a recipient or content")
        }

        let recipient = v[0].to_string();
        let content = v[1].to_string();

        if let Err(e) = tx.blocking_send(ClientMessage::SendChatMessage {
            recipient,
            message: content,
        }) {
            eprintln!("GameLoop: Failed to post chat message: {e}");
        }

        if let Ok(ServerMessage::RecvChatMessage { from, message }) = rx.try_recv() {
            println!("You received a message from {from}: {message}");
        }
    }
}
