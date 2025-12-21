//! Asset Value Adjustment Entity
//!
//! Tracks all changes to fixed asset values for SAF-T reporting and accounting compliance.
//! According to Bulgarian SAF-T requirements, this tracks transaction types such as:
//! - Acquisition (придобиване)
//! - Improvement (увеличение/подобрение)
//! - Impairment (намаление/обезценка)
//! - Revaluation (преоценка)
//! - Disposal (отписване/продажба)
//! - InternalTransfer (вътрешен трансфер)
//! - Scrap (брак)

use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "asset_value_adjustments")]
#[graphql(name = "AssetValueAdjustment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// Foreign key to fixed_assets
    pub fixed_asset_id: i32,

    /// Foreign key to companies
    pub company_id: i32,

    /// Date when the adjustment occurred
    pub adjustment_date: Date,

    /// Type of adjustment: acquisition, improvement, impairment, revaluation,
    /// disposal, internal_transfer, scrap
    pub adjustment_type: String,

    /// Amount of the adjustment (positive or negative)
    pub adjustment_amount: Decimal,

    // Book value changes
    /// Accounting book value before adjustment
    pub accounting_value_before: Decimal,

    /// Accounting book value after adjustment
    pub accounting_value_after: Decimal,

    /// Tax book value before adjustment
    pub tax_value_before: Decimal,

    /// Tax book value after adjustment
    pub tax_value_after: Decimal,

    // New depreciation rates/lives after adjustment (if changed)
    /// New accounting depreciation rate (if changed)
    pub new_accounting_depreciation_rate: Option<Decimal>,

    /// New tax depreciation rate (if changed)
    pub new_tax_depreciation_rate: Option<Decimal>,

    /// New accounting useful life in months (if changed)
    pub new_accounting_useful_life: Option<i32>,

    /// New tax useful life in months (if changed)
    pub new_tax_useful_life: Option<i32>,

    // Description and documentation
    /// Reason for the adjustment
    pub reason: String,

    /// Document number (e.g., invoice number, protocol number)
    pub document_number: Option<String>,

    /// Additional notes
    pub notes: Option<String>,

    // Journal entry reference
    /// Reference to the journal entry that posts this adjustment
    pub journal_entry_id: Option<i32>,

    /// Whether this adjustment has been posted to the general ledger
    pub is_posted: bool,

    /// When this adjustment was posted
    pub posted_at: Option<DateTime>,

    /// User who posted this adjustment
    pub posted_by: Option<i32>,

    // Audit fields
    /// User who created this adjustment
    pub created_by: i32,

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

    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::CreatedBy",
        to = "super::user::Column::Id"
    )]
    CreatedByUser,
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

impl ActiveModelBehavior for ActiveModel {}

/// Asset adjustment types according to SAF-T standard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetAdjustmentType {
    /// Initial acquisition of the asset
    Acquisition,
    /// Value increase due to improvements or additions
    Improvement,
    /// Value decrease due to impairment or damage
    Impairment,
    /// Revaluation (fair value adjustment)
    Revaluation,
    /// Disposal or sale of the asset
    Disposal,
    /// Transfer between locations or departments
    InternalTransfer,
    /// Asset scrapped or written off
    Scrap,
}

impl AssetAdjustmentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Acquisition => "acquisition",
            Self::Improvement => "improvement",
            Self::Impairment => "impairment",
            Self::Revaluation => "revaluation",
            Self::Disposal => "disposal",
            Self::InternalTransfer => "internal_transfer",
            Self::Scrap => "scrap",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "acquisition" => Some(Self::Acquisition),
            "improvement" => Some(Self::Improvement),
            "impairment" => Some(Self::Impairment),
            "revaluation" => Some(Self::Revaluation),
            "disposal" => Some(Self::Disposal),
            "internal_transfer" => Some(Self::InternalTransfer),
            "scrap" => Some(Self::Scrap),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Acquisition => "Придобиване",
            Self::Improvement => "Увеличение",
            Self::Impairment => "Намаление",
            Self::Revaluation => "Преоценка",
            Self::Disposal => "Отписване",
            Self::InternalTransfer => "Вътрешен трансфер",
            Self::Scrap => "Брак",
        }
    }

    /// Returns true if this adjustment type increases asset value
    pub fn increases_value(&self) -> bool {
        matches!(self, Self::Acquisition | Self::Improvement | Self::Revaluation)
    }

    /// Returns true if this adjustment type decreases asset value
    pub fn decreases_value(&self) -> bool {
        matches!(self, Self::Impairment | Self::Disposal | Self::Scrap)
    }

    /// Returns true if this adjustment type is a SAF-T reportable transaction
    pub fn is_saft_reportable(&self) -> bool {
        // All adjustment types are reportable in SAF-T
        true
    }
}

impl Model {
    /// Get the adjustment type as enum
    pub fn get_adjustment_type(&self) -> Option<AssetAdjustmentType> {
        AssetAdjustmentType::from_str(&self.adjustment_type)
    }

    /// Get the change in accounting value
    pub fn get_accounting_value_change(&self) -> Decimal {
        self.accounting_value_after - self.accounting_value_before
    }

    /// Get the change in tax value
    pub fn get_tax_value_change(&self) -> Decimal {
        self.tax_value_after - self.tax_value_before
    }

    /// Check if this adjustment can be posted
    pub fn can_be_posted(&self) -> bool {
        !self.is_posted
    }

    /// Check if this adjustment can be unposted
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

    /// Get adjustment type display name
    pub fn get_type_display(&self) -> String {
        self.get_adjustment_type()
            .map(|t| t.display_name().to_string())
            .unwrap_or_else(|| self.adjustment_type.clone())
    }

    /// Validate adjustment before saving
    pub fn validate(&self) -> Result<(), String> {
        // Validate adjustment type
        if AssetAdjustmentType::from_str(&self.adjustment_type).is_none() {
            return Err(format!("Invalid adjustment type: {}", self.adjustment_type));
        }

        // Validate amounts
        if self.adjustment_amount == Decimal::from(0) {
            return Err("Adjustment amount cannot be zero".to_string());
        }

        // Validate book value changes
        let accounting_change = self.get_accounting_value_change();
        let tax_change = self.get_tax_value_change();

        if accounting_change == Decimal::from(0) && tax_change == Decimal::from(0) {
            return Err("At least one book value must change".to_string());
        }

        // Validate that values cannot be negative after adjustment
        if self.accounting_value_after < Decimal::from(0) {
            return Err("Accounting value cannot be negative after adjustment".to_string());
        }

        if self.tax_value_after < Decimal::from(0) {
            return Err("Tax value cannot be negative after adjustment".to_string());
        }

        Ok(())
    }

    /// Check if depreciation rates or useful life changed
    pub fn has_depreciation_changes(&self) -> bool {
        self.new_accounting_depreciation_rate.is_some()
            || self.new_tax_depreciation_rate.is_some()
            || self.new_accounting_useful_life.is_some()
            || self.new_tax_useful_life.is_some()
    }
}
