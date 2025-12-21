use async_graphql::{InputObject, SimpleObject};
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "ai_accounting_settings")]
#[graphql(name = "AiAccountingSetting")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub company_id: i32,

    // Sales accounts
    pub sales_revenue_account: String,
    pub sales_services_account: String,
    pub sales_receivables_account: String,

    // Purchase accounts
    pub purchase_expense_account: String,
    pub purchase_payables_account: String,

    // VAT accounts
    pub vat_input_account: String,
    pub vat_output_account: String,

    // Special cases
    pub non_registered_persons_account: Option<String>,
    pub non_registered_vat_operation: String,

    // Account code length (3 or 4 digits)
    pub account_code_length: i32,

    // Description templates
    pub sales_description_template: String,
    pub purchase_description_template: String,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::CompanyId",
        to = "super::company::Column::Id",
        on_delete = "Cascade"
    )]
    Company,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Input types for GraphQL
#[derive(InputObject)]
pub struct CreateAiAccountingSettingInput {
    pub company_id: i32,
    pub sales_revenue_account: Option<String>,
    pub sales_services_account: Option<String>,
    pub sales_receivables_account: Option<String>,
    pub purchase_expense_account: Option<String>,
    pub purchase_payables_account: Option<String>,
    pub vat_input_account: Option<String>,
    pub vat_output_account: Option<String>,
    pub non_registered_persons_account: Option<String>,
    pub non_registered_vat_operation: Option<String>,
    pub account_code_length: Option<i32>,
    pub sales_description_template: Option<String>,
    pub purchase_description_template: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateAiAccountingSettingInput {
    pub sales_revenue_account: Option<String>,
    pub sales_services_account: Option<String>,
    pub sales_receivables_account: Option<String>,
    pub purchase_expense_account: Option<String>,
    pub purchase_payables_account: Option<String>,
    pub vat_input_account: Option<String>,
    pub vat_output_account: Option<String>,
    pub non_registered_persons_account: Option<String>,
    pub non_registered_vat_operation: Option<String>,
    pub account_code_length: Option<i32>,
    pub sales_description_template: Option<String>,
    pub purchase_description_template: Option<String>,
}

impl From<CreateAiAccountingSettingInput> for ActiveModel {
    fn from(input: CreateAiAccountingSettingInput) -> Self {
        Self {
            company_id: Set(input.company_id),
            sales_revenue_account: Set(input.sales_revenue_account.unwrap_or_else(|| "701".to_string())),
            sales_services_account: Set(input.sales_services_account.unwrap_or_else(|| "703".to_string())),
            sales_receivables_account: Set(input.sales_receivables_account.unwrap_or_else(|| "411".to_string())),
            purchase_expense_account: Set(input.purchase_expense_account.unwrap_or_else(|| "602".to_string())),
            purchase_payables_account: Set(input.purchase_payables_account.unwrap_or_else(|| "401".to_string())),
            vat_input_account: Set(input.vat_input_account.unwrap_or_else(|| "4531".to_string())),
            vat_output_account: Set(input.vat_output_account.unwrap_or_else(|| "4531".to_string())),
            non_registered_persons_account: Set(input.non_registered_persons_account),
            non_registered_vat_operation: Set(input.non_registered_vat_operation.unwrap_or_else(|| "про09".to_string())),
            account_code_length: Set(input.account_code_length.unwrap_or(3)),
            sales_description_template: Set(input.sales_description_template.unwrap_or_else(|| "{counterpart} - {document_number}".to_string())),
            purchase_description_template: Set(input.purchase_description_template.unwrap_or_else(|| "{counterpart} - {document_number}".to_string())),
            ..Default::default()
        }
    }
}

impl Model {
    /// Get the appropriate revenue account based on invoice type
    pub fn get_revenue_account(&self, is_service: bool) -> &str {
        if is_service {
            &self.sales_services_account
        } else {
            &self.sales_revenue_account
        }
    }

    /// Format account code with proper length
    pub fn format_account_code(&self, base_code: &str) -> String {
        if self.account_code_length == 4 && base_code.len() == 3 {
            format!("{}1", base_code)
        } else {
            base_code.to_string()
        }
    }

    /// Format description using template
    pub fn format_description(&self, template: &str, counterpart: &str, document_number: &str) -> String {
        template
            .replace("{counterpart}", counterpart)
            .replace("{document_number}", document_number)
    }
}
