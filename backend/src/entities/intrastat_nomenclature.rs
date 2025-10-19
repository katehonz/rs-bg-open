use async_graphql::{InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "intrastat_nomenclature")]
#[graphql(concrete(name = "IntrastatNomenclature", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub cn_code: String,
    pub description_bg: String,
    pub description_en: Option<String>,
    pub unit_of_measure: String,
    pub unit_description: String,
    pub parent_code: Option<String>,
    pub level: i32,
    pub is_active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::intrastat_account_mapping::Entity")]
    AccountMappings,
    #[sea_orm(has_many = "super::intrastat_declaration_item::Entity")]
    DeclarationItems,
}

impl Related<super::intrastat_account_mapping::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccountMappings.def()
    }
}

impl Related<super::intrastat_declaration_item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DeclarationItems.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, InputObject)]
pub struct CreateIntrastatNomenclatureInput {
    pub cn_code: String,
    pub description_bg: String,
    pub description_en: Option<String>,
    pub unit_of_measure: String,
    pub unit_description: String,
    pub parent_code: Option<String>,
    pub level: i32,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, InputObject)]
pub struct UpdateIntrastatNomenclatureInput {
    pub cn_code: Option<String>,
    pub description_bg: Option<String>,
    pub description_en: Option<String>,
    pub unit_of_measure: Option<String>,
    pub unit_description: Option<String>,
    pub parent_code: Option<String>,
    pub level: Option<i32>,
    pub is_active: Option<bool>,
}
