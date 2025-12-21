//! Asset Value Adjustment Service
//!
//! Handles changes to fixed asset values with proper depreciation recalculation
//! according to Bulgarian accounting and tax requirements.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use sea_orm::*;

use crate::entities::{
    account, asset_value_adjustment, depreciation_journal, entry_line, fixed_asset,
    fixed_asset_category, journal_entry, AssetAdjustmentType,
};
use crate::services::depreciation_service::DepreciationService;

/// Service for managing asset value adjustments
pub struct AssetAdjustmentService;

/// Result of applying an asset value adjustment
#[derive(Debug)]
pub struct AdjustmentResult {
    pub adjustment_id: i32,
    pub journal_entry_id: Option<i32>,
    pub depreciation_periods_calculated: Vec<NaiveDate>,
    pub message: String,
}

/// Error types for asset adjustments
#[derive(Debug)]
pub enum AdjustmentError {
    AssetNotFound,
    InvalidAdjustmentType,
    InvalidAmount,
    DepreciationNotSequential(String),
    AccountNotFound(String),
    DatabaseError(String),
}

impl std::fmt::Display for AdjustmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssetNotFound => write!(f, "Asset not found"),
            Self::InvalidAdjustmentType => write!(f, "Invalid adjustment type"),
            Self::InvalidAmount => write!(f, "Invalid adjustment amount"),
            Self::DepreciationNotSequential(msg) => write!(f, "Depreciation error: {}", msg),
            Self::AccountNotFound(code) => write!(f, "Account not found: {}", code),
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for AdjustmentError {}

impl From<DbErr> for AdjustmentError {
    fn from(err: DbErr) -> Self {
        Self::DatabaseError(err.to_string())
    }
}

impl AssetAdjustmentService {
    pub fn new() -> Self {
        Self
    }

    /// Apply an improvement (increase in value) to a fixed asset
    ///
    /// Steps:
    /// 1. Calculate and save all depreciation up to the adjustment date
    /// 2. Apply the value increase to the asset
    /// 3. Optionally update depreciation rates/useful life
    /// 4. Create journal entry for the improvement
    pub async fn apply_improvement(
        &self,
        db: &DatabaseConnection,
        asset_id: i32,
        adjustment_date: NaiveDate,
        improvement_amount: Decimal,
        reason: String,
        document_number: Option<String>,
        new_accounting_rate: Option<Decimal>,
        new_tax_rate: Option<Decimal>,
        new_accounting_life: Option<i32>,
        new_tax_life: Option<i32>,
        user_id: i32,
    ) -> Result<AdjustmentResult, AdjustmentError> {
        // Step 1: Load asset and validate
        let asset = fixed_asset::Entity::find_by_id(asset_id)
            .one(db)
            .await?
            .ok_or(AdjustmentError::AssetNotFound)?;

        if asset.status != "active" {
            return Err(AdjustmentError::DatabaseError(
                "Asset must be active to apply improvement".to_string(),
            ));
        }

        if improvement_amount <= Decimal::ZERO {
            return Err(AdjustmentError::InvalidAmount);
        }

        // Step 2: Calculate depreciation up to adjustment month
        let depreciation_periods = self
            .calculate_depreciation_up_to_date(db, &asset, adjustment_date)
            .await?;

        // Step 3: Reload asset to get updated book values after depreciation
        let asset = fixed_asset::Entity::find_by_id(asset_id)
            .one(db)
            .await?
            .ok_or(AdjustmentError::AssetNotFound)?;

        // Step 4: Calculate new values
        let accounting_value_before = asset.accounting_book_value;
        let tax_value_before = asset.tax_book_value;

        // For improvements, both accounting and tax values increase by the improvement amount
        let accounting_value_after = accounting_value_before + improvement_amount;
        let tax_value_after = tax_value_before + improvement_amount;

        // Also increase the acquisition cost
        let new_acquisition_cost = asset.acquisition_cost + improvement_amount;

        // Step 5: Apply new depreciation parameters if provided
        let final_accounting_rate = new_accounting_rate.unwrap_or(asset.accounting_depreciation_rate);
        let final_tax_rate = new_tax_rate.unwrap_or(asset.tax_depreciation_rate);
        let final_accounting_life = new_accounting_life.unwrap_or(asset.accounting_useful_life);
        let final_tax_life = new_tax_life.or(asset.tax_useful_life);

        // Step 6: Update asset with new values
        let asset_update = fixed_asset::ActiveModel {
            id: Set(asset_id),
            acquisition_cost: Set(new_acquisition_cost),
            accounting_book_value: Set(accounting_value_after),
            tax_book_value: Set(tax_value_after),
            accounting_depreciation_rate: Set(final_accounting_rate),
            tax_depreciation_rate: Set(final_tax_rate),
            accounting_useful_life: Set(final_accounting_life),
            tax_useful_life: Set(final_tax_life),
            ..Default::default()
        };

        fixed_asset::Entity::update(asset_update).exec(db).await?;

        // Step 7: Create adjustment record
        let adjustment = asset_value_adjustment::ActiveModel {
            fixed_asset_id: Set(asset_id),
            company_id: Set(asset.company_id),
            adjustment_date: Set(adjustment_date),
            adjustment_type: Set(AssetAdjustmentType::Improvement.as_str().to_string()),
            adjustment_amount: Set(improvement_amount),
            accounting_value_before: Set(accounting_value_before),
            accounting_value_after: Set(accounting_value_after),
            tax_value_before: Set(tax_value_before),
            tax_value_after: Set(tax_value_after),
            new_accounting_depreciation_rate: Set(new_accounting_rate),
            new_tax_depreciation_rate: Set(new_tax_rate),
            new_accounting_useful_life: Set(new_accounting_life),
            new_tax_useful_life: Set(new_tax_life),
            reason: Set(reason.clone()),
            document_number: Set(document_number.clone()),
            is_posted: Set(false), // Will be posted after journal entry creation
            created_by: Set(user_id),
            ..Default::default()
        };

        let adjustment_result = asset_value_adjustment::Entity::insert(adjustment)
            .exec(db)
            .await?;

        let adjustment_id = adjustment_result.last_insert_id;

        // Step 8: Create journal entry for the improvement
        let journal_entry_id = self
            .create_improvement_journal_entry(
                db,
                asset_id,
                asset.company_id,
                adjustment_date,
                improvement_amount,
                reason,
                document_number,
                user_id,
            )
            .await?;

        // Step 9: Mark adjustment as posted
        let adjustment_update = asset_value_adjustment::ActiveModel {
            id: Set(adjustment_id),
            is_posted: Set(true),
            journal_entry_id: Set(Some(journal_entry_id)),
            posted_at: Set(Some(chrono::Utc::now().naive_utc())),
            posted_by: Set(Some(user_id)),
            ..Default::default()
        };

        asset_value_adjustment::Entity::update(adjustment_update)
            .exec(db)
            .await?;

        Ok(AdjustmentResult {
            adjustment_id,
            journal_entry_id: Some(journal_entry_id),
            depreciation_periods_calculated: depreciation_periods,
            message: format!(
                "Successfully applied improvement of {} to asset",
                improvement_amount
            ),
        })
    }

