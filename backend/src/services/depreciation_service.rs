//! Depreciation Service
//!
//! Handles fixed asset depreciation calculations according to Bulgarian accounting
//! and tax standards (ЗКПО - Corporate Income Tax Act)

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use sea_orm::*;
use std::collections::HashMap;

use crate::entities::{
    account, depreciation_journal, entry_line, fixed_asset, fixed_asset_category, journal_entry,
};

/// Service for managing fixed asset depreciation calculations
pub struct DepreciationService;

/// Monthly depreciation calculation result
#[derive(Debug, Clone)]
pub struct MonthlyDepreciation {
    pub fixed_asset_id: i32,
    pub period: NaiveDate,
    pub accounting_amount: Decimal,
    pub accounting_book_value_before: Decimal,
    pub accounting_book_value_after: Decimal,
    pub tax_amount: Decimal,
    pub tax_book_value_before: Decimal,
    pub tax_book_value_after: Decimal,
}

/// Bulk depreciation calculation for a company
#[derive(Debug)]
pub struct BulkDepreciationResult {
    pub calculated: Vec<MonthlyDepreciation>,
    pub errors: Vec<DepreciationError>,
    pub total_accounting_amount: Decimal,
    pub total_tax_amount: Decimal,
}

/// Depreciation calculation error
#[derive(Debug)]
pub struct DepreciationError {
    pub fixed_asset_id: i32,
    pub asset_name: String,
    pub error_message: String,
}

/// Journal entry creation result
#[derive(Debug)]
pub struct DepreciationJournalEntry {
    pub journal_entry_id: i32,
    pub total_amount: Decimal,
    pub assets_count: i32,
}

impl DepreciationService {
    pub fn new() -> Self {
        Self
    }

    /// Calculate monthly depreciation for a single asset
    pub async fn calculate_monthly_depreciation(
        &self,
        db: &DatabaseConnection,
        asset_id: i32,
        period: NaiveDate,
    ) -> Result<MonthlyDepreciation, Box<dyn std::error::Error + Send + Sync>> {
        // Load the fixed asset with category
        let asset = fixed_asset::Entity::find_by_id(asset_id)
            .one(db)
            .await?
            .ok_or("Fixed asset not found")?;

        if asset.status != "active" {
            return Err(format!("Asset {} is not active", asset.name).into());
        }

        // Check if depreciation already calculated for this period
        let existing = depreciation_journal::Entity::find()
            .filter(depreciation_journal::Column::FixedAssetId.eq(asset_id))
            .filter(depreciation_journal::Column::Period.eq(period))
            .one(db)
            .await?;

        if existing.is_some() {
            return Err(format!(
                "Depreciation already calculated for asset {} in period {}-{}",
                asset.name,
                period.year(),
                period.month()
            )
            .into());
        }

        // Check if asset was put into service before or during this period
        if let Some(service_date) = asset.put_into_service_date {
            if service_date > period {
                return Err(format!(
                    "Asset {} not yet in service for period {}-{}",
                    asset.name,
                    period.year(),
                    period.month()
                )
                .into());
            }
        }

        // Validate sequential period calculation - check if previous period is calculated
        self.validate_sequential_period(db, asset_id, &asset, period).await?;

        // Calculate accounting depreciation
        let accounting_monthly = self.calculate_accounting_depreciation(&asset, period)?;
        let accounting_book_value_before = asset.accounting_book_value;
        let accounting_book_value_after =
            (accounting_book_value_before - accounting_monthly).max(asset.accounting_salvage_value);

        // Calculate tax depreciation
        let tax_monthly = self.calculate_tax_depreciation(&asset, period)?;
        let tax_book_value_before = asset.tax_book_value;
        let tax_book_value_after = (tax_book_value_before - tax_monthly).max(Decimal::from(0));

        Ok(MonthlyDepreciation {
            fixed_asset_id: asset_id,
            period,
            accounting_amount: accounting_monthly,
            accounting_book_value_before,
            accounting_book_value_after,
            tax_amount: tax_monthly,
            tax_book_value_before,
            tax_book_value_after,
        })
    }

