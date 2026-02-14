use std::net::{IpAddr, Ipv4Addr, SocketAddr};

mod server;
mod users;

#[tokio::main]
async fn main() {
    let ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    let addr = SocketAddr::new(ip, 10000);
    server::run(addr).await;
}
