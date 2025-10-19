//! Inventory Service
//!
//! Manages inventory movements and average cost calculations

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::*;
use std::collections::HashMap;

use crate::entities::{
    average_cost_correction, entry_line, inventory_balance, inventory_movement, journal_entry,
    AverageCostCorrection, InventoryBalance, InventoryMovement,
};
use sea_orm::sea_query::Expr;

pub struct InventoryService;

#[derive(Debug, Clone)]
pub struct AverageCostInfo {
    pub account_id: i32,
    pub current_quantity: Decimal,
    pub current_amount: Decimal,
    pub average_cost: Decimal,
}

impl InventoryService {
    pub fn new() -> Self {
        Self
    }

    /// Calculate current average cost for a material account
    pub async fn get_average_cost(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        account_id: i32,
        as_of_date: Option<NaiveDate>,
    ) -> Result<AverageCostInfo, DbErr> {
        // Get or create balance record
        let balance = self
            .get_or_create_balance(db, company_id, account_id)
            .await?;

        // If as_of_date is specified, recalculate from movements up to that date
        if let Some(date) = as_of_date {
            self.calculate_average_cost_at_date(db, company_id, account_id, date)
                .await
        } else {
            Ok(AverageCostInfo {
                account_id,
                current_quantity: balance.current_quantity,
                current_amount: balance.current_amount,
                average_cost: balance.current_average_cost,
            })
        }
    }

    /// Process an entry line and create inventory movement
    pub async fn process_entry_line(
        &self,
        db: &DatabaseConnection,
        entry_line_id: i32,
    ) -> Result<Option<inventory_movement::Model>, DbErr> {
        let line = entry_line::Entity::find_by_id(entry_line_id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                "Entry line not found".to_string(),
            ))?;

        // Skip if no quantity
        if line.quantity.is_none() || line.quantity.unwrap() == Decimal::ZERO {
            return Ok(None);
        }

        // Check if already processed
        let existing = InventoryMovement::find()
            .filter(inventory_movement::Column::EntryLineId.eq(entry_line_id))
            .one(db)
            .await?;

        if existing.is_some() {
            return Ok(existing);
        }

        // Get journal entry for company_id and date
        let journal = journal_entry::Entity::find_by_id(line.journal_entry_id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Journal entry not found".to_string()))?;

        let quantity = line.quantity.unwrap();
        let movement_type = if line.debit_amount > Decimal::ZERO {
            "DEBIT"
        } else {
            "CREDIT"
        };
        let amount = line.debit_amount.max(line.credit_amount);
        let unit_price = if quantity > Decimal::ZERO {
            amount / quantity
        } else {
            Decimal::ZERO
        };

        // Get current balance
        let balance = self
            .get_or_create_balance(db, journal.company_id, line.account_id)
            .await?;

        // Calculate new balance
        let (new_quantity, new_amount, new_avg_cost) = if movement_type == "DEBIT" {
            // Receipt
            let new_qty = balance.current_quantity + quantity;
            let new_amt = balance.current_amount + amount;
            let new_avg = if new_qty > Decimal::ZERO {
                new_amt / new_qty
            } else {
                Decimal::ZERO
            };
            (new_qty, new_amt, new_avg)
        } else {
            // Issue - use average cost
            let issue_value = balance.current_average_cost * quantity;
            (
                balance.current_quantity - quantity,
                balance.current_amount - issue_value,
                balance.current_average_cost,
            )
        };

        // Create movement
        let movement = inventory_movement::ActiveModel {
            company_id: Set(journal.company_id),
            account_id: Set(line.account_id),
            entry_line_id: Set(entry_line_id),
            journal_entry_id: Set(line.journal_entry_id),
            movement_date: Set(journal.accounting_date),
            movement_type: Set(movement_type.to_string()),
            quantity: Set(quantity),
            unit_price: Set(unit_price),
            total_amount: Set(amount),
            unit_of_measure: Set(line.unit_of_measure_code.clone()),
            description: Set(line.description.clone()),
            balance_after_quantity: Set(new_quantity),
            balance_after_amount: Set(new_amount),
            average_cost_at_time: Set(new_avg_cost),
            ..Default::default()
        };

        let movement_record = movement.insert(db).await?;

        // Update balance
        let mut balance_active: inventory_balance::ActiveModel = balance.into();
        balance_active.current_quantity = Set(new_quantity);
        balance_active.current_amount = Set(new_amount);
        balance_active.current_average_cost = Set(new_avg_cost);
        balance_active.last_movement_date = Set(Some(journal.accounting_date));
        balance_active.last_movement_id = Set(Some(movement_record.id));
        balance_active.update(db).await?;

        Ok(Some(movement_record))
    }

