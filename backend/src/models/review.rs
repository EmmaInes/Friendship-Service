use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Review {
    pub id: String,
    pub request_id: String,
    pub reviewer_id: String,
    pub reviewee_id: String,
    pub reviewer_role: String,
    pub rating: i32,
    pub comment: String,
    pub created_at: String,
}
