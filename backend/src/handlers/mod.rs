pub mod auth;
pub mod messages;
pub mod reviews;
pub mod services;
pub mod surveys;

use actix_web::HttpRequest;
use crate::auth as jwt;

/// Shared helper — extracts and validates the Bearer token from any request.
/// Returns the user_id (sub claim) or an error string.
pub fn extract_user_id(req: &HttpRequest, jwt_secret: &str) -> Result<String, &'static str> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or("Missing or invalid Authorization header")?;

    jwt::validate_token(token, jwt_secret)
        .map(|c| c.sub)
        .map_err(|_| "Invalid or expired token")
}
