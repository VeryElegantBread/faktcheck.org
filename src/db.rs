use serde::Serialize;
use sqlx::sqlite::SqlitePool;

use crate::pages;

#[derive(Serialize, sqlx::FromRow)]
pub struct Fakt {
    pub id: i32,
    pub content: String,
    pub keyword: String,
    pub source: String,
}

pub async fn get_pool() -> SqlitePool {
    let db_name = match std::env::var("FAKTCHECK_DB_NAME") {
        Ok(name) => name,
        Err(_) => {
            println!("FAKTCHECK_DB_NAME not set; using fakts.db");
            "fakts.db".to_string()
        }
    };
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

pub async fn search(pool: SqlitePool, term: &str) -> Vec<Fakt> {
    let pattern = format!("%{}%", term);

    sqlx::query_as::<_, Fakt>(
        "SELECT id, content, keyword, source FROM fakts WHERE content LIKE ? ORDER BY keyword COLLATE NOCASE",
    )
    .bind(pattern)
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

pub async fn add(pool: SqlitePool, payload: pages::AddForm) -> Result<i32, String> {
    let Ok(result) = sqlx::query("INSERT INTO fakts (content, keyword, source) VALUES (?, ?, ?)")
        .bind(&payload.fakt)
        .bind(&payload.keyword)
        .bind(&payload.source)
        .execute(&pool)
        .await
    else {
        return Err("Error: failed to query database.".to_string());
    };

    Ok(result.last_insert_rowid().try_into().unwrap())
}

pub async fn del(pool: SqlitePool, id: i32) -> Result<(), String> {
    let Ok(_) = sqlx::query("DELETE FROM fakts WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
    else {
        return Err("Error: failed to query database.".to_string());
    };

    Ok(())
}

pub async fn edit(pool: SqlitePool, row: Fakt) -> Result<(), String> {
    let Ok(_) = sqlx::query("UPDATE fakts SET content = ?, keyword = ?, source = ? WHERE id = ?")
        .bind(row.content)
        .bind(row.keyword)
        .bind(row.source)
        .bind(row.id)
        .execute(&pool)
        .await
    else {
        return Err("Error: failed to query database.".to_string());
    };

    Ok(())
}
