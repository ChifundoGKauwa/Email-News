pub mod handlers;
pub mod model;

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use handlers::{create_author, delete_author, get_author, get_authors, update_author};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_authors).post(create_author))
        .route(
            "/{id}",
            get(get_author).put(update_author).delete(delete_author),
        )
}
