use async_graphql::{InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "intrastat_settings")]
#[graphql(concrete(name = "IntrastatSettings", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub company_id: i32,
    pub is_enabled: bool,
    pub arrival_threshold_bgn: Decimal,
    pub dispatch_threshold_bgn: Decimal,
    pub current_arrival_threshold_bgn: Decimal,
    pub current_dispatch_threshold_bgn: Decimal,
    pub auto_generate_declarations: bool,
    pub default_transport_mode: Option<i32>,
    pub default_delivery_terms: Option<String>,
    pub default_transaction_nature: Option<String>,
    pub responsible_person_name: Option<String>,
    pub responsible_person_phone: Option<String>,
    pub responsible_person_email: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::CompanyId",
        to = "super::company::Column::Id"
    )]
    Company,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, InputObject)]
pub struct CreateIntrastatSettingsInput {
    pub company_id: i32,
    pub is_enabled: bool,
    pub arrival_threshold_bgn: Decimal,
    pub dispatch_threshold_bgn: Decimal,
    pub auto_generate_declarations: bool,
    pub default_transport_mode: Option<i32>,
    pub default_delivery_terms: Option<String>,
    pub default_transaction_nature: Option<String>,
    pub responsible_person_name: Option<String>,
    pub responsible_person_phone: Option<String>,
    pub responsible_person_email: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, InputObject)]
pub struct UpdateIntrastatSettingsInput {
    pub is_enabled: Option<bool>,
    pub arrival_threshold_bgn: Option<Decimal>,
    pub dispatch_threshold_bgn: Option<Decimal>,
    pub auto_generate_declarations: Option<bool>,
    pub default_transport_mode: Option<i32>,
    pub default_delivery_terms: Option<String>,
    pub default_transaction_nature: Option<String>,
    pub responsible_person_name: Option<String>,
    pub responsible_person_phone: Option<String>,
    pub responsible_person_email: Option<String>,
}
