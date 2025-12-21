//! Technology Card Stage Entity
//!
//! Represents individual stages in a production technology card with accounting accounts and formulas

use async_graphql::{InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "technology_card_stages")]
#[graphql(name = "TechnologyCardStage")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// Technology card ID
    pub technology_card_id: i32,

    /// Stage number in sequence (1, 2, 3...)
    pub stage_number: i32,

    /// Name of the stage
    pub name: String,

    /// Debit account ID for the accounting entry
    pub debit_account_id: i32,

    /// Credit account ID for the accounting entry
    pub credit_account_id: i32,

    /// Formula for calculating quantity (e.g., "input_quantity", "input_quantity * 1.05")
    pub quantity_formula: Option<String>,

    /// Unit of measure for this stage
    pub unit_of_measure: Option<String>,

    /// Formula for calculating amount (e.g., "quantity * unit_price", "previous_stage_amount")
    pub amount_formula: Option<String>,

    /// Description of the stage
    pub description: Option<String>,

    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::technology_card::Entity",
        from = "Column::TechnologyCardId",
        to = "super::technology_card::Column::Id"
    )]
    TechnologyCard,

    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::DebitAccountId",
        to = "super::account::Column::Id"
    )]
    DebitAccount,

    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::CreditAccountId",
        to = "super::account::Column::Id"
    )]
    CreditAccount,
}

impl Related<super::technology_card::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TechnologyCard.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Input types for GraphQL mutations
#[derive(InputObject, Debug, Clone)]
pub struct CreateTechnologyCardStageInput {
    pub technology_card_id: i32,
    pub stage_number: i32,
    pub name: String,
    pub debit_account_id: i32,
    pub credit_account_id: i32,
    pub quantity_formula: Option<String>,
    pub unit_of_measure: Option<String>,
    pub amount_formula: Option<String>,
    pub description: Option<String>,
}

#[derive(InputObject, Debug, Clone)]
pub struct UpdateTechnologyCardStageInput {
    pub id: i32,
    pub name: Option<String>,
    pub debit_account_id: Option<i32>,
    pub credit_account_id: Option<i32>,
    pub quantity_formula: Option<String>,
    pub unit_of_measure: Option<String>,
    pub amount_formula: Option<String>,
    pub description: Option<String>,
}