    /// Calculate accounting depreciation amount for a single month
    fn calculate_accounting_depreciation(
        &self,
        asset: &fixed_asset::Model,
        _period: NaiveDate,
    ) -> Result<Decimal, Box<dyn std::error::Error + Send + Sync>> {
        if asset.accounting_book_value <= asset.accounting_salvage_value {
            return Ok(Decimal::from(0)); // Fully depreciated
        }

        match asset.accounting_depreciation_method.as_str() {
            "straight_line" => {
                let depreciable_amount = asset.acquisition_cost - asset.accounting_salvage_value;
                let annual_amount =
                    depreciable_amount * (asset.accounting_depreciation_rate / Decimal::from(100));
                Ok(annual_amount / Decimal::from(12))
            }
            "declining_balance" => {
                // Double-declining balance method
                let book_value = asset.accounting_book_value;
                let annual_rate = asset.accounting_depreciation_rate / Decimal::from(100);
                let monthly_rate = annual_rate / Decimal::from(12);
                let monthly_amount = book_value * monthly_rate;

                // Ensure we don't depreciate below salvage value
                let max_depreciation = asset.accounting_book_value - asset.accounting_salvage_value;
                Ok(monthly_amount.min(max_depreciation))
            }
            _ => Err(format!(
                "Unsupported depreciation method: {}",
                asset.accounting_depreciation_method
            )
            .into()),
        }
    }

    /// Calculate tax depreciation amount for a single month
    fn calculate_tax_depreciation(
        &self,
        asset: &fixed_asset::Model,
        _period: NaiveDate,
    ) -> Result<Decimal, Box<dyn std::error::Error + Send + Sync>> {
        if asset.tax_book_value <= Decimal::from(0) {
            return Ok(Decimal::from(0)); // Fully depreciated for tax purposes
        }

        // Tax depreciation is always straight-line in Bulgaria
        let annual_amount =
            asset.acquisition_cost * (asset.tax_depreciation_rate / Decimal::from(100));
        let monthly_amount = annual_amount / Decimal::from(12);

        // Ensure we don't depreciate below zero
        Ok(monthly_amount.min(asset.tax_book_value))
    }

    /// Calculate monthly depreciation for all active assets in a company
    pub async fn calculate_bulk_depreciation(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        period: NaiveDate,
    ) -> Result<BulkDepreciationResult, Box<dyn std::error::Error + Send + Sync>> {
        // Get all active fixed assets for the company
        let assets = fixed_asset::Entity::find()
            .filter(fixed_asset::Column::CompanyId.eq(company_id))
            .filter(fixed_asset::Column::Status.eq("active"))
            .all(db)
            .await?;

        let mut calculated = Vec::new();
        let mut errors = Vec::new();
        let mut total_accounting_amount = Decimal::from(0);
        let mut total_tax_amount = Decimal::from(0);

        for asset in assets {
            match self
                .calculate_monthly_depreciation(db, asset.id, period)
                .await
            {
                Ok(depreciation) => {
                    total_accounting_amount += depreciation.accounting_amount;
                    total_tax_amount += depreciation.tax_amount;
                    calculated.push(depreciation);
                }
                Err(err) => {
                    errors.push(DepreciationError {
                        fixed_asset_id: asset.id,
                        asset_name: asset.name.clone(),
                        error_message: err.to_string(),
                    });
                }
            }
        }

        Ok(BulkDepreciationResult {
            calculated,
            errors,
            total_accounting_amount,
            total_tax_amount,
        })
    }

