use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServiceRequest {
    pub id: String,
    pub service_id: String,
    pub seeker_id: String,
    pub message: String,
    pub status: String,
    pub work_status: String,
    pub decline_reason: String,
    pub declined_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
