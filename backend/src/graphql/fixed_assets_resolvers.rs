//! GraphQL Resolvers for Fixed Assets Management
//!
//! Handles CRUD operations for fixed assets, categories, and depreciation

use async_graphql::{Context, FieldResult, InputObject, Object, SimpleObject};
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use std::sync::Arc;

use crate::entities::{
    depreciation_journal, fixed_asset, fixed_asset_category, DepreciationJournal, FixedAsset,
    FixedAssetCategory,
};
use crate::services::depreciation_service::{DepreciationService, MonthlyDepreciation};

// Input Types
#[derive(InputObject)]
pub struct CreateFixedAssetInput {
    pub inventory_number: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: i32,
    pub acquisition_cost: Decimal,
    pub acquisition_date: NaiveDate,
    pub put_into_service_date: Option<NaiveDate>,
    pub accounting_useful_life: i32,
    pub accounting_depreciation_rate: Decimal,
    pub accounting_depreciation_method: Option<String>,
    pub accounting_salvage_value: Option<Decimal>,
    pub tax_useful_life: Option<i32>,
    pub tax_depreciation_rate: Decimal,
    pub is_new_first_time_investment: Option<bool>,
    pub location: Option<String>,
    pub responsible_person: Option<String>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub notes: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateFixedAssetInput {
    pub id: i32,
    pub inventory_number: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub responsible_person: Option<String>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub notes: Option<String>,
    pub status: Option<String>,
    pub disposal_date: Option<NaiveDate>,
    pub disposal_amount: Option<Decimal>,
}

#[derive(InputObject)]
pub struct CreateFixedAssetCategoryInput {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub tax_category: i32,
    pub max_tax_depreciation_rate: Decimal,
    pub default_accounting_depreciation_rate: Option<Decimal>,
    pub min_useful_life: Option<i32>,
    pub max_useful_life: Option<i32>,
    pub asset_account_code: String,
    pub depreciation_account_code: String,
    pub expense_account_code: String,
}

#[derive(InputObject)]
pub struct UpdateAssetCategoryInput {
    pub asset_account_code: Option<String>,
    pub depreciation_account_code: Option<String>,
    pub expense_account_code: Option<String>,
    pub default_accounting_depreciation_rate: Option<Decimal>,
}

#[derive(InputObject)]
pub struct CalculateDepreciationInput {
    pub company_id: i32,
    pub year: i32,
    pub month: u32,
    pub asset_ids: Option<Vec<i32>>, // If None, calculate for all assets
}

#[derive(InputObject)]
pub struct PostDepreciationInput {
    pub company_id: i32,
    pub year: i32,
    pub month: u32,
    pub reference: Option<String>,
}

// Output Types
#[derive(SimpleObject)]
pub struct FixedAssetWithCategory {
    #[graphql(flatten)]
    pub asset: fixed_asset::Model,
    #[graphql(name = "category")]
    pub category: Option<fixed_asset_category::Model>,
}

#[derive(SimpleObject)]
pub struct DepreciationCalculationResult {
    pub success: bool,
    pub calculated_count: i32,
    pub error_count: i32,
    pub total_accounting_amount: Decimal,
    pub total_tax_amount: Decimal,
    pub errors: Vec<String>,
}

#[derive(SimpleObject)]
pub struct DepreciationPostingResult {
    pub success: bool,
    pub journal_entry_id: i32,
    pub total_amount: Decimal,
    pub assets_count: i32,
    pub message: String,
}

#[derive(SimpleObject)]
pub struct FixedAssetSummary {
    pub total_assets: i32,
    pub total_acquisition_cost: Decimal,
    pub total_accounting_book_value: Decimal,
    pub total_tax_book_value: Decimal,
    pub total_accumulated_depreciation: Decimal,
    pub active_assets: i32,
    pub disposed_assets: i32,
}

#[derive(SimpleObject)]
pub struct CalculatedPeriod {
    pub year: i32,
    pub month: u32,
    pub period_display: String,
    pub is_posted: bool,
}

#[derive(SimpleObject)]
pub struct AssetDepreciationStatus {
    pub asset_id: i32,
    pub asset_name: String,
    pub calculated_periods: Vec<CalculatedPeriod>,
    pub next_period_year: Option<i32>,
    pub next_period_month: Option<u32>,
}

// Query Resolvers
#[derive(Default)]
pub struct FixedAssetsQuery;

#[Object]
impl FixedAssetsQuery {
    /// Get fixed asset by ID
    async fn fixed_asset(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> FieldResult<Option<FixedAssetWithCategory>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        let asset = FixedAsset::find_by_id(id).one(db.as_ref()).await?;

        if let Some(asset) = asset {
            let category = FixedAssetCategory::find_by_id(asset.category_id)
                .one(db.as_ref())
                .await?;

            Ok(Some(FixedAssetWithCategory { asset, category }))
        } else {
            Ok(None)
        }
    }