    /// Apply an impairment (decrease in value) to a fixed asset
    pub async fn apply_impairment(
        &self,
        db: &DatabaseConnection,
        asset_id: i32,
        adjustment_date: NaiveDate,
        impairment_amount: Decimal,
        reason: String,
        document_number: Option<String>,
        user_id: i32,
    ) -> Result<AdjustmentResult, AdjustmentError> {
        // Similar logic to improvement but decreases value
        let asset = fixed_asset::Entity::find_by_id(asset_id)
            .one(db)
            .await?
            .ok_or(AdjustmentError::AssetNotFound)?;

        if asset.status != "active" {
            return Err(AdjustmentError::DatabaseError(
                "Asset must be active to apply impairment".to_string(),
            ));
        }

        if impairment_amount <= Decimal::ZERO {
            return Err(AdjustmentError::InvalidAmount);
        }

        // Calculate depreciation up to adjustment month
        let depreciation_periods = self
            .calculate_depreciation_up_to_date(db, &asset, adjustment_date)
            .await?;

        // Reload asset after depreciation
        let asset = fixed_asset::Entity::find_by_id(asset_id)
            .one(db)
            .await?
            .ok_or(AdjustmentError::AssetNotFound)?;

        let accounting_value_before = asset.accounting_book_value;
        let tax_value_before = asset.tax_book_value;

        // For impairment, reduce both values
        let accounting_value_after = (accounting_value_before - impairment_amount)
            .max(asset.accounting_salvage_value);
        let tax_value_after = (tax_value_before - impairment_amount).max(Decimal::ZERO);

        // Validate that values don't go negative
        if accounting_value_after < Decimal::ZERO || tax_value_after < Decimal::ZERO {
            return Err(AdjustmentError::InvalidAmount);
        }

        // Update asset
        let asset_update = fixed_asset::ActiveModel {
            id: Set(asset_id),
            accounting_book_value: Set(accounting_value_after),
            tax_book_value: Set(tax_value_after),
            ..Default::default()
        };

        fixed_asset::Entity::update(asset_update).exec(db).await?;

        // Create adjustment record
        let adjustment = asset_value_adjustment::ActiveModel {
            fixed_asset_id: Set(asset_id),
            company_id: Set(asset.company_id),
            adjustment_date: Set(adjustment_date),
            adjustment_type: Set(AssetAdjustmentType::Impairment.as_str().to_string()),
            adjustment_amount: Set(-impairment_amount), // Negative for impairment
            accounting_value_before: Set(accounting_value_before),
            accounting_value_after: Set(accounting_value_after),
            tax_value_before: Set(tax_value_before),
            tax_value_after: Set(tax_value_after),
            new_accounting_depreciation_rate: Set(None),
            new_tax_depreciation_rate: Set(None),
            new_accounting_useful_life: Set(None),
            new_tax_useful_life: Set(None),
            reason: Set(reason.clone()),
            document_number: Set(document_number.clone()),
            is_posted: Set(false),
            created_by: Set(user_id),
            ..Default::default()
        };

        let adjustment_result = asset_value_adjustment::Entity::insert(adjustment)
            .exec(db)
            .await?;

        let adjustment_id = adjustment_result.last_insert_id;

        // Create journal entry
        let journal_entry_id = self
            .create_impairment_journal_entry(
                db,
                asset_id,
                asset.company_id,
                adjustment_date,
                impairment_amount,
                reason,
                document_number,
                user_id,
            )
            .await?;

        // Mark as posted
        let adjustment_update = asset_value_adjustment::ActiveModel {
            id: Set(adjustment_id),
            is_posted: Set(true),
            journal_entry_id: Set(Some(journal_entry_id)),
            posted_at: Set(Some(chrono::Utc::now().naive_utc())),
            posted_by: Set(Some(user_id)),
            ..Default::default()
        };

        asset_value_adjustment::Entity::update(adjustment_update)
            .exec(db)
            .await?;

        Ok(AdjustmentResult {
            adjustment_id,
            journal_entry_id: Some(journal_entry_id),
            depreciation_periods_calculated: depreciation_periods,
            message: format!(
                "Successfully applied impairment of {} to asset",
                impairment_amount
            ),
        })
    }

