use crate::AppState;
use crate::readers::AuthenticatedReader;
use crate::entities::{bookmarks, claps, comments, highlights, newsletters};
use crate::entities::readers::Entity as ReaderEntity;
use axum::{Extension, Json, Router, extract::{Path, State}, http::StatusCode, middleware, routing::{get, post, put, delete}};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)] pub struct ClapCount { pub story_id: Uuid, pub total: u64, pub reader_clapped: bool }
#[derive(Serialize)] pub struct CommentResponse { pub id: Uuid, pub story_id: Uuid, pub reader_id: Uuid, pub reader_name: String, pub body: String, pub created_at: chrono::NaiveDateTime, pub updated_at: chrono::NaiveDateTime }
#[derive(Deserialize)] pub struct CommentInput { pub body: String }
#[derive(Serialize)] pub struct BookmarkResponse { pub id: Uuid, pub story_id: Uuid, pub story_title: String, pub created_at: chrono::NaiveDateTime }
#[derive(Deserialize)] pub struct HighlightInput { pub text: String, pub start_offset: i32, pub end_offset: i32 }
#[derive(Serialize)] pub struct HighlightResponse { pub id: Uuid, pub story_id: Uuid, pub text: String, pub start_offset: i32, pub end_offset: i32, pub created_at: chrono::NaiveDateTime }

async fn require_published_story(state: &AppState, story_id: Uuid) -> Result<(), StatusCode> {
    newsletters::Entity::find_by_id(story_id)
        .filter(newsletters::Column::Published.eq(true))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|_| ())
        .ok_or(StatusCode::NOT_FOUND)
}

pub fn routes(jwt_secret: String, db: sea_orm::DatabaseConnection) -> Router<AppState> {
    let public = Router::new()
        .route("/claps/{story_id}", get(claps_total))
        .route("/comments/{story_id}", get(list_comments));
    let protected = Router::new()
        .route("/claps/{story_id}", post(add_clap).delete(remove_clap))
        .route("/comments/{story_id}", post(create_comment))
        .route("/comments/{id}", put(update_comment).delete(delete_comment))
        .route("/bookmarks", get(list_bookmarks))
        .route("/bookmarks/{story_id}", put(add_bookmark).delete(remove_bookmark))
        .route("/highlights/{story_id}", get(list_highlights).post(add_highlight))
        .route("/highlights/{id}", delete(delete_highlight))
        .route_layer(middleware::from_fn_with_state(crate::readers::ReaderAuthState { secret: jwt_secret, db }, crate::readers::reader_middleware));
    public.merge(protected)
}

pub async fn claps_total(State(state): State<AppState>, Path(story_id): Path<Uuid>) -> Result<Json<ClapCount>, StatusCode> {
    require_published_story(&state, story_id).await?;
    let total = claps::Entity::find().filter(claps::Column::StoryId.eq(story_id)).count(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ClapCount { story_id, total, reader_clapped: false }))
}

