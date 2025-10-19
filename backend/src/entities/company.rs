use async_graphql::{InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "companies")]
#[graphql(concrete(name = "Company", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub eik: String,
    pub vat_number: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub contact_person: Option<String>,
    pub manager_name: Option<String>,
    pub authorized_person: Option<String>,
    pub manager_egn: Option<String>,
    pub authorized_person_egn: Option<String>,
    pub is_active: bool,
    pub contragent_api_url: Option<String>,
    pub contragent_api_key: Option<String>,
    pub enable_vies_validation: bool,
    pub enable_ai_mapping: bool,
    pub auto_validate_on_import: bool,
    pub base_currency_id: Option<i32>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::account::Entity")]
    Accounts,
    #[sea_orm(has_many = "super::journal_entry::Entity")]
    JournalEntries,
    #[sea_orm(
        belongs_to = "super::currency::Entity",
        from = "Column::BaseCurrencyId",
        to = "super::currency::Column::Id"
    )]
    BaseCurrency,
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Accounts.def()
    }
}

impl Related<super::journal_entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::JournalEntries.def()
    }
}

// Input types for GraphQL mutations
#[derive(InputObject, Deserialize, Serialize)]
pub struct CreateCompanyInput {
    pub name: String,
    pub eik: String,
    pub vat_number: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub contact_person: Option<String>,
    pub manager_name: Option<String>,
    pub authorized_person: Option<String>,
    pub manager_egn: Option<String>,
    pub authorized_person_egn: Option<String>,
    pub contragent_api_url: Option<String>,
    pub contragent_api_key: Option<String>,
    pub enable_vies_validation: Option<bool>,
    pub enable_ai_mapping: Option<bool>,
    pub auto_validate_on_import: Option<bool>,
    pub base_currency_id: Option<i32>,
}

#[derive(InputObject, Deserialize, Serialize)]
pub struct UpdateCompanyInput {
    pub name: Option<String>,
    pub eik: Option<String>,
    pub vat_number: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub contact_person: Option<String>,
    pub manager_name: Option<String>,
    pub authorized_person: Option<String>,
    pub manager_egn: Option<String>,
    pub authorized_person_egn: Option<String>,
    pub is_active: Option<bool>,
    pub contragent_api_url: Option<String>,
    pub contragent_api_key: Option<String>,
    pub enable_vies_validation: Option<bool>,
    pub enable_ai_mapping: Option<bool>,
    pub auto_validate_on_import: Option<bool>,
    pub base_currency_id: Option<i32>,
}

impl From<CreateCompanyInput> for ActiveModel {
    fn from(input: CreateCompanyInput) -> Self {
        ActiveModel {
            name: Set(input.name),
            eik: Set(input.eik),
            vat_number: Set(input.vat_number),
            address: Set(input.address),
            city: Set(input.city),
            country: Set(input.country.or(Some("България".to_string()))),
            phone: Set(input.phone),
            email: Set(input.email),
            contact_person: Set(input.contact_person),
            manager_name: Set(input.manager_name),
            authorized_person: Set(input.authorized_person),
            manager_egn: Set(input.manager_egn),
            authorized_person_egn: Set(input.authorized_person_egn),
            is_active: Set(true),
            contragent_api_url: Set(input.contragent_api_url),
            contragent_api_key: Set(input.contragent_api_key),
            enable_vies_validation: Set(input.enable_vies_validation.unwrap_or(false)),
            enable_ai_mapping: Set(input.enable_ai_mapping.unwrap_or(false)),
            auto_validate_on_import: Set(input.auto_validate_on_import.unwrap_or(false)),
            base_currency_id: Set(input.base_currency_id),
            ..Default::default()
        }
    }
}
impl ActiveModelBehavior for ActiveModel {}