    /// Get all fixed assets for a company
    async fn fixed_assets(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        status: Option<String>,
        category_id: Option<i32>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> FieldResult<Vec<FixedAssetWithCategory>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        let mut query = FixedAsset::find().filter(fixed_asset::Column::CompanyId.eq(company_id));

        if let Some(status) = status {
            query = query.filter(fixed_asset::Column::Status.eq(status));
        }

        if let Some(category_id) = category_id {
            query = query.filter(fixed_asset::Column::CategoryId.eq(category_id));
        }

        query = query.order_by_asc(fixed_asset::Column::Name);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let assets = query.all(db.as_ref()).await?;

        let mut result = Vec::new();
        for asset in assets {
            let category = FixedAssetCategory::find_by_id(asset.category_id)
                .one(db.as_ref())
                .await?;
            result.push(FixedAssetWithCategory { asset, category });
        }

        Ok(result)
    }

    /// Get fixed asset categories
    async fn fixed_asset_categories(
        &self,
        ctx: &Context<'_>,
    ) -> FieldResult<Vec<fixed_asset_category::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        let categories = FixedAssetCategory::find()
            .filter(fixed_asset_category::Column::IsActive.eq(true))
            .order_by_asc(fixed_asset_category::Column::TaxCategory)
            .order_by_asc(fixed_asset_category::Column::Name)
            .all(db.as_ref())
            .await?;

        Ok(categories)
    }

    /// Get depreciation journal for a period
    async fn depreciation_journal(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        year: i32,
        month: Option<u32>,
    ) -> FieldResult<Vec<depreciation_journal::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        let mut query = DepreciationJournal::find()
            .filter(depreciation_journal::Column::CompanyId.eq(company_id));

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

        let entries = query
            .order_by_desc(depreciation_journal::Column::Period)
            .order_by_asc(depreciation_journal::Column::FixedAssetId)
            .all(db.as_ref())
            .await?;

