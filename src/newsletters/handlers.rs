use crate::AppState;
use crate::entities::newsletters::{self, Entity as NewsletterEntity};
use crate::newsletters::model::{CreateNewsletter, Newsletter, UpdateNewsletter};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;

pub async fn get_newsletters(
    State(state): State<AppState>,
) -> Result<Json<Vec<Newsletter>>, StatusCode> {
    let newsletters = NewsletterEntity::find()
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = newsletters
        .into_iter()
        .map(|n| Newsletter {
            id: n.id,
            author_id: n.author_id,
            title: n.title,
            content: n.content,
            published: n.published,
            created_at: n.created_at,
            updated_at: n.updated_at,
        })
        .collect();

    Ok(Json(result))
}

pub async fn get_newsletter(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Newsletter>, StatusCode> {
    let newsletter = NewsletterEntity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(Newsletter {
        id: newsletter.id,
        author_id: newsletter.author_id,
        title: newsletter.title,
        content: newsletter.content,
        published: newsletter.published,
        created_at: newsletter.created_at,
        updated_at: newsletter.updated_at,
    }))
}

pub async fn create_newsletter(
    State(state): State<AppState>,
    Json(payload): Json<CreateNewsletter>,
) -> Result<Json<Newsletter>, StatusCode> {
    let now = Utc::now().naive_utc();

    let new_newsletter = newsletters::ActiveModel {
        author_id: Set(payload.author_id),
        title: Set(payload.title.clone()),
        content: Set(payload.content.clone()),
        published: Set(false),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let saved = new_newsletter
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Newsletter {
        id: saved.id,
        author_id: saved.author_id,
        title: saved.title,
        content: saved.content,
        published: saved.published,
        created_at: saved.created_at,
        updated_at: saved.updated_at,
    }))
}

pub async fn update_newsletter(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateNewsletter>,
) -> Result<Json<Newsletter>, StatusCode> {
    let newsletter = NewsletterEntity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut active: newsletters::ActiveModel = newsletter.into();

    if let Some(title) = payload.title {
        active.title = Set(title);
    }
    if let Some(content) = payload.content {
        active.content = Set(content);
    }
    if let Some(published) = payload.published {
        active.published = Set(published);
    }
    active.updated_at = Set(Utc::now().naive_utc());

    let updated = active
        .update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Newsletter {
        id: updated.id,
        author_id: updated.author_id,
        title: updated.title,
        content: updated.content,
        published: updated.published,
        created_at: updated.created_at,
        updated_at: updated.updated_at,
    }))
}

pub async fn delete_newsletter(State(state): State<AppState>, Path(id): Path<Uuid>) -> StatusCode {
    match NewsletterEntity::delete_by_id(id).exec(&state.db).await {
        Ok(res) if res.rows_affected > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
