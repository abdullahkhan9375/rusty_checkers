use std::net::{IpAddr, Ipv4Addr, SocketAddr};

mod client;
mod game_loop;

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let Some(username) = args.get(1) else {
        eprintln!("Must specify username");
        std::process::exit(1);
    };

    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let addr = SocketAddr::new(ip, 10000);
    client::run(username, addr).await;
}
