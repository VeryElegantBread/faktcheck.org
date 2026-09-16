use axum::{Json, Router, routing::get};

use crate::db::Fakt;

mod db;

#[tokio::main]
async fn main() {
    let pool = db::get_pool();
    let port = std::env::var("FAKTCHECK_PORT").unwrap_or(String::from("3000"));
    let bind = format!("0.0.0.0:{}", port);

    let app = Router::new().route("/", get(root)).with_state(pool.await);

    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    let _ = axum::serve(listener, app).await;
}

async fn root(
    axum::extract::State(pool): axum::extract::State<sqlx::SqlitePool>,
) -> Json<Vec<Fakt>> {
    Json(db::all_fakts(pool).await)
}
