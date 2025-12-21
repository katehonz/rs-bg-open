//! Production Batch Stage Entity
//!
//! Represents completed stages of production batches with automatic accounting entries

use async_graphql::{Enum, InputObject, SimpleObject};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Enum, Copy)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum ProductionBatchStageStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "production_batch_stages")]
#[graphql(name = "ProductionBatchStage")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// Production batch ID
    pub production_batch_id: i32,

    /// Stage number in sequence
    pub stage_number: i32,

    /// Technology card stage ID this is based on
    pub technology_card_stage_id: i32,

    /// Automatically created journal entry ID
    pub journal_entry_id: Option<i32>,

    /// Actual quantity
    pub quantity: Option<Decimal>,

    /// Actual amount
    pub amount: Option<Decimal>,

    /// Unit of measure
    pub unit_of_measure: Option<String>,

    /// When the stage was completed
    pub completed_at: Option<DateTimeUtc>,

    /// Stage status
    pub status: ProductionBatchStageStatus,

    /// Additional notes
    pub notes: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::production_batch::Entity",
        from = "Column::ProductionBatchId",
        to = "super::production_batch::Column::Id"
    )]
    ProductionBatch,

    #[sea_orm(
        belongs_to = "super::technology_card_stage::Entity",
        from = "Column::TechnologyCardStageId",
        to = "super::technology_card_stage::Column::Id"
    )]
    TechnologyCardStage,

    #[sea_orm(
        belongs_to = "super::journal_entry::Entity",
        from = "Column::JournalEntryId",
        to = "super::journal_entry::Column::Id"
    )]
    JournalEntry,
}

impl Related<super::production_batch::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductionBatch.def()
    }
}

impl Related<super::technology_card_stage::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TechnologyCardStage.def()
    }
}

impl Related<super::journal_entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::JournalEntry.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Input types for GraphQL mutations
#[derive(InputObject, Debug, Clone)]
pub struct CompleteProductionBatchStageInput {
    pub production_batch_id: i32,
    pub stage_number: i32,
    pub notes: Option<String>,
}