    /// Save calculated depreciation to the database
    pub async fn save_depreciation(
        &self,
        db: &DatabaseConnection,
        depreciation: MonthlyDepreciation,
        company_id: i32,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let depreciation_model = depreciation_journal::ActiveModel {
            fixed_asset_id: Set(depreciation.fixed_asset_id),
            period: Set(depreciation.period),
            company_id: Set(company_id),
            accounting_depreciation_amount: Set(depreciation.accounting_amount),
            accounting_book_value_before: Set(depreciation.accounting_book_value_before),
            accounting_book_value_after: Set(depreciation.accounting_book_value_after),
            tax_depreciation_amount: Set(depreciation.tax_amount),
            tax_book_value_before: Set(depreciation.tax_book_value_before),
            tax_book_value_after: Set(depreciation.tax_book_value_after),
            is_posted: Set(false),
            ..Default::default()
        };

        let result = depreciation_journal::Entity::insert(depreciation_model)
            .exec(db)
            .await?;

        // Update the fixed asset book values
        let asset_update = fixed_asset::ActiveModel {
            id: Set(depreciation.fixed_asset_id),
            accounting_book_value: Set(depreciation.accounting_book_value_after),
            tax_book_value: Set(depreciation.tax_book_value_after),
            accounting_accumulated_depreciation: Set(depreciation.accounting_book_value_before
                - depreciation.accounting_book_value_after),
            tax_accumulated_depreciation: Set(
                depreciation.tax_book_value_before - depreciation.tax_book_value_after
            ),
            ..Default::default()
        };

        fixed_asset::Entity::update(asset_update).exec(db).await?;

        Ok(result.last_insert_id)
    }

    /// Create journal entry for depreciation postings
    pub async fn create_depreciation_journal_entry(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        period: NaiveDate,
        user_id: i32,
        reference: Option<String>,
    ) -> Result<DepreciationJournalEntry, Box<dyn std::error::Error + Send + Sync>> {
        // Get all unposted depreciation for the period and company
        let depreciations = depreciation_journal::Entity::find()
            .filter(depreciation_journal::Column::CompanyId.eq(company_id))
            .filter(depreciation_journal::Column::Period.eq(period))
            .filter(depreciation_journal::Column::IsPosted.eq(false))
            .all(db)
            .await?;

        if depreciations.is_empty() {
            return Err("No unposted depreciation entries found for the specified period".into());
        }

        // Filter out depreciations with zero amounts
        let non_zero_depreciations: Vec<_> = depreciations
            .into_iter()
            .filter(|d| d.accounting_depreciation_amount > Decimal::ZERO)
            .collect();

        if non_zero_depreciations.is_empty() {
            return Err("All depreciation entries have zero amounts".into());
        }

        // Group by asset categories to get proper account codes
        let mut category_groups: HashMap<i32, (Decimal, Vec<i32>)> = HashMap::new();

        for depreciation in &non_zero_depreciations {
            let asset = fixed_asset::Entity::find_by_id(depreciation.fixed_asset_id)
                .one(db)
                .await?
                .ok_or("Asset not found")?;

            let entry = category_groups
                .entry(asset.category_id)
                .or_insert((Decimal::from(0), Vec::new()));
            entry.0 += depreciation.accounting_depreciation_amount;
            entry.1.push(depreciation.id);
        }

        let total_amount = non_zero_depreciations
            .iter()
            .map(|d| d.accounting_depreciation_amount)
            .fold(Decimal::from(0), |acc, amount| acc + amount);

        // Generate entry number
        let entry_number = format!(
            "DEP-{}-{:02}-{}",
            period.year(),
            period.month(),
            chrono::Utc::now().format("%H%M%S")
        );

        // Create the main journal entry
        let journal_entry = journal_entry::ActiveModel {
            entry_number: Set(entry_number),
            company_id: Set(company_id),
            document_date: Set(period),
            accounting_date: Set(period),
            document_number: Set(reference.map(|r| r.clone()).or_else(|| {
                Some(format!(
                    "Амортизация за {}-{:02}",
                    period.year(),
                    period.month()
                ))
            })),
            description: Set(format!(
                "Месечна амортизация за {}-{:02}",
                period.year(),
                period.month()
            )),
            vat_document_type: Set(Some("DEPRECIATION".to_string())),
            total_amount: Set(total_amount),
            total_vat_amount: Set(Decimal::from(0)),
            is_posted: Set(true),
            created_by: Set(user_id),
            ..Default::default()
        };

        let journal_result = journal_entry::Entity::insert(journal_entry)
            .exec(db)
            .await?;
        let journal_entry_id = journal_result.last_insert_id;

        // Create entry lines for each category
        for (category_id, (amount, _)) in category_groups {
            let category = fixed_asset_category::Entity::find_by_id(category_id)
                .one(db)
                .await?
                .ok_or("Asset category not found")?;

            // Find accounts
            let expense_account = account::Entity::find()
                .filter(account::Column::Code.eq(&category.expense_account_code))
                .filter(account::Column::CompanyId.eq(company_id))
                .one(db)
                .await?
                .ok_or(format!(
                    "Expense account {} not found",
                    category.expense_account_code
                ))?;

            let depreciation_account = account::Entity::find()
                .filter(account::Column::Code.eq(&category.depreciation_account_code))
                .filter(account::Column::CompanyId.eq(company_id))
                .one(db)
                .await?
                .ok_or(format!(
                    "Depreciation account {} not found",
                    category.depreciation_account_code
                ))?;

            // Debit expense account (603)
            let expense_line = entry_line::ActiveModel {
                journal_entry_id: Set(journal_entry_id),
                account_id: Set(expense_account.id),
                debit_amount: Set(amount),
                credit_amount: Set(Decimal::from(0)),
                description: Set(Some(format!(
                    "Амортизация {} - {}",
                    category.code, category.name
                ))),
                line_order: Set(1),
                base_amount: Set(amount),
                vat_amount: Set(Decimal::from(0)),
                ..Default::default()
            };

            entry_line::Entity::insert(expense_line).exec(db).await?;

            // Credit accumulated depreciation account (241)
            let depreciation_line = entry_line::ActiveModel {
                journal_entry_id: Set(journal_entry_id),
                account_id: Set(depreciation_account.id),
                debit_amount: Set(Decimal::from(0)),
                credit_amount: Set(amount),
                description: Set(Some(format!(
                    "Натрупана амортизация {} - {}",
                    category.code, category.name
                ))),
                line_order: Set(2),
                base_amount: Set(amount),
                vat_amount: Set(Decimal::from(0)),
                ..Default::default()
            };

            entry_line::Entity::insert(depreciation_line)
                .exec(db)
                .await?;
        }

        // Mark all depreciation entries as posted
        let now = chrono::Utc::now().naive_utc();
        let depreciation_count = non_zero_depreciations.len();
        for depreciation in non_zero_depreciations {
            let update_model = depreciation_journal::ActiveModel {
                id: Set(depreciation.id),
                is_posted: Set(true),
                journal_entry_id: Set(Some(journal_entry_id)),
                posted_at: Set(Some(now)),
                posted_by: Set(Some(user_id)),
                ..Default::default()
            };

            depreciation_journal::Entity::update(update_model)
                .exec(db)
                .await?;
        }

        Ok(DepreciationJournalEntry {
            journal_entry_id,
            total_amount,
            assets_count: depreciation_count as i32,
        })
    }

