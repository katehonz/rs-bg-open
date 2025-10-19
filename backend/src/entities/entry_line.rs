use async_graphql::{InputObject, SimpleObject};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "entry_lines")]
#[graphql(concrete(name = "EntryLine", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub journal_entry_id: i32,
    pub account_id: i32,
    pub debit_amount: Decimal,
    pub credit_amount: Decimal,
    pub counterpart_id: Option<i32>,
    pub currency_code: Option<String>,
    pub currency_amount: Option<Decimal>,
    pub exchange_rate: Option<Decimal>,
    pub base_amount: Decimal,
    pub vat_amount: Decimal,
    pub vat_rate_id: Option<i32>,
    pub quantity: Option<Decimal>,
    pub unit_of_measure_code: Option<String>,
    pub description: Option<String>,
    pub line_order: i32,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::journal_entry::Entity",
        from = "Column::JournalEntryId",
        to = "super::journal_entry::Column::Id"
    )]
    JournalEntry,
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id"
    )]
    Account,
    #[sea_orm(
        belongs_to = "super::counterpart::Entity",
        from = "Column::CounterpartId",
        to = "super::counterpart::Column::Id"
    )]
    Counterpart,
}

impl Related<super::journal_entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::JournalEntry.def()
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::counterpart::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Counterpart.def()
    }
}

// Input types for GraphQL mutations
#[derive(InputObject, Deserialize, Serialize)]
pub struct CreateEntryLineInput {
    pub account_id: i32,
    pub debit_amount: Option<Decimal>,
    pub credit_amount: Option<Decimal>,
    pub counterpart_id: Option<i32>,
    pub currency_code: Option<String>,
    pub currency_amount: Option<Decimal>,
    pub exchange_rate: Option<Decimal>,
    pub vat_amount: Option<Decimal>,
    pub vat_rate_id: Option<i32>,
    pub quantity: Option<Decimal>,
    pub unit_of_measure_code: Option<String>,
    pub description: Option<String>,
    pub line_order: Option<i32>,
}

#[derive(InputObject, Deserialize, Serialize)]
pub struct UpdateEntryLineInput {
    pub account_id: Option<i32>,
    pub debit_amount: Option<Decimal>,
    pub credit_amount: Option<Decimal>,
    pub counterpart_id: Option<i32>,
    pub currency_code: Option<String>,
    pub currency_amount: Option<Decimal>,
    pub exchange_rate: Option<Decimal>,
    pub vat_amount: Option<Decimal>,
    pub vat_rate_id: Option<i32>,
    pub quantity: Option<Decimal>,
    pub unit_of_measure_code: Option<String>,
    pub description: Option<String>,
    pub line_order: Option<i32>,
}

impl From<CreateEntryLineInput> for ActiveModel {
    fn from(input: CreateEntryLineInput) -> Self {
        ActiveModel {
            account_id: Set(input.account_id),
            debit_amount: Set(input.debit_amount.unwrap_or(Decimal::ZERO)),
            credit_amount: Set(input.credit_amount.unwrap_or(Decimal::ZERO)),
            counterpart_id: Set(input.counterpart_id),
            currency_code: Set(input.currency_code.or_else(|| Some("BGN".to_string()))),
            currency_amount: Set(input.currency_amount),
            exchange_rate: Set(input.exchange_rate.or_else(|| Some(Decimal::ONE))),
            vat_amount: Set(input.vat_amount.unwrap_or(Decimal::ZERO)),
            vat_rate_id: Set(input.vat_rate_id),
            quantity: Set(input.quantity),
            unit_of_measure_code: Set(input.unit_of_measure_code),
            description: Set(input.description),
            line_order: Set(input.line_order.unwrap_or(1)),
            ..Default::default()
        }
    }
}

// Helper struct for accounting reports
#[derive(SimpleObject, Serialize)]
pub struct EntryLineWithDetails {
    pub entry_line: Model,
    pub account: super::account::Model,
    pub counterpart: Option<super::counterpart::Model>,
    pub journal_entry: super::journal_entry::Model,
}

// Summary struct for account balances
#[derive(SimpleObject, Serialize)]
pub struct AccountBalance {
    pub account_id: i32,
    pub account_code: String,
    pub account_name: String,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub balance: Decimal,
    pub balance_type: String, // "DEBIT" or "CREDIT"
}
impl ActiveModelBehavior for ActiveModel {}
