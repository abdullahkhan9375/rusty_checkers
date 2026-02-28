use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use std::net::SocketAddr;

const WASM_DATA: &[u8] = include_bytes!("../../web-client/pkg/web_client_bg.wasm");
const JS_DATA: &str = include_str!("../../web-client/pkg/web_client.js");
const INDEX_DATA: &str = include_str!("../../web-client/index.html");

pub(crate) async fn run(addr: SocketAddr) {
    // build our application with a route
    let app = Router::new()
        .route("/", get(index))
        .route("/index.html", get(index))
        .route("/web_client_bg.wasm", get(wasm))
        .route("/web_client.js", get(js))
        ;

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await;
}

async fn wasm() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/wasm")],
        WASM_DATA,
    )
}
async fn js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript")],
        JS_DATA,
    )
}
async fn index() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html")],
        INDEX_DATA,
    )
}
