use crate::AppState;
use crate::auth::{AuthenticatedUser, hash_password, require_owner};
use crate::authors::model::{Author, CreateAuthor, PublicAuthor, UpdateAuthor};
use crate::entities::authors::{self, Entity as AuthorEntity};
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

pub async fn get_authors(State(state): State<AppState>) -> Result<Json<Vec<PublicAuthor>>, StatusCode> {
    let authors = AuthorEntity::find()
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = authors
        .into_iter()
        .map(|a| PublicAuthor {
            id: a.id,
            name: a.name,
            bio: a.bio,
            created_at: a.created_at,
        })
        .collect();

    Ok(Json(result))
}

pub async fn get_author(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PublicAuthor>, StatusCode> {
    let author = AuthorEntity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(PublicAuthor {
        id: author.id,
        name: author.name,
        bio: author.bio,
        created_at: author.created_at,
    }))
}

pub async fn create_author(
    State(state): State<AppState>,
    Json(payload): Json<CreateAuthor>,
) -> Result<Json<Author>, StatusCode> {
    let email = payload.email.trim().to_lowercase();
    let name = payload.name.trim();
    if name.is_empty() || !email_address::EmailAddress::is_valid(&email) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if payload.password.trim().len() < 8 {
        return Err(StatusCode::BAD_REQUEST);
    }

    if AuthorEntity::find()
        .filter(authors::Column::Email.eq(&email))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some()
    {
        return Err(StatusCode::CONFLICT);
    }

    let password_hash =
        hash_password(payload.password.trim()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let new_author = authors::ActiveModel {
        name: Set(name.to_string()),
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
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateAuthor>,
) -> Result<Json<Author>, StatusCode> {
    let author = AuthorEntity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    require_owner(&user, author.id)?;

    let mut active: authors::ActiveModel = author.into();

    if let Some(name) = payload.name {
        let name = name.trim();
        if name.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
        active.name = Set(name.to_string());
    }
    if let Some(email) = payload.email {
        let email = email.trim().to_lowercase();
        if !email_address::EmailAddress::is_valid(&email) {
            return Err(StatusCode::BAD_REQUEST);
        }
        if AuthorEntity::find()
            .filter(authors::Column::Email.eq(&email))
            .filter(authors::Column::Id.ne(id))
            .one(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .is_some()
        {
            return Err(StatusCode::CONFLICT);
        }
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

pub async fn delete_author(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let author = AuthorEntity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    require_owner(&user, author.id)?;

    match AuthorEntity::delete_by_id(id).exec(&state.db).await {
        Ok(res) if res.rows_affected > 0 => Ok(StatusCode::NO_CONTENT),
        Ok(_) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
