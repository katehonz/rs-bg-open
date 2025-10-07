use async_graphql::{Enum, InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use sea_orm::prelude::StringLen;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, Copy, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Enum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(10))")]
pub enum UserCompanyRole {
    #[sea_orm(string_value = "admin")]
    Admin,
    #[sea_orm(string_value = "user")]
    User,
    #[sea_orm(string_value = "viewer")]
    Viewer,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "user_companies")]
#[graphql(concrete(name = "UserCompany", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub company_id: i32,
    pub role: UserCompanyRole,
    pub is_active: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::CompanyId",
        to = "super::company::Column::Id"
    )]
    Company,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Input structs for GraphQL
#[derive(InputObject)]
pub struct CreateUserCompanyInput {
    pub user_id: i32,
    pub company_id: i32,
    pub role: UserCompanyRole,
    pub is_active: Option<bool>,
}

#[derive(InputObject)]
pub struct UpdateUserCompanyInput {
    pub role: Option<UserCompanyRole>,
    pub is_active: Option<bool>,
}

// Extended model with related data
#[derive(SimpleObject, Clone, Debug)]
pub struct UserCompanyWithDetails {
    pub id: i32,
    pub user_id: i32,
    pub company_id: i32,
    pub role: UserCompanyRole,
    pub is_active: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    // Related data
    pub user: Option<super::user::Model>,
    pub company: Option<super::company::Model>,
}

impl Model {
    pub fn can_admin_company(&self) -> bool {
        self.role == UserCompanyRole::Admin && self.is_active
    }

    pub fn can_edit_company(&self) -> bool {
        matches!(self.role, UserCompanyRole::Admin | UserCompanyRole::User) && self.is_active
    }

    pub fn can_view_company(&self) -> bool {
        self.is_active
    }
}
