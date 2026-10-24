pub mod handlers;
pub mod model;

use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::AppState;
use handlers::{
    confirm_subscription, create_subscriber, delete_subscriber, get_subscriber, get_subscribers,
};

pub fn routes(jwt_secret: String) -> Router<AppState> {
    let public = Router::new()
        .route("/", post(create_subscriber))
        .route("/confirm/{token}", get(confirm_subscription));
    let protected = Router::new()
        .route("/", get(get_subscribers))
        .route("/{id}", get(get_subscriber).delete(delete_subscriber))
        .route_layer(middleware::from_fn_with_state(
            jwt_secret,
            crate::auth::auth_middleware,
        ));

    public.merge(protected)
}
