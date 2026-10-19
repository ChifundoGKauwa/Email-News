use chrono::{Duration, NaiveDateTime, Utc};
use email_address::EmailAddress;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PENDING_SUBSCRIPTION_STATUS: &str = "pending_confirmation";
pub const CONFIRMED_SUBSCRIPTION_STATUS: &str = "confirmed";

#[derive(Debug, Serialize)]
pub struct Subscriber {
    pub id: Uuid,
    pub email: String,
    pub subscribed_at: NaiveDateTime,
    pub name: String,
    pub status: String,
    pub confirmation_token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSubscriber {
    pub email: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subscriberresponse {
    pub email: String,
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfirmationClaims {
    sub: String,
    exp: usize,
    iat: usize,
}

pub fn validate_email(email: &str) -> Result<(), String> {
    let trimmed = email.trim();

    if trimmed.is_empty() {
        return Err("Email address is required.".to_string());
    }

    if !EmailAddress::is_valid(trimmed) {
        return Err("Please provide a valid email address.".to_string());
    }

    Ok(())
}

pub fn generate_confirmation_token(subscriber_id: Uuid) -> Result<String, String> {
    let secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "email-news-dev-secret".to_string());
    let now = Utc::now();
    let claims = ConfirmationClaims {
        sub: subscriber_id.to_string(),
        exp: (now + Duration::hours(24)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|err| format!("failed to encode confirmation token: {err}"))
}

pub fn verify_confirmation_token(token: &str) -> Result<Uuid, String> {
    let secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "email-news-dev-secret".to_string());
    let decoded = decode::<ConfirmationClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|err| format!("invalid or expired confirmation token: {err}"))?;

    Uuid::parse_str(&decoded.claims.sub)
        .map_err(|err| format!("invalid subscriber id in confirmation token: {err}"))
}
