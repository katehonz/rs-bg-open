use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "controlisy_imports")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub company_id: i32,
    pub import_date: NaiveDateTime,
    pub file_name: String,
    pub document_type: String,          // 'purchase' or 'sale'
    pub raw_xml: Option<String>,        // Original XML content
    pub parsed_data: serde_json::Value, // Parsed JSON data
    pub status: String,                 // staged, reviewed, processing, completed, failed
    pub processed: bool,
    pub error_message: Option<String>,
    pub imported_documents: Option<i32>,
    pub imported_contractors: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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

impl Model {
    pub fn get_status_display(&self) -> &'static str {
        match self.status.as_str() {
            "staged" => "За преглед",
            "reviewed" => "Прегледан",
            "processing" => "Обработва се",
            "completed" => "Завършен",
            "failed" => "Грешка",
            _ => "Непознат",
        }
    }
}
