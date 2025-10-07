use async_graphql::{InputObject, SimpleObject};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use sea_orm::{sea_query::StringLen, Set};
use serde::{Deserialize, Serialize};

// Use VatDirection from account module to avoid GraphQL name conflict
pub use super::account::VatDirection;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "vat_rates")]
#[graphql(concrete(name = "VatRate", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub code: String,
    pub name: String,
    pub rate: Decimal,
    pub vat_direction: VatDirection,
    pub is_active: bool,
    pub valid_from: Date,
    pub valid_to: Option<Date>,
    pub company_id: i32,
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
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

// Input types for GraphQL mutations
#[derive(InputObject, Deserialize, Serialize)]
pub struct CreateVatRateInput {
    pub code: String,
    pub name: String,
    pub rate: Decimal,
    pub vat_direction: VatDirection,
    pub valid_from: Date,
    pub valid_to: Option<Date>,
    pub company_id: i32,
}

#[derive(InputObject, Deserialize, Serialize)]
pub struct UpdateVatRateInput {
    pub code: Option<String>,
    pub name: Option<String>,
    pub rate: Option<Decimal>,
    pub vat_direction: Option<VatDirection>,
    pub valid_from: Option<Date>,
    pub valid_to: Option<Date>,
    pub is_active: Option<bool>,
}

impl From<CreateVatRateInput> for ActiveModel {
    fn from(input: CreateVatRateInput) -> Self {
        ActiveModel {
            code: Set(input.code),
            name: Set(input.name),
            rate: Set(input.rate),
            vat_direction: Set(input.vat_direction),
            valid_from: Set(input.valid_from),
            valid_to: Set(input.valid_to),
            company_id: Set(input.company_id),
            ..Default::default()
        }
    }
}

// Filter input for queries
#[derive(InputObject, Deserialize)]
pub struct VatRateFilter {
    pub company_id: Option<i32>,
    pub is_active: Option<bool>,
    pub vat_direction: Option<VatDirection>,
    pub valid_on_date: Option<Date>,
    pub code_contains: Option<String>,
}

// Bulgarian standard VAT rates
impl Model {
    /// Get Bulgarian standard VAT rates
    pub fn bulgarian_standard_rates() -> Vec<(String, String, Decimal, VatDirection)> {
        vec![
            (
                "VAT20".to_string(),
                "ДДС 20% (стандартна ставка)".to_string(),
                Decimal::from(20),
                VatDirection::Both,
            ),
            (
                "VAT9".to_string(),
                "ДДС 9% (намалена ставка)".to_string(),
                Decimal::from(9),
                VatDirection::Both,
            ),
            (
                "VAT0".to_string(),
                "ДДС 0% (освободени доставки)".to_string(),
                Decimal::ZERO,
                VatDirection::Output,
            ),
            (
                "NOVAT".to_string(),
                "Без ДДС (необлагаеми операции)".to_string(),
                Decimal::ZERO,
                VatDirection::None,
            ),
        ]
    }

    /// Check if VAT rate is valid for a given date
    pub fn is_valid_on_date(&self, date: Date) -> bool {
        if !self.is_active {
            return false;
        }

        if date < self.valid_from {
            return false;
        }

        if let Some(valid_to) = self.valid_to {
            if date > valid_to {
                return false;
            }
        }

        true
    }

    /// Calculate VAT amount from base amount
    pub fn calculate_vat_amount(&self, base_amount: Decimal) -> Decimal {
        base_amount * self.rate / Decimal::from(100)
    }

    /// Calculate base amount from total amount (including VAT)
    pub fn calculate_base_from_total(&self, total_amount: Decimal) -> Decimal {
        if self.rate == Decimal::ZERO {
            return total_amount;
        }

        let divisor = Decimal::from(100) + self.rate;
        total_amount * Decimal::from(100) / divisor
    }

    /// Get description for Bulgarian VAT reporting
    pub fn get_bulgarian_description(&self) -> String {
        match self.rate.to_string().as_str() {
            "20" => "Доставки с ДДС 20%".to_string(),
            "9" => "Доставки с ДДС 9%".to_string(),
            "0" => "Освободени доставки с право на приспадане".to_string(),
            _ => format!("Доставки с ДДС {}%", self.rate),
        }
    }
}

// Helper struct for VAT calculations
#[derive(SimpleObject, Serialize)]
pub struct VatCalculation {
    pub base_amount: Decimal,
    pub vat_rate: Decimal,
    pub vat_amount: Decimal,
    pub total_amount: Decimal,
    pub rate_code: String,
    pub rate_name: String,
}
impl ActiveModelBehavior for ActiveModel {}
