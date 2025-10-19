use crate::auth::{generate_token, JwtConfig};
use crate::entities::user::{self, AuthResponse, LoginInput};
use actix_web::{web, HttpResponse, Result};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::json;
use std::sync::Arc;

/// Public login endpoint (no authentication required)
pub async fn login(
    db: web::Data<Arc<DatabaseConnection>>,
    jwt_config: web::Data<Arc<JwtConfig>>,
    input: web::Json<LoginInput>,
) -> Result<HttpResponse> {
    let db = db.as_ref().as_ref();

    let user = user::Entity::find()
        .filter(user::Column::Username.eq(&input.username))
        .filter(user::Column::IsActive.eq(true))
        .one(db)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
        })?;

    if let Some(user) = user {
        let is_valid = user
            .verify_password(&input.password)
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!(
                    "Password verification failed: {}",
                    e
                ))
            })?;

        if is_valid {
            // Generate JWT token
            let token = generate_token(&user, jwt_config.as_ref().as_ref()).map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Token generation failed: {}", e))
            })?;

            let expires_at =
                chrono::Utc::now() + chrono::Duration::hours(jwt_config.expiration_hours);

            let response = AuthResponse {
                user,
                token,
                expires_at,
            };

            Ok(HttpResponse::Ok().json(response))
        } else {
            Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Invalid credentials"
            })))
        }
    } else {
        Ok(HttpResponse::Unauthorized().json(json!({
            "error": "Invalid credentials"
        })))
    }
}

/// Health check endpoint (public)
pub async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "rs-ac-bg accounting backend"
    })))
}
