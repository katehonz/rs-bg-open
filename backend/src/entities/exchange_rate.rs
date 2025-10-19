use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{Datelike, NaiveDate, Weekday};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use sea_orm::{sea_query::StringLen, Set};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Enum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(10))")]
pub enum RateSource {
    #[sea_orm(string_value = "BNB")]
    Bnb,
    #[sea_orm(string_value = "MANUAL")]
    Manual,
    #[sea_orm(string_value = "ECB")]
    Ecb,
    #[sea_orm(string_value = "API")]
    Api,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "exchange_rates")]
#[graphql(concrete(name = "ExchangeRate", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub from_currency_id: i32,
    pub to_currency_id: i32,
    pub rate: Decimal,
    pub reverse_rate: Decimal,
    pub valid_date: Date,
    pub rate_source: RateSource,
    pub bnb_rate_id: Option<String>,
    pub is_active: bool,
    pub notes: Option<String>,
    pub created_by: Option<i32>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::currency::Entity",
        from = "Column::FromCurrencyId",
        to = "super::currency::Column::Id"
    )]
    FromCurrency,
    #[sea_orm(
        belongs_to = "super::currency::Entity",
        from = "Column::ToCurrencyId",
        to = "super::currency::Column::Id"
    )]
    ToCurrency,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::CreatedBy",
        to = "super::user::Column::Id"
    )]
    CreatedByUser,
}

impl Related<super::currency::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FromCurrency.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatedByUser.def()
    }
}

// Input types for GraphQL mutations
#[derive(InputObject, Deserialize, Serialize)]
pub struct CreateExchangeRateInput {
    pub from_currency_id: i32,
    pub to_currency_id: i32,
    pub rate: Decimal,
    pub valid_date: Date,
    pub rate_source: Option<RateSource>,
    pub bnb_rate_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(InputObject, Deserialize, Serialize)]
pub struct UpdateExchangeRateInput {
    pub rate: Option<Decimal>,
    pub valid_date: Option<Date>,
    pub rate_source: Option<RateSource>,
    pub is_active: Option<bool>,
    pub notes: Option<String>,
}

impl From<CreateExchangeRateInput> for ActiveModel {
    fn from(input: CreateExchangeRateInput) -> Self {
        ActiveModel {
            from_currency_id: Set(input.from_currency_id),
            to_currency_id: Set(input.to_currency_id),
            rate: Set(input.rate),
            valid_date: Set(input.valid_date),
            rate_source: Set(input.rate_source.unwrap_or(RateSource::Manual)),
            bnb_rate_id: Set(input.bnb_rate_id),
            notes: Set(input.notes),
            ..Default::default()
        }
    }
}

// Filter input for queries
#[derive(InputObject, Deserialize)]
pub struct ExchangeRateFilter {
    pub from_currency_id: Option<i32>,
    pub to_currency_id: Option<i32>,
    pub valid_date: Option<Date>,
    pub date_from: Option<Date>,
    pub date_to: Option<Date>,
    pub rate_source: Option<RateSource>,
    pub is_active: Option<bool>,
}

impl Model {
    /// Convert amount using this exchange rate
    pub fn convert_amount(&self, amount: Decimal) -> Decimal {
        amount * self.rate
    }

    /// Convert amount in reverse direction
    pub fn convert_amount_reverse(&self, amount: Decimal) -> Decimal {
        amount * self.reverse_rate
    }

    /// Get rate description in Bulgarian
    pub fn get_rate_description(&self) -> String {
        match self.rate_source {
            RateSource::Bnb => "БНБ".to_string(),
            RateSource::Manual => "Ръчно въведен".to_string(),
            RateSource::Ecb => "ЕЦБ".to_string(),
            RateSource::Api => "API".to_string(),
        }
    }

    /// Check if rate is up to date (within last business day)
    pub fn is_up_to_date(&self) -> bool {
        let today = chrono::Utc::now().date_naive();
        let days_diff = (today - self.valid_date).num_days();

        // Consider weekends - if today is Monday, rate from Friday is OK
        match today.weekday() {
            chrono::Weekday::Mon => days_diff <= 3, // Friday to Monday
            chrono::Weekday::Tue => days_diff <= 1, // Monday to Tuesday
            _ => days_diff <= 1,                    // Any other day
        }
    }

    /// Get age description in Bulgarian
    pub fn get_age_description(&self) -> String {
        let today = chrono::Utc::now().date_naive();
        let days = (today - self.valid_date).num_days();

        match days {
            0 => "Днес".to_string(),
            1 => "Вчера".to_string(),
            2..=7 => format!("Преди {} дни", days),
            8..=30 => format!("Преди {} дни", days),
            31..=365 => format!("Преди {} месеца", days / 30),
            _ => format!("Преди {} години", days / 365),
        }
    }
}

// Helper structs for BNB integration
#[derive(SimpleObject, Serialize)]
pub struct ExchangeRateWithCurrencies {
    pub exchange_rate: Model,
    pub from_currency: super::currency::Model,
    pub to_currency: super::currency::Model,
    pub is_up_to_date: bool,
    pub age_description: String,
}

#[derive(SimpleObject, Serialize)]
pub struct CurrencyConversion {
    pub from_currency: String,
    pub to_currency: String,
    pub from_amount: Decimal,
    pub to_amount: Decimal,
    pub exchange_rate: Decimal,
    pub conversion_date: Date,
    pub rate_source: String,
}

// BNB API response structure
#[derive(Debug, Deserialize)]
pub struct BnbRate {
    pub code: String,
    pub name: String,
    pub rate: String,
    pub date: String,
    pub extrainfo: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BnbResponse {
    pub rates: Vec<BnbRate>,
}
impl ActiveModelBehavior for ActiveModel {}
