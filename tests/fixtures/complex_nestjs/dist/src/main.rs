#![allow(unused)]

use axum::Router;
use tokio::net::TcpListener;
use std::sync::Arc;
use axum::Extension;

#[tokio::main]
async fn main() {

    // Build router
    let app = axum::Router::new()
        .merge(tyrus_app::controllers::payment_controller::PaymentController::router());

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
