use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use tokio::net::TcpListener;

async fn handle_request(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let response = Response::new(Body::from("Hello, Hyper with Tokio! 🚀"));
    Ok(response)
}

#[tokio::main]
async fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();

    // Create a Hyper server service
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_request))
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("🚀 Server running at http://{}", addr);

    // Start the server
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}

