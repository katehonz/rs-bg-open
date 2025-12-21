//! Technology Card Entity
//!
//! Represents production recipes/technology cards with stages and formulas

use async_graphql::{InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "technology_cards")]
#[graphql(name = "TechnologyCard")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// Company ID
    pub company_id: i32,

    /// Name of the technology card/recipe
    pub name: String,

    /// Description of the production process
    pub description: Option<String>,

    /// Unit of measure for the output product (kg, pcs, L)
    pub output_unit: String,

    /// Whether this card is active and can be used
    pub is_active: bool,

    /// User who created this card
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
        belongs_to = "super::user::Entity",
        from = "Column::CreatedBy",
        to = "super::user::Column::Id"
    )]
    CreatedBy,

    #[sea_orm(has_many = "super::technology_card_stage::Entity")]
    Stages,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatedBy.def()
    }
}

impl Related<super::technology_card_stage::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Stages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Input types for GraphQL mutations
#[derive(InputObject, Debug, Clone)]
pub struct CreateTechnologyCardInput {
    pub company_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub output_unit: String,
}

#[derive(InputObject, Debug, Clone)]
pub struct UpdateTechnologyCardInput {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub output_unit: Option<String>,
    pub is_active: Option<bool>,
}
