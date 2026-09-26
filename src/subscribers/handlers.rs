use crate::AppState;
use crate::entities::subscribers::{self, Entity as SubscriberEntity};
use crate::subscribers::model::{CreateSubscriber, Subscriber};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;

// get all subscribers
pub async fn get_subscribers(
    State(state): State<AppState>,
) -> Result<Json<Vec<Subscriber>>, StatusCode> {
    let subscribers = SubscriberEntity::find()
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = subscribers
        .into_iter()
        .map(|s| Subscriber {
            id: s.id,
            email: s.email,
            subscribed_at: s.subscribed_at,
            name: s.name,
        })
        .collect();

    Ok(Json(result))
}

// get subscriber by ID
pub async fn get_subscriber(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Subscriber>, StatusCode> {
    let subscriber = SubscriberEntity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(Subscriber {
        id: subscriber.id,
        email: subscriber.email,
        subscribed_at: subscriber.subscribed_at,
        name: subscriber.name,
    }))
}

// create subscriber
pub async fn create_subscriber(
    State(state): State<AppState>,
    Json(payload): Json<CreateSubscriber>,
) -> Result<Json<Subscriber>, StatusCode> {
    println!("creating the subscriber: {:?}", payload);

    let new_subscriber = subscribers::ActiveModel {
        email: Set(payload.email.clone()),
        name: Set(payload.name.clone()),
        subscribed_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    let saved = new_subscriber
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Subscriber {
        id: saved.id,
        email: saved.email,
        subscribed_at: saved.subscribed_at,
        name: saved.name,
    }))
}

// delete subscriber
pub async fn delete_subscriber(State(state): State<AppState>, Path(id): Path<Uuid>) -> StatusCode {
    println!("deleting the subscriber {}", id);

    match SubscriberEntity::delete_by_id(id).exec(&state.db).await {
        Ok(res) if res.rows_affected > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
