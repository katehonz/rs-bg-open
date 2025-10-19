use async_graphql::{Context, FieldResult, Object};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    Set,
};
use std::collections::HashMap;
use std::sync::Arc;

use crate::entities::currency::{
    CreateCurrencyInput, CurrencyFilter, CurrencyWithRate, UpdateCurrencyInput,
};
use crate::entities::exchange_rate::{
    CreateExchangeRateInput, CurrencyConversion, ExchangeRateFilter, ExchangeRateWithCurrencies,
    RateSource, UpdateExchangeRateInput,
};
use crate::entities::{currency, exchange_rate};
use crate::services::bnb_service::BnbService;
use crate::services::ecb_service::EcbService;

#[derive(Default)]
pub struct CurrencyQuery;

#[Object]
impl CurrencyQuery {
    /// Get all currencies with optional filtering
    async fn currencies(
        &self,
        ctx: &Context<'_>,
        filter: Option<CurrencyFilter>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> FieldResult<Vec<currency::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut query = currency::Entity::find();

        if let Some(f) = filter {
            let mut condition = Condition::all();

            if let Some(is_active) = f.is_active {
                condition = condition.add(currency::Column::IsActive.eq(is_active));
            }

            if let Some(is_base) = f.is_base_currency {
                condition = condition.add(currency::Column::IsBaseCurrency.eq(is_base));
            }

            if let Some(supports_bnb) = f.supports_bnb {
                if supports_bnb {
                    condition = condition
                        .add(currency::Column::BnbCode.is_not_null())
                        .add(currency::Column::IsBaseCurrency.eq(false));
                } else {
                    condition = condition.add(currency::Column::BnbCode.is_null());
                }
            }

            if let Some(code_search) = f.code_contains {
                condition = condition.add(currency::Column::Code.contains(&code_search));
            }

            query = query.filter(condition);
        }

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let currencies = query.order_by_asc(currency::Column::Code).all(db).await?;

        Ok(currencies)
    }

    /// Get currency by ID
    async fn currency(&self, ctx: &Context<'_>, id: i32) -> FieldResult<Option<currency::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let currency = currency::Entity::find_by_id(id).one(db).await?;
        Ok(currency)
    }

