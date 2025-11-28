use axum::Router;
use tokio::net::TcpListener;
use std::sync::Arc;
use axum::Extension;

#[tokio::main]
async fn main() {
    let http_client = Arc::new(typerust_app::utils::http-client::HttpClient::new());
    let user_processor = Arc::new(typerust_app::services::user-processor::UserProcessor::new());

    let app = Router::new()        .layer(Extension(user_processor.clone()))
        .layer(Extension(http_client.clone()))
;

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
