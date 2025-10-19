use async_graphql::{InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "global_contragents")]
#[graphql(concrete(name = "GlobalContragent", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub vat_number: String,
    pub eik: Option<String>,
    pub registration_number: Option<String>,
    pub customer_id: Option<String>,
    pub supplier_id: Option<String>,
    pub company_name: Option<String>,
    pub company_name_bg: Option<String>,
    pub legal_form: Option<String>,
    pub status: Option<String>,
    pub address: Option<String>,
    pub long_address: Option<String>,
    pub street_name: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub contact_person: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub r#type: Option<String>,
    pub iban: Option<String>,
    pub bic: Option<String>,
    pub bank_name: Option<String>,
    pub vat_valid: bool,
    pub eik_valid: bool,
    pub valid: bool,
    pub last_validated_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(InputObject, Deserialize, Serialize, Default)]
pub struct GlobalContragentFilter {
    pub search: Option<String>,
    pub vat_number: Option<String>,
    pub eik: Option<String>,
    pub vat_valid: Option<bool>,
    pub eik_valid: Option<bool>,
    pub valid: Option<bool>,
}

#[derive(SimpleObject, Deserialize, Serialize)]
pub struct GlobalContragentSummary {
    pub total: i64,
    pub valid_count: i64,
    pub invalid_count: i64,
    pub last_synced_at: Option<DateTimeUtc>,
}
