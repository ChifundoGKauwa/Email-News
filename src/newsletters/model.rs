use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Newsletter {
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub content: String,
    pub published: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateNewsletter {
    pub author_id: Uuid,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNewsletter {
    pub title: Option<String>,
    pub content: Option<String>,
    pub published: Option<bool>,
}
