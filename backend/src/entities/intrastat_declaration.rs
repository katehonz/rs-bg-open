use async_graphql::{Enum, InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Enum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(10))")]
pub enum DeclarationType {
    #[sea_orm(string_value = "ARRIVAL")]
    Arrival,
    #[sea_orm(string_value = "DISPATCH")]
    Dispatch,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Enum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(15))")]
pub enum DeclarationStatus {
    #[sea_orm(string_value = "DRAFT")]
    Draft,
    #[sea_orm(string_value = "SUBMITTED")]
    Submitted,
    #[sea_orm(string_value = "ACCEPTED")]
    Accepted,
    #[sea_orm(string_value = "REJECTED")]
    Rejected,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "intrastat_declaration")]
#[graphql(concrete(name = "IntrastatDeclaration", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub company_id: i32,
    pub declaration_type: DeclarationType,
    pub reference_period: String,
    pub year: i32,
    pub month: i32,
    pub declaration_number: Option<String>,
    pub declarant_eik: String,
    pub declarant_name: String,
    pub contact_person: String,
    pub contact_phone: String,
    pub contact_email: String,
    pub total_items: i32,
    pub total_statistical_value: Decimal,
    pub total_invoice_value: Decimal,
    pub status: DeclarationStatus,
    pub submission_date: Option<DateTime>,
    pub xml_file_path: Option<String>,
    pub created_by: i32,
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
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::CreatedBy",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(has_many = "super::intrastat_declaration_item::Entity")]
    Items,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::intrastat_declaration_item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Items.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, InputObject)]
pub struct CreateIntrastatDeclarationInput {
    pub company_id: i32,
    pub declaration_type: DeclarationType,
    pub year: i32,
    pub month: i32,
    pub declarant_eik: String,
    pub declarant_name: String,
    pub contact_person: String,
    pub contact_phone: String,
    pub contact_email: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, InputObject)]
pub struct UpdateIntrastatDeclarationInput {
    pub declarant_name: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub status: Option<DeclarationStatus>,
}
