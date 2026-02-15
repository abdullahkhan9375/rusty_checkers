use std::net::{IpAddr, Ipv4Addr, SocketAddr};

mod client;

#[tokio::main]
async fn main() {
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let addr = SocketAddr::new(ip, 10000);
    client::run(addr).await;
}
