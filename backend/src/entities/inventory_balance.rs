//! Inventory Balance Entity
//!
//! Maintains current balances and average costs for each material account

use async_graphql::SimpleObject;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "inventory_balances")]
#[graphql(name = "InventoryBalance")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// Company ID
    pub company_id: i32,

    /// Material account ID (class 3)
    pub account_id: i32,

    /// Current quantity on hand
    pub current_quantity: Decimal,

    /// Current total amount value
    pub current_amount: Decimal,

    /// Current weighted average cost per unit
    pub current_average_cost: Decimal,

    /// Date of last movement
    pub last_movement_date: Option<NaiveDate>,

    /// ID of last movement
    pub last_movement_id: Option<i32>,

    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::CompanyId",
        to = "super::company::Column::Id"
    )]
    Company,

    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id"
    )]
    Account,

    #[sea_orm(
        belongs_to = "super::inventory_movement::Entity",
        from = "Column::LastMovementId",
        to = "super::inventory_movement::Column::Id"
    )]
    LastMovement,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::inventory_movement::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LastMovement.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Check if inventory is in stock
    pub fn is_in_stock(&self) -> bool {
        self.current_quantity > Decimal::ZERO
    }

    /// Check if inventory is out of stock
    pub fn is_out_of_stock(&self) -> bool {
        self.current_quantity <= Decimal::ZERO
    }

    /// Calculate new average cost after a receipt
    /// Formula: (current_amount + receipt_amount) / (current_quantity + receipt_quantity)
    pub fn calculate_new_average_cost_after_receipt(
        &self,
        receipt_quantity: Decimal,
        receipt_amount: Decimal,
    ) -> Decimal {
        let new_total_quantity = self.current_quantity + receipt_quantity;
        let new_total_amount = self.current_amount + receipt_amount;

        if new_total_quantity > Decimal::ZERO {
            new_total_amount / new_total_quantity
        } else {
            Decimal::ZERO
        }
    }

    /// Calculate value of an issue at current average cost
    pub fn calculate_issue_value(&self, issue_quantity: Decimal) -> Decimal {
        issue_quantity * self.current_average_cost
    }
}
