pub mod handlers;
pub mod model;

use axum::{
    Router, middleware,
    routing::{get, post, put},
};

use crate::AppState;
use handlers::{
    create_newsletter, delete_newsletter, get_newsletter, get_newsletters, update_newsletter,
};

pub fn routes(jwt_secret: String) -> Router<AppState> {
    let public = Router::new()
        .route("/", get(get_newsletters))
        .route("/{id}", get(get_newsletter));
    let protected = Router::new()
        .route("/", post(create_newsletter))
        .route("/{id}", put(update_newsletter).delete(delete_newsletter))
        .route_layer(middleware::from_fn_with_state(
            jwt_secret,
            crate::auth::auth_middleware,
        ));

    public.merge(protected)
}
