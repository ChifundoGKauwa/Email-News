pub mod handlers;
pub mod model;

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use handlers::{
    create_newsletter, delete_newsletter, get_newsletter, get_newsletters, update_newsletter,
};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_newsletters).post(create_newsletter))
        .route(
            "/{id}",
            get(get_newsletter)
                .put(update_newsletter)
                .delete(delete_newsletter),
        )
}
