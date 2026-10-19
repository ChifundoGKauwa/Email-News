use crate::AppState;
use crate::entities::subscribers::{self, Entity as SubscriberEntity};
use crate::subscribers::model::{
    CONFIRMED_SUBSCRIPTION_STATUS, CreateSubscriber, PENDING_SUBSCRIPTION_STATUS, Subscriber,
    generate_confirmation_token, validate_email, verify_confirmation_token,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use lettre::{
    Message, SmtpTransport, Transport, message::Mailbox,
    transport::smtp::authentication::Credentials,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::env;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
pub enum SmtpSecurity {
    StartTls,
    Tls,
}

fn smtp_security_mode(smtp_host: &str, smtp_port: u16) -> SmtpSecurity {
    if uses_gmail_smtp(smtp_host) || smtp_port == 587 || smtp_port == 25 {
        SmtpSecurity::StartTls
    } else if smtp_port == 465 {
        SmtpSecurity::Tls
    } else {
        SmtpSecurity::StartTls
    }
}

fn resolve_sender_email() -> String {
    let configured_sender = env::var("EMAIL_FROM").ok();
    let smtp_username = env::var("SMTP_USERNAME").ok();

    let use_gmail_sender = |value: &str| {
        value.trim().to_ascii_lowercase().ends_with("@gmail.com")
            || value
                .trim()
                .to_ascii_lowercase()
                .ends_with("@googlemail.com")
    };

    if let Some(sender) = configured_sender
        .as_ref()
        .filter(|value| use_gmail_sender(value))
    {
        return sender.clone();
    }

    if let Some(username) = smtp_username
        .as_ref()
        .filter(|value| use_gmail_sender(value))
    {
        return username.clone();
    }

    if let Some(username) = smtp_username {
        return username;
    }

    configured_sender.unwrap_or_else(|| "no-reply@example.com".to_string())
}

fn uses_gmail_smtp(smtp_host: &str) -> bool {
    let host = smtp_host.trim().to_ascii_lowercase();
    host == "smtp.gmail.com"
        || host == "smtp.googlemail.com"
        || host.ends_with(".gmail.com")
        || host.ends_with(".googlemail.com")
}

fn normalize_smtp_password(password: &str) -> String {
    password.split_whitespace().collect()
}

fn validate_smtp_configuration(smtp_host: &str) -> Result<(), String> {
    if !uses_gmail_smtp(smtp_host) {
        return Ok(());
    }

    let username = env::var("SMTP_USERNAME").ok();
    let password = env::var("SMTP_PASSWORD")
        .ok()
        .map(|value| normalize_smtp_password(&value));

    match (username, password) {
        (Some(_), Some(password)) if !password.is_empty() => Ok(()),
        _ => Err(
            "Gmail SMTP requires an app password. Set SMTP_USERNAME to your full Gmail address and SMTP_PASSWORD to a 16-character Google app password, not your normal Gmail password."
                .to_string(),
        ),
    }
}

fn build_confirmation_link(token: &str) -> String {
    let base_url = env::var("APP_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8001".to_string());
    format!("{base_url}/subscribers/confirm/{token}")
}

fn send_confirmation_email(recipient: &str, link: &str) -> Result<(), String> {
    let smtp_host = env::var("SMTP_HOST").ok();
    if smtp_host.is_none() {
        println!("[EMAIL] Confirmation link for {recipient}: {link}");
        return Ok(());
    }

    let smtp_host = smtp_host.unwrap();
    validate_smtp_configuration(&smtp_host)?;

    let smtp_port = env::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string());
    let from_email = resolve_sender_email();
    let username = env::var("SMTP_USERNAME").ok();
    let password = env::var("SMTP_PASSWORD")
        .ok()
        .map(|value| normalize_smtp_password(&value));

    let from: Mailbox = format!("Email News <{from_email}>")
        .parse()
        .map_err(|err| format!("invalid sender email: {err}"))?;
    let to: Mailbox = recipient
        .parse()
        .map_err(|err| format!("invalid recipient email: {err}"))?;

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject("Confirm your newsletter subscription")
        .body(format!(
            "You are receiving this email because you subscribed to our newsletter.\n\nClick the link below to confirm your subscription:\n\n{link}\n\nIf you did not request this, you can safely ignore this email."
        ))
        .map_err(|err| format!("failed to build email message: {err}"))?;

    let smtp_port_number = smtp_port.parse::<u16>().unwrap_or(587);
    let mut transport = match smtp_security_mode(&smtp_host, smtp_port_number) {
        SmtpSecurity::Tls => SmtpTransport::relay(&smtp_host),
        SmtpSecurity::StartTls => SmtpTransport::starttls_relay(&smtp_host),
    }
    .map_err(|err| format!("failed to configure mail transport: {err}"))?;

    if let (Some(username), Some(password)) = (username, password) {
        transport = transport.credentials(Credentials::new(username, password));
    }

    transport = transport.port(smtp_port_number);
    let mailer = transport.build();

    mailer
        .send(&email)
        .map_err(|err| format!("failed to send confirmation email: {err}"))?;
    Ok(())
}

// get all subscribers
pub async fn get_subscribers(
    State(state): State<AppState>,
) -> Result<Json<Vec<Subscriber>>, StatusCode> {
    let subscribers = SubscriberEntity::find()
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = subscribers
        .into_iter()
        .map(|s| Subscriber {
            id: s.id,
            email: s.email,
            subscribed_at: s.subscribed_at,
            name: s.name,
            status: s.status,
            confirmation_token: Some(s.confirmation_token),
        })
        .collect();

    Ok(Json(result))
}

// get subscriber by ID
pub async fn get_subscriber(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Subscriber>, StatusCode> {
    let subscriber = SubscriberEntity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(Subscriber {
        id: subscriber.id,
        email: subscriber.email,
        subscribed_at: subscriber.subscribed_at,
        name: subscriber.name,
        status: subscriber.status,
        confirmation_token: Some(subscriber.confirmation_token),
    }))
}

// create subscriber
pub async fn create_subscriber(
    State(state): State<AppState>,
    Json(payload): Json<CreateSubscriber>,
) -> Result<Json<Subscriber>, StatusCode> {
    validate_email(&payload.email).map_err(|message| {
        eprintln!("{message}");
        StatusCode::BAD_REQUEST
    })?;

    let existing_subscriber = SubscriberEntity::find()
        .filter(crate::entities::subscribers::Column::Email.eq(payload.email.trim()))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing_subscriber.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    let subscriber_id = Uuid::new_v4();
    let confirmation_token = generate_confirmation_token(subscriber_id).map_err(|err| {
        eprintln!("{err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let confirmation_link = build_confirmation_link(&confirmation_token);

    let new_subscriber = subscribers::ActiveModel {
        id: Set(subscriber_id),
        email: Set(payload.email.trim().to_string()),
        name: Set(payload.name.trim().to_string()),
        subscribed_at: Set(Utc::now().naive_utc()),
        status: Set(PENDING_SUBSCRIPTION_STATUS.to_string()),
        confirmation_token: Set(confirmation_token.clone()),
    };

    let saved = new_subscriber.insert(&state.db).await.map_err(|err| {
        eprintln!("failed to insert subscriber: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Err(err) = send_confirmation_email(&saved.email, &confirmation_link) {
        eprintln!("failed to send confirmation email: {err}");
    }

    Ok(Json(Subscriber {
        id: saved.id,
        email: saved.email,
        subscribed_at: saved.subscribed_at,
        name: saved.name,
        status: saved.status,
        confirmation_token: Some(saved.confirmation_token),
    }))
}

pub async fn confirm_subscription(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    let subscriber_id = match verify_confirmation_token(&token) {
        Ok(id) => id,
        Err(err) => {
            eprintln!("{err}");
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "message": "Invalid or expired confirmation token." })),
            );
        }
    };

    let subscriber = match SubscriberEntity::find_by_id(subscriber_id)
        .one(&state.db)
        .await
        .map_err(|err| {
            eprintln!("failed to query subscriber: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }) {
        Ok(Some(subscriber)) => subscriber,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "message": "Subscriber not found." })),
            );
        }
        Err(status_code) => {
            return (
                status_code,
                Json(serde_json::json!({ "message": "Internal server error." })),
            );
        }
    };

    let subscriber_email = subscriber.email.clone();
    let mut active_subscriber: subscribers::ActiveModel = subscriber.into();
    active_subscriber.status = Set(CONFIRMED_SUBSCRIPTION_STATUS.to_string());
    active_subscriber.confirmation_token = Set(String::new());

    if let Err(err) = active_subscriber.update(&state.db).await {
        eprintln!("failed to confirm subscriber: {err}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": "Could not confirm subscription." })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Subscription confirmed successfully.",
            "email": subscriber_email,
            "status": CONFIRMED_SUBSCRIPTION_STATUS,
        })),
    )
}

// delete subscriber
pub async fn delete_subscriber(State(state): State<AppState>, Path(id): Path<Uuid>) -> StatusCode {
    println!("deleting the subscriber {}", id);

    match SubscriberEntity::delete_by_id(id).exec(&state.db).await {
        Ok(res) if res.rows_affected > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
