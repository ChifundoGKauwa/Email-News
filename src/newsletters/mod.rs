pub mod handlers;
pub mod model;

use axum::{Router, routing::get};

use crate::AppState;
use handlers::{
    create_newsletter, delete_newsletter, get_newsletter, get_newsletters, update_newsletter,
};

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
