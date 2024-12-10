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
use mongodb::{ 
	bson::{Document, doc, to_document},
	Client,
	Collection 
};

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Environment variables
    let host = env::var("HOST").expect("Host not defined");
    let port = env::var("PORT").expect("Port not defined");
    let db_uri = env::var("DATABASE_URI").expect("Database URI not defined");
    let db_name = env::var("DATABASE_NAME").expect("Database NAME not defined");

    let client = Client::with_uri_str(db_uri).await.unwrap();
    let db = client.database(&db_name);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", get(|| async {"tiggit server"}))
        .nest("/", simple::git_storage_router())
        .layer(TraceLayer::new_for_http());
    
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    
}
