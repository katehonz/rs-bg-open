//! Fixed Asset Entity
//!
//! Represents individual fixed assets with separate accounting and tax depreciation

use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "fixed_assets")]
#[graphql(name = "FixedAsset")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// Unique inventory number
    pub inventory_number: String,

    /// Asset name
    pub name: String,

    /// Detailed description
    pub description: Option<String>,

    /// Foreign key to fixed_asset_categories
    pub category_id: i32,

    /// Foreign key to companies
    pub company_id: i32,

    // Financial data
    /// Initial acquisition cost
    pub acquisition_cost: Decimal,

    /// Date when asset was acquired
    pub acquisition_date: Date,

    /// Date when asset was put into service (can be different from acquisition)
    pub put_into_service_date: Option<Date>,

    // Accounting depreciation (book depreciation)
    /// Useful life for accounting purposes in months
    pub accounting_useful_life: i32,

    /// Annual accounting depreciation rate as percentage
    pub accounting_depreciation_rate: Decimal,

    /// Depreciation method (straight_line, declining_balance, etc.)
    pub accounting_depreciation_method: String,

    /// Salvage value for accounting depreciation
    pub accounting_salvage_value: Decimal,

    /// Accumulated accounting depreciation to date
    pub accounting_accumulated_depreciation: Decimal,

    // Tax depreciation (ЗКПО)
    /// Useful life for tax purposes in months (can differ from accounting)
    pub tax_useful_life: Option<i32>,

    /// Annual tax depreciation rate as percentage (limited by ЗКПО)
    pub tax_depreciation_rate: Decimal,

    /// Accumulated tax depreciation to date
    pub tax_accumulated_depreciation: Decimal,

    /// Whether this is a new first-time investment (allows higher rates)
    pub is_new_first_time_investment: bool,

    // Book values
    /// Current accounting book value (cost - accumulated accounting depreciation)
    pub accounting_book_value: Decimal,

    /// Current tax book value (cost - accumulated tax depreciation)
    pub tax_book_value: Decimal,

    // Status
    /// Asset status: active, disposed, sold
    pub status: String,

    /// Date when asset was disposed (if applicable)
    pub disposal_date: Option<Date>,

    /// Disposal amount (if disposed or sold)
    pub disposal_amount: Option<Decimal>,

    // Metadata
    /// Physical location of the asset
    pub location: Option<String>,

    /// Person responsible for the asset
    pub responsible_person: Option<String>,

    /// Serial number
    pub serial_number: Option<String>,

    /// Manufacturer
    pub manufacturer: Option<String>,

    /// Model
    pub model: Option<String>,

    /// Additional notes
    pub notes: Option<String>,

    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::fixed_asset_category::Entity",
        from = "Column::CategoryId",
        to = "super::fixed_asset_category::Column::Id"
    )]
    Category,

    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::CompanyId",
        to = "super::company::Column::Id"
    )]
    Company,

    #[sea_orm(has_many = "super::depreciation_journal::Entity")]
    DepreciationJournal,
}

impl Related<super::fixed_asset_category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl Related<super::depreciation_journal::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DepreciationJournal.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Calculate monthly accounting depreciation amount
    pub fn calculate_monthly_accounting_depreciation(&self) -> Decimal {
        let depreciable_amount = self.acquisition_cost - self.accounting_salvage_value;
        let annual_amount =
            depreciable_amount * (self.accounting_depreciation_rate / Decimal::from(100));
        annual_amount / Decimal::from(12)
    }

    /// Calculate monthly tax depreciation amount
    pub fn calculate_monthly_tax_depreciation(&self) -> Decimal {
        let annual_amount =
            self.acquisition_cost * (self.tax_depreciation_rate / Decimal::from(100));
        annual_amount / Decimal::from(12)
    }

    /// Check if asset is fully depreciated for accounting purposes
    pub fn is_fully_depreciated_accounting(&self) -> bool {
        self.accounting_book_value <= self.accounting_salvage_value
    }

    /// Check if asset is fully depreciated for tax purposes
    pub fn is_fully_depreciated_tax(&self) -> bool {
        self.tax_book_value <= Decimal::from(0)
    }

    /// Calculate remaining accounting life in months
    pub fn remaining_accounting_life_months(&self) -> i32 {
        if self.is_fully_depreciated_accounting() {
            return 0;
        }

        let monthly_depreciation = self.calculate_monthly_accounting_depreciation();
        if monthly_depreciation == Decimal::from(0) {
            return i32::MAX;
        }

        let remaining_value = self.accounting_book_value - self.accounting_salvage_value;
        (remaining_value / monthly_depreciation)
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0)
            .ceil() as i32
    }

    /// Calculate remaining tax life in months  
    pub fn remaining_tax_life_months(&self) -> i32 {
        if self.is_fully_depreciated_tax() {
            return 0;
        }

        let monthly_depreciation = self.calculate_monthly_tax_depreciation();
        if monthly_depreciation == Decimal::from(0) {
            return i32::MAX;
        }

        (self.tax_book_value / monthly_depreciation)
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0)
            .ceil() as i32
    }

    /// Get the difference between accounting and tax book values (for deferred taxes)
    pub fn get_tax_accounting_difference(&self) -> Decimal {
        self.tax_book_value - self.accounting_book_value
    }

    /// Check if asset is active
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }

    /// Get status display name
    pub fn get_status_display(&self) -> &'static str {
        match self.status.as_str() {
            "active" => "Активен",
            "disposed" => "Ликвидиран",
            "sold" => "Продаден",
            _ => "Неизвестен статус",
        }
    }
}
