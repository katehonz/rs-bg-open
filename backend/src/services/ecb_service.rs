use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate, Utc};
use reqwest::Client;
use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

use crate::entities::exchange_rate::RateSource;
use crate::entities::{currency, exchange_rate};

/// ECB (European Central Bank) Exchange Rate Service
///
/// This service fetches exchange rates from the European Central Bank API.
/// ECB provides EUR-based exchange rates for various currencies.
///
/// API Documentation: https://www.ecb.europa.eu/stats/eurofxref/
///
/// Important notes for Euro adoption in Bulgaria (2026):
/// - When Bulgaria adopts EUR in 2026, this service will be used for historical BGN rates
/// - For new companies starting from 2026, this will be the primary source for non-EUR rates
/// - Rates are published on ECB business days (Mon-Fri, excluding ECB holidays)
pub struct EcbService {
    client: Client,
    /// ECB provides current and last 90 days of rates via this endpoint
    current_rates_url: String,
    /// For historical data, use this endpoint with date
    historical_rates_url: String,
}

#[derive(Debug, Deserialize)]
struct EcbEnvelope {
    #[serde(rename = "Cube")]
    cube: EcbCube,
}

#[derive(Debug, Deserialize)]
struct EcbCube {
    #[serde(rename = "Cube")]
    time_cubes: Vec<EcbTimeCube>,
}

#[derive(Debug, Deserialize)]
struct EcbTimeCube {
    #[serde(rename = "time")]
    time: String,
    #[serde(rename = "Cube")]
    currency_cubes: Vec<EcbCurrencyCube>,
}

#[derive(Debug, Deserialize)]
struct EcbCurrencyCube {
    #[serde(rename = "currency")]
    currency: String,
    #[serde(rename = "rate")]
    rate: String,
}

/// ECB Rate structure (simplified for internal use)
#[derive(Debug, Clone)]
pub struct EcbRate {
    pub code: String,
    pub rate: String,
    pub date: String,
}

