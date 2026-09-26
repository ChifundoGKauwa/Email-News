use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Subscriber {
    pub id: Uuid,
    pub email: String,
    pub subscribed_at: NaiveDateTime,
    pub name: String,
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
