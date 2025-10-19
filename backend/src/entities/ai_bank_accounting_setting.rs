use async_graphql::{InputObject, SimpleObject};
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "ai_bank_accounting_settings")]
#[graphql(name = "AiBankAccountingSetting")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub company_id: i32,

    // Pattern matching
    pub pattern_name: String,
    pub description_keywords: Option<String>,

    // Transaction classification
    pub transaction_type: String,

    // Accounting configuration
    pub account_id: Option<i32>,
    pub counterpart_account_id: Option<i32>,
    pub vat_account_id: Option<i32>,

    // Transaction direction
    pub direction: String, // "debit" or "credit"

    // Description template
    pub description_template: Option<String>,

    // Priority for pattern matching
    pub priority: i32,

    // Active flag
    pub is_active: bool,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::CompanyId",
        to = "super::company::Column::Id",
        on_delete = "Cascade"
    )]
    Company,

    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id",
        on_delete = "SetNull"
    )]
    Account,

    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::CounterpartAccountId",
        to = "super::account::Column::Id",
        on_delete = "SetNull"
    )]
    CounterpartAccount,

    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::VatAccountId",
        to = "super::account::Column::Id",
        on_delete = "SetNull"
    )]
    VatAccount,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Input types for GraphQL
#[derive(InputObject)]
pub struct CreateAiBankAccountingSettingInput {
    pub company_id: i32,
    pub pattern_name: String,
    pub description_keywords: Option<String>,
    pub transaction_type: String,
    pub account_id: Option<i32>,
    pub counterpart_account_id: Option<i32>,
    pub vat_account_id: Option<i32>,
    pub direction: Option<String>, // defaults to "debit"
    pub description_template: Option<String>,
    pub priority: Option<i32>, // defaults to 0
    pub is_active: Option<bool>, // defaults to true
}

#[derive(InputObject)]
pub struct UpdateAiBankAccountingSettingInput {
    pub pattern_name: Option<String>,
    pub description_keywords: Option<String>,
    pub transaction_type: Option<String>,
    pub account_id: Option<i32>,
    pub counterpart_account_id: Option<i32>,
    pub vat_account_id: Option<i32>,
    pub direction: Option<String>,
    pub description_template: Option<String>,
    pub priority: Option<i32>,
    pub is_active: Option<bool>,
}

impl From<CreateAiBankAccountingSettingInput> for ActiveModel {
    fn from(input: CreateAiBankAccountingSettingInput) -> Self {
        Self {
            company_id: Set(input.company_id),
            pattern_name: Set(input.pattern_name),
            description_keywords: Set(input.description_keywords),
            transaction_type: Set(input.transaction_type),
            account_id: Set(input.account_id),
            counterpart_account_id: Set(input.counterpart_account_id),
            vat_account_id: Set(input.vat_account_id),
            direction: Set(input.direction.unwrap_or_else(|| "debit".to_string())),
            description_template: Set(input.description_template),
            priority: Set(input.priority.unwrap_or(0)),
            is_active: Set(input.is_active.unwrap_or(true)),
            ..Default::default()
        }
    }
}

impl Model {
    /// Check if this pattern matches the given description keywords
    pub fn matches_description(&self, description: &str) -> bool {
        if !self.is_active {
            return false;
        }

        let description_lower = description.to_lowercase();

        // Check if description_keywords are present
        if let Some(ref keywords) = self.description_keywords {
            let keywords_list: Vec<&str> = keywords.split(',').map(|s| s.trim()).collect();

            // Check if ANY keyword is present in the description
            for keyword in keywords_list {
                if !keyword.is_empty() && description_lower.contains(&keyword.to_lowercase()) {
                    return true;
                }
            }
            false
        } else {
            // If no keywords specified, match by pattern name
            description_lower.contains(&self.pattern_name.to_lowercase())
        }
    }

    /// Get formatted description using template
    pub fn format_description(&self, counterpart: Option<&str>, description: &str) -> String {
        if let Some(ref template) = self.description_template {
            template
                .replace("{counterpart}", counterpart.unwrap_or(""))
                .replace("{description}", description)
        } else {
            counterpart.unwrap_or(description).to_string()
        }
    }
}
