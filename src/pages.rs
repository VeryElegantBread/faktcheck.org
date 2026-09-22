use std::env;

use askama::Template;
use axum::{
    Form,
    extract::{Path, State},
    response::Html,
};
use sqlx::SqlitePool;

use crate::db::{self, Fakt};

#[derive(Template)]
#[template(path = "home.html")]
struct HomePage {
    id: i32,
    fakt: String,
}

#[derive(Template)]
#[template(path = "fakt.html")]
struct FaktPage {
    keyword: String,
    fakt: String,
    source: String,
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundPage;

#[derive(Template)]
#[template(path = "faktlist.html")]
struct FaktListPage {
    faktlist: String,
}

#[derive(Template)]
#[template(path = "add.html")]
struct AddPage {
    message: String,
}

#[derive(serde::Deserialize)]
pub struct AddForm {
    pub fakt: String,
    pub keyword: String,
    pub source: String,
    password: String,
}

#[derive(Template)]
#[template(path = "del.html")]
struct DelPage {
    id: i32,
    fakt: String,
    source: String,
    message: String,
}

#[derive(serde::Deserialize)]
pub struct DelForm {
    password: String,
}

#[derive(Template)]
#[template(path = "edit.html")]
struct EditPage {
    id: i32,
    keyword: String,
    fakt: String,
    source: String,
    content: String,
    message: String,
}

#[derive(serde::Deserialize)]
pub struct EditForm {
    pub fakt: String,
    pub keyword: String,
    pub source: String,
    password: String,
}

pub async fn root(State(pool): State<SqlitePool>) -> Html<String> {
    let fakt = db::random(pool).await;
    let page = HomePage {
        id: fakt.id,
        fakt: format_fakt(&fakt),
    };

    Html(page.render().expect("Failed to render page"))
}

pub async fn not_found() -> Html<String> {
    Html(NotFoundPage.render().expect("Failed to render page"))
}

pub async fn fakt(Path(id): Path<i32>, State(pool): State<SqlitePool>) -> Html<String> {
    let Some(fakt) = db::id(pool, id).await else {
        return not_found().await;
    };

    let page = FaktPage {
        fakt: format_fakt(&fakt),
        keyword: fakt.keyword,
        source: fakt.source,
    };

    Html(page.render().expect("Failed to render page"))
}

pub async fn faktlist(State(pool): State<SqlitePool>) -> Html<String> {
    let fakts = db::all_fakts(pool);

    let faktlist = format_faktlist(fakts.await);

    let page = FaktListPage { faktlist };

    Html(page.render().expect("Failed to render page"))
}

pub async fn add() -> Html<String> {
    let page = AddPage {
        message: String::new(),
    };
    Html(page.render().expect("Failed to render page"))
}

pub async fn add_post(
    State(pool): State<SqlitePool>,
    Form(payload): Form<AddForm>,
) -> Html<String> {
    if !is_correct_password(&payload.password) {
        let page = AddPage {
            message: "Error: incorrect password.".to_string(),
        };
        return Html(page.render().expect("Failed to render page"));
    }

    if payload.fakt.trim().is_empty()
        || payload.keyword.trim().is_empty()
        || payload.source.trim().is_empty()
    {
        let page = AddPage {
            message: "Error: fields cannot be empty.".to_string(),
        };
        return Html(page.render().expect("Failed to render page"));
    }

    let id = match db::add(pool, payload).await {
        Ok(id) => id,
        Err(error) => {
            let page = AddPage { message: error };
            return Html(page.render().expect("Failed to render page"));
        }
    };

    let mut message = "New fakt: <a href=\"/fakt/".to_string();
    message.push_str(&id.to_string());
    message.push_str("\">");
    message.push_str(&id.to_string());
    message.push_str("</a>");

    let page = AddPage { message };
    Html(page.render().expect("Failed to render page"))
}

pub async fn del(Path(id): Path<i32>, State(pool): State<SqlitePool>) -> Html<String> {
    let Some(fakt) = db::id(pool, id).await else {
        return not_found().await;
    };

    let page = DelPage {
        id,
        fakt: format_fakt(&fakt),
        source: fakt.source,
        message: String::new(),
    };

    Html(page.render().expect("Failed to render page"))
}

pub async fn del_post(
    Path(id): Path<i32>,
    State(pool): State<SqlitePool>,
    Form(payload): Form<DelForm>,
) -> Html<String> {
    let Some(fakt) = db::id(pool.clone(), id).await else {
        return not_found().await;
    };

    if !is_correct_password(&payload.password) {
        let page = DelPage {
            id,
            fakt: format_fakt(&fakt),
            source: fakt.source,
            message: "Error: incorrect password.".to_string(),
        };
        return Html(page.render().expect("Failed to render page"));
    }

    if let Err(error) = db::del(pool, id).await {
        let page = DelPage {
            id,
            fakt: format_fakt(&fakt),
            source: fakt.source,
            message: error,
        };
        return Html(page.render().expect("Failed to render page"));
    };

    let page = DelPage {
        id,
        fakt: format_fakt(&fakt),
        source: fakt.source,
        message: "Successfully deleted fakt. <a href=\"/fakts\">Fakt List</a>".to_string(),
    };

    Html(page.render().expect("Failed to render page"))
}

pub async fn edit(Path(id): Path<i32>, State(pool): State<SqlitePool>) -> Html<String> {
    let Some(fakt) = db::id(pool, id).await else {
        return not_found().await;
    };

    let page = EditPage {
        id,
        fakt: format_fakt(&fakt),
        content: fakt.content,
        keyword: fakt.keyword,
        source: fakt.source,
        message: String::new(),
    };

    Html(page.render().expect("Failed to render page"))
}

pub async fn edit_post(
    Path(id): Path<i32>,
    State(pool): State<SqlitePool>,
    Form(payload): Form<EditForm>,
) -> Html<String> {
    let Some(fakt) = db::id(pool.clone(), id).await else {
        return not_found().await;
    };

    if !is_correct_password(&payload.password) {
        let page = EditPage {
            id,
            fakt: format_fakt(&fakt),
            content: fakt.content,
            keyword: fakt.keyword,
            source: fakt.source,
            message: "Error: incorrect password.".to_string(),
        };
        return Html(page.render().expect("Failed to render page"));
    }

    if payload.fakt.trim().is_empty()
        || payload.keyword.trim().is_empty()
        || payload.source.trim().is_empty()
    {
        let page = EditPage {
            id,
            fakt: format_fakt(&fakt),
            content: fakt.content,
            keyword: fakt.keyword,
            source: fakt.source,
            message: "Error: fields cannot be empty.".to_string(),
        };
        return Html(page.render().expect("Failed to render page"));
    }

    let row = Fakt {
        id,
        content: payload.fakt.clone(),
        keyword: payload.keyword.clone(),
        source: payload.source.clone(),
    };

    if let Err(error) = db::edit(pool, row).await {
        let page = EditPage {
            id,
            fakt: format_fakt(&fakt),
            content: fakt.content,
            keyword: fakt.keyword,
            source: fakt.source,
            message: error,
        };
        return Html(page.render().expect("Failed to render page"));
    };

    let mut message = "Edited fakt: <a href=\"/fakt/".to_string();
    message.push_str(&id.to_string());
    message.push_str("\">");
    message.push_str(&id.to_string());
    message.push_str("</a>");

    let page = EditPage {
        id,
        fakt: format_content(&payload.fakt, &payload.keyword),
        content: payload.fakt,
        keyword: payload.keyword,
        source: payload.source,
        message,
    };

    Html(page.render().expect("Failed to render page"))
}

fn format_fakt(fakt: &Fakt) -> String {
    format_content(&fakt.content, &fakt.keyword)
}

fn format_content(content: &str, keyword: &str) -> String {
    let length = keyword.len();
    let mut last_index = 0;
    let mut formatted = String::new();
    let lower_content = content.to_lowercase();
    let lower_keyword = keyword.to_lowercase();
    for (start_index, _) in lower_content.match_indices(&lower_keyword) {
        formatted.push_str(&content[last_index..start_index]);
        formatted.push_str("<b><i>");
        formatted.push_str(&content[start_index..start_index + length]);
        formatted.push_str("</i></b>");
        last_index = start_index + length;
    }
    formatted.push_str(&content[last_index..]);
    formatted
}

fn format_faktlist(fakts: Vec<Fakt>) -> String {
    let mut faktlist = String::new();

    let mut last_letter = 'A';
    for fakt in fakts {
        let letter = fakt.keyword.chars().next().unwrap();
        let lower = letter.to_lowercase().next().unwrap();
        if lower != last_letter {
            last_letter = lower;
            faktlist.push_str("<h2>");
            faktlist.push(letter.to_uppercase().next().unwrap());
            faktlist.push_str("</h2>");
        }
        faktlist.push_str("<p><a href=\"/fakt/");
        faktlist.push_str(&fakt.id.to_string());
        faktlist.push_str("\">");
        faktlist.push_str(&format_fakt(&fakt));
        faktlist.push_str("</a></p>");
    }

    faktlist
}

fn is_correct_password(pass: &str) -> bool {
    let Ok(correct_pass) = env::var("FAKTCHECK_ADMIN_PASSWORD") else {
        return false;
    };
    pass == correct_pass
}
