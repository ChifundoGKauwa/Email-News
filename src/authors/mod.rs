pub mod handlers;
pub mod model;

use axum::{
    Router, middleware,
    routing::{get, post, put},
};

use crate::AppState;
use handlers::{create_author, delete_author, get_author, get_authors, update_author};

pub fn routes(jwt_secret: String) -> Router<AppState> {
    let public = Router::new()
        .route("/", get(get_authors))
        .route("/{id}", get(get_author));
    let protected = Router::new()
        .route("/", post(create_author))
        .route("/{id}", put(update_author).delete(delete_author))
        .route_layer(middleware::from_fn_with_state(
            jwt_secret,
            crate::auth::auth_middleware,
        ));

    public.merge(protected)
}
