pub mod handlers;
pub mod model;

use axum::{Router, routing::get};

use handlers::{create_subscriber, delete_subscriber, get_subscriber, get_subscribers};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_subscribers).post(create_subscriber))
        .route("/{id}", get(get_subscriber).delete(delete_subscriber))
}
