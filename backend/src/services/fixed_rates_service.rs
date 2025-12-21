use anyhow::Result;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::str::FromStr;

use crate::entities::exchange_rate::RateSource;
use crate::entities::{currency, exchange_rate};

/// Service for managing fixed exchange rates (like EUR/BGN currency board)
pub struct FixedRatesService;

impl FixedRatesService {
    pub fn new() -> Self {
        Self
    }

    /// EUR/BGN is fixed at 1.95583 (Bulgarian currency board)
    pub const EUR_BGN_RATE: &'static str = "1.95583";

    /// Ensure fixed EUR/BGN rate exists in database for a specific date
    pub async fn ensure_eur_bgn_rate(
        &self,
        db: &DatabaseConnection,
        date: NaiveDate,
    ) -> Result<()> {
        let eur_currency = currency::Entity::find()
            .filter(currency::Column::Code.eq("EUR"))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("EUR currency not found"))?;

        let bgn_currency = currency::Entity::find()
            .filter(currency::Column::Code.eq("BGN"))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("BGN currency not found"))?;

        let rate = Decimal::from_str(Self::EUR_BGN_RATE)?;
        let reverse_rate = Decimal::ONE / rate;

        // Check if rate already exists for this date
        let existing = exchange_rate::Entity::find()
            .filter(exchange_rate::Column::FromCurrencyId.eq(eur_currency.id))
            .filter(exchange_rate::Column::ToCurrencyId.eq(bgn_currency.id))
            .filter(exchange_rate::Column::ValidDate.eq(date))
            .one(db)
            .await?;

        if let Some(existing_rate) = existing {
            // Update to ensure it's always the fixed rate
            let mut active_model: exchange_rate::ActiveModel = existing_rate.into();
            active_model.rate = Set(rate);
            active_model.reverse_rate = Set(reverse_rate);
            active_model.rate_source = Set(RateSource::Manual); // Mark as manual/fixed
            active_model.notes = Set(Some("Fixed EUR/BGN currency board rate".to_string()));

            exchange_rate::Entity::update(active_model).exec(db).await?;
            tracing::info!("Updated fixed EUR/BGN rate for {}: {}", date, rate);
        } else {
            // Create new fixed rate
            let new_rate = exchange_rate::ActiveModel {
                from_currency_id: Set(eur_currency.id),
                to_currency_id: Set(bgn_currency.id),
                rate: Set(rate),
                reverse_rate: Set(reverse_rate),
                valid_date: Set(date),
                rate_source: Set(RateSource::Manual),
                bnb_rate_id: Set(Some(format!("FIXED_EUR_BGN_{}", date))),
                notes: Set(Some("Fixed EUR/BGN currency board rate".to_string())),
                created_by: Set(None),
                ..Default::default()
            };

            exchange_rate::Entity::insert(new_rate).exec(db).await?;
            tracing::info!("Created fixed EUR/BGN rate for {}: {}", date, rate);
        }

        Ok(())
    }

    /// Get the fixed EUR/BGN rate (always returns 1.95583)
    pub fn get_eur_bgn_rate() -> Decimal {
        Decimal::from_str(Self::EUR_BGN_RATE).unwrap()
    }

    /// Check if a currency pair is a fixed rate
    pub fn is_fixed_rate(from_code: &str, to_code: &str) -> bool {
        (from_code == "EUR" && to_code == "BGN") || (from_code == "BGN" && to_code == "EUR")
    }

    /// List of legacy EU currencies with fixed EUR conversion rates
    /// These currencies no longer exist but may appear in historical data
    pub fn get_legacy_eu_fixed_rates() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            // (Currency Code, Name, Fixed Rate to EUR)
            ("HRK", "Croatian Kuna", "7.53450"),      // Croatia joined EUR 2023-01-01
            ("LTL", "Lithuanian Litas", "3.45280"),   // Lithuania joined EUR 2015-01-01
            ("LVL", "Latvian Lats", "0.702804"),      // Latvia joined EUR 2014-01-01
            ("EEK", "Estonian Kroon", "15.6466"),     // Estonia joined EUR 2011-01-01
            ("SKK", "Slovak Koruna", "30.1260"),      // Slovakia joined EUR 2009-01-01
            ("CYP", "Cypriot Pound", "0.585274"),     // Cyprus joined EUR 2008-01-01
            ("MTL", "Maltese Lira", "0.429300"),      // Malta joined EUR 2008-01-01
            ("SIT", "Slovenian Tolar", "239.640"),    // Slovenia joined EUR 2007-01-01
        ]
    }
}
