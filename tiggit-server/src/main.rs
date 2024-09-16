mod storage;
use storage::simple;
use axum::{
    routing::get,
    Router,
};
use dotenvy::dotenv;
use std::env;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Environment variables
    let host = env::var("HOST").expect("Host not defined");
    let port = env::var("PORT").expect("Port not defined");

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", get(|| async {"hello world"}))
        .nest("/", simple::git_storage_router())
        .layer(TraceLayer::new_for_http());
    
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    
}