    /// Get or create inventory balance for an account
    async fn get_or_create_balance(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        account_id: i32,
    ) -> Result<inventory_balance::Model, DbErr> {
        match InventoryBalance::find()
            .filter(inventory_balance::Column::CompanyId.eq(company_id))
            .filter(inventory_balance::Column::AccountId.eq(account_id))
            .one(db)
            .await?
        {
            Some(balance) => Ok(balance),
            None => {
                let new_balance = inventory_balance::ActiveModel {
                    company_id: Set(company_id),
                    account_id: Set(account_id),
                    current_quantity: Set(Decimal::ZERO),
                    current_amount: Set(Decimal::ZERO),
                    current_average_cost: Set(Decimal::ZERO),
                    ..Default::default()
                };
                new_balance.insert(db).await
            }
        }
    }

    /// Calculate average cost at a specific date
    async fn calculate_average_cost_at_date(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        account_id: i32,
        as_of_date: NaiveDate,
    ) -> Result<AverageCostInfo, DbErr> {
        let movements = InventoryMovement::find()
            .filter(inventory_movement::Column::CompanyId.eq(company_id))
            .filter(inventory_movement::Column::AccountId.eq(account_id))
            .filter(inventory_movement::Column::MovementDate.lte(as_of_date))
            .order_by_asc(inventory_movement::Column::MovementDate)
            .order_by_asc(inventory_movement::Column::Id)
            .all(db)
            .await?;

        let mut quantity = Decimal::ZERO;
        let mut amount = Decimal::ZERO;

        for movement in movements {
            if movement.movement_type == "DEBIT" {
                quantity += movement.quantity;
                amount += movement.total_amount;
            } else {
                let avg_cost = if quantity > Decimal::ZERO {
                    amount / quantity
                } else {
                    Decimal::ZERO
                };
                quantity -= movement.quantity;
                amount -= avg_cost * movement.quantity;
            }
        }

        let average_cost = if quantity > Decimal::ZERO {
            amount / quantity
        } else {
            Decimal::ZERO
        };

        Ok(AverageCostInfo {
            account_id,
            current_quantity: quantity,
            current_amount: amount,
            average_cost,
        })
    }

