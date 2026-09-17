use serde::Serialize;
use sqlx::sqlite::SqlitePool;

#[derive(Serialize, sqlx::FromRow)]
pub struct Fakt {
    pub id: i32,
    pub content: String,
    pub keyword: String,
    pub source: String,
}

pub async fn get_pool() -> SqlitePool {
    let db_name = std::env::var("FAKTCHECK_DB_NAME").unwrap_or(String::from("fakts.db"));
    let db_url = format!("sqlite:{}", db_name);

    let pool = SqlitePool::connect(&db_url)
        .await
        .expect("Failed to connect to database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS fakts (
            id INTEGER PRIMARY KEY,
            content TEXT NOT NULL,
            keyword TEXT NOT NULL,
            source TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create table");

    pool
}

pub async fn all_fakts(pool: SqlitePool) -> Vec<Fakt> {
    sqlx::query_as::<_, Fakt>(
        "SELECT id, content, keyword, source FROM fakts ORDER BY keyword COLLATE NOCASE",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default()
}

pub async fn id(pool: SqlitePool, id: i32) -> Option<Fakt> {
    sqlx::query_as::<_, Fakt>("SELECT id, content, keyword, source FROM fakts WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .expect("Failed to query database")
}

pub async fn random(pool: SqlitePool) -> Fakt {
    sqlx::query_as::<_, Fakt>(
        "SELECT id, content, keyword, source FROM fakts ORDER BY RANDOM() LIMIT 1;",
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to query database")
    .expect("Empty database")
}
