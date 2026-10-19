use crate::AppState;
use crate::auth::hash_password;
use crate::authors::model::{Author, CreateAuthor, UpdateAuthor};
use crate::entities::authors::{self, Entity as AuthorEntity};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;

pub async fn get_authors(State(state): State<AppState>) -> Result<Json<Vec<Author>>, StatusCode> {
    let authors = AuthorEntity::find()
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = authors
        .into_iter()
        .map(|a| Author {
            id: a.id,
            name: a.name,
            email: a.email,
            bio: a.bio,
            created_at: a.created_at,
        })
        .collect();

    Ok(Json(result))
}

pub async fn get_author(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Author>, StatusCode> {
    let author = AuthorEntity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(Author {
        id: author.id,
        name: author.name,
        email: author.email,
        bio: author.bio,
        created_at: author.created_at,
    }))
}

pub async fn create_author(
    State(state): State<AppState>,
    Json(payload): Json<CreateAuthor>,
) -> Result<Json<Author>, StatusCode> {
    let email = payload.email.trim().to_lowercase();
    if email.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let password_hash =
        hash_password(payload.password.trim()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let new_author = authors::ActiveModel {
        name: Set(payload.name.trim().to_string()),
        email: Set(email.clone()),
        email_hash: Set(String::new()),
        email_salt: Set(String::new()),
        password_hash: Set(password_hash),
        password_salt: Set(String::new()),
        bio: Set(payload.bio.clone()),
        created_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    let saved = new_author
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Author {
        id: saved.id,
        name: saved.name,
        email: saved.email,
        bio: saved.bio,
        created_at: saved.created_at,
    }))
}

pub async fn update_author(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateAuthor>,
) -> Result<Json<Author>, StatusCode> {
    let author = AuthorEntity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut active: authors::ActiveModel = author.into();

    if let Some(name) = payload.name {
        active.name = Set(name);
    }
    if let Some(email) = payload.email {
        active.email = Set(email);
    }
    if let Some(bio) = payload.bio {
        active.bio = Set(Some(bio));
    }

    let updated = active
        .update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Author {
        id: updated.id,
        name: updated.name,
        email: updated.email,
        bio: updated.bio,
        created_at: updated.created_at,
    }))
}

pub async fn delete_author(State(state): State<AppState>, Path(id): Path<Uuid>) -> StatusCode {
    match AuthorEntity::delete_by_id(id).exec(&state.db).await {
        Ok(res) if res.rows_affected > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
