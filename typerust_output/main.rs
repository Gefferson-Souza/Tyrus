use axum::Router;
use tokio::net::TcpListener;
use std::sync::Arc;
use axum::Extension;

#[tokio::main]
async fn main() {
    let cats_service = Arc::new(typerust_app::src::cats_service::CatsService::new());
    let cats_controller = Arc::new(typerust_app::src::cats_controller::CatsController::new(cats_service.clone()));

    let app = Router::new()
        .merge(typerust_app::src::cats_controller::CatsController::router())
        .layer(Extension(cats_controller.clone()))
        .layer(Extension(cats_service.clone()));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