    /// Calculate and save depreciation up to the specified date
    /// Returns list of periods that were calculated
    async fn calculate_depreciation_up_to_date(
        &self,
        db: &DatabaseConnection,
        asset: &fixed_asset::Model,
        target_date: NaiveDate,
    ) -> Result<Vec<NaiveDate>, AdjustmentError> {
        let depreciation_service = DepreciationService::new();

        // Get the starting date (put into service or acquisition)
        let start_date = asset
            .put_into_service_date
            .unwrap_or(asset.acquisition_date);

        // Get already calculated periods
        let calculated_periods = depreciation_service
            .get_calculated_periods(db, asset.id)
            .await
            .map_err(|e| AdjustmentError::DatabaseError(e.to_string()))?;

        // Find the last calculated period
        let last_calculated = calculated_periods.last().map(|d| d.period);

        // Determine which period to start from
        let start_period = if let Some(last) = last_calculated {
            // Calculate next period after last calculated
            let mut year = last.year();
            let mut month = last.month() + 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
            NaiveDate::from_ymd_opt(year, month, 1)
                .ok_or(AdjustmentError::DatabaseError("Invalid date".to_string()))?
        } else {
            // Start from the first month of service
            NaiveDate::from_ymd_opt(start_date.year(), start_date.month(), 1)
                .ok_or(AdjustmentError::DatabaseError("Invalid date".to_string()))?
        };

        // Calculate end period (month before adjustment)
        let target_period = NaiveDate::from_ymd_opt(target_date.year(), target_date.month(), 1)
            .ok_or(AdjustmentError::DatabaseError("Invalid date".to_string()))?;

        let mut calculated = Vec::new();
        let mut current_period = start_period;

        // Calculate depreciation for each month up to (but not including) adjustment month
        while current_period < target_period {
            match depreciation_service
                .calculate_monthly_depreciation(db, asset.id, current_period)
                .await
            {
                Ok(depreciation) => {
                    // Save the depreciation
                    depreciation_service
                        .save_depreciation(db, depreciation.clone(), asset.company_id)
                        .await
                        .map_err(|e| AdjustmentError::DatabaseError(e.to_string()))?;

                    calculated.push(current_period);
                }
                Err(e) => {
                    return Err(AdjustmentError::DepreciationNotSequential(e.to_string()));
                }
            }

            // Move to next month
            let mut year = current_period.year();
            let mut month = current_period.month() + 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
            current_period = NaiveDate::from_ymd_opt(year, month, 1)
                .ok_or(AdjustmentError::DatabaseError("Invalid date".to_string()))?;
        }

        Ok(calculated)
    }

