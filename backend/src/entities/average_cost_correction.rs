//! Average Cost Correction Entity
//!
//! Tracks corrections when entries are added out of chronological order

use async_graphql::SimpleObject;
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "average_cost_corrections")]
#[graphql(name = "AverageCostCorrection")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// Company ID
    pub company_id: i32,

    /// Material account ID
    pub account_id: i32,

    /// The movement that was added and triggered recalculation
    pub triggering_movement_id: i32,

    /// The later movement that needs correction
    pub affected_movement_id: i32,

    /// The correction journal entry created (if applied)
    pub correction_journal_entry_id: Option<i32>,

    /// Old average cost before correction
    pub old_average_cost: Decimal,

    /// New average cost after correction
    pub new_average_cost: Decimal,

    /// Difference to be corrected (positive or negative)
    pub correction_amount: Decimal,

    /// Whether the correction has been applied
    pub is_applied: bool,

    /// When the correction was applied
    pub applied_at: Option<DateTimeUtc>,

    pub created_at: DateTimeUtc,
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
        belongs_to = "super::inventory_movement::Entity",
        from = "Column::TriggeringMovementId",
        to = "super::inventory_movement::Column::Id"
    )]
    TriggeringMovement,

    #[sea_orm(
        belongs_to = "super::inventory_movement::Entity",
        from = "Column::AffectedMovementId",
        to = "super::inventory_movement::Column::Id"
    )]
    AffectedMovement,

    #[sea_orm(
        belongs_to = "super::journal_entry::Entity",
        from = "Column::CorrectionJournalEntryId",
        to = "super::journal_entry::Column::Id"
    )]
    CorrectionJournalEntry,
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

impl Related<super::journal_entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CorrectionJournalEntry.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Check if correction is pending
    pub fn is_pending(&self) -> bool {
        !self.is_applied
    }

    /// Get the absolute value of correction
    pub fn correction_abs(&self) -> Decimal {
        self.correction_amount.abs()
    }

    /// Check if correction increases the cost
    pub fn is_increase(&self) -> bool {
        self.correction_amount > Decimal::ZERO
    }

    /// Check if correction decreases the cost
    pub fn is_decrease(&self) -> bool {
        self.correction_amount < Decimal::ZERO
    }

    /// Get status display text
    pub fn status_text(&self) -> &'static str {
        if self.is_applied {
            "Приложена"
        } else {
            "Изчакваща"
        }
    }
}
