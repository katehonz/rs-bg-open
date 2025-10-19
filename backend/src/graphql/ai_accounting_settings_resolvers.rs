use async_graphql::{Context, FieldResult, Object};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

use crate::entities::ai_accounting_setting::{
    self, CreateAiAccountingSettingInput, UpdateAiAccountingSettingInput,
};

#[derive(Default)]
pub struct AiAccountingSettingsQuery;

#[Object]
impl AiAccountingSettingsQuery {
    /// Get AI accounting settings for a company
    async fn ai_accounting_settings(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
    ) -> FieldResult<Option<ai_accounting_setting::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let settings = ai_accounting_setting::Entity::find()
            .filter(ai_accounting_setting::Column::CompanyId.eq(company_id))
            .one(db)
            .await?;

        Ok(settings)
    }

    /// Get all AI accounting settings (admin only)
    async fn all_ai_accounting_settings(
        &self,
        ctx: &Context<'_>,
    ) -> FieldResult<Vec<ai_accounting_setting::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let settings = ai_accounting_setting::Entity::find().all(db).await?;

        Ok(settings)
    }
}

#[derive(Default)]
pub struct AiAccountingSettingsMutation;

#[Object]
impl AiAccountingSettingsMutation {
    /// Create AI accounting settings for a company
    async fn create_ai_accounting_settings(
        &self,
        ctx: &Context<'_>,
        input: CreateAiAccountingSettingInput,
    ) -> FieldResult<ai_accounting_setting::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Check if settings already exist for this company
        let existing = ai_accounting_setting::Entity::find()
            .filter(ai_accounting_setting::Column::CompanyId.eq(input.company_id))
            .one(db)
            .await?;

        if existing.is_some() {
            return Err("AI accounting settings already exist for this company. Use update instead.".into());
        }

        let settings_model = ai_accounting_setting::ActiveModel::from(input);
        let settings = ai_accounting_setting::Entity::insert(settings_model)
            .exec_with_returning(db)
            .await?;

        Ok(settings)
    }

    /// Update AI accounting settings for a company
    async fn update_ai_accounting_settings(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        input: UpdateAiAccountingSettingInput,
    ) -> FieldResult<ai_accounting_setting::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Find existing settings
        let mut settings: ai_accounting_setting::ActiveModel =
            ai_accounting_setting::Entity::find()
                .filter(ai_accounting_setting::Column::CompanyId.eq(company_id))
                .one(db)
                .await?
                .ok_or_else(|| {
                    async_graphql::Error::new(
                        "AI accounting settings not found for this company. Create them first.",
                    )
                })?
                .into();

        // Update fields
        if let Some(sales_revenue_account) = input.sales_revenue_account {
            settings.sales_revenue_account = Set(sales_revenue_account);
        }
        if let Some(sales_services_account) = input.sales_services_account {
            settings.sales_services_account = Set(sales_services_account);
        }
        if let Some(sales_receivables_account) = input.sales_receivables_account {
            settings.sales_receivables_account = Set(sales_receivables_account);
        }
        if let Some(purchase_expense_account) = input.purchase_expense_account {
            settings.purchase_expense_account = Set(purchase_expense_account);
        }
        if let Some(purchase_payables_account) = input.purchase_payables_account {
            settings.purchase_payables_account = Set(purchase_payables_account);
        }
        if let Some(vat_input_account) = input.vat_input_account {
            settings.vat_input_account = Set(vat_input_account);
        }
        if let Some(vat_output_account) = input.vat_output_account {
            settings.vat_output_account = Set(vat_output_account);
        }
        if let Some(non_registered_persons_account) = input.non_registered_persons_account {
            settings.non_registered_persons_account = Set(Some(non_registered_persons_account));
        }
        if let Some(non_registered_vat_operation) = input.non_registered_vat_operation {
            settings.non_registered_vat_operation = Set(non_registered_vat_operation);
        }
        if let Some(account_code_length) = input.account_code_length {
            settings.account_code_length = Set(account_code_length);
        }
        if let Some(sales_description_template) = input.sales_description_template {
            settings.sales_description_template = Set(sales_description_template);
        }
        if let Some(purchase_description_template) = input.purchase_description_template {
            settings.purchase_description_template = Set(purchase_description_template);
        }

        settings.updated_at = Set(chrono::Utc::now().naive_utc());

        let updated_settings = settings.update(db).await?;
        Ok(updated_settings)
    }

    /// Delete AI accounting settings for a company
    async fn delete_ai_accounting_settings(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
    ) -> FieldResult<bool> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Find the settings
        let settings = ai_accounting_setting::Entity::find()
            .filter(ai_accounting_setting::Column::CompanyId.eq(company_id))
            .one(db)
            .await?
            .ok_or("AI accounting settings not found for this company")?;

        // Delete the settings
        ai_accounting_setting::Entity::delete_by_id(settings.id)
            .exec(db)
            .await?;

        Ok(true)
    }

    /// Create or update AI accounting settings (upsert)
    async fn upsert_ai_accounting_settings(
        &self,
        ctx: &Context<'_>,
        input: CreateAiAccountingSettingInput,
    ) -> FieldResult<ai_accounting_setting::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Check if settings exist
        let existing = ai_accounting_setting::Entity::find()
            .filter(ai_accounting_setting::Column::CompanyId.eq(input.company_id))
            .one(db)
            .await?;

        if let Some(existing_settings) = existing {
            // Update existing
            let mut settings: ai_accounting_setting::ActiveModel = existing_settings.into();

            settings.sales_revenue_account = Set(input.sales_revenue_account.unwrap_or_else(|| "701".to_string()));
            settings.sales_services_account = Set(input.sales_services_account.unwrap_or_else(|| "703".to_string()));
            settings.sales_receivables_account = Set(input.sales_receivables_account.unwrap_or_else(|| "411".to_string()));
            settings.purchase_expense_account = Set(input.purchase_expense_account.unwrap_or_else(|| "602".to_string()));
            settings.purchase_payables_account = Set(input.purchase_payables_account.unwrap_or_else(|| "401".to_string()));
            settings.vat_input_account = Set(input.vat_input_account.unwrap_or_else(|| "4531".to_string()));
            settings.vat_output_account = Set(input.vat_output_account.unwrap_or_else(|| "4531".to_string()));
            settings.non_registered_persons_account = Set(input.non_registered_persons_account);
            settings.non_registered_vat_operation = Set(input.non_registered_vat_operation.unwrap_or_else(|| "про09".to_string()));
            settings.account_code_length = Set(input.account_code_length.unwrap_or(3));
            settings.sales_description_template = Set(input.sales_description_template.unwrap_or_else(|| "{counterpart} - {document_number}".to_string()));
            settings.purchase_description_template = Set(input.purchase_description_template.unwrap_or_else(|| "{counterpart} - {document_number}".to_string()));
            settings.updated_at = Set(chrono::Utc::now().naive_utc());

            let updated_settings = settings.update(db).await?;
            Ok(updated_settings)
        } else {
            // Create new
            let settings_model = ai_accounting_setting::ActiveModel::from(input);
            let settings = ai_accounting_setting::Entity::insert(settings_model)
                .exec_with_returning(db)
                .await?;
            Ok(settings)
        }
    }
}