    /// Create journal entry for asset improvement
    /// Debit: Asset account (e.g., 201, 204, etc.)
    /// Credit: Supplier or expense account (from category settings)
    async fn create_improvement_journal_entry(
        &self,
        db: &DatabaseConnection,
        asset_id: i32,
        company_id: i32,
        date: NaiveDate,
        amount: Decimal,
        reason: String,
        document_number: Option<String>,
        user_id: i32,
    ) -> Result<i32, AdjustmentError> {
        // Load asset and category
        let asset = fixed_asset::Entity::find_by_id(asset_id)
            .one(db)
            .await?
            .ok_or(AdjustmentError::AssetNotFound)?;

        let category = fixed_asset_category::Entity::find_by_id(asset.category_id)
            .one(db)
            .await?
            .ok_or(AdjustmentError::DatabaseError("Category not found".to_string()))?;

        // Get accounts
        let asset_account = account::Entity::find()
            .filter(account::Column::Code.eq(&category.asset_account_code))
            .filter(account::Column::CompanyId.eq(company_id))
            .one(db)
            .await?
            .ok_or(AdjustmentError::AccountNotFound(
                category.asset_account_code.clone(),
            ))?;

        // For credit, use the improvement credit account or default to supplier account
        let credit_account_code = category
            .improvement_credit_account_code
            .as_deref()
            .unwrap_or("401"); // Default to 401 (Suppliers)

        let credit_account = account::Entity::find()
            .filter(account::Column::Code.eq(credit_account_code))
            .filter(account::Column::CompanyId.eq(company_id))
            .one(db)
            .await?
            .ok_or(AdjustmentError::AccountNotFound(credit_account_code.to_string()))?;

        // Create journal entry
        let entry_number = format!(
            "IMP-{}-{}",
            date.format("%Y%m"),
            chrono::Utc::now().format("%H%M%S")
        );

        let doc_number = document_number.unwrap_or_else(|| "Увеличение на актив".to_string());

        let journal_entry = journal_entry::ActiveModel {
            entry_number: Set(entry_number),
            company_id: Set(company_id),
            document_date: Set(date),
            accounting_date: Set(date),
            document_number: Set(Some(doc_number)),
            description: Set(format!("Увеличение на {} - {}", asset.name, reason)),
            document_type: Set(Some("ASSET_IMPROVEMENT".to_string())),
            vat_document_type: Set(None),
            total_amount: Set(amount),
            total_vat_amount: Set(Decimal::ZERO),
            is_posted: Set(true),
            created_by: Set(user_id),
            ..Default::default()
        };

        let journal_result = journal_entry::Entity::insert(journal_entry)
            .exec(db)
            .await?;
        let journal_entry_id = journal_result.last_insert_id;

        // Debit asset account
        let debit_line = entry_line::ActiveModel {
            journal_entry_id: Set(journal_entry_id),
            account_id: Set(asset_account.id),
            debit_amount: Set(amount),
            credit_amount: Set(Decimal::ZERO),
            description: Set(Some(format!("Увеличение на {}", asset.name))),
            line_order: Set(1),
            base_amount: Set(amount),
            vat_amount: Set(Decimal::ZERO),
            ..Default::default()
        };

        entry_line::Entity::insert(debit_line).exec(db).await?;

        // Credit supplier/expense account
        let credit_line = entry_line::ActiveModel {
            journal_entry_id: Set(journal_entry_id),
            account_id: Set(credit_account.id),
            debit_amount: Set(Decimal::ZERO),
            credit_amount: Set(amount),
            description: Set(Some(format!("Увеличение на {}", asset.name))),
            line_order: Set(2),
            base_amount: Set(amount),
            vat_amount: Set(Decimal::ZERO),
            ..Default::default()
        };

        entry_line::Entity::insert(credit_line).exec(db).await?;

        Ok(journal_entry_id)
    }