    /// Get quantity turnover statement for a period
    pub async fn get_quantity_turnover(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> Result<Vec<QuantityTurnoverRow>, DbErr> {
        // This would be a complex SQL query - simplified version here
        let movements = InventoryMovement::find()
            .filter(inventory_movement::Column::CompanyId.eq(company_id))
            .filter(inventory_movement::Column::MovementDate.gte(from_date))
            .filter(inventory_movement::Column::MovementDate.lte(to_date))
            .order_by_asc(inventory_movement::Column::AccountId)
            .order_by_asc(inventory_movement::Column::MovementDate)
            .all(db)
            .await?;

        let mut rows: HashMap<i32, QuantityTurnoverRow> = HashMap::new();

        for movement in movements {
            let row = rows.entry(movement.account_id).or_insert(QuantityTurnoverRow {
                account_id: movement.account_id,
                opening_quantity: Decimal::ZERO,
                opening_amount: Decimal::ZERO,
                receipt_quantity: Decimal::ZERO,
                receipt_amount: Decimal::ZERO,
                issue_quantity: Decimal::ZERO,
                issue_amount: Decimal::ZERO,
                closing_quantity: Decimal::ZERO,
                closing_amount: Decimal::ZERO,
            });

            if movement.movement_type == "DEBIT" {
                row.receipt_quantity += movement.quantity;
                row.receipt_amount += movement.total_amount;
            } else {
                row.issue_quantity += movement.quantity;
                row.issue_amount += movement.total_amount;
            }
        }

        Ok(rows.into_values().collect())
    }

    /// Check for corrections needed when adding retroactive entry
    /// Returns list of affected movements with their corresponding debit accounts
    pub async fn check_retroactive_corrections(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        account_id: i32,
        new_entry_date: NaiveDate,
    ) -> Result<Vec<CorrectionNeeded>, DbErr> {
        // Find all CREDIT movements after the new entry date
        let affected_movements = InventoryMovement::find()
            .filter(inventory_movement::Column::CompanyId.eq(company_id))
            .filter(inventory_movement::Column::AccountId.eq(account_id))
            .filter(inventory_movement::Column::MovementType.eq("CREDIT"))
            .filter(inventory_movement::Column::MovementDate.gt(new_entry_date))
            .order_by_asc(inventory_movement::Column::MovementDate)
            .all(db)
            .await?;

        let mut corrections = Vec::new();

        for movement in affected_movements {
            // Find the corresponding journal entry and its lines
            let entry_lines = entry_line::Entity::find()
                .filter(entry_line::Column::JournalEntryId.eq(movement.journal_entry_id))
                .all(db)
                .await?;

            // Find the debit account in the same journal entry
            // The credit is on our material account, debit should be expense account
            let debit_account_id = entry_lines
                .iter()
                .find(|line| {
                    line.account_id != account_id
                        && line.debit_amount > Decimal::ZERO
                        && line.debit_amount == movement.total_amount
                })
                .map(|line| line.account_id);

            if let Some(debit_acc_id) = debit_account_id {
                // Calculate what the new average cost should be at this point
                let new_avg_cost = self
                    .calculate_average_cost_at_date(db, company_id, account_id, movement.movement_date)
                    .await?
                    .average_cost;

                let old_avg_cost = movement.average_cost_at_time;
                let quantity = movement.quantity;

                let correction_amount = (new_avg_cost - old_avg_cost) * quantity;

                if correction_amount.abs() > Decimal::new(1, 2) {
                    // Only create correction if difference > 0.01
                    corrections.push(CorrectionNeeded {
                        movement_id: movement.id,
                        movement_date: movement.movement_date,
                        material_account_id: account_id,
                        expense_account_id: debit_acc_id,
                        quantity,
                        old_average_cost: old_avg_cost,
                        new_average_cost: new_avg_cost,
                        correction_amount,
                        description: format!(
                            "Корекция СПЦ за {} бр от {:.2} на {:.2} лв",
                            quantity, old_avg_cost, new_avg_cost
                        ),
                    });
                }
            }
        }

        Ok(corrections)
    }

    /// Generate correction journal entry
    pub async fn generate_correction_entry(
        &self,
        _db: &DatabaseConnection,
        _company_id: i32,
        corrections: Vec<CorrectionNeeded>,
        _accounting_date: NaiveDate,
    ) -> Result<Vec<CorrectionEntryLine>, DbErr> {
        let mut entry_lines = Vec::new();

        for correction in corrections {
            let is_positive = correction.correction_amount > Decimal::ZERO;

            // If positive: Debit expense, Credit material (additional expense)
            // If negative: Debit expense (-), Credit material (-) (storno)
            entry_lines.push(CorrectionEntryLine {
                material_account_id: correction.material_account_id,
                expense_account_id: correction.expense_account_id,
                debit_amount: if is_positive { Decimal::ZERO } else { correction.correction_amount.abs() },
                credit_amount: if is_positive { correction.correction_amount } else { Decimal::ZERO },
                quantity: correction.quantity,
                description: correction.description.clone(),
                is_storno: !is_positive,
                movement_id: correction.movement_id,
            });
        }

        Ok(entry_lines)
    }
}

#[derive(Debug, Clone)]
pub struct QuantityTurnoverRow {
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct CorrectionEntryLine {
    pub material_account_id: i32,
    pub expense_account_id: i32,
    pub debit_amount: Decimal,
    pub credit_amount: Decimal,
    pub quantity: Decimal,
    pub description: String,
    pub is_storno: bool,
    pub movement_id: i32,
}
