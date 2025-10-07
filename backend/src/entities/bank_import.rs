use async_graphql::{Enum, SimpleObject};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "bank_imports")]
#[graphql(concrete(name = "BankImport", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub bank_profile_id: i32,
    pub company_id: i32,
    pub file_name: String,
    pub import_format: String,
    pub imported_at: DateTimeUtc,
    pub transactions_count: i32,
    pub total_credit: Decimal,
    pub total_debit: Decimal,
    pub created_journal_entries: i32,
    pub journal_entry_ids: Option<Value>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_by: Option<i32>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::bank_profile::Entity",
        from = "Column::BankProfileId",
        to = "super::bank_profile::Column::Id"
    )]
    BankProfile,
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::CompanyId",
        to = "super::company::Column::Id"
    )]
    Company,
}

impl Related<super::bank_profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BankProfile.def()
    }
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum BankImportStatus {
    #[graphql(name = "COMPLETED")]
    Completed,
    #[graphql(name = "FAILED")]
    Failed,
    #[graphql(name = "IN_PROGRESS")]
    InProgress,
}

impl BankImportStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BankImportStatus::Completed => "completed",
            BankImportStatus::Failed => "failed",
            BankImportStatus::InProgress => "in_progress",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "in_progress" => Some(Self::InProgress),
            _ => None,
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