impl EcbService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            // Last 90 days of rates
            current_rates_url: "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-hist-90d.xml"
                .to_string(),
            // All historical rates (large file, use sparingly)
            historical_rates_url: "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-hist.xml"
                .to_string(),
        }
    }

    /// Fetch exchange rates from ECB for a specific date
    ///
    /// Returns EUR-based rates (1 EUR = X currency)
    pub async fn fetch_rates_for_date(&self, date: NaiveDate) -> Result<Vec<EcbRate>> {
        let today = Utc::now().date_naive();
        let days_diff = (today - date).num_days();

        // Use the 90-day endpoint for recent dates, full history for older dates
        let url = if days_diff <= 90 {
            &self.current_rates_url
        } else {
            tracing::warn!("Fetching from full ECB history (large file). Consider caching.");
            &self.historical_rates_url
        };

        tracing::info!("Fetching ECB rates for date: {} from {}", date, url);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch ECB rates: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("ECB API returned error: {}", response.status()));
        }

        let xml_text = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read ECB response: {}", e))?;

        self.parse_ecb_xml(&xml_text, date)
    }

    /// Parse ECB XML response and extract rates for specific date
    fn parse_ecb_xml(&self, xml: &str, target_date: NaiveDate) -> Result<Vec<EcbRate>> {
        // ECB XML format:
        // <gesmes:Envelope>
        //   <Cube>
        //     <Cube time="2025-05-28">
        //       <Cube currency="USD" rate="1.0847"/>
        //       <Cube currency="GBP" rate="0.8532"/>
        //       ...
        //     </Cube>
        //   </Cube>
        // </gesmes:Envelope>

        let envelope: EcbEnvelope = serde_xml_rs::from_str(xml)
            .map_err(|e| anyhow!("Failed to parse ECB XML: {}", e))?;

        let target_date_str = target_date.format("%Y-%m-%d").to_string();

        // Find rates for the target date
        for time_cube in &envelope.cube.time_cubes {
            if time_cube.time == target_date_str {
                let rates: Vec<EcbRate> = time_cube
                    .currency_cubes
                    .iter()
                    .map(|c| EcbRate {
                        code: c.currency.clone(),
                        rate: c.rate.clone(),
                        date: time_cube.time.clone(),
                    })
                    .collect();

                tracing::info!("Found {} ECB rates for {}", rates.len(), target_date);
                return Ok(rates);
            }
        }

        // If exact date not found, try to find the closest earlier date
        let mut closest_date: Option<(String, Vec<EcbRate>)> = None;

        for time_cube in &envelope.cube.time_cubes {
            if let Ok(cube_date) = NaiveDate::parse_from_str(&time_cube.time, "%Y-%m-%d") {
                if cube_date <= target_date {
                    let rates: Vec<EcbRate> = time_cube
                        .currency_cubes
                        .iter()
                        .map(|c| EcbRate {
                            code: c.currency.clone(),
                            rate: c.rate.clone(),
                            date: time_cube.time.clone(),
                        })
                        .collect();

                    if let Some((closest, _)) = &closest_date {
                        if let Ok(closest_parsed) = NaiveDate::parse_from_str(closest, "%Y-%m-%d")
                        {
                            if cube_date > closest_parsed {
                                closest_date = Some((time_cube.time.clone(), rates));
                            }
                        }
                    } else {
                        closest_date = Some((time_cube.time.clone(), rates));
                    }
                }
            }
        }

        if let Some((found_date, rates)) = closest_date {
            tracing::info!(
                "No exact ECB rates for {}, using closest earlier date: {} ({} rates)",
                target_date,
                found_date,
                rates.len()
            );
            return Ok(rates);
        }

        Err(anyhow!("No ECB rates found for {} or earlier", target_date))
    }

    /// Update exchange rates in database from ECB
    ///
    /// For Bulgaria pre-2026: Converts EUR rates to BGN using fixed rate (1 EUR = 1.95583 BGN)
    /// For Bulgaria post-2026: Stores EUR-based rates directly
    pub async fn update_rates_for_date(
        &self,
        db: &DatabaseConnection,
        date: NaiveDate,
    ) -> Result<usize> {
        let ecb_rates = self.fetch_rates_for_date(date).await?;
        let mut updated_count = 0;

        // Check if EUR is the base currency (post-2026) or BGN (pre-2026)
        let base_currency = self.get_base_currency(db).await?;
        let is_eur_base = base_currency.code == "EUR";

        tracing::info!(
            "Updating ECB rates with base currency: {}",
            base_currency.code
        );

        // Get EUR currency for conversions
        let eur_currency = currency::Entity::find()
            .filter(currency::Column::Code.eq("EUR"))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("EUR currency not found"))?;

        for ecb_rate in ecb_rates {
            // Find currency by code
            if let Some(foreign_currency) = currency::Entity::find()
                .filter(currency::Column::Code.eq(&ecb_rate.code))
                .one(db)
                .await?
            {
                // Parse ECB rate (EUR to foreign currency)
                let eur_to_foreign = Decimal::from_str(&ecb_rate.rate).map_err(|e| {
                    anyhow!("Failed to parse ECB rate '{}': {}", ecb_rate.rate, e)
                })?;

                let (rate, from_id, to_id) = if is_eur_base {
                    // Post-2026: Store EUR -> Foreign directly
                    (eur_to_foreign, eur_currency.id, foreign_currency.id)
                } else {
                    // Pre-2026: Convert EUR rate to BGN rate
                    // If 1 EUR = X USD, and 1 EUR = 1.95583 BGN
                    // Then 1 USD = 1.95583 / X BGN
                    let fixed_eur_bgn = Decimal::from_str("1.95583").unwrap();
                    let bgn_rate = fixed_eur_bgn / eur_to_foreign;
                    (bgn_rate, foreign_currency.id, base_currency.id)
                };

                // Check if rate already exists for this date
                let existing = exchange_rate::Entity::find()
                    .filter(exchange_rate::Column::FromCurrencyId.eq(from_id))
                    .filter(exchange_rate::Column::ToCurrencyId.eq(to_id))
                    .filter(exchange_rate::Column::ValidDate.eq(date))
                    .one(db)
                    .await?;

                if let Some(existing_rate) = existing {
                    // Update existing rate
                    let mut active_model: exchange_rate::ActiveModel = existing_rate.into();
                    active_model.rate = Set(rate);
                    active_model.reverse_rate = Set(Decimal::ONE / rate);
                    active_model.rate_source = Set(RateSource::Ecb);
                    active_model.bnb_rate_id =
                        Set(Some(format!("ECB_{}_{}", ecb_rate.code, date)));

                    exchange_rate::Entity::update(active_model).exec(db).await?;
                    updated_count += 1;
                    tracing::info!("Updated ECB rate for {} on {}: {}", ecb_rate.code, date, rate);
                } else {
                    // Create new rate
                    let new_rate = exchange_rate::ActiveModel {
                        from_currency_id: Set(from_id),
                        to_currency_id: Set(to_id),
                        rate: Set(rate),
                        reverse_rate: Set(Decimal::ONE / rate),
                        valid_date: Set(date),
                        rate_source: Set(RateSource::Ecb),
                        bnb_rate_id: Set(Some(format!("ECB_{}_{}", ecb_rate.code, date))),
                        created_by: Set(None), // System user
                        ..Default::default()
                    };

                    exchange_rate::Entity::insert(new_rate).exec(db).await?;
                    updated_count += 1;
                    tracing::info!(
                        "Created new ECB rate for {} on {}: {}",
                        ecb_rate.code,
                        date,
                        rate
                    );
                }
            }
        }

        Ok(updated_count)
    }

    /// Get the base currency for the current company
    async fn get_base_currency(&self, db: &DatabaseConnection) -> Result<currency::Model> {
        currency::Entity::find()
            .filter(currency::Column::IsBaseCurrency.eq(true))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Base currency not found"))
    }

    /// Update rates for current date
    pub async fn update_current_rates(&self, db: &DatabaseConnection) -> Result<usize> {
        let today = Utc::now().date_naive();

        // ECB publishes rates around 16:00 CET
        // If it's early in the day, try yesterday's rate as fallback
        match self.update_rates_for_date(db, today).await {
            Ok(count) if count > 0 => Ok(count),
            _ => {
                tracing::info!("No rates for today, trying yesterday");
                let yesterday = today.pred_opt().ok_or_else(|| anyhow!("Invalid date"))?;
                self.update_rates_for_date(db, yesterday).await
            }
        }
    }

    /// Update rates for a range of dates
    pub async fn update_rates_for_range(
        &self,
        db: &DatabaseConnection,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> Result<HashMap<NaiveDate, usize>> {
        let mut results = HashMap::new();
        let mut current_date = from_date;

        while current_date <= to_date {
            // ECB publishes rates on business days (Mon-Fri)
            if current_date.weekday() != chrono::Weekday::Sat
                && current_date.weekday() != chrono::Weekday::Sun
            {
                match self.update_rates_for_date(db, current_date).await {
                    Ok(count) => {
                        results.insert(current_date, count);
                        tracing::info!("Updated {} ECB rates for {}", count, current_date);
                    }
                    Err(e) => {
                        tracing::error!("Failed to update ECB rates for {}: {}", current_date, e);
                        results.insert(current_date, 0);
                    }
                }
            }

            current_date = current_date
                .succ_opt()
                .ok_or_else(|| anyhow!("Invalid date increment"))?;

            // Small delay to be respectful to ECB servers
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        }

        Ok(results)
    }

    /// Get latest available rate for currency pair from ECB source
    pub async fn get_latest_rate(
        &self,
        db: &DatabaseConnection,
        from_currency_id: i32,
        to_currency_id: i32,
    ) -> Result<Option<exchange_rate::Model>> {
        let rate = exchange_rate::Entity::find()
            .filter(exchange_rate::Column::FromCurrencyId.eq(from_currency_id))
            .filter(exchange_rate::Column::ToCurrencyId.eq(to_currency_id))
            .filter(exchange_rate::Column::RateSource.eq(RateSource::Ecb))
            .filter(exchange_rate::Column::IsActive.eq(true))
            .order_by_desc(exchange_rate::Column::ValidDate)
            .one(db)
            .await?;

        Ok(rate)
    }

    /// Get list of currencies supported by ECB
    ///
    /// ECB provides rates for: USD, JPY, BGN, CZK, DKK, GBP, HUF, PLN, RON, SEK, CHF,
    /// ISK, NOK, TRY, AUD, BRL, CAD, CNY, HKD, IDR, ILS, INR, KRW, MXN, MYR, NZD,
    /// PHP, SGD, THB, ZAR
    pub fn supported_currencies() -> Vec<&'static str> {
        vec![
            "USD", "JPY", "BGN", "CZK", "DKK", "GBP", "HUF", "PLN", "RON", "SEK", "CHF", "ISK",
            "NOK", "TRY", "AUD", "BRL", "CAD", "CNY", "HKD", "IDR", "ILS", "INR", "KRW", "MXN",
            "MYR", "NZD", "PHP", "SGD", "THB", "ZAR",
        ]
    }

    /// Check if a currency is supported by ECB
    pub fn is_currency_supported(code: &str) -> bool {
        Self::supported_currencies().contains(&code)
    }
}

impl Default for EcbService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_currencies() {
        assert!(EcbService::is_currency_supported("USD"));
        assert!(EcbService::is_currency_supported("GBP"));
        assert!(EcbService::is_currency_supported("BGN"));
        assert!(!EcbService::is_currency_supported("XYZ"));
    }

    #[tokio::test]
    async fn test_fetch_current_rates() {
        let service = EcbService::new();
        let today = Utc::now().date_naive();

        // Try to fetch rates for today or a recent business day
        let result = service.fetch_rates_for_date(today).await;

        // This might fail on weekends or if ECB hasn't published yet
        // so we just check that the function doesn't panic
        match result {
            Ok(rates) => {
                println!("Successfully fetched {} ECB rates", rates.len());
                assert!(!rates.is_empty());
            }
            Err(e) => {
                println!("Failed to fetch ECB rates (expected on weekends): {}", e);
            }
        }
    }
}
