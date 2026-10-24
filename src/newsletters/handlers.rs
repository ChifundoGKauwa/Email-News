use crate::AppState;
use crate::auth::{AuthenticatedUser, require_owner};
use crate::entities::newsletters::{self, Entity as NewsletterEntity};
use crate::newsletters::model::{CreateNewsletter, Newsletter, UpdateNewsletter};
use axum::{
    Json,
    extract::{Extension, Path, State},
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
            image_url: n.image_url,
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
        image_url: newsletter.image_url,
        published: newsletter.published,
        created_at: newsletter.created_at,
        updated_at: newsletter.updated_at,
    }))
}

pub async fn create_newsletter(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<CreateNewsletter>,
) -> Result<Json<Newsletter>, StatusCode> {
    require_owner(&user, payload.author_id)?;
    let now = Utc::now().naive_utc();

    let new_newsletter = newsletters::ActiveModel {
        author_id: Set(user.author_id),
        title: Set(payload.title.clone()),
        content: Set(payload.content.clone()),
        image_url: Set(payload.image_url.clone()),
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
        image_url: saved.image_url,
        published: saved.published,
        created_at: saved.created_at,
        updated_at: saved.updated_at,
    }))
}

pub async fn update_newsletter(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateNewsletter>,
) -> Result<Json<Newsletter>, StatusCode> {
    let newsletter = NewsletterEntity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    require_owner(&user, newsletter.author_id)?;

    let mut active: newsletters::ActiveModel = newsletter.into();

    if let Some(title) = payload.title {
        active.title = Set(title);
    }
    if let Some(content) = payload.content {
        active.content = Set(content);
    }
    if let Some(image_url) = payload.image_url {
        active.image_url = Set(Some(image_url));
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
        image_url: updated.image_url,
        published: updated.published,
        created_at: updated.created_at,
        updated_at: updated.updated_at,
    }))
}

pub async fn delete_newsletter(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let newsletter = NewsletterEntity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    require_owner(&user, newsletter.author_id)?;

    match NewsletterEntity::delete_by_id(id).exec(&state.db).await {
        Ok(res) if res.rows_affected > 0 => Ok(StatusCode::NO_CONTENT),
        Ok(_) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
