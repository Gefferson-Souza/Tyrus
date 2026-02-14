#![allow(unused)]

use axum::Router;
use tokio::net::TcpListener;
use std::sync::Arc;
use axum::Extension;

#[tokio::main]
async fn main() {
    let fraud_service = Arc::new(tyrus_app::services::fraud_service::FraudService::new_di());
    let payment_service = Arc::new(tyrus_app::services::payment_service::PaymentService::new_di(fraud_service.clone()));
    let create_payment_dto = Arc::new(tyrus_app::dtos::payment_dto::CreatePaymentDto::new_di());
    let payment_controller = Arc::new(tyrus_app::controllers::payment_controller::PaymentController::new_di(payment_service.clone()));

    // Build router
    let app = axum::Router::new()
        .merge(tyrus_app::controllers::payment_controller::PaymentController::router())
        .layer(Extension(payment_service.clone()))
        .layer(Extension(create_payment_dto.clone()))
        .layer(Extension(fraud_service.clone()))
        .layer(Extension(payment_controller.clone()));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
