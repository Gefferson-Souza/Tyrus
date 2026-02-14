#![allow(unused)]

use axum::Router;
use tokio::net::TcpListener;
use std::sync::Arc;
use axum::Extension;

#[tokio::main]
async fn main() {
    let user_processor = Arc::new(tyrus_app::services::user_processor::UserProcessor::new_di());

    // Build router
    let app = axum::Router::new()
        .layer(Extension(user_processor.clone()));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
