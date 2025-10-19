use async_graphql::{Context, FieldResult, Object};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use std::sync::Arc;

use crate::entities::ai_bank_accounting_setting::{
    self, CreateAiBankAccountingSettingInput, UpdateAiBankAccountingSettingInput,
};

#[derive(Default)]
pub struct AiBankAccountingSettingsQuery;

#[Object]
impl AiBankAccountingSettingsQuery {
    /// Get all AI bank accounting settings for a company
    async fn ai_bank_accounting_settings(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
    ) -> FieldResult<Vec<ai_bank_accounting_setting::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let settings = ai_bank_accounting_setting::Entity::find()
            .filter(ai_bank_accounting_setting::Column::CompanyId.eq(company_id))
            .filter(ai_bank_accounting_setting::Column::IsActive.eq(true))
            .order_by_desc(ai_bank_accounting_setting::Column::Priority)
            .all(db)
            .await?;

        Ok(settings)
    }

    /// Get single AI bank accounting setting by ID
    async fn ai_bank_accounting_setting(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> FieldResult<Option<ai_bank_accounting_setting::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let setting = ai_bank_accounting_setting::Entity::find_by_id(id)
            .one(db)
            .await?;

        Ok(setting)
    }

    /// Get all AI bank accounting settings (including inactive, admin only)
    async fn all_ai_bank_accounting_settings(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
    ) -> FieldResult<Vec<ai_bank_accounting_setting::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let settings = ai_bank_accounting_setting::Entity::find()
            .filter(ai_bank_accounting_setting::Column::CompanyId.eq(company_id))
            .order_by_desc(ai_bank_accounting_setting::Column::Priority)
            .all(db)
            .await?;

        Ok(settings)
    }
}

#[derive(Default)]
pub struct AiBankAccountingSettingsMutation;

#[Object]
impl AiBankAccountingSettingsMutation {
    /// Create AI bank accounting setting
    async fn create_ai_bank_accounting_setting(
        &self,
        ctx: &Context<'_>,
        input: CreateAiBankAccountingSettingInput,
    ) -> FieldResult<ai_bank_accounting_setting::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let settings_model = ai_bank_accounting_setting::ActiveModel::from(input);
        let settings = ai_bank_accounting_setting::Entity::insert(settings_model)
            .exec_with_returning(db)
            .await?;

        Ok(settings)
    }

    /// Update AI bank accounting setting
    async fn update_ai_bank_accounting_setting(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateAiBankAccountingSettingInput,
    ) -> FieldResult<ai_bank_accounting_setting::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Find existing setting
        let mut setting: ai_bank_accounting_setting::ActiveModel =
            ai_bank_accounting_setting::Entity::find_by_id(id)
                .one(db)
                .await?
                .ok_or_else(|| {
                    async_graphql::Error::new("AI bank accounting setting not found")
                })?
                .into();

        // Update fields
        if let Some(pattern_name) = input.pattern_name {
            setting.pattern_name = Set(pattern_name);
        }
        if let Some(description_keywords) = input.description_keywords {
            setting.description_keywords = Set(Some(description_keywords));
        }
        if let Some(transaction_type) = input.transaction_type {
            setting.transaction_type = Set(transaction_type);
        }
        if let Some(account_id) = input.account_id {
            setting.account_id = Set(Some(account_id));
        }
        if let Some(counterpart_account_id) = input.counterpart_account_id {
            setting.counterpart_account_id = Set(Some(counterpart_account_id));
        }
        if let Some(vat_account_id) = input.vat_account_id {
            setting.vat_account_id = Set(Some(vat_account_id));
        }
        if let Some(direction) = input.direction {
            setting.direction = Set(direction);
        }
        if let Some(description_template) = input.description_template {
            setting.description_template = Set(Some(description_template));
        }
        if let Some(priority) = input.priority {
            setting.priority = Set(priority);
        }
        if let Some(is_active) = input.is_active {
            setting.is_active = Set(is_active);
        }

        setting.updated_at = Set(chrono::Utc::now().naive_utc());

        let updated_setting = setting.update(db).await?;
        Ok(updated_setting)
    }

    /// Delete AI bank accounting setting
    async fn delete_ai_bank_accounting_setting(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> FieldResult<bool> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        ai_bank_accounting_setting::Entity::delete_by_id(id)
            .exec(db)
            .await?;

        Ok(true)
    }

    /// Toggle active status
    async fn toggle_ai_bank_accounting_setting_status(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> FieldResult<ai_bank_accounting_setting::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let setting = ai_bank_accounting_setting::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Setting not found"))?;

        let mut active_model: ai_bank_accounting_setting::ActiveModel = setting.into();
        active_model.is_active = Set(!active_model.is_active.unwrap());
        active_model.updated_at = Set(chrono::Utc::now().naive_utc());

        let updated = active_model.update(db).await?;
        Ok(updated)
    }
}
