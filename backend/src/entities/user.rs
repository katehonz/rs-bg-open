use async_graphql::{InputObject, SimpleObject};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Datelike;
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "users")]
#[graphql(concrete(name = "User", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    #[graphql(skip)]
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub group_id: i32,
    pub is_active: bool,
    // Personal input periods - Documents
    pub document_period_start: Date,
    pub document_period_end: Date,
    pub document_period_active: bool,
    // Personal input periods - Accounting
    pub accounting_period_start: Date,
    pub accounting_period_end: Date,
    pub accounting_period_active: bool,
    // Personal input periods - VAT
    pub vat_period_start: Date,
    pub vat_period_end: Date,
    pub vat_period_active: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    // Recovery code fields
    #[graphql(skip)]
    pub recovery_code_hash: Option<String>,
    #[graphql(skip)]
    pub recovery_code_created_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user_group::Entity",
        from = "Column::GroupId",
        to = "super::user_group::Column::Id"
    )]
    UserGroup,
}

impl Related<super::user_group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserGroup.def()
    }
}

// Input types for GraphQL mutations
#[derive(InputObject, Deserialize, Serialize)]
pub struct CreateUserInput {
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub group_id: i32,
    pub is_active: Option<bool>,
}

#[derive(InputObject, Deserialize, Serialize)]
pub struct UpdateUserInput {
    pub username: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub group_id: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(InputObject, Deserialize, Serialize)]
pub struct UpdateInputPeriodsInput {
    // Document periods
    pub document_period_start: Option<Date>,
    pub document_period_end: Option<Date>,
    pub document_period_active: Option<bool>,
    // Accounting periods
    pub accounting_period_start: Option<Date>,
    pub accounting_period_end: Option<Date>,
    pub accounting_period_active: Option<bool>,
    // VAT periods
    pub vat_period_start: Option<Date>,
    pub vat_period_end: Option<Date>,
    pub vat_period_active: Option<bool>,
}

#[derive(SimpleObject, Serialize)]
pub struct InputPeriods {
    pub user_id: i32,
    // Document periods
    pub document_period_start: Date,
    pub document_period_end: Date,
    pub document_period_active: bool,
    // Accounting periods
    pub accounting_period_start: Date,
    pub accounting_period_end: Date,
    pub accounting_period_active: bool,
    // VAT periods
    pub vat_period_start: Date,
    pub vat_period_end: Date,
    pub vat_period_active: bool,
}

impl From<&Model> for InputPeriods {
    fn from(user: &Model) -> Self {
        Self {
            user_id: user.id,
            document_period_start: user.document_period_start,
            document_period_end: user.document_period_end,
            document_period_active: user.document_period_active,
            accounting_period_start: user.accounting_period_start,
            accounting_period_end: user.accounting_period_end,
            accounting_period_active: user.accounting_period_active,
            vat_period_start: user.vat_period_start,
            vat_period_end: user.vat_period_end,
            vat_period_active: user.vat_period_active,
        }
    }
}

impl Model {
    /// Hash a password using bcrypt
    pub fn hash_password(password: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(hash(password, DEFAULT_COST)?)
    }

    /// Verify a password against the stored hash
    pub fn verify_password(&self, password: &str) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(verify(password, &self.password_hash)?)
    }

    /// Verify a recovery code against the stored hash
    pub fn verify_recovery_code(&self, code: &str) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(ref hash) = self.recovery_code_hash {
            Ok(verify(code, hash)?)
        } else {
            Ok(false)
        }
    }

    /// Check if recovery code is expired (older than 90 days)
    pub fn is_recovery_code_expired(&self) -> bool {
        if let Some(created_at) = self.recovery_code_created_at {
            let now = chrono::Utc::now();
            let age = now.signed_duration_since(created_at);
            age.num_days() > 90
        } else {
            true // No code = expired
        }
    }

    /// Check if user can input documents for a given date
    pub fn can_input_document(&self, date: chrono::NaiveDate) -> bool {
        if !self.document_period_active || !self.is_active {
            return false;
        }
        date >= self.document_period_start && date <= self.document_period_end
    }

    /// Check if user can input accounting entries for a given date
    pub fn can_input_accounting(&self, date: chrono::NaiveDate) -> bool {
        if !self.accounting_period_active || !self.is_active {
            return false;
        }
        date >= self.accounting_period_start && date <= self.accounting_period_end
    }

    /// Check if user can input VAT entries for a given date
    pub fn can_input_vat(&self, date: chrono::NaiveDate) -> bool {
        if !self.vat_period_active || !self.is_active {
            return false;
        }
        date >= self.vat_period_start && date <= self.vat_period_end
    }

    /// Get user's full name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Check if all input periods are active
    pub fn has_all_periods_active(&self) -> bool {
        self.document_period_active && self.accounting_period_active && self.vat_period_active
    }
}

impl From<CreateUserInput> for ActiveModel {
    fn from(input: CreateUserInput) -> Self {
        ActiveModel {
            username: Set(input.username),
            email: Set(input.email),
            password_hash: Set(Self::hash_password(&input.password).unwrap_or_default()),
            first_name: Set(input.first_name),
            last_name: Set(input.last_name),
            group_id: Set(input.group_id),
            is_active: Set(input.is_active.unwrap_or(true)),
            // Set default periods for new users
            document_period_start: Set(chrono::Utc::now().date_naive()),
            document_period_end: Set(chrono::Utc::now().date_naive()),
            document_period_active: Set(true),
            accounting_period_start: Set(chrono::Utc::now().date_naive()),
            accounting_period_end: Set(chrono::Utc::now().date_naive()),
            accounting_period_active: Set(true),
            vat_period_start: Set(chrono::Utc::now().date_naive()),
            vat_period_end: Set(chrono::Utc::now().date_naive()),
            vat_period_active: Set(true),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
    }
}

impl ActiveModel {
    /// Hash a password using bcrypt
    pub fn hash_password(password: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(hash(password, DEFAULT_COST)?)
    }
}

// Authentication related structs
#[derive(InputObject, Deserialize, Serialize)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(SimpleObject, Serialize)]
pub struct AuthResponse {
    pub user: Model,
    pub token: String,
    pub expires_at: DateTimeUtc,
}

// User filtering and searching
#[derive(InputObject, Deserialize)]
pub struct UserFilter {
    pub group_id: Option<i32>,
    pub is_active: Option<bool>,
    pub search_term: Option<String>, // Search in name, username, email
    pub has_document_period_active: Option<bool>,
    pub has_accounting_period_active: Option<bool>,
    pub has_vat_period_active: Option<bool>,
}

// User with role information
#[derive(SimpleObject, Serialize)]
pub struct UserWithRole {
    pub user: Model,
    pub role: super::user_group::Model,
    pub input_periods: InputPeriods,
}
impl ActiveModelBehavior for ActiveModel {}