    /// Get depreciation summary for a company and period
    pub async fn get_depreciation_summary(
        &self,
        _db: &DatabaseConnection,
        company_id: i32,
        year: i32,
        month: Option<u32>,
    ) -> Result<Vec<(String, Decimal, Decimal, i32)>, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut query = depreciation_journal::Entity::find()
            .filter(depreciation_journal::Column::CompanyId.eq(company_id));

        // Add date filters
        if let Some(month) = month {
            let period = NaiveDate::from_ymd_opt(year, month, 1).ok_or("Invalid year/month")?;
            query = query.filter(depreciation_journal::Column::Period.eq(period));
        } else {
            let start_date = NaiveDate::from_ymd_opt(year, 1, 1).ok_or("Invalid year")?;
            let end_date = NaiveDate::from_ymd_opt(year, 12, 1).ok_or("Invalid year")?;
            query = query
                .filter(depreciation_journal::Column::Period.gte(start_date))
                .filter(depreciation_journal::Column::Period.lte(end_date));
        }

        // This would need to be implemented with proper SQL grouping
        // For now, returning empty result as this requires more complex SQL
        Ok(Vec::new())
    }

    /// Check which assets need depreciation for a given period
    pub async fn get_assets_needing_depreciation(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        period: NaiveDate,
    ) -> Result<Vec<fixed_asset::Model>, Box<dyn std::error::Error + Send + Sync>> {
        // Get active assets that don't have depreciation calculated for this period
        let assets = fixed_asset::Entity::find()
            .filter(fixed_asset::Column::CompanyId.eq(company_id))
            .filter(fixed_asset::Column::Status.eq("active"))
            .filter(
                Condition::all()
                    .add(fixed_asset::Column::PutIntoServiceDate.lte(period))
                    .add(
                        Condition::any()
                            .add(fixed_asset::Column::AccountingBookValue.gt(Decimal::from(0)))
                            .add(fixed_asset::Column::TaxBookValue.gt(Decimal::from(0))),
                    ),
            )
            .all(db)
            .await?;

        // Filter out assets that already have depreciation for this period
        let mut result = Vec::new();
        for asset in assets {
            let existing = depreciation_journal::Entity::find()
                .filter(depreciation_journal::Column::FixedAssetId.eq(asset.id))
                .filter(depreciation_journal::Column::Period.eq(period))
                .one(db)
                .await?;

            if existing.is_none() {
                result.push(asset);
            }
        }

        Ok(result)
    }

    /// Validate that depreciation is calculated sequentially
    /// Checks if all previous periods since put-into-service date are calculated
    async fn validate_sequential_period(
        &self,
        db: &DatabaseConnection,
        asset_id: i32,
        asset: &fixed_asset::Model,
        period: NaiveDate,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get the put-into-service date or acquisition date as starting point
        let start_date = asset.put_into_service_date.unwrap_or(asset.acquisition_date);

        // If this is the first month of service, no previous period to check
        let start_period = NaiveDate::from_ymd_opt(start_date.year(), start_date.month(), 1)
            .ok_or("Invalid start date")?;

        if period <= start_period {
            // This is the first or same period as service start, so it's valid
            return Ok(());
        }

        // Calculate previous period
        let mut check_year = period.year();
        let mut check_month = period.month();

        if check_month == 1 {
            check_month = 12;
            check_year -= 1;
        } else {
            check_month -= 1;
        }

        let previous_period = NaiveDate::from_ymd_opt(check_year, check_month, 1)
            .ok_or("Invalid previous period")?;

        // Check if previous period exists
        let previous_exists = depreciation_journal::Entity::find()
            .filter(depreciation_journal::Column::FixedAssetId.eq(asset_id))
            .filter(depreciation_journal::Column::Period.eq(previous_period))
            .one(db)
            .await?;

        if previous_exists.is_none() && previous_period >= start_period {
            return Err(format!(
                "Cannot calculate depreciation for {}-{:02}. Previous period {}-{:02} must be calculated first",
                period.year(),
                period.month(),
                previous_period.year(),
                previous_period.month()
            )
            .into());
        }

        Ok(())
    }

    /// Get calculated periods for a fixed asset
    pub async fn get_calculated_periods(
        &self,
        db: &DatabaseConnection,
        asset_id: i32,
    ) -> Result<Vec<depreciation_journal::Model>, Box<dyn std::error::Error + Send + Sync>> {
        let periods = depreciation_journal::Entity::find()
            .filter(depreciation_journal::Column::FixedAssetId.eq(asset_id))
            .order_by_asc(depreciation_journal::Column::Period)
            .all(db)
            .await?;

        Ok(periods)
    }

    /// Get all calculated periods for a company
    pub async fn get_company_calculated_periods(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
    ) -> Result<Vec<(i32, u32)>, Box<dyn std::error::Error + Send + Sync>> {
        use sea_orm::QuerySelect;

        let periods = depreciation_journal::Entity::find()
            .filter(depreciation_journal::Column::CompanyId.eq(company_id))
            .filter(depreciation_journal::Column::IsPosted.eq(true))
            .select_only()
            .column(depreciation_journal::Column::Period)
            .distinct()
            .all(db)
            .await?;

        let mut result: Vec<(i32, u32)> = periods
            .iter()
            .map(|p| (p.period.year(), p.period.month()))
            .collect();

        result.sort();
        result.dedup();

        Ok(result)
    }
}
