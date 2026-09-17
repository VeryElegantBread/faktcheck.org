use axum::{Router, routing::get};

mod db;
mod pages;

#[tokio::main]
async fn main() {
    let pool = db::get_pool();
    let port = std::env::var("FAKTCHECK_PORT").unwrap_or(String::from("3000"));
    let bind = format!("0.0.0.0:{}", port);

    let app = Router::new()
        .route("/", get(pages::root))
        .route("/fakt/{id}", get(pages::fakt))
        .route("/fakts", get(pages::faktlist))
        .fallback(pages::not_found)
        .with_state(pool.await);

    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    let _ = axum::serve(listener, app).await;
}