    /// Get currency by code
    async fn currency_by_code(
        &self,
        ctx: &Context<'_>,
        code: String,
    ) -> FieldResult<Option<currency::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let currency = currency::Entity::find()
            .filter(currency::Column::Code.eq(code.to_uppercase()))
            .one(db)
            .await?;
        Ok(currency)
    }

    /// Get currencies with their latest exchange rates
    async fn currencies_with_rates(&self, ctx: &Context<'_>) -> FieldResult<Vec<CurrencyWithRate>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let bnb_service = BnbService::new();

        let currencies = currency::Entity::find()
            .filter(currency::Column::IsActive.eq(true))
            .order_by_asc(currency::Column::Code)
            .all(db)
            .await?;

        let bgn_currency = currency::Entity::find()
            .filter(currency::Column::Code.eq("BGN"))
            .one(db)
            .await?
            .ok_or("BGN currency not found")?;

        let mut results = Vec::new();

        for curr in currencies {
            if curr.is_base_currency {
                results.push(CurrencyWithRate {
                    currency: curr,
                    latest_rate: Some(Decimal::ONE),
                    rate_date: Some(chrono::Utc::now().date_naive()),
                    rate_source: Some("BASE".to_string()),
                });
            } else {
                if let Ok(Some(rate)) = bnb_service
                    .get_latest_rate(db, curr.id, bgn_currency.id)
                    .await
                {
                    results.push(CurrencyWithRate {
                        currency: curr,
                        latest_rate: Some(rate.rate),
                        rate_date: Some(rate.valid_date),
                        rate_source: Some(rate.get_rate_description()),
                    });
                } else {
                    results.push(CurrencyWithRate {
                        currency: curr,
                        latest_rate: None,
                        rate_date: None,
                        rate_source: None,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Get all exchange rates with optional filtering
    async fn exchange_rates(
        &self,
        ctx: &Context<'_>,
        filter: Option<ExchangeRateFilter>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> FieldResult<Vec<exchange_rate::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut query = exchange_rate::Entity::find();

        if let Some(f) = filter {
            let mut condition = Condition::all();

            if let Some(from_id) = f.from_currency_id {
                condition = condition.add(exchange_rate::Column::FromCurrencyId.eq(from_id));
            }

            if let Some(to_id) = f.to_currency_id {
                condition = condition.add(exchange_rate::Column::ToCurrencyId.eq(to_id));
            }

            if let Some(valid_date) = f.valid_date {
                condition = condition.add(exchange_rate::Column::ValidDate.eq(valid_date));
            }

            if let Some(date_from) = f.date_from {
                condition = condition.add(exchange_rate::Column::ValidDate.gte(date_from));
            }

            if let Some(date_to) = f.date_to {
                condition = condition.add(exchange_rate::Column::ValidDate.lte(date_to));
            }

            if let Some(source) = f.rate_source {
                condition = condition.add(exchange_rate::Column::RateSource.eq(source));
            }

            if let Some(is_active) = f.is_active {
                condition = condition.add(exchange_rate::Column::IsActive.eq(is_active));
            }

            query = query.filter(condition);
        }

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let rates = query
            .order_by_desc(exchange_rate::Column::ValidDate)
            .order_by_asc(exchange_rate::Column::FromCurrencyId)
            .all(db)
            .await?;

        Ok(rates)
    }

    /// Get exchange rate for specific currency pair and date
    async fn exchange_rate_for_date(
        &self,
        ctx: &Context<'_>,
        from_currency_id: i32,
        to_currency_id: i32,
        date: NaiveDate,
    ) -> FieldResult<Option<exchange_rate::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let bnb_service = BnbService::new();

        let rate = bnb_service
            .get_rate_for_date(db, from_currency_id, to_currency_id, date)
            .await?;
        Ok(rate)
    }

    /// Convert amount between currencies
    async fn convert_currency(
        &self,
        ctx: &Context<'_>,
        from_currency_id: i32,
        to_currency_id: i32,
        amount: Decimal,
        date: Option<NaiveDate>,
    ) -> FieldResult<CurrencyConversion> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let bnb_service = BnbService::new();

        let conversion_date = date.unwrap_or_else(|| chrono::Utc::now().date_naive());

        // Get currencies
        let from_currency = currency::Entity::find_by_id(from_currency_id)
            .one(db)
            .await?
            .ok_or("From currency not found")?;
        let to_currency = currency::Entity::find_by_id(to_currency_id)
            .one(db)
            .await?
            .ok_or("To currency not found")?;

        // Handle same currency
        if from_currency_id == to_currency_id {
            return Ok(CurrencyConversion {
                from_currency: from_currency.code,
                to_currency: to_currency.code,
                from_amount: amount,
                to_amount: amount,
                exchange_rate: Decimal::ONE,
                conversion_date,
                rate_source: "SAME".to_string(),
            });
        }

        // Get exchange rate
        let rate = bnb_service
            .get_rate_for_date(db, from_currency_id, to_currency_id, conversion_date)
            .await?
            .ok_or("Exchange rate not found")?;

        let converted_amount = rate.convert_amount(amount);

        Ok(CurrencyConversion {
            from_currency: from_currency.code,
            to_currency: to_currency.code,
            from_amount: amount,
            to_amount: converted_amount,
            exchange_rate: rate.rate,
            conversion_date,
            rate_source: rate.get_rate_description(),
        })
    }

    /// Get exchange rates with currency details
    async fn exchange_rates_with_currencies(
        &self,
        ctx: &Context<'_>,
        date: Option<NaiveDate>,
    ) -> FieldResult<Vec<ExchangeRateWithCurrencies>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let target_date = date.unwrap_or_else(|| chrono::Utc::now().date_naive());

        let rates = exchange_rate::Entity::find()
            .filter(exchange_rate::Column::ValidDate.eq(target_date))
            .filter(exchange_rate::Column::IsActive.eq(true))
            .all(db)
            .await?;

        let mut results = Vec::new();

        for rate in rates {
            let from_currency = currency::Entity::find_by_id(rate.from_currency_id)
                .one(db)
                .await?
                .ok_or("From currency not found")?;
            let to_currency = currency::Entity::find_by_id(rate.to_currency_id)
                .one(db)
                .await?
                .ok_or("To currency not found")?;

            results.push(ExchangeRateWithCurrencies {
                is_up_to_date: rate.is_up_to_date(),
                age_description: rate.get_age_description(),
                exchange_rate: rate,
                from_currency,
                to_currency,
            });
        }

        Ok(results)
    }
}

#[derive(Default)]
pub struct CurrencyMutation;

#[Object]
impl CurrencyMutation {
    /// Create a new currency
    async fn create_currency(
        &self,
        ctx: &Context<'_>,
        input: CreateCurrencyInput,
    ) -> FieldResult<currency::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Check if currency code already exists
        let existing = currency::Entity::find()
            .filter(currency::Column::Code.eq(&input.code.to_uppercase()))
            .one(db)
            .await?;

        if existing.is_some() {
            return Err("Currency code already exists".into());
        }

        let currency_model = currency::ActiveModel::from(input);
        let currency = currency::Entity::insert(currency_model)
            .exec_with_returning(db)
            .await?;

        Ok(currency)
    }

    /// Update currency
    async fn update_currency(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateCurrencyInput,
    ) -> FieldResult<currency::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut currency: currency::ActiveModel =
            if let Some(currency) = currency::Entity::find_by_id(id).one(db).await? {
                currency.into()
            } else {
                return Err("Currency not found".into());
            };

        if let Some(name) = input.name {
            currency.name = Set(name);
        }

        if let Some(name_bg) = input.name_bg {
            currency.name_bg = Set(name_bg);
        }

        if let Some(symbol) = input.symbol {
            currency.symbol = Set(Some(symbol));
        }

        if let Some(decimal_places) = input.decimal_places {
            currency.decimal_places = Set(decimal_places);
        }

        if let Some(is_active) = input.is_active {
            currency.is_active = Set(is_active);
        }

        if let Some(bnb_code) = input.bnb_code {
            currency.bnb_code = Set(Some(bnb_code.to_uppercase()));
        }

        let updated_currency = currency::Entity::update(currency).exec(db).await?;
        Ok(updated_currency)
    }

    /// Create or update exchange rate
    async fn create_exchange_rate(
        &self,
        ctx: &Context<'_>,
        input: CreateExchangeRateInput,
    ) -> FieldResult<exchange_rate::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Check if rate already exists for this date
        let existing = exchange_rate::Entity::find()
            .filter(exchange_rate::Column::FromCurrencyId.eq(input.from_currency_id))
            .filter(exchange_rate::Column::ToCurrencyId.eq(input.to_currency_id))
            .filter(exchange_rate::Column::ValidDate.eq(input.valid_date))
            .one(db)
            .await?;

        if existing.is_some() {
            return Err("Exchange rate already exists for this date".into());
        }

        let mut rate_model = exchange_rate::ActiveModel::from(input);
        rate_model.created_by = Set(Some(1)); // TODO: Get from auth context

        let rate = exchange_rate::Entity::insert(rate_model)
            .exec_with_returning(db)
            .await?;

        Ok(rate)
    }

    /// Update exchange rate
    async fn update_exchange_rate(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateExchangeRateInput,
    ) -> FieldResult<exchange_rate::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut rate: exchange_rate::ActiveModel =
            if let Some(rate) = exchange_rate::Entity::find_by_id(id).one(db).await? {
                rate.into()
            } else {
                return Err("Exchange rate not found".into());
            };

        if let Some(new_rate) = input.rate {
            rate.rate = Set(new_rate);
        }

        if let Some(valid_date) = input.valid_date {
            rate.valid_date = Set(valid_date);
        }

        if let Some(rate_source) = input.rate_source {
            rate.rate_source = Set(rate_source);
        }

        if let Some(is_active) = input.is_active {
            rate.is_active = Set(is_active);
        }

        if let Some(notes) = input.notes {
            rate.notes = Set(Some(notes));
        }

        let updated_rate = exchange_rate::Entity::update(rate).exec(db).await?;
        Ok(updated_rate)
    }

    /// Update exchange rates from BNB for specific date
    async fn update_bnb_rates_for_date(
        &self,
        ctx: &Context<'_>,
        date: NaiveDate,
    ) -> FieldResult<i32> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let bnb_service = BnbService::new();

        let updated_count = bnb_service
            .update_rates_for_date(db, date)
            .await
            .map_err(|e| format!("Failed to update BNB rates: {}", e))?;

        Ok(updated_count as i32)
    }

    /// Update current exchange rates from BNB
    async fn update_current_bnb_rates(&self, ctx: &Context<'_>) -> FieldResult<i32> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let bnb_service = BnbService::new();

        let updated_count = bnb_service
            .update_current_rates(db)
            .await
            .map_err(|e| format!("Failed to update current BNB rates: {}", e))?;

        Ok(updated_count as i32)
    }

    /// Update BNB rates for date range
    async fn update_bnb_rates_for_range(
        &self,
        ctx: &Context<'_>,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> FieldResult<Vec<String>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let bnb_service = BnbService::new();

        let results = bnb_service
            .update_rates_for_range(db, from_date, to_date)
            .await
            .map_err(|e| format!("Failed to update BNB rates for range: {}", e))?;

        let mut response = Vec::new();
        for (date, count) in results {
            response.push(format!("{}: {} rates updated", date, count));
        }

        Ok(response)
    }

    /// Update exchange rates from ECB for specific date
    async fn update_ecb_rates_for_date(
        &self,
        ctx: &Context<'_>,
        date: NaiveDate,
    ) -> FieldResult<i32> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let ecb_service = EcbService::new();

        let updated_count = ecb_service
            .update_rates_for_date(db, date)
            .await
            .map_err(|e| format!("Failed to update ECB rates: {}", e))?;

        Ok(updated_count as i32)
    }

    /// Update current exchange rates from ECB
    async fn update_current_ecb_rates(&self, ctx: &Context<'_>) -> FieldResult<i32> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let ecb_service = EcbService::new();

        let updated_count = ecb_service
            .update_current_rates(db)
            .await
            .map_err(|e| format!("Failed to update current ECB rates: {}", e))?;

        Ok(updated_count as i32)
    }

    /// Update ECB rates for date range
    async fn update_ecb_rates_for_range(
        &self,
        ctx: &Context<'_>,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> FieldResult<Vec<String>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let ecb_service = EcbService::new();

        let results = ecb_service
            .update_rates_for_range(db, from_date, to_date)
            .await
            .map_err(|e| format!("Failed to update ECB rates for range: {}", e))?;

        let mut response = Vec::new();
        for (date, count) in results {
            response.push(format!("{}: {} rates updated", date, count));
        }

        Ok(response)
    }
}
