//! Production Batch Entity
//!
//! Represents specific production batches/runs using technology cards

use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Enum, Copy)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum ProductionBatchStatus {
    #[sea_orm(string_value = "draft")]
    Draft,
    #[sea_orm(string_value = "in_progress")]
    InProgress,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "production_batches")]
#[graphql(name = "ProductionBatch")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// Company ID
    pub company_id: i32,

    /// Technology card ID being used
    pub technology_card_id: i32,

    /// Batch number
    pub batch_number: String,

    /// Input quantity for production
    pub input_quantity: Decimal,

    /// Production date
    pub production_date: NaiveDate,

    /// Batch status
    pub status: ProductionBatchStatus,

    /// Additional notes
    pub notes: Option<String>,

    /// User who created this batch
    pub created_by: Option<i32>,

    pub created_at: DateTimeUtc,
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
        belongs_to = "super::technology_card::Entity",
        from = "Column::TechnologyCardId",
        to = "super::technology_card::Column::Id"
    )]
    TechnologyCard,

    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::CreatedBy",
        to = "super::user::Column::Id"
    )]
    CreatedBy,

    #[sea_orm(has_many = "super::production_batch_stage::Entity")]
    Stages,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl Related<super::technology_card::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TechnologyCard.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatedBy.def()
    }
}

impl Related<super::production_batch_stage::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Stages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Input types for GraphQL mutations
#[derive(InputObject, Debug, Clone)]
pub struct CreateProductionBatchInput {
    pub company_id: i32,
    pub technology_card_id: i32,
    pub batch_number: String,
    pub input_quantity: String, // Using String for Decimal in GraphQL
    pub production_date: NaiveDate,
    pub notes: Option<String>,
}

#[derive(InputObject, Debug, Clone)]
pub struct UpdateProductionBatchInput {
    pub id: i32,
    pub input_quantity: Option<String>,
    pub production_date: Option<NaiveDate>,
    pub status: Option<ProductionBatchStatus>,
    pub notes: Option<String>,
}
