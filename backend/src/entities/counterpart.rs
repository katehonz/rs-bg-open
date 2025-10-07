use async_graphql::{Enum, InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use sea_orm::{sea_query::StringLen, Set};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Enum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
pub enum CounterpartType {
    #[sea_orm(string_value = "CUSTOMER")]
    Customer,
    #[sea_orm(string_value = "SUPPLIER")]
    Supplier,
    #[sea_orm(string_value = "EMPLOYEE")]
    Employee,
    #[sea_orm(string_value = "BANK")]
    Bank,
    #[sea_orm(string_value = "GOVERNMENT")]
    Government,
    #[sea_orm(string_value = "OTHER")]
    Other,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "counterparts")]
#[graphql(concrete(name = "Counterpart", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub eik: Option<String>,
    pub vat_number: Option<String>,
    pub street: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub contact_person: Option<String>,
    pub counterpart_type: CounterpartType,
    pub is_customer: bool,
    pub is_supplier: bool,
    pub is_vat_registered: bool,
    pub is_active: bool,
    pub company_id: i32,
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
    #[sea_orm(has_many = "super::entry_line::Entity")]
    EntryLines,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl Related<super::entry_line::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EntryLines.def()
    }
}

// Input types for GraphQL mutations
#[derive(InputObject, Deserialize, Serialize)]
pub struct CreateCounterpartInput {
    pub name: String,
    pub eik: Option<String>,
    pub vat_number: Option<String>,
    pub street: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub contact_person: Option<String>,
    pub counterpart_type: Option<CounterpartType>,
    pub is_customer: Option<bool>,
    pub is_supplier: Option<bool>,
    pub is_vat_registered: Option<bool>,
    pub company_id: i32,
}

#[derive(InputObject, Deserialize, Serialize)]
pub struct UpdateCounterpartInput {
    pub name: Option<String>,
    pub eik: Option<String>,
    pub vat_number: Option<String>,
    pub street: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub contact_person: Option<String>,
    pub counterpart_type: Option<CounterpartType>,
    pub is_customer: Option<bool>,
    pub is_supplier: Option<bool>,
    pub is_vat_registered: Option<bool>,
    pub is_active: Option<bool>,
}

impl From<CreateCounterpartInput> for ActiveModel {
    fn from(input: CreateCounterpartInput) -> Self {
        ActiveModel {
            name: Set(input.name),
            eik: Set(input.eik),
            vat_number: Set(input.vat_number),
            street: Set(input.street),
            address: Set(input.address),
            city: Set(input.city),
            postal_code: Set(input.postal_code),
            country: Set(input.country.or(Some("България".to_string()))),
            phone: Set(input.phone),
            email: Set(input.email),
            contact_person: Set(input.contact_person),
            counterpart_type: Set(input.counterpart_type.unwrap_or(CounterpartType::Other)),
            is_customer: Set(input.is_customer.unwrap_or(false)),
            is_supplier: Set(input.is_supplier.unwrap_or(false)),
            is_vat_registered: Set(input.is_vat_registered.unwrap_or(false)),
            company_id: Set(input.company_id),
            is_active: Set(true),
            ..Default::default()
        }
    }
}

// Filter input for queries
#[derive(InputObject, Deserialize)]
pub struct CounterpartFilter {
    pub company_id: Option<i32>,
    pub counterpart_type: Option<CounterpartType>,
    pub is_customer: Option<bool>,
    pub is_supplier: Option<bool>,
    pub is_vat_registered: Option<bool>,
    pub is_active: Option<bool>,
    pub name_contains: Option<String>,
    pub vat_number: Option<String>,
}

// Helper struct for counterpart with balance
#[derive(SimpleObject, Serialize)]
pub struct CounterpartWithBalance {
    pub counterpart: Model,
    pub total_debit: rust_decimal::Decimal,
    pub total_credit: rust_decimal::Decimal,
    pub balance: rust_decimal::Decimal,
    pub last_transaction_date: Option<chrono::NaiveDate>,
}

// Common counterpart categories for Bulgarian accounting
impl CounterpartType {
    pub fn bulgarian_categories() -> Vec<Self> {
        vec![
            CounterpartType::Customer,
            CounterpartType::Supplier,
            CounterpartType::Employee,
            CounterpartType::Bank,
            CounterpartType::Government,
            CounterpartType::Other,
        ]
    }

    pub fn description(&self) -> &'static str {
        match self {
            CounterpartType::Customer => "Клиент",
            CounterpartType::Supplier => "Доставчик",
            CounterpartType::Employee => "Служител",
            CounterpartType::Bank => "Банка",
            CounterpartType::Government => "Държавна институция",
            CounterpartType::Other => "Друго",
        }
    }
}
impl ActiveModelBehavior for ActiveModel {}
