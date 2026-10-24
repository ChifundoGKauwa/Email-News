mod auth;
mod authors;
mod newsletters;
mod subscribers;

use axum::Router;
use dotenvy::dotenv;
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement};
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
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let port = std::env::var("PORT").unwrap_or_else(|_| "8004".to_string());
    let port = port.parse::<u16>().expect("PORT must be a valid TCP port");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
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
