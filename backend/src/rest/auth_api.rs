use crate::auth::{generate_token, JwtConfig};
use crate::entities::user::{self, AuthResponse, LoginInput};
use actix_web::{web, HttpResponse, Result};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Deserialize)]
pub struct RecoverPasswordInput {
    pub username: String,
    pub recovery_code: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct RecoverPasswordResponse {
    pub success: bool,
    pub message: String,
}

/// Password recovery endpoint (public)
/// NOTE: This is a temporary implementation without recovery code validation
/// TODO: Add recovery_code_hash field to users table and implement proper validation
pub async fn recover_password(
    db: web::Data<Arc<DatabaseConnection>>,
    input: web::Json<RecoverPasswordInput>,
) -> Result<HttpResponse> {
    let db = db.as_ref().as_ref();

    // Find user by username
    let user = user::Entity::find()
        .filter(user::Column::Username.eq(&input.username))
        .one(db)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
        })?;

    if let Some(user) = user {
        // Validate recovery code format
        if input.recovery_code.len() != 9 || !input.recovery_code.contains('-') {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Невалиден формат на код за възстановяване"
            })));
        }

        // Check if user has a recovery code
        if user.recovery_code_hash.is_none() {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Нямате генериран код за възстановяване"
            })));
        }

        // Check if recovery code is expired
        if user.is_recovery_code_expired() {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Кодът за възстановяване е изтекъл. Моля, генерирайте нов код."
            })));
        }

        // Verify the recovery code
        let is_valid = user.verify_recovery_code(&input.recovery_code)
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!(
                    "Recovery code verification failed: {}",
                    e
                ))
            })?;

        if !is_valid {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Невалиден код за възстановяване"
            })));
        }

        // Validate new password length
        if input.new_password.len() < 6 {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Новата парола трябва да е поне 6 символа"
            })));
        }

        // Hash the new password
        let password_hash = user::Model::hash_password(&input.new_password)
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!(
                    "Password hashing failed: {}",
                    e
                ))
            })?;

        // Update user password and clear recovery code (one-time use)
        let mut user_active: user::ActiveModel = user.into();
        user_active.password_hash = Set(password_hash);
        user_active.recovery_code_hash = Set(None);
        user_active.recovery_code_created_at = Set(None);

        user_active.update(db).await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to update password: {}", e))
        })?;

        Ok(HttpResponse::Ok().json(RecoverPasswordResponse {
            success: true,
            message: "Паролата е променена успешно".to_string(),
        }))
    } else {
        Ok(HttpResponse::BadRequest().json(json!({
            "error": "Потребителят не е намерен"
        })))
    }
}