        Ok(entries)
    }

    /// Get fixed assets summary for a company
    async fn fixed_assets_summary(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
    ) -> FieldResult<FixedAssetSummary> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        let assets = FixedAsset::find()
            .filter(fixed_asset::Column::CompanyId.eq(company_id))
            .all(db.as_ref())
            .await?;

        let total_assets = assets.len() as i32;
        let mut total_acquisition_cost = Decimal::from(0);
        let mut total_accounting_book_value = Decimal::from(0);
        let mut total_tax_book_value = Decimal::from(0);
        let mut total_accumulated_depreciation = Decimal::from(0);
        let mut active_assets = 0;
        let mut disposed_assets = 0;

        for asset in assets {
            total_acquisition_cost += asset.acquisition_cost;
            total_accounting_book_value += asset.accounting_book_value;
            total_tax_book_value += asset.tax_book_value;
            total_accumulated_depreciation += asset.accounting_accumulated_depreciation;

            match asset.status.as_str() {
                "active" => active_assets += 1,
                "disposed" | "sold" => disposed_assets += 1,
                _ => {}
            }
        }

        Ok(FixedAssetSummary {
            total_assets,
            total_acquisition_cost,
            total_accounting_book_value,
            total_tax_book_value,
            total_accumulated_depreciation,
            active_assets,
            disposed_assets,
        })
    }

    /// Get calculated and posted periods for a company
    async fn company_calculated_periods(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
    ) -> FieldResult<Vec<CalculatedPeriod>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        let periods = DepreciationJournal::find()
            .filter(depreciation_journal::Column::CompanyId.eq(company_id))
            .select_only()
            .column(depreciation_journal::Column::Period)
            .column(depreciation_journal::Column::IsPosted)
            .distinct()
            .all(db.as_ref())
            .await?;

        let month_names = [
            "", "Януари", "Февруари", "Март", "Април", "Май", "Юни",
            "Юли", "Август", "Септември", "Октомври", "Ноември", "Декември",
        ];

        let mut result: Vec<CalculatedPeriod> = periods
            .iter()
            .map(|p| {
                let year = p.period.year();
                let month = p.period.month();
                CalculatedPeriod {
                    year,
                    month,
                    period_display: format!("{} {}", month_names[month as usize], year),
                    is_posted: p.is_posted,
                }
            })
            .collect();

        // Sort by year and month
        result.sort_by(|a, b| {
            match a.year.cmp(&b.year) {
                std::cmp::Ordering::Equal => a.month.cmp(&b.month),
                other => other,
            }
        });

        Ok(result)
    }

    /// Get depreciation status for an asset
    async fn asset_depreciation_status(
        &self,
        ctx: &Context<'_>,
        asset_id: i32,
    ) -> FieldResult<AssetDepreciationStatus> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let service = ctx.data::<Arc<DepreciationService>>()?;

        let asset = FixedAsset::find_by_id(asset_id)
            .one(db.as_ref())
            .await?
            .ok_or("Asset not found")?;

        let calculated = service.get_calculated_periods(db.as_ref(), asset_id).await?;

        let month_names = [
            "", "Януари", "Февруари", "Март", "Април", "Май", "Юни",
            "Юли", "Август", "Септември", "Октомври", "Ноември", "Декември",
        ];

        let calculated_periods: Vec<CalculatedPeriod> = calculated
            .iter()
            .map(|p| {
                let year = p.period.year();
                let month = p.period.month();
                CalculatedPeriod {
                    year,
                    month,
                    period_display: format!("{} {}", month_names[month as usize], year),
                    is_posted: p.is_posted,
                }
            })
            .collect();

        // Calculate next period
        let (next_year, next_month) = if let Some(last) = calculated.last() {
            let mut year = last.period.year();
            let mut month = last.period.month() + 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
            (Some(year), Some(month))
        } else {
            // If no periods calculated, start from put-into-service date
            if let Some(service_date) = asset.put_into_service_date {
                (Some(service_date.year()), Some(service_date.month()))
            } else {
                (Some(asset.acquisition_date.year()), Some(asset.acquisition_date.month()))
            }
        };

        Ok(AssetDepreciationStatus {
            asset_id,
            asset_name: asset.name,
            calculated_periods,
            next_period_year: next_year,
            next_period_month: next_month,
        })
    }
}

// Mutation Resolvers
#[derive(Default)]
pub struct FixedAssetsMutation;

#[Object]
impl FixedAssetsMutation {
    /// Create a new fixed asset
    async fn create_fixed_asset(
        &self,
        ctx: &Context<'_>,
        input: CreateFixedAssetInput,
        company_id: i32,
    ) -> FieldResult<fixed_asset::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        // Calculate initial book values
        let accounting_book_value = input.acquisition_cost;
        let tax_book_value = input.acquisition_cost;

