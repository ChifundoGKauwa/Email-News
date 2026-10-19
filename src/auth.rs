use crate::{
    AppState,
    authors::model::{Author, CreateAuthor},
    entities::authors::{self, Entity as AuthorEntity},
};
use axum::{
    Json, Router,
    extract::{Request, State},
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
    routing::post,
};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use email_address::EmailAddress;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn generate_auth_token(subject: &str) -> Result<String, String> {
    let secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "email-news-dev-secret".to_string());
    let now = Utc::now();
    let claims = AuthClaims {
        sub: subject.to_string(),
        exp: (now + Duration::hours(12)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|err| format!("failed to generate auth token: {err}"))
}

pub fn verify_auth_token(token: &str) -> Result<String, String> {
    let secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "email-news-dev-secret".to_string());
    let claims = decode::<AuthClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|err| format!("invalid auth token: {err}"))?
    .claims;

    Ok(claims.sub)
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub fn hash_password(password: &str) -> Result<String, String> {
    hash(password, DEFAULT_COST).map_err(|err| format!("failed to hash password: {err}"))
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    verify(password, password_hash).unwrap_or(false)
}

async fn find_author_by_email(
    state: &AppState,
    email: &str,
) -> Result<Option<authors::Model>, StatusCode> {
    AuthorEntity::find()
        .filter(authors::Column::Email.eq(email.trim().to_lowercase()))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub author: Author,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register_author))
        .route("/login", post(login_author))
}

pub async fn register_author(
    State(state): State<AppState>,
    Json(payload): Json<CreateAuthor>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let email = payload.email.trim().to_lowercase();
    if email.is_empty() || !EmailAddress::is_valid(&email) {
        return Err(StatusCode::BAD_REQUEST);
    }

    if payload.password.trim().len() < 8 {
        return Err(StatusCode::BAD_REQUEST);
    }

    if AuthorEntity::find()
        .filter(authors::Column::Email.eq(&email))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some()
    {
        return Err(StatusCode::CONFLICT);
    }

    let password_hash =
        hash_password(payload.password.trim()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let now = Utc::now().naive_utc();
    let new_author = authors::ActiveModel {
        name: Set(payload.name.trim().to_string()),
        email: Set(email.clone()),
        email_hash: Set(String::new()),
        email_salt: Set(String::new()),
        password_hash: Set(password_hash),
        password_salt: Set(String::new()),
        bio: Set(payload.bio.clone()),
        created_at: Set(now),
        ..Default::default()
    };

    let saved = new_author
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token = generate_auth_token(&saved.id.to_string())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        token,
        author: Author {
            id: saved.id,
            name: saved.name,
            email: saved.email,
            bio: saved.bio,
            created_at: saved.created_at,
        },
    }))
}

pub async fn login_author(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let email = payload.email.trim().to_lowercase();
    if email.is_empty() || !EmailAddress::is_valid(&email) {
        return Err(StatusCode::BAD_REQUEST);
    }

    if payload.password.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let author = find_author_by_email(&state, &email)
        .await?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !verify_password(payload.password.trim(), &author.password_hash) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = generate_auth_token(&author.id.to_string())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        token,
        author: Author {
            id: author.id,
            name: author.name,
            email: author.email,
            bio: author.bio,
            created_at: author.created_at,
        },
    }))
}

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let authorization = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = authorization
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let subject = verify_auth_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(AuthenticatedUser { subject });

    Ok(next.run(req).await)
}

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub subject: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_and_verifies_password_with_bcrypt() {
        let password_hash = hash_password("StrongPass!123").unwrap();
        assert!(!password_hash.is_empty());
        assert!(verify_password("StrongPass!123", &password_hash));
        assert!(!verify_password("WrongPass!123", &password_hash));
    }
}
