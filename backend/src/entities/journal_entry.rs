use async_graphql::{InputObject, SimpleObject};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "journal_entries")]
#[graphql(concrete(name = "JournalEntry", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub entry_number: String,
    // Bulgarian triple date system
    pub document_date: Date,
    pub vat_date: Option<Date>,
    pub accounting_date: Date,
    pub document_number: Option<String>,
    pub description: String,
    pub total_amount: Decimal,
    pub total_vat_amount: Decimal,
    pub is_posted: bool,
    pub posted_by: Option<i32>,
    pub posted_at: Option<DateTimeUtc>,
    pub created_by: i32,
    pub company_id: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,

    // Bulgarian VAT codes according to PPZDDS requirements
    pub vat_document_type: Option<String>,
    pub vat_purchase_operation: Option<String>,
    pub vat_sales_operation: Option<String>,
    pub vat_additional_operation: Option<String>,
    pub vat_additional_data: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::CompanyId",
        to = "super::company::Column::Id"
    )]
    Company,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::CreatedBy",
        to = "super::user::Column::Id"
    )]
    CreatedByUser,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::PostedBy",
        to = "super::user::Column::Id"
    )]
    PostedByUser,
    #[sea_orm(has_many = "super::entry_line::Entity")]
    EntryLines,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl Related<super::entry_line::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EntryLines.def()
    }
}

// Input types for GraphQL mutations
#[derive(Clone, InputObject, Deserialize, Serialize)]
pub struct CreateJournalEntryInput {
    pub entry_number: Option<String>,
    pub document_date: Date,
    pub vat_date: Option<Date>,
    pub accounting_date: Date,
    pub document_number: Option<String>,
    pub description: String,
    pub company_id: i32,
    pub lines: Vec<CreateEntryLineInput>,

    // Bulgarian VAT codes according to PPZDDS requirements
    pub vat_document_type: Option<String>,
    pub vat_purchase_operation: Option<String>,
    pub vat_sales_operation: Option<String>,
    pub vat_additional_operation: Option<String>,
    pub vat_additional_data: Option<String>,
}

#[derive(InputObject, Deserialize, Serialize)]
pub struct UpdateJournalEntryInput {
    pub document_date: Option<Date>,
    pub vat_date: Option<Date>,
    pub accounting_date: Option<Date>,
    pub document_number: Option<String>,
    pub description: Option<String>,
    pub lines: Option<Vec<CreateEntryLineInput>>,
}

#[derive(Clone, InputObject, Deserialize, Serialize)]
pub struct CreateEntryLineInput {
    pub account_id: i32,
    pub debit_amount: Option<Decimal>,
    pub credit_amount: Option<Decimal>,
    pub counterpart_id: Option<i32>,
    pub currency_code: Option<String>,
    pub currency_amount: Option<Decimal>,
    pub exchange_rate: Option<Decimal>,
    pub vat_amount: Option<Decimal>,
    pub quantity: Option<Decimal>,
    pub unit_of_measure_code: Option<String>,
    pub description: Option<String>,
    pub line_order: Option<i32>,
}

// Response type that includes entry lines
#[derive(SimpleObject, Serialize)]
pub struct JournalEntryWithLines {
    #[graphql(flatten)]
    pub journal_entry: Model,
    pub lines: Vec<super::entry_line::Model>,
}

// Filter input for queries
#[derive(InputObject, Deserialize)]
pub struct JournalEntryFilter {
    pub company_id: Option<i32>,
    pub from_date: Option<Date>,
    pub to_date: Option<Date>,
    pub account_id: Option<i32>,
    pub is_posted: Option<bool>,
    pub created_by: Option<i32>,
    pub document_number: Option<String>,
}

impl From<CreateJournalEntryInput> for ActiveModel {
    fn from(input: CreateJournalEntryInput) -> Self {
        ActiveModel {
            entry_number: Set(input.entry_number.unwrap_or_else(|| {
                // Generate entry number - will be improved with proper sequence
                format!("JE-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"))
            })),
            document_date: Set(input.document_date),
            vat_date: Set(input.vat_date),
            accounting_date: Set(input.accounting_date),
            document_number: Set(input.document_number),
            description: Set(input.description),
            company_id: Set(input.company_id),
            vat_document_type: Set(input.vat_document_type),
            vat_purchase_operation: Set(input.vat_purchase_operation),
            vat_sales_operation: Set(input.vat_sales_operation),
            vat_additional_operation: Set(input.vat_additional_operation),
            vat_additional_data: Set(input.vat_additional_data),
            // total_amount and total_vat_amount will be calculated from lines
            ..Default::default()
        }
    }
}
impl ActiveModelBehavior for ActiveModel {}
