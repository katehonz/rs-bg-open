//! Depreciation Journal Entity
//!
//! Tracks monthly depreciation calculations for both accounting and tax purposes

use async_graphql::SimpleObject;
use chrono::Datelike;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "depreciation_journal")]
#[graphql(name = "DepreciationJournal")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// Foreign key to fixed_assets
    pub fixed_asset_id: i32,

    /// Depreciation period (YYYY-MM-01 format)
    pub period: Date,

    /// Foreign key to companies
    pub company_id: i32,

    // Accounting depreciation for this period
    /// Accounting depreciation amount for this month
    pub accounting_depreciation_amount: Decimal,

    /// Book value before accounting depreciation
    pub accounting_book_value_before: Decimal,

    /// Book value after accounting depreciation
    pub accounting_book_value_after: Decimal,

    // Tax depreciation for this period
    /// Tax depreciation amount for this month
    pub tax_depreciation_amount: Decimal,

    /// Book value before tax depreciation
    pub tax_book_value_before: Decimal,

    /// Book value after tax depreciation
    pub tax_book_value_after: Decimal,

    // Journal entry reference
    /// Reference to the journal entry that posts this depreciation (optional)
    pub journal_entry_id: Option<i32>,

    /// Whether this depreciation has been posted to the general ledger
    pub is_posted: bool,

    /// When this depreciation was posted
    pub posted_at: Option<DateTime>,

    /// User who posted this depreciation
    pub posted_by: Option<i32>,

    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::fixed_asset::Entity",
        from = "Column::FixedAssetId",
        to = "super::fixed_asset::Column::Id"
    )]
    FixedAsset,

    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::CompanyId",
        to = "super::company::Column::Id"
    )]
    Company,

    #[sea_orm(
        belongs_to = "super::journal_entry::Entity",
        from = "Column::JournalEntryId",
        to = "super::journal_entry::Column::Id"
    )]
    JournalEntry,

    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::PostedBy",
        to = "super::user::Column::Id"
    )]
    PostedByUser,
}

impl Related<super::fixed_asset::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FixedAsset.def()
    }
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl Related<super::journal_entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::JournalEntry.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PostedByUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Get period as year and month
    pub fn get_year_month(&self) -> (i32, u32) {
        (self.period.year(), self.period.month())
    }

    /// Get period display string (e.g., "Януари 2024")
    pub fn get_period_display(&self) -> String {
        let month_names = [
            "",
            "Януари",
            "Февруари",
            "Март",
            "Април",
            "Май",
            "Юни",
            "Юли",
            "Август",
            "Септември",
            "Октомври",
            "Ноември",
            "Декември",
        ];

        format!(
            "{} {}",
            month_names[self.period.month() as usize],
            self.period.year()
        )
    }

    /// Calculate the difference between accounting and tax depreciation for this period
    pub fn get_depreciation_difference(&self) -> Decimal {
        self.tax_depreciation_amount - self.accounting_depreciation_amount
    }

    /// Calculate the difference in book values after depreciation
    pub fn get_book_value_difference(&self) -> Decimal {
        self.tax_book_value_after - self.accounting_book_value_after
    }

    /// Check if there's a timing difference for deferred tax purposes
    pub fn has_timing_difference(&self) -> bool {
        self.get_depreciation_difference() != Decimal::from(0)
    }

    /// Check if this entry can be posted (not already posted)
    pub fn can_be_posted(&self) -> bool {
        !self.is_posted
    }

    /// Check if this entry can be unposted (already posted and no dependent entries)
    pub fn can_be_unposted(&self) -> bool {
        self.is_posted && self.journal_entry_id.is_some()
    }

    /// Get status display
    pub fn get_status_display(&self) -> &'static str {
        if self.is_posted {
            "Приключен"
        } else {
            "Неприключен"
        }
    }

    /// Create period date from year and month
    pub fn create_period_date(year: i32, month: u32) -> Result<Date, String> {
        Date::from_ymd_opt(year, month, 1)
            .ok_or_else(|| format!("Invalid year/month: {}/{}", year, month))
    }

    /// Get next period
    pub fn get_next_period(&self) -> Result<Date, String> {
        let mut year = self.period.year();
        let mut month = self.period.month() + 1;

        if month > 12 {
            month = 1;
            year += 1;
        }

        Self::create_period_date(year, month)
    }

    /// Get previous period  
    pub fn get_previous_period(&self) -> Result<Date, String> {
        let mut year = self.period.year();
        let mut month = self.period.month();

        if month == 1 {
            month = 12;
            year -= 1;
        } else {
            month -= 1;
        }

        Self::create_period_date(year, month)
    }
}
