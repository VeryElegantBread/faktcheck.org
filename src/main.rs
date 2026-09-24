use axum::{
    Router,
    routing::{get, post},
};
use dotenv::dotenv;
use tokio::signal;

mod db;
mod pages;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let pool = db::get_pool();
    let port = match std::env::var("FAKTCHECK_PORT") {
        Ok(port) => port,
        Err(_) => {
            println!("FAKTCHECK_PORT not set; using 8080");
            "8080".to_string()
        }
    };
    let bind = format!("0.0.0.0:{}", port);

    let app = Router::new()
        .route("/style.css", get(pages::style))
        .route("/", get(pages::root))
        .route("/fakt/{id}", get(pages::fakt))
        .route("/fakts", get(pages::faktlist))
        .route("/search", get(pages::search))
        .route("/add", get(pages::add))
        .route("/add", post(pages::add_post))
        .route("/del/{id}", get(pages::del))
        .route("/del/{id}", post(pages::del_post))
        .route("/edit/{id}", get(pages::edit))
        .route("/edit/{id}", post(pages::edit_post))
        .fallback(pages::not_found)
        .with_state(pool.await);

    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    let _ = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await;
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("Received CTRL+C, shutting down...");
        }
        _ = terminate => {
            println!("Received SIGTERM, shutting down...");
        }
    }
}
