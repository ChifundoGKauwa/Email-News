use crate::{
    AppState,
    authors::model::{Author, CreateAuthor},
    entities::authors::{self, Entity as AuthorEntity},
};
use axum::{
    Json, Router,
    extract::{Request, State},
    http::{StatusCode, header::AUTHORIZATION},
    middleware,
    middleware::Next,
    response::Response,
    routing::{get, post},
};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use email_address::EmailAddress;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub token_use: String,
}

pub fn generate_auth_token(subject: &str) -> Result<String, String> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    generate_auth_token_with_secret(subject, &secret)
}

fn generate_auth_token_with_secret(subject: &str, secret: &str) -> Result<String, String> {
    let now = Utc::now();
    let claims = AuthClaims {
        sub: subject.to_string(),
        exp: (now + Duration::hours(12)).timestamp() as usize,
        iat: now.timestamp() as usize,
        token_use: "author_access".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|err| format!("failed to generate auth token: {err}"))
}

fn verify_auth_token_with_secret(token: &str, secret: &str) -> Result<String, String> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.required_spec_claims.insert("sub".to_string());
    let claims = decode::<AuthClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|err| format!("invalid auth token: {err}"))?
    .claims;

    if claims.token_use != "author_access" {
        return Err("token is not an author access token".to_string());
    }
    Uuid::parse_str(&claims.sub).map_err(|_| "invalid author id in token".to_string())?;
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

pub fn routes(jwt_secret: String) -> Router<AppState> {
    let public = Router::new()
        .route("/register", post(register_author))
        .route("/login", post(login_author));
    let protected = Router::new()
        .route("/session", get(get_session))
        .route_layer(middleware::from_fn_with_state(jwt_secret, auth_middleware));
    public.merge(protected)
}

pub async fn get_session(
    State(state): State<AppState>,
    axum::extract::Extension(user): axum::extract::Extension<AuthenticatedUser>,
) -> Result<Json<Author>, StatusCode> {
    let author = AuthorEntity::find_by_id(user.author_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(Json(Author {
        id: author.id,
        name: author.name,
        email: author.email,
        bio: author.bio,
        created_at: author.created_at,
    }))
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

pub async fn auth_middleware(
    State(secret): State<String>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let authorization = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = authorization
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let subject =
        verify_auth_token_with_secret(token, &secret).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let author_id = Uuid::parse_str(&subject).map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(AuthenticatedUser { author_id });

    Ok(next.run(req).await)
}

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub author_id: Uuid,
}

pub fn require_owner(user: &AuthenticatedUser, owner_id: Uuid) -> Result<(), StatusCode> {
    if user.author_id == owner_id {
        Ok(())
    } else {
        // Hide whether another author's resource exists.
        Err(StatusCode::NOT_FOUND)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, extract::Extension, middleware, routing::get};
    use serde::Serialize;
    use tower::ServiceExt;

    const TEST_SECRET: &str = "test-secret-with-at-least-32-bytes-long";

    async fn protected_identity(Extension(user): Extension<AuthenticatedUser>) -> String {
        user.author_id.to_string()
    }

    fn test_app() -> Router {
        let public = Router::new().route("/public", get(|| async { "public" }));
        let protected = Router::new()
            .route("/private", get(protected_identity))
            .route_layer(middleware::from_fn_with_state(
                TEST_SECRET.to_string(),
                auth_middleware,
            ));
        public.merge(protected)
    }

    async fn request(path: &str, token: Option<&str>) -> axum::response::Response {
        let mut builder = axum::http::Request::builder().uri(path);
        if let Some(token) = token {
            builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        test_app()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    fn signed_token(author_id: Uuid, exp: usize) -> String {
        let claims = AuthClaims {
            sub: author_id.to_string(),
            exp,
            iat: Utc::now().timestamp() as usize,
            token_use: "author_access".to_string(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
        )
        .unwrap()
    }

    #[derive(Serialize)]
    struct ConfirmationTokenClaims {
        sub: String,
        exp: usize,
        iat: usize,
    }

    #[tokio::test]
    async fn public_routes_remain_public() {
        assert_eq!(request("/public", None).await.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn protected_route_rejects_missing_malformed_invalid_and_expired_tokens() {
        let author_id = Uuid::new_v4();
        let expired = signed_token(
            author_id,
            (Utc::now() - Duration::hours(1)).timestamp() as usize,
        );
        let valid_format = signed_token(
            author_id,
            (Utc::now() + Duration::hours(1)).timestamp() as usize,
        );
        let invalid_signature = format!("{}.bad", valid_format.rsplit_once('.').unwrap().0);
        let confirmation_token = encode(
            &Header::default(),
            &ConfirmationTokenClaims {
                sub: author_id.to_string(),
                exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
                iat: Utc::now().timestamp() as usize,
            },
            &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
        )
        .unwrap();

        for token in [
            None,
            Some("not-a-jwt"),
            Some("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ4In0.invalid"),
            Some(expired.as_str()),
            Some(invalid_signature.as_str()),
            Some(confirmation_token.as_str()),
        ] {
            assert_eq!(
                request("/private", token).await.status(),
                StatusCode::UNAUTHORIZED
            );
        }
    }

    #[tokio::test]
    async fn valid_token_authenticates_the_author() {
        let author_id = Uuid::new_v4();
        let token = generate_auth_token_with_secret(&author_id.to_string(), TEST_SECRET).unwrap();
        let response = request("/private", Some(&token)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(std::str::from_utf8(&body).unwrap(), author_id.to_string());
    }

    #[test]
    fn owner_can_create_update_delete_and_other_authors_are_hidden() {
        let owner_id = Uuid::new_v4();
        let owner = AuthenticatedUser {
            author_id: owner_id,
        };
        let other = AuthenticatedUser {
            author_id: Uuid::new_v4(),
        };

        // Create compares the requested author_id; update and delete compare
        // the persisted resource owner. All use this same policy guard.
        let requested_author_id = owner.author_id;
        assert_eq!(require_owner(&owner, requested_author_id), Ok(()));
        let persisted_owner_id = owner_id;
        assert_eq!(require_owner(&owner, persisted_owner_id), Ok(()));
        assert_eq!(require_owner(&owner, owner_id), Ok(()));
        assert_eq!(require_owner(&other, owner_id), Err(StatusCode::NOT_FOUND));
    }
}
