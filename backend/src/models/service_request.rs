use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceRequest {
    pub id: String,
    pub service_id: String,
    pub seeker_id: String,
    pub message: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
