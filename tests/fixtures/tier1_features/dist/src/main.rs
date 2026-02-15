#![allow(unused)]

use axum::Router;
use tokio::net::TcpListener;
use std::sync::Arc;
use axum::Extension;

#[tokio::main]
async fn main() {

    // Build router
    let app = axum::Router::new();

    let addr = "0.0.0.0:3000".parse().unwrap();
    println!("Server running on http://0.0.0.0:3000");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
