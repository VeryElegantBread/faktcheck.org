use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let port = std::env::var("FAKTCHECK_PORT").unwrap_or(String::from("3000"));
    let bind = format!("0.0.0.0:{}", port);

    let app = Router::new().route("/", get(root));

    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    let _ = axum::serve(listener, app).await;
}

async fn root() -> &'static str {
    "Hello, World!"
}
