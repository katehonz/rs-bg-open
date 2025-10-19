use async_graphql::{InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "intrastat_declaration_item")]
#[graphql(concrete(name = "IntrastatDeclarationItem", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub declaration_id: i32,
    pub item_number: i32,
    pub cn_code: String,
    pub nomenclature_id: Option<i32>,
    pub country_of_origin: String,
    pub country_of_consignment: String,
    pub transaction_nature_code: String,
    pub transport_mode: i32,
    pub delivery_terms: String,
    pub statistical_procedure: Option<String>,
    pub net_mass_kg: Decimal,
    pub supplementary_unit: Option<Decimal>,
    pub invoice_value: Decimal,
    pub statistical_value: Decimal,
    pub currency_code: String,
    pub description: String,
    pub region_code: Option<String>,
    pub port_code: Option<String>,
    pub journal_entry_id: Option<i32>,
    pub entry_line_id: Option<i32>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::intrastat_declaration::Entity",
        from = "Column::DeclarationId",
        to = "super::intrastat_declaration::Column::Id"
    )]
    Declaration,
    #[sea_orm(
        belongs_to = "super::intrastat_nomenclature::Entity",
        from = "Column::NomenclatureId",
        to = "super::intrastat_nomenclature::Column::Id"
    )]
    Nomenclature,
    #[sea_orm(
        belongs_to = "super::journal_entry::Entity",
        from = "Column::JournalEntryId",
        to = "super::journal_entry::Column::Id"
    )]
    JournalEntry,
    #[sea_orm(
        belongs_to = "super::entry_line::Entity",
        from = "Column::EntryLineId",
        to = "super::entry_line::Column::Id"
    )]
    EntryLine,
}

impl Related<super::intrastat_declaration::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Declaration.def()
    }
}

impl Related<super::intrastat_nomenclature::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nomenclature.def()
    }
}

impl Related<super::journal_entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::JournalEntry.def()
    }
}

impl Related<super::entry_line::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EntryLine.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, InputObject)]
pub struct CreateIntrastatDeclarationItemInput {
    pub declaration_id: i32,
    pub item_number: i32,
    pub cn_code: String,
    pub nomenclature_id: Option<i32>,
    pub country_of_origin: String,
    pub country_of_consignment: String,
    pub transaction_nature_code: String,
    pub transport_mode: i32,
    pub delivery_terms: String,
    pub statistical_procedure: Option<String>,
    pub net_mass_kg: Decimal,
    pub supplementary_unit: Option<Decimal>,
    pub invoice_value: Decimal,
    pub statistical_value: Decimal,
    pub currency_code: String,
    pub description: String,
    pub region_code: Option<String>,
    pub port_code: Option<String>,
    pub journal_entry_id: Option<i32>,
    pub entry_line_id: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, InputObject)]
pub struct UpdateIntrastatDeclarationItemInput {
    pub cn_code: Option<String>,
    pub country_of_origin: Option<String>,
    pub country_of_consignment: Option<String>,
    pub transaction_nature_code: Option<String>,
    pub transport_mode: Option<i32>,
    pub delivery_terms: Option<String>,
    pub statistical_procedure: Option<String>,
    pub net_mass_kg: Option<Decimal>,
    pub supplementary_unit: Option<Decimal>,
    pub invoice_value: Option<Decimal>,
    pub statistical_value: Option<Decimal>,
    pub description: Option<String>,
}
