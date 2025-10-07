//! Fixed Asset Category Entity
//!
//! Categories for Bulgarian ЗКПО (Corporate Income Tax Act) classification
//! Each category has specific depreciation limits and rules

use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "fixed_asset_categories")]
#[graphql(name = "FixedAssetCategory")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// Category code (e.g., "BUILDINGS", "MACHINERY")
    pub code: String,

    /// Category name in Bulgarian
    pub name: String,

    /// Detailed description
    pub description: Option<String>,

    /// ЗКПО tax category number (1-7)
    pub tax_category: i32,

    /// Maximum tax depreciation rate as percentage (e.g., 4.00 for 4%)
    pub max_tax_depreciation_rate: Decimal,

    /// Default accounting depreciation rate (can be different from tax)
    pub default_accounting_depreciation_rate: Option<Decimal>,

    /// Minimum useful life in months
    pub min_useful_life: Option<i32>,

    /// Maximum useful life in months
    pub max_useful_life: Option<i32>,

    /// Default asset account code (e.g., "201", "204")
    pub asset_account_code: String,

    /// Default accumulated depreciation account code (e.g., "241")
    pub depreciation_account_code: String,

    /// Default depreciation expense account code (e.g., "603")
    pub expense_account_code: String,

    /// Whether this category is active
    pub is_active: bool,

    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::fixed_asset::Entity")]
    FixedAssets,
}

impl Related<super::fixed_asset::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FixedAssets.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Check if the given depreciation rate is within limits for this category
    pub fn is_rate_within_limits(&self, rate: Decimal) -> bool {
        rate <= self.max_tax_depreciation_rate
    }

    /// Get the display name for tax category
    pub fn get_tax_category_display(&self) -> &'static str {
        match self.tax_category {
            1 => "Категория I - Сгради и съоръжения",
            2 => "Категория II - Машини и оборудване",
            3 => "Категория III - Транспортни средства",
            4 => "Категория IV - Компютри и софтуер",
            5 => "Категория V - Автомобили",
            7 => "Категория VII - Други ДМА",
            _ => "Неизвестна категория",
        }
    }

    /// Check if this category allows accelerated depreciation for new investments
    pub fn allows_accelerated_depreciation(&self) -> bool {
        // Category II (Machinery) allows 50% instead of 30% for new first-time investments
        self.tax_category == 2
    }

    /// Get accelerated rate for new investments (only for Category II)
    pub fn get_accelerated_rate(&self) -> Option<Decimal> {
        if self.allows_accelerated_depreciation() {
            Some(Decimal::from(50)) // 50% for new machinery investments
        } else {
            None
        }
    }
}
