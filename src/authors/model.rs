use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Author {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub bio: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateAuthor {
    pub name: String,
    pub email: String,
    pub bio: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAuthor {
    pub name: Option<String>,
    pub email: Option<String>,
    pub bio: Option<String>,
}