        let asset = fixed_asset::ActiveModel {
            inventory_number: Set(input.inventory_number),
            name: Set(input.name),
            description: Set(input.description),
            category_id: Set(input.category_id),
            company_id: Set(company_id),
            acquisition_cost: Set(input.acquisition_cost),
            acquisition_date: Set(input.acquisition_date),
            put_into_service_date: Set(input.put_into_service_date),
            accounting_useful_life: Set(input.accounting_useful_life),
            accounting_depreciation_rate: Set(input.accounting_depreciation_rate),
            accounting_depreciation_method: Set(input
                .accounting_depreciation_method
                .unwrap_or_else(|| "straight_line".to_string())),
            accounting_salvage_value: Set(input
                .accounting_salvage_value
                .unwrap_or_else(|| Decimal::from(0))),
            accounting_accumulated_depreciation: Set(Decimal::from(0)),
            tax_useful_life: Set(input.tax_useful_life),
            tax_depreciation_rate: Set(input.tax_depreciation_rate),
            tax_accumulated_depreciation: Set(Decimal::from(0)),
            is_new_first_time_investment: Set(input.is_new_first_time_investment.unwrap_or(false)),
            accounting_book_value: Set(accounting_book_value),
            tax_book_value: Set(tax_book_value),
            status: Set("active".to_string()),
            location: Set(input.location),
            responsible_person: Set(input.responsible_person),
            serial_number: Set(input.serial_number),
            manufacturer: Set(input.manufacturer),
            model: Set(input.model),
            notes: Set(input.notes),
            ..Default::default()
        };

        let result = asset.insert(db.as_ref()).await?;

