use crate::entities::{user, user_company};
use actix_web::{dev::ServiceRequest, Error as ActixError, HttpMessage};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32, // user_id
    pub username: String,
    pub group_id: i32,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub group_id: i32,
    pub is_active: bool,
    pub user: user::Model,
}

pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: i64,
}

impl JwtConfig {
    pub fn from_env() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your_jwt_secret_change_in_production".to_string()),
            expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(24),
        }
    }
}

/// Generate JWT token for user
pub fn generate_token(user: &user::Model, config: &JwtConfig) -> Result<String, String> {
    let now = Utc::now();
    let exp = now + Duration::hours(config.expiration_hours);

    let claims = Claims {
        sub: user.id,
        username: user.username.clone(),
        group_id: user.group_id,
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|e| format!("Failed to generate token: {}", e))
}

/// Validate JWT token and extract claims
pub fn validate_token(token: &str, config: &JwtConfig) -> Result<Claims, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| format!("Invalid token: {}", e))
}

/// Extract user from database using claims
pub async fn get_user_from_claims(
    claims: &Claims,
    db: &DatabaseConnection,
) -> Result<AuthenticatedUser, String> {
    let user = user::Entity::find_by_id(claims.sub)
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "User not found".to_string())?;

    if !user.is_active {
        return Err("User is not active".to_string());
    }

    Ok(AuthenticatedUser {
        id: user.id,
        username: user.username.clone(),
        email: user.email.clone(),
        group_id: user.group_id,
        is_active: user.is_active,
        user: user.clone(),
    })
}

/// Extract Bearer token from Authorization header
pub fn extract_bearer_token(req: &ServiceRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|auth| {
            if auth.starts_with("Bearer ") {
                Some(auth[7..].to_string())
            } else {
                None
            }
        })
}

/// Check if user has permission for a specific company
pub async fn user_can_access_company(
    user_id: i32,
    company_id: i32,
    db: &DatabaseConnection,
) -> Result<user_company::Model, String> {
    user_company::Entity::find()
        .filter(user_company::Column::UserId.eq(user_id))
        .filter(user_company::Column::CompanyId.eq(company_id))
        .filter(user_company::Column::IsActive.eq(true))
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Access denied to this company".to_string())
}

/// Check if user has admin role for a company
pub async fn user_is_company_admin(
    user_id: i32,
    company_id: i32,
    db: &DatabaseConnection,
) -> Result<bool, String> {
    let user_company = user_can_access_company(user_id, company_id, db).await?;
    Ok(user_company.role == user_company::UserCompanyRole::Admin)
}

/// Check if user can manage other users (global permission)
pub async fn user_can_manage_users(
    user: &AuthenticatedUser,
    db: &DatabaseConnection,
) -> Result<bool, String> {
    use crate::entities::user_group;

    let group = user_group::Entity::find_by_id(user.group_id)
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "User group not found".to_string())?;

    Ok(group.can_manage_users)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate_token() {
        let config = JwtConfig {
            secret: "test_secret".to_string(),
            expiration_hours: 24,
        };

        let user = user::Model {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            group_id: 1,
            is_active: true,
            document_period_start: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            document_period_end: chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            document_period_active: true,
            accounting_period_start: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            accounting_period_end: chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            accounting_period_active: true,
            vat_period_start: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            vat_period_end: chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            vat_period_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let token = generate_token(&user, &config).unwrap();
        let claims = validate_token(&token, &config).unwrap();

        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.group_id, user.group_id);
    }
}