    /// Create journal entry for asset impairment
    /// Debit: Impairment expense account
    /// Credit: Asset account or accumulated depreciation
    async fn create_impairment_journal_entry(
        &self,
        db: &DatabaseConnection,
        asset_id: i32,
        company_id: i32,
        date: NaiveDate,
        amount: Decimal,
        reason: String,
        document_number: Option<String>,
        user_id: i32,
    ) -> Result<i32, AdjustmentError> {
        let asset = fixed_asset::Entity::find_by_id(asset_id)
            .one(db)
            .await?
            .ok_or(AdjustmentError::AssetNotFound)?;

        let category = fixed_asset_category::Entity::find_by_id(asset.category_id)
            .one(db)
            .await?
            .ok_or(AdjustmentError::DatabaseError("Category not found".to_string()))?;

        // For impairment, typically debit an expense account and credit accumulated depreciation
        let debit_account_code = category
            .impairment_debit_account_code
            .as_deref()
            .unwrap_or("683"); // Default to 683 (Other operating expenses)

        let debit_account = account::Entity::find()
            .filter(account::Column::Code.eq(debit_account_code))
            .filter(account::Column::CompanyId.eq(company_id))
            .one(db)
            .await?
            .ok_or(AdjustmentError::AccountNotFound(debit_account_code.to_string()))?;

        // Credit accumulated depreciation account
        let credit_account = account::Entity::find()
            .filter(account::Column::Code.eq(&category.depreciation_account_code))
            .filter(account::Column::CompanyId.eq(company_id))
            .one(db)
            .await?
            .ok_or(AdjustmentError::AccountNotFound(
                category.depreciation_account_code.clone(),
            ))?;

        // Create journal entry
        let entry_number = format!(
            "IMP-{}-{}",
            date.format("%Y%m"),
            chrono::Utc::now().format("%H%M%S")
        );

        let doc_number = document_number.unwrap_or_else(|| "Обезценка на актив".to_string());

        let journal_entry = journal_entry::ActiveModel {
            entry_number: Set(entry_number),
            company_id: Set(company_id),
            document_date: Set(date),
            accounting_date: Set(date),
            document_number: Set(Some(doc_number)),
            description: Set(format!("Обезценка на {} - {}", asset.name, reason)),
            document_type: Set(Some("ASSET_IMPAIRMENT".to_string())),
            vat_document_type: Set(None),
            total_amount: Set(amount),
            total_vat_amount: Set(Decimal::ZERO),
            is_posted: Set(true),
            created_by: Set(user_id),
            ..Default::default()
        };

        let journal_result = journal_entry::Entity::insert(journal_entry)
            .exec(db)
            .await?;
        let journal_entry_id = journal_result.last_insert_id;

        // Debit expense account
        let debit_line = entry_line::ActiveModel {
            journal_entry_id: Set(journal_entry_id),
            account_id: Set(debit_account.id),
            debit_amount: Set(amount),
            credit_amount: Set(Decimal::ZERO),
            description: Set(Some(format!("Обезценка на {}", asset.name))),
            line_order: Set(1),
            base_amount: Set(amount),
            vat_amount: Set(Decimal::ZERO),
            ..Default::default()
        };

        entry_line::Entity::insert(debit_line).exec(db).await?;

        // Credit accumulated depreciation
        let credit_line = entry_line::ActiveModel {
            journal_entry_id: Set(journal_entry_id),
            account_id: Set(credit_account.id),
            debit_amount: Set(Decimal::ZERO),
            credit_amount: Set(amount),
            description: Set(Some(format!("Обезценка на {}", asset.name))),
            line_order: Set(2),
            base_amount: Set(amount),
            vat_amount: Set(Decimal::ZERO),
            ..Default::default()
        };

        entry_line::Entity::insert(credit_line).exec(db).await?;

        Ok(journal_entry_id)
    }

    /// Get all adjustments for a fixed asset
    pub async fn get_asset_adjustments(
        &self,
        db: &DatabaseConnection,
        asset_id: i32,
    ) -> Result<Vec<asset_value_adjustment::Model>, AdjustmentError> {
        let adjustments = asset_value_adjustment::Entity::find()
            .filter(asset_value_adjustment::Column::FixedAssetId.eq(asset_id))
            .order_by_asc(asset_value_adjustment::Column::AdjustmentDate)
            .all(db)
            .await?;

        Ok(adjustments)
    }

    /// Get all adjustments for a company within a date range
    pub async fn get_company_adjustments(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>,
    ) -> Result<Vec<asset_value_adjustment::Model>, AdjustmentError> {
        let mut query = asset_value_adjustment::Entity::find()
            .filter(asset_value_adjustment::Column::CompanyId.eq(company_id));

        if let Some(from) = from_date {
            query = query.filter(asset_value_adjustment::Column::AdjustmentDate.gte(from));
        }

        if let Some(to) = to_date {
            query = query.filter(asset_value_adjustment::Column::AdjustmentDate.lte(to));
        }

        let adjustments = query
            .order_by_desc(asset_value_adjustment::Column::AdjustmentDate)
            .all(db)
            .await?;

        Ok(adjustments)
    }
}
