use async_graphql::{InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "currency_revaluation_settings")]
#[graphql(concrete(name = "CurrencyRevaluationSettings", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub company_id: i32,
    pub expense_account_id: i32,
    pub revenue_account_id: i32,
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
        belongs_to = "super::account::Entity",
        from = "Column::ExpenseAccountId",
        to = "super::account::Column::Id"
    )]
    ExpenseAccount,

    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::RevenueAccountId",
        to = "super::account::Column::Id"
    )]
    RevenueAccount,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Input types for GraphQL mutations
#[derive(InputObject, Deserialize, Serialize)]
pub struct RevaluationSettingsInput {
    pub company_id: i32,
    pub expense_account_id: i32,
    pub revenue_account_id: i32,
}

impl From<RevaluationSettingsInput> for ActiveModel {
    fn from(input: RevaluationSettingsInput) -> Self {
        ActiveModel {
            company_id: Set(input.company_id),
            expense_account_id: Set(input.expense_account_id),
            revenue_account_id: Set(input.revenue_account_id),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
    }
}
