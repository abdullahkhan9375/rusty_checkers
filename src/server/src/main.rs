use std::net::{IpAddr, Ipv4Addr, SocketAddr};

mod connection;
mod game_states;
mod server;
mod tcp;
mod users;
mod websocket;
mod http;

#[tokio::main]
async fn main() {
    let ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

    let c = server::ServerConfig {
        tcp_addr: SocketAddr::new(ip, 10000),
        websocket_addr: SocketAddr::new(ip, 11000),
    };

    tokio::join!(
        server::run(c),
        http::run(SocketAddr::new(ip, 12000)),
    );
}
