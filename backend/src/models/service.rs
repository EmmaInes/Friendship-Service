use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Service {
    pub id: String,
    pub provider_id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub price_cents: Option<i64>,
    pub price_type: String,
    pub location: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
