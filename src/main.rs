mod auth;
mod authors;
mod newsletters;
mod subscribers;

use axum::Router;
use dotenvy::dotenv;
#[cfg(test)]
use sea_orm::ConnectOptions;
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement};
use std::env;
mod entities;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

async fn ensure_subscriber_schema(db: &DatabaseConnection) {
    let statements = [
        "ALTER TABLE subscribers ADD COLUMN IF NOT EXISTS status VARCHAR NOT NULL DEFAULT 'pending_confirmation';",
        "ALTER TABLE subscribers ADD COLUMN IF NOT EXISTS confirmation_token VARCHAR NOT NULL DEFAULT '';",
    ];

    for sql in statements {
        if let Err(err) = db
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                sql.to_string(),
            ))
            .await
        {
            eprintln!("failed to ensure subscriber schema: {err}");
        }
    }
}

async fn ensure_author_auth_schema(db: &DatabaseConnection) {
    let statements = [
        "ALTER TABLE authors ADD COLUMN IF NOT EXISTS password_hash VARCHAR NOT NULL DEFAULT '';",
        "ALTER TABLE authors ADD COLUMN IF NOT EXISTS password_salt VARCHAR NOT NULL DEFAULT '';",
        "ALTER TABLE authors ADD COLUMN IF NOT EXISTS email_hash VARCHAR NOT NULL DEFAULT '';",
        "ALTER TABLE authors ADD COLUMN IF NOT EXISTS email_salt VARCHAR NOT NULL DEFAULT '';",
    ];

    for sql in statements {
        if let Err(err) = db
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                sql.to_string(),
            ))
            .await
        {
            eprintln!("failed to ensure author auth schema: {err}");
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let port = env::var("PORT").unwrap_or_else(|_| "8004".to_string());
    let port = port.parse::<u16>().expect("PORT must be a valid TCP port");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    assert!(
        jwt_secret.len() >= 32,
        "JWT_SECRET must be at least 32 bytes long"
    );
    let db = Database::connect(&db_url)
        .await
        .expect("failed to connect to database");

    ensure_subscriber_schema(&db).await;
    ensure_author_auth_schema(&db).await;

    let state = AppState { db };

    let app = Router::new()
        .nest("/auth", auth::routes(jwt_secret.clone()))
        .nest("/authors", authors::routes(jwt_secret.clone()))
        .nest("/newsletters", newsletters::routes(jwt_secret.clone()))
        .nest("/subscribers", subscribers::routes(jwt_secret))
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|err| panic!("failed to bind to {addr}: {err}"));
    println!("server running on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod route_policy_tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    use uuid::Uuid;

    const TEST_SECRET: &str = "test-secret-with-at-least-32-bytes-long";

    async fn test_app() -> Router {
        let mut options = ConnectOptions::new("postgres://localhost/noted_test");
        options.connect_lazy(true);
        let state = AppState {
            db: Database::connect(options).await.unwrap(),
        };
        Router::new()
            .nest("/auth", auth::routes(TEST_SECRET.to_string()))
            .nest("/authors", authors::routes(TEST_SECRET.to_string()))
            .nest("/newsletters", newsletters::routes(TEST_SECRET.to_string()))
            .nest("/subscribers", subscribers::routes(TEST_SECRET.to_string()))
            .with_state(state)
    }

    async fn status(app: &Router, method: &str, uri: &str, body: &str) -> axum::http::StatusCode {
        app.clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_owned()))
                    .unwrap(),
            )
            .await
            .unwrap()
            .status()
    }

    #[tokio::test]
    async fn session_endpoint_requires_a_validated_bearer_token() {
        let app = test_app().await;
        assert_eq!(
            status(&app, "GET", "/auth/session", "").await,
            axum::http::StatusCode::UNAUTHORIZED,
        );
    }
}
