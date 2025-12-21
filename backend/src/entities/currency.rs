use async_graphql::{InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "currencies")]
#[graphql(concrete(name = "Currency", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub code: String,
    pub name: String,
    pub name_bg: String,
    pub symbol: Option<String>,
    pub decimal_places: i32,
    pub is_active: bool,
    pub is_base_currency: bool,
    pub bnb_code: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::exchange_rate::Entity")]
    FromExchangeRates,
    #[sea_orm(has_many = "super::exchange_rate::Entity")]
    ToExchangeRates,
}

// Input types for GraphQL mutations
#[derive(InputObject, Deserialize, Serialize)]
pub struct CreateCurrencyInput {
    pub code: String,
    pub name: String,
    pub name_bg: String,
    pub symbol: Option<String>,
    pub decimal_places: Option<i32>,
    pub bnb_code: Option<String>,
}

#[derive(InputObject, Deserialize, Serialize)]
pub struct UpdateCurrencyInput {
    pub name: Option<String>,
    pub name_bg: Option<String>,
    pub symbol: Option<String>,
    pub decimal_places: Option<i32>,
    pub is_active: Option<bool>,
    pub bnb_code: Option<String>,
}

impl From<CreateCurrencyInput> for ActiveModel {
    fn from(input: CreateCurrencyInput) -> Self {
        ActiveModel {
            code: Set(input.code.to_uppercase()),
            name: Set(input.name),
            name_bg: Set(input.name_bg),
            symbol: Set(input.symbol),
            decimal_places: Set(input.decimal_places.unwrap_or(2)),
            bnb_code: Set(input.bnb_code.map(|c| c.to_uppercase())),
            ..Default::default()
        }
    }
}

impl Model {
    /// Format amount according to currency's decimal places
    /// Always uses ISO code instead of symbols (€, $, лв., etc.)
    pub fn format_amount(&self, amount: rust_decimal::Decimal) -> String {
        // Always use ISO code, never symbols
        let code = &self.code;
        match self.decimal_places {
            0 => format!("{} {}", amount.round(), code),
            1 => format!("{:.1} {}", amount, code),
            2 => format!("{:.2} {}", amount, code),
            3 => format!("{:.3} {}", amount, code),
            4 => format!("{:.4} {}", amount, code),
            _ => format!("{:.6} {}", amount, code),
        }
    }

    /// Get display name (Bulgarian name if available)
    pub fn get_display_name(&self) -> &str {
        if self.name_bg.is_empty() {
            &self.name
        } else {
            &self.name_bg
        }
    }

    /// Check if currency supports BNB automatic updates
    pub fn supports_bnb_updates(&self) -> bool {
        self.bnb_code.is_some() && !self.is_base_currency
    }

    /// Get common currency pairs for Bulgaria (ISO codes only, no symbols)
    pub fn bulgarian_common_currencies() -> Vec<(&'static str, &'static str)> {
        vec![
            ("BGN", "Български лев"),
            ("EUR", "Евро"),
            ("USD", "Американски долар"),
            ("GBP", "Британска лира"),
            ("CHF", "Швейцарски франк"),
            ("JPY", "Японска йена"),
            ("RUB", "Руска рубла"),
            ("TRY", "Турска лира"),
        ]
    }
}

// Filter input for queries
#[derive(InputObject, Deserialize)]
pub struct CurrencyFilter {
    pub is_active: Option<bool>,
    pub is_base_currency: Option<bool>,
    pub supports_bnb: Option<bool>,
    pub code_contains: Option<String>,
}

// Helper struct for currency with latest rate
#[derive(SimpleObject, Serialize)]
pub struct CurrencyWithRate {
    pub currency: Model,
    pub latest_rate: Option<rust_decimal::Decimal>,
    pub rate_date: Option<chrono::NaiveDate>,
    pub rate_source: Option<String>,
}
impl ActiveModelBehavior for ActiveModel {}
