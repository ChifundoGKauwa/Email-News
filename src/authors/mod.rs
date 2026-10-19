pub mod handlers;
pub mod model;

use axum::{Router, routing::get};

use crate::AppState;
use handlers::{create_author, delete_author, get_author, get_authors, update_author};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_authors).post(create_author))
        .route(
            "/{id}",
            get(get_author).put(update_author).delete(delete_author),
        )
}