        Ok(result)
    }

    /// Update a fixed asset
    async fn update_fixed_asset(
        &self,
        ctx: &Context<'_>,
        input: UpdateFixedAssetInput,
    ) -> FieldResult<fixed_asset::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        let mut asset = fixed_asset::ActiveModel {
            id: Set(input.id),
            ..Default::default()
        };

        if let Some(inventory_number) = input.inventory_number {
            asset.inventory_number = Set(inventory_number);
        }
        if let Some(name) = input.name {
            asset.name = Set(name);
        }
        if let Some(description) = input.description {
            asset.description = Set(Some(description));
        }
        if let Some(location) = input.location {
            asset.location = Set(Some(location));
        }
        if let Some(responsible_person) = input.responsible_person {
            asset.responsible_person = Set(Some(responsible_person));
        }
        if let Some(serial_number) = input.serial_number {
            asset.serial_number = Set(Some(serial_number));
        }
        if let Some(manufacturer) = input.manufacturer {
            asset.manufacturer = Set(Some(manufacturer));
        }
        if let Some(model) = input.model {
            asset.model = Set(Some(model));
        }
        if let Some(notes) = input.notes {
            asset.notes = Set(Some(notes));
        }
        if let Some(status) = input.status {
            asset.status = Set(status);
        }
        if let Some(disposal_date) = input.disposal_date {
            asset.disposal_date = Set(Some(disposal_date));
        }
        if let Some(disposal_amount) = input.disposal_amount {
            asset.disposal_amount = Set(Some(disposal_amount));
        }

        let result = asset.update(db.as_ref()).await?;

        Ok(result)
    }

    /// Create a fixed asset category
    async fn create_fixed_asset_category(
        &self,
        ctx: &Context<'_>,
        input: CreateFixedAssetCategoryInput,
    ) -> FieldResult<fixed_asset_category::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        let category = fixed_asset_category::ActiveModel {
            code: Set(input.code),
            name: Set(input.name),
            description: Set(input.description),
            tax_category: Set(input.tax_category),
            max_tax_depreciation_rate: Set(input.max_tax_depreciation_rate),
            default_accounting_depreciation_rate: Set(input.default_accounting_depreciation_rate),
            min_useful_life: Set(input.min_useful_life),
            max_useful_life: Set(input.max_useful_life),
            asset_account_code: Set(input.asset_account_code),
            depreciation_account_code: Set(input.depreciation_account_code),
            expense_account_code: Set(input.expense_account_code),
            is_active: Set(true),
            ..Default::default()
        };

        let result = category.insert(db.as_ref()).await?;

        Ok(result)
    }

    /// Update a fixed asset category
    async fn update_fixed_asset_category(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateAssetCategoryInput,
    ) -> FieldResult<fixed_asset_category::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        let mut category = fixed_asset_category::ActiveModel {
            id: Set(id),
            ..Default::default()
        };

        if let Some(asset_account_code) = input.asset_account_code {
            category.asset_account_code = Set(asset_account_code);
        }
        if let Some(depreciation_account_code) = input.depreciation_account_code {
            category.depreciation_account_code = Set(depreciation_account_code);
        }
        if let Some(expense_account_code) = input.expense_account_code {
            category.expense_account_code = Set(expense_account_code);
        }
        if let Some(rate) = input.default_accounting_depreciation_rate {
            category.default_accounting_depreciation_rate = Set(Some(rate));
        }

        let result = category.update(db.as_ref()).await?;

        Ok(result)
    }

    /// Calculate monthly depreciation for assets
    async fn calculate_depreciation(
        &self,
        ctx: &Context<'_>,
        input: CalculateDepreciationInput,
    ) -> FieldResult<DepreciationCalculationResult> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let service = DepreciationService::new();

        let period =
            NaiveDate::from_ymd_opt(input.year, input.month, 1).ok_or("Invalid year/month")?;

        let bulk_result = service
            .calculate_bulk_depreciation(db.as_ref(), input.company_id, period)
            .await?;

        // Save calculated depreciation
        let mut saved_count = 0;
        for depreciation in bulk_result.calculated {
            match service
                .save_depreciation(db.as_ref(), depreciation, input.company_id)
                .await
            {
                Ok(_) => saved_count += 1,
                Err(e) => eprintln!("Error saving depreciation: {}", e),
            }
        }

        Ok(DepreciationCalculationResult {
            success: bulk_result.errors.is_empty(),
            calculated_count: saved_count,
            error_count: bulk_result.errors.len() as i32,
            total_accounting_amount: bulk_result.total_accounting_amount,
            total_tax_amount: bulk_result.total_tax_amount,
            errors: bulk_result
                .errors
                .into_iter()
                .map(|e| e.error_message)
                .collect(),
        })
    }

    /// Post calculated depreciation to journal
    async fn post_depreciation(
        &self,
        ctx: &Context<'_>,
        input: PostDepreciationInput,
        user_id: i32,
    ) -> FieldResult<DepreciationPostingResult> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let service = DepreciationService::new();

        let period =
            NaiveDate::from_ymd_opt(input.year, input.month, 1).ok_or("Invalid year/month")?;

        let result = service
            .create_depreciation_journal_entry(
                db.as_ref(),
                input.company_id,
                period,
                user_id,
                input.reference,
            )
            .await?;

        Ok(DepreciationPostingResult {
            success: true,
            journal_entry_id: result.journal_entry_id,
            total_amount: result.total_amount,
            assets_count: result.assets_count,
            message: format!(
                "Successfully posted depreciation for {} assets",
                result.assets_count
            ),
        })
    }

    /// Delete a fixed asset
    async fn delete_fixed_asset(&self, ctx: &Context<'_>, id: i32) -> FieldResult<bool> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        // Check if asset has depreciation entries
        let depreciation_count = DepreciationJournal::find()
            .filter(depreciation_journal::Column::FixedAssetId.eq(id))
            .count(db.as_ref())
            .await?;

        if depreciation_count > 0 {
            return Err(
                "Cannot delete asset with depreciation history. Set status to 'disposed' instead."
                    .into(),
            );
        }

        let result = FixedAsset::delete_by_id(id).exec(db.as_ref()).await?;

        Ok(result.rows_affected > 0)
    }
}
