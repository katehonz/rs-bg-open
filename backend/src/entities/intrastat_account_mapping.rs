use async_graphql::{Enum, InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Enum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(10))")]
pub enum FlowDirection {
    #[sea_orm(string_value = "ARRIVAL")]
    Arrival,
    #[sea_orm(string_value = "DISPATCH")]
    Dispatch,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "intrastat_account_mapping")]
#[graphql(concrete(name = "IntrastatAccountMapping", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub account_id: i32,
    pub nomenclature_id: i32,
    pub flow_direction: FlowDirection,
    pub transaction_nature_code: String,
    pub is_quantity_tracked: bool,
    pub default_country_code: Option<String>,
    pub default_transport_mode: Option<i32>,
    pub is_optional: bool,
    pub min_threshold_bgn: Option<Decimal>,
    pub company_id: i32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id"
    )]
    Account,
    #[sea_orm(
        belongs_to = "super::intrastat_nomenclature::Entity",
        from = "Column::NomenclatureId",
        to = "super::intrastat_nomenclature::Column::Id"
    )]
    Nomenclature,
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::CompanyId",
        to = "super::company::Column::Id"
    )]
    Company,
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::intrastat_nomenclature::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nomenclature.def()
    }
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, InputObject)]
pub struct CreateIntrastatAccountMappingInput {
    pub account_id: i32,
    pub nomenclature_id: i32,
    pub flow_direction: FlowDirection,
    pub transaction_nature_code: String,
    pub is_quantity_tracked: bool,
    pub default_country_code: Option<String>,
    pub default_transport_mode: Option<i32>,
    pub is_optional: bool,
    pub min_threshold_bgn: Option<Decimal>,
    pub company_id: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, InputObject)]
pub struct UpdateIntrastatAccountMappingInput {
    pub account_id: Option<i32>,
    pub nomenclature_id: Option<i32>,
    pub flow_direction: Option<FlowDirection>,
    pub transaction_nature_code: Option<String>,
    pub is_quantity_tracked: Option<bool>,
    pub default_country_code: Option<String>,
    pub default_transport_mode: Option<i32>,
    pub is_optional: Option<bool>,
    pub min_threshold_bgn: Option<Decimal>,
}
