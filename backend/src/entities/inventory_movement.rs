//! Inventory Movement Entity
//!
//! Tracks all inventory movements (receipts and issues) for material accounts

use async_graphql::SimpleObject;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "inventory_movements")]
#[graphql(name = "InventoryMovement")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// Company ID
    pub company_id: i32,

    /// Material account ID (class 3)
    pub account_id: i32,

    /// Reference to entry line that created this movement
    pub entry_line_id: i32,

    /// Reference to journal entry
    pub journal_entry_id: i32,

    /// Date of the movement
    pub movement_date: NaiveDate,

    /// Type of movement: "DEBIT" (receipt) or "CREDIT" (issue)
    pub movement_type: String,

    /// Quantity moved
    pub quantity: Decimal,

    /// Unit price at the time of movement
    pub unit_price: Decimal,

    /// Total amount (quantity × unit_price)
    pub total_amount: Decimal,

    /// Unit of measure
    pub unit_of_measure: Option<String>,

    /// Description
    pub description: Option<String>,

    /// Balance quantity after this movement
    pub balance_after_quantity: Decimal,

    /// Balance amount after this movement
    pub balance_after_amount: Decimal,

    /// Average cost at the time of this movement
    pub average_cost_at_time: Decimal,

    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
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
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id"
    )]
    Account,

    #[sea_orm(
        belongs_to = "super::entry_line::Entity",
        from = "Column::EntryLineId",
        to = "super::entry_line::Column::Id"
    )]
    EntryLine,

    #[sea_orm(
        belongs_to = "super::journal_entry::Entity",
        from = "Column::JournalEntryId",
        to = "super::journal_entry::Column::Id"
    )]
    JournalEntry,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::entry_line::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EntryLine.def()
    }
}

impl Related<super::journal_entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::JournalEntry.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Check if this is a receipt (debit) movement
    pub fn is_receipt(&self) -> bool {
        self.movement_type == "DEBIT"
    }

    /// Check if this is an issue (credit) movement
    pub fn is_issue(&self) -> bool {
        self.movement_type == "CREDIT"
    }

    /// Get the signed quantity (positive for receipts, negative for issues)
    pub fn signed_quantity(&self) -> Decimal {
        if self.is_receipt() {
            self.quantity
        } else {
            -self.quantity
        }
    }

    /// Get the signed amount (positive for receipts, negative for issues)
    pub fn signed_amount(&self) -> Decimal {
        if self.is_receipt() {
            self.total_amount
        } else {
            -self.total_amount
        }
    }
}
