use async_graphql::{InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "contragent_settings")]
#[graphql(concrete(name = "ContragentSetting", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub key: String,
    pub value: Option<String>,
    pub description: Option<String>,
    pub encrypted: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(InputObject, Deserialize, Serialize)]
pub struct UpsertContragentSettingInput {
    pub key: String,
    pub value: Option<String>,
    pub description: Option<String>,
    pub encrypted: Option<bool>,
}
