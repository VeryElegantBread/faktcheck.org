use axum::{
    Router,
    routing::{get, post},
};
use dotenv::dotenv;

mod db;
mod pages;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let pool = db::get_pool();
    let port = match std::env::var("FAKTCHECK_PORT") {
        Ok(port) => port,
        Err(_) => {
            println!("FAKTCHECK_PORT not set; using 3000");
            "3000".to_string()
        }
    };
    let bind = format!("0.0.0.0:{}", port);

    let app = Router::new()
        .route("/", get(pages::root))
        .route("/fakt/{id}", get(pages::fakt))
        .route("/fakts", get(pages::faktlist))
        .route("/add", get(pages::add))
        .route("/add", post(pages::add_post))
        .route("/del/{id}", get(pages::del))
        .route("/del/{id}", post(pages::del_post))
        .fallback(pages::not_found)
        .with_state(pool.await);

    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    let _ = axum::serve(listener, app).await;
}
