use async_graphql::{Enum, InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use sea_orm::{sea_query::StringLen, Set};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Enum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(1))")]
pub enum AccountType {
    #[sea_orm(string_value = "ASSET")]
    Asset,
    #[sea_orm(string_value = "LIABILITY")]
    Liability,
    #[sea_orm(string_value = "EQUITY")]
    Equity,
    #[sea_orm(string_value = "REVENUE")]
    Revenue,
    #[sea_orm(string_value = "EXPENSE")]
    Expense,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Enum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(1))")]
pub enum VatDirection {
    #[sea_orm(string_value = "NONE")]
    None,
    #[sea_orm(string_value = "INPUT")]
    Input,
    #[sea_orm(string_value = "OUTPUT")]
    Output,
    #[sea_orm(string_value = "BOTH")]
    Both,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "accounts")]
#[graphql(concrete(name = "Account", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub account_class: i32,
    pub parent_id: Option<i32>,
    pub level: i32,
    pub is_vat_applicable: bool,
    pub vat_direction: VatDirection,
    pub is_active: bool,
    pub is_analytical: bool,
    pub company_id: i32,
    pub supports_quantities: bool,
    pub default_unit: Option<String>,
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
    #[sea_orm(has_many = "super::intrastat_account_mapping::Entity")]
    IntrastatMappings,
    // Self-referencing relationship for parent-child hierarchy
    #[sea_orm(belongs_to = "Entity", from = "Column::ParentId", to = "Column::Id")]
    SelfRef,
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

impl Related<super::intrastat_account_mapping::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::IntrastatMappings.def()
    }
}

// Input types for GraphQL mutations
#[derive(InputObject, Deserialize, Serialize)]
pub struct CreateAccountInput {
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub account_class: i32,
    pub parent_id: Option<i32>,
    pub is_vat_applicable: Option<bool>,
    pub vat_direction: Option<VatDirection>,
    pub supports_quantities: Option<bool>,
    pub default_unit: Option<String>,
    pub company_id: i32,
}

#[derive(InputObject, Deserialize, Serialize)]
pub struct UpdateAccountInput {
    pub code: Option<String>,
    pub name: Option<String>,
    pub account_type: Option<AccountType>,
    pub parent_id: Option<i32>,
    pub is_vat_applicable: Option<bool>,
    pub vat_direction: Option<VatDirection>,
    pub supports_quantities: Option<bool>,
    pub default_unit: Option<String>,
    pub is_active: Option<bool>,
}

impl From<CreateAccountInput> for ActiveModel {
    fn from(input: CreateAccountInput) -> Self {
        // Automatically set is_analytical = true if parentId is provided
        let is_analytical = input.parent_id.is_some();

        ActiveModel {
            code: Set(input.code),
            name: Set(input.name),
            account_type: Set(input.account_type),
            account_class: Set(input.account_class),
            parent_id: Set(input.parent_id),
            is_analytical: Set(is_analytical),
            is_vat_applicable: Set(input.is_vat_applicable.unwrap_or(false)),
            vat_direction: Set(input.vat_direction.unwrap_or(VatDirection::None)),
            supports_quantities: Set(input
                .supports_quantities
                .unwrap_or(input.account_class == 2 || input.account_class == 3)),
            default_unit: Set(input.default_unit.or_else(|| {
                // Auto-set default unit for material/production accounts
                if input.account_class == 2 || input.account_class == 3 {
                    Some("бр".to_string())
                } else {
                    None
                }
            })),
            company_id: Set(input.company_id),
            ..Default::default()
        }
    }
}

// Helper struct for account hierarchy queries
#[derive(SimpleObject, Serialize)]
pub struct AccountWithBalance {
    pub account: Model,
    pub debit_balance: rust_decimal::Decimal,
    pub credit_balance: rust_decimal::Decimal,
    pub net_balance: rust_decimal::Decimal,
}
impl ActiveModelBehavior for ActiveModel {}