pub async fn add_clap(State(state): State<AppState>, Extension(user): Extension<AuthenticatedReader>, Path(story_id): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    require_published_story(&state, story_id).await?;
    if claps::Entity::find().filter(claps::Column::StoryId.eq(story_id)).filter(claps::Column::ReaderId.eq(user.reader_id)).one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.is_some() { return Ok(StatusCode::NO_CONTENT); }
    claps::ActiveModel { reader_id: Set(user.reader_id), story_id: Set(story_id), created_at: Set(Utc::now().naive_utc()), ..Default::default() }.insert(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}
pub async fn remove_clap(State(state): State<AppState>, Extension(user): Extension<AuthenticatedReader>, Path(story_id): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    let deleted = claps::Entity::delete_many().filter(claps::Column::StoryId.eq(story_id)).filter(claps::Column::ReaderId.eq(user.reader_id)).exec(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted.rows_affected == 0 { Err(StatusCode::NOT_FOUND) } else { Ok(StatusCode::NO_CONTENT) }
}

pub async fn list_comments(State(state): State<AppState>, Path(story_id): Path<Uuid>) -> Result<Json<Vec<CommentResponse>>, StatusCode> {
    require_published_story(&state, story_id).await?;
    let rows = comments::Entity::find().filter(comments::Column::StoryId.eq(story_id)).all(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut result = Vec::with_capacity(rows.len());
    for row in rows { let name = ReaderEntity::find_by_id(row.reader_id).one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.map(|r| r.name).unwrap_or_else(|| "Deleted reader".to_string()); result.push(CommentResponse { id: row.id, story_id: row.story_id, reader_id: row.reader_id, reader_name: name, body: row.body, created_at: row.created_at, updated_at: row.updated_at }); }
    Ok(Json(result))
}
pub async fn create_comment(State(state): State<AppState>, Extension(user): Extension<AuthenticatedReader>, Path(story_id): Path<Uuid>, Json(input): Json<CommentInput>) -> Result<Json<CommentResponse>, StatusCode> {
    require_published_story(&state, story_id).await?;
    let body = input.body.trim(); if body.is_empty() { return Err(StatusCode::BAD_REQUEST); }
    if body.chars().count() > 10_000 { return Err(StatusCode::BAD_REQUEST); }
    let now = Utc::now().naive_utc(); let saved = comments::ActiveModel { reader_id: Set(user.reader_id), story_id: Set(story_id), body: Set(body.to_string()), created_at: Set(now), updated_at: Set(now), ..Default::default() }.insert(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let reader_name = ReaderEntity::find_by_id(user.reader_id).one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.map(|r| r.name).unwrap_or_else(|| "Reader".to_string());
    Ok(Json(CommentResponse { id: saved.id, story_id: saved.story_id, reader_id: saved.reader_id, reader_name, body: saved.body, created_at: saved.created_at, updated_at: saved.updated_at }))
}
pub async fn update_comment(State(state): State<AppState>, Extension(user): Extension<AuthenticatedReader>, Path(id): Path<Uuid>, Json(input): Json<CommentInput>) -> Result<Json<CommentResponse>, StatusCode> {
    let row = comments::Entity::find_by_id(id).one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::NOT_FOUND)?; if row.reader_id != user.reader_id { return Err(StatusCode::NOT_FOUND); }
    let body = input.body.trim(); if body.is_empty() || body.chars().count() > 10_000 { return Err(StatusCode::BAD_REQUEST); } let mut active: comments::ActiveModel = row.into(); active.body = Set(body.to_string()); active.updated_at = Set(Utc::now().naive_utc()); let saved = active.update(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let name = ReaderEntity::find_by_id(saved.reader_id).one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.map(|r| r.name).unwrap_or_else(|| "Reader".to_string()); Ok(Json(CommentResponse { id: saved.id, story_id: saved.story_id, reader_id: saved.reader_id, reader_name: name, body: saved.body, created_at: saved.created_at, updated_at: saved.updated_at }))
}
pub async fn delete_comment(State(state): State<AppState>, Extension(user): Extension<AuthenticatedReader>, Path(id): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    let deleted = comments::Entity::delete_many().filter(comments::Column::Id.eq(id)).filter(comments::Column::ReaderId.eq(user.reader_id)).exec(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?; if deleted.rows_affected == 0 { Err(StatusCode::NOT_FOUND) } else { Ok(StatusCode::NO_CONTENT) }
}

pub async fn list_bookmarks(State(state): State<AppState>, Extension(user): Extension<AuthenticatedReader>) -> Result<Json<Vec<BookmarkResponse>>, StatusCode> {
    let rows = bookmarks::Entity::find().filter(bookmarks::Column::ReaderId.eq(user.reader_id)).all(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?; let mut result = Vec::with_capacity(rows.len()); for row in rows { let title = newsletters::Entity::find_by_id(row.story_id).filter(newsletters::Column::Published.eq(true)).one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.map(|n| n.title).unwrap_or_else(|| "Unavailable story".to_string()); result.push(BookmarkResponse { id: row.id, story_id: row.story_id, story_title: title, created_at: row.created_at }); } Ok(Json(result))
}
pub async fn add_bookmark(State(state): State<AppState>, Extension(user): Extension<AuthenticatedReader>, Path(story_id): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    require_published_story(&state, story_id).await?;
    if bookmarks::Entity::find().filter(bookmarks::Column::ReaderId.eq(user.reader_id)).filter(bookmarks::Column::StoryId.eq(story_id)).one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.is_some() { return Ok(StatusCode::NO_CONTENT); } bookmarks::ActiveModel { reader_id: Set(user.reader_id), story_id: Set(story_id), created_at: Set(Utc::now().naive_utc()), ..Default::default() }.insert(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?; Ok(StatusCode::NO_CONTENT)
}
pub async fn remove_bookmark(State(state): State<AppState>, Extension(user): Extension<AuthenticatedReader>, Path(story_id): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    let deleted = bookmarks::Entity::delete_many().filter(bookmarks::Column::ReaderId.eq(user.reader_id)).filter(bookmarks::Column::StoryId.eq(story_id)).exec(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?; if deleted.rows_affected == 0 { Err(StatusCode::NOT_FOUND) } else { Ok(StatusCode::NO_CONTENT) }
}

pub async fn list_highlights(State(state): State<AppState>, Extension(user): Extension<AuthenticatedReader>, Path(story_id): Path<Uuid>) -> Result<Json<Vec<HighlightResponse>>, StatusCode> {
    require_published_story(&state, story_id).await?;
    let rows = highlights::Entity::find().filter(highlights::Column::ReaderId.eq(user.reader_id)).filter(highlights::Column::StoryId.eq(story_id)).all(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?; Ok(Json(rows.into_iter().map(|h| HighlightResponse { id: h.id, story_id: h.story_id, text: h.text, start_offset: h.start_offset, end_offset: h.end_offset, created_at: h.created_at }).collect()))
}
pub async fn add_highlight(State(state): State<AppState>, Extension(user): Extension<AuthenticatedReader>, Path(story_id): Path<Uuid>, Json(input): Json<HighlightInput>) -> Result<Json<HighlightResponse>, StatusCode> {
    require_published_story(&state, story_id).await?;
    if input.text.trim().is_empty() || input.text.chars().count() > 10_000 || input.start_offset < 0 || input.end_offset <= input.start_offset { return Err(StatusCode::BAD_REQUEST); } let saved = highlights::ActiveModel { reader_id: Set(user.reader_id), story_id: Set(story_id), text: Set(input.text.trim().to_string()), start_offset: Set(input.start_offset), end_offset: Set(input.end_offset), created_at: Set(Utc::now().naive_utc()), ..Default::default() }.insert(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?; Ok(Json(HighlightResponse { id: saved.id, story_id: saved.story_id, text: saved.text, start_offset: saved.start_offset, end_offset: saved.end_offset, created_at: saved.created_at }))
}
pub async fn delete_highlight(State(state): State<AppState>, Extension(user): Extension<AuthenticatedReader>, Path(id): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    let deleted = highlights::Entity::delete_many().filter(highlights::Column::Id.eq(id)).filter(highlights::Column::ReaderId.eq(user.reader_id)).exec(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?; if deleted.rows_affected == 0 { Err(StatusCode::NOT_FOUND) } else { Ok(StatusCode::NO_CONTENT) }
}
