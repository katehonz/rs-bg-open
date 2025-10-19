//! GraphQL Resolvers for Inventory Management

use async_graphql::{Context, FieldResult, InputObject, Object, SimpleObject};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::services::inventory_service::{
    CorrectionNeeded as ServiceCorrectionNeeded, InventoryService, QuantityTurnoverRow,
};

#[derive(Default)]
pub struct InventoryQuery;

#[derive(SimpleObject)]
pub struct AverageCostResult {
    pub account_id: i32,
    pub current_quantity: Decimal,
    pub current_amount: Decimal,
    pub average_cost: Decimal,
}

#[derive(SimpleObject)]
pub struct QuantityTurnover {
    pub account_id: i32,
    pub opening_quantity: Decimal,
    pub opening_amount: Decimal,
    pub receipt_quantity: Decimal,
    pub receipt_amount: Decimal,
    pub issue_quantity: Decimal,
    pub issue_amount: Decimal,
    pub closing_quantity: Decimal,
    pub closing_amount: Decimal,
}

#[derive(SimpleObject)]
pub struct CorrectionNeeded {
    pub movement_id: i32,
    pub movement_date: NaiveDate,
    pub material_account_id: i32,
    pub expense_account_id: i32,
    pub quantity: Decimal,
    pub old_average_cost: Decimal,
    pub new_average_cost: Decimal,
    pub correction_amount: Decimal,
    pub description: String,
}

impl From<QuantityTurnoverRow> for QuantityTurnover {
    fn from(row: QuantityTurnoverRow) -> Self {
        Self {
            account_id: row.account_id,
            opening_quantity: row.opening_quantity,
            opening_amount: row.opening_amount,
            receipt_quantity: row.receipt_quantity,
            receipt_amount: row.receipt_amount,
            issue_quantity: row.issue_quantity,
            issue_amount: row.issue_amount,
            closing_quantity: row.closing_quantity,
            closing_amount: row.closing_amount,
        }
    }
}

impl From<ServiceCorrectionNeeded> for CorrectionNeeded {
    fn from(correction: ServiceCorrectionNeeded) -> Self {
        Self {
            movement_id: correction.movement_id,
            movement_date: correction.movement_date,
            material_account_id: correction.material_account_id,
            expense_account_id: correction.expense_account_id,
            quantity: correction.quantity,
            old_average_cost: correction.old_average_cost,
            new_average_cost: correction.new_average_cost,
            correction_amount: correction.correction_amount,
            description: correction.description,
        }
    }
}

#[Object]
impl InventoryQuery {
    /// Get average cost for a material account
    async fn get_average_cost(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        account_id: i32,
        as_of_date: Option<NaiveDate>,
    ) -> FieldResult<AverageCostResult> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let service = InventoryService::new();

        let info = service
            .get_average_cost(db.as_ref(), company_id, account_id, as_of_date)
            .await?;

        Ok(AverageCostResult {
            account_id: info.account_id,
            current_quantity: info.current_quantity,
            current_amount: info.current_amount,
            average_cost: info.average_cost,
        })
    }

    /// Get quantity turnover statement
    async fn get_quantity_turnover(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> FieldResult<Vec<QuantityTurnover>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let service = InventoryService::new();

        let rows = service
            .get_quantity_turnover(db.as_ref(), company_id, from_date, to_date)
            .await?;

        Ok(rows.into_iter().map(QuantityTurnover::from).collect())
    }

    /// Check for needed corrections when adding retroactive entry
    async fn check_retroactive_corrections(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        account_id: i32,
        new_entry_date: NaiveDate,
    ) -> FieldResult<Vec<CorrectionNeeded>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let service = InventoryService::new();

        let corrections = service
            .check_retroactive_corrections(db.as_ref(), company_id, account_id, new_entry_date)
            .await?;

        Ok(corrections.into_iter().map(CorrectionNeeded::from).collect())
    }
}

#[derive(Default)]
pub struct InventoryMutation;

#[derive(InputObject)]
pub struct ProcessEntryLineInput {
    pub entry_line_id: i32,
}

#[derive(SimpleObject)]
pub struct ProcessEntryLineResult {
    pub success: bool,
    pub movement_id: Option<i32>,
    pub message: String,
}

#[Object]
impl InventoryMutation {
    /// Process entry line to create inventory movement
    async fn process_entry_line(
        &self,
        ctx: &Context<'_>,
        input: ProcessEntryLineInput,
    ) -> FieldResult<ProcessEntryLineResult> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let service = InventoryService::new();

        match service
            .process_entry_line(db.as_ref(), input.entry_line_id)
            .await
        {
            Ok(Some(movement)) => Ok(ProcessEntryLineResult {
                success: true,
                movement_id: Some(movement.id),
                message: "Inventory movement created successfully".to_string(),
            }),
            Ok(None) => Ok(ProcessEntryLineResult {
                success: true,
                movement_id: None,
                message: "No movement created (no quantity or already processed)".to_string(),
            }),
            Err(e) => Ok(ProcessEntryLineResult {
                success: false,
                movement_id: None,
                message: format!("Error: {}", e),
            }),
        }
    }
}
