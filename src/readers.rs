use crate::AppState;
use crate::entities::readers::{self, Entity as ReaderEntity};
use axum::{Json, Router, extract::{Request, State}, http::{StatusCode, header::AUTHORIZATION}, middleware, middleware::Next, response::Response, routing::{get, post}};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use email_address::EmailAddress;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug)] pub struct AuthenticatedReader { pub reader_id: Uuid }
#[derive(Clone)]
pub struct ReaderAuthState {
    pub secret: String,
    pub db: sea_orm::DatabaseConnection,
}
#[derive(Deserialize)] pub struct RegisterReader { pub name: String, pub email: String, pub password: String }
#[derive(Deserialize)] pub struct LoginReader { pub email: String, pub password: String }
#[derive(Serialize)] pub struct ReaderResponse { pub id: Uuid, pub name: String, pub email: String, pub created_at: chrono::NaiveDateTime }
#[derive(Serialize)] pub struct AuthResponse { pub token: String, pub reader: ReaderResponse }
#[derive(Serialize, Deserialize)] struct Claims { sub: String, exp: usize, iat: usize, token_use: String }

fn secret() -> String { std::env::var("JWT_SECRET").expect("JWT_SECRET must be set") }
fn generate_token(id: Uuid) -> Result<String, String> {
    let now = Utc::now(); let claims = Claims { sub: id.to_string(), exp: (now + Duration::hours(12)).timestamp() as usize, iat: now.timestamp() as usize, token_use: "reader_access".to_string() };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret().as_bytes())).map_err(|e| e.to_string())
}

pub fn routes(jwt_secret: String, db: sea_orm::DatabaseConnection) -> Router<AppState> {
    let public = Router::new().route("/register", post(register)).route("/login", post(login));
    let protected = Router::new().route("/session", get(session)).route_layer(middleware::from_fn_with_state(ReaderAuthState { secret: jwt_secret, db }, reader_middleware));
    public.merge(protected)
}

pub async fn register(State(state): State<AppState>, Json(payload): Json<RegisterReader>) -> Result<Json<AuthResponse>, StatusCode> {
    let email = payload.email.trim().to_ascii_lowercase(); let name = payload.name.trim();
    if name.is_empty() || !EmailAddress::is_valid(&email) || payload.password.trim().len() < 8 { return Err(StatusCode::BAD_REQUEST); }
    if ReaderEntity::find().filter(readers::Column::Email.eq(&email)).one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.is_some() { return Err(StatusCode::CONFLICT); }
    let password_hash = hash(payload.password.trim(), DEFAULT_COST).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let now = Utc::now().naive_utc();
    let saved = readers::ActiveModel { name: Set(name.to_string()), email: Set(email), password_hash: Set(password_hash), created_at: Set(now), ..Default::default() }.insert(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let reader = ReaderResponse { id: saved.id, name: saved.name, email: saved.email, created_at: saved.created_at };
    let token = generate_token(reader.id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(AuthResponse { token, reader }))
}

pub async fn login(State(state): State<AppState>, Json(payload): Json<LoginReader>) -> Result<Json<AuthResponse>, StatusCode> {
    let email = payload.email.trim().to_ascii_lowercase();
    let reader = ReaderEntity::find().filter(readers::Column::Email.eq(&email)).one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::UNAUTHORIZED)?;
    if !verify(payload.password.trim(), &reader.password_hash).unwrap_or(false) { return Err(StatusCode::UNAUTHORIZED); }
    let response = ReaderResponse { id: reader.id, name: reader.name, email: reader.email, created_at: reader.created_at };
    let token = generate_token(response.id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(AuthResponse { token, reader: response }))
}

pub async fn session(State(state): State<AppState>, axum::extract::Extension(user): axum::extract::Extension<AuthenticatedReader>) -> Result<Json<ReaderResponse>, StatusCode> {
    let reader = ReaderEntity::find_by_id(user.reader_id).one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(ReaderResponse { id: reader.id, name: reader.name, email: reader.email, created_at: reader.created_at }))
}

pub async fn reader_middleware(State(auth): State<ReaderAuthState>, mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let token = req.headers().get(AUTHORIZATION).and_then(|v| v.to_str().ok()).and_then(|v| v.strip_prefix("Bearer ")).ok_or(StatusCode::UNAUTHORIZED)?;
    let mut validation = Validation::new(Algorithm::HS256); validation.required_spec_claims.insert("sub".to_string());
    let claims = decode::<Claims>(token, &DecodingKey::from_secret(auth.secret.as_bytes()), &validation).map_err(|_| StatusCode::UNAUTHORIZED)?.claims;
    if claims.token_use != "reader_access" { return Err(StatusCode::UNAUTHORIZED); }
    let reader_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;
    if ReaderEntity::find_by_id(reader_id).one(&auth.db).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_none() { return Err(StatusCode::UNAUTHORIZED); }
    req.extensions_mut().insert(AuthenticatedReader { reader_id });
    Ok(next.run(req).await)
}
