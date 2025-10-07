use async_graphql::{Context, FieldResult, InputObject, Object, SimpleObject};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::{NaiveDate, TimeZone, Utc};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use std::{convert::TryFrom, sync::Arc};

use crate::entities::{
    bank_import, bank_profile, BankImportModel, BankImportStatus, BankProfileActiveModel,
    BankProfileModel, CreateBankProfileInput, UpdateBankProfileInput,
};
use crate::services::bank_imports::{BankImportService, ImportSummary};

#[derive(Default)]
pub struct BankQuery;

#[Object]
impl BankQuery {
    async fn bank_profiles(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        active_only: Option<bool>,
    ) -> FieldResult<Vec<BankProfileModel>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut query = bank_profile::Entity::find()
            .filter(bank_profile::Column::CompanyId.eq(company_id))
            .order_by_asc(bank_profile::Column::Name);

        if active_only.unwrap_or(false) {
            query = query.filter(bank_profile::Column::IsActive.eq(true));
        }

        let profiles = query.all(db).await?;
        Ok(profiles)
    }

    async fn bank_profile(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> FieldResult<Option<BankProfileModel>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let profile = bank_profile::Entity::find_by_id(id).one(db).await?;
        Ok(profile)
    }

    async fn bank_imports(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        bank_profile_id: Option<i32>,
        limit: Option<i32>,
        offset: Option<i32>,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>,
    ) -> FieldResult<Vec<BankImportModel>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut query = bank_import::Entity::find()
            .filter(bank_import::Column::CompanyId.eq(company_id))
            .order_by_desc(bank_import::Column::ImportedAt)
            .order_by_desc(bank_import::Column::Id);

        if let Some(profile_id) = bank_profile_id {
            query = query.filter(bank_import::Column::BankProfileId.eq(profile_id));
        }

        if let Some(from_date) = from_date {
            if let Some(start) = from_date.and_hms_opt(0, 0, 0) {
                let start = Utc.from_utc_datetime(&start);
                query = query.filter(bank_import::Column::ImportedAt.gte(start));
            }
        }

        if let Some(to_date) = to_date {
            if let Some(end) = to_date.and_hms_milli_opt(23, 59, 59, 999) {
                let end = Utc.from_utc_datetime(&end);
                query = query.filter(bank_import::Column::ImportedAt.lte(end));
            }
        }

        if let Some(limit) = limit {
            query = query.limit(limit as u64);
        }

        if let Some(offset) = offset {
            query = query.offset(offset as u64);
        }

        let imports = query.all(db).await?;
        Ok(imports)
    }
}

#[derive(Default)]
pub struct BankMutation;

#[Object]
impl BankMutation {
    async fn create_bank_profile(
        &self,
        ctx: &Context<'_>,
        input: CreateBankProfileInput,
    ) -> FieldResult<BankProfileModel> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Ensure unique name per company
        let existing = bank_profile::Entity::find()
            .filter(bank_profile::Column::CompanyId.eq(input.company_id))
            .filter(bank_profile::Column::Name.eq(input.name.clone()))
            .one(db)
            .await?;

        if existing.is_some() {
            return Err("Банка с такова име вече съществува".into());
        }

        // TODO: Get real user id from auth context
        let mut model = BankProfileActiveModel::from(input);
        model.created_by = Set(Some(1));
        model.created_at = Set(Utc::now());
        model.updated_at = Set(Utc::now());

        let record = model.insert(db).await?;
        Ok(record)
    }

    async fn update_bank_profile(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateBankProfileInput,
    ) -> FieldResult<BankProfileModel> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let model = bank_profile::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or("Банковият профил не е намерен")?;

        let mut active: BankProfileActiveModel = model.into();

        if let Some(name) = input.name {
            if !name.trim().is_empty() {
                active.name = Set(name.trim().to_string());
            }
        }
        if let Some(iban_opt) = input.iban {
            active.iban = Set(iban_opt);
        }
        if let Some(account_id) = input.account_id {
            active.account_id = Set(account_id);
        }
        if let Some(buffer_account_id) = input.buffer_account_id {
            active.buffer_account_id = Set(buffer_account_id);
        }
        if let Some(currency_code) = input.currency_code {
            active.currency_code = Set(currency_code.to_uppercase());
        }
        if let Some(format) = input.import_format {
            active.import_format = Set(format.as_str().to_string());
        }
        if let Some(is_active) = input.is_active {
            active.is_active = Set(is_active);
        }
        if let Some(settings) = input.settings {
            active.settings = Set(settings);
        }

        active.updated_at = Set(Utc::now());

        let record = active.update(db).await?;
        Ok(record)
    }

    async fn import_bank_statement(
        &self,
        ctx: &Context<'_>,
        input: ImportBankStatementInput,
    ) -> FieldResult<BankImportSummaryPayload> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let profile = bank_profile::Entity::find_by_id(input.bank_profile_id)
            .one(db)
            .await?
            .ok_or("Банковият профил не е намерен")?;

        if let Some(company_id) = input.company_id {
            if company_id != profile.company_id {
                return Err("Банковият профил не принадлежи на избраната компания".into());
            }
        }

        if !profile.is_active {
            return Err("Банковият профил е деактивиран".into());
        }

        let file_name = input.file_name.trim();
        if file_name.is_empty() {
            return Err("Името на файла е задължително".into());
        }

        let file_bytes = decode_bank_document(&input.file_base64)?;
        if file_bytes.is_empty() {
            return Err("Файлът е празен".into());
        }

        let summary = BankImportService::import_statement(
            db,
            &profile,
            file_name,
            &file_bytes,
            input.created_by,
        )
        .await
        .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(BankImportSummaryPayload::from(summary))
    }

    async fn delete_bank_profile(&self, ctx: &Context<'_>, id: i32) -> FieldResult<bool> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Prevent deleting profiles that have imports
        let count = bank_import::Entity::find()
            .filter(bank_import::Column::BankProfileId.eq(id))
            .count(db)
            .await?;

        if count > 0 {
            return Err("Не може да изтриете банка с налична история на импорт".into());
        }

        let res = bank_profile::Entity::delete_by_id(id).exec(db).await?;
        Ok(res.rows_affected > 0)
    }
}

#[derive(InputObject)]
pub struct ImportBankStatementInput {
    pub bank_profile_id: i32,
    pub company_id: Option<i32>,
    pub file_name: String,
    /// Base64 съдържание на файла. Поддържа се и data URI (`data:<mime>;base64,....`).
    pub file_base64: String,
    pub created_by: Option<i32>,
}

#[derive(SimpleObject)]
pub struct BankImportSummaryPayload {
    pub bank_import: BankImportModel,
    pub transactions: i32,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub journal_entry_ids: Vec<i32>,
}

impl From<ImportSummary> for BankImportSummaryPayload {
    fn from(summary: ImportSummary) -> Self {
        let ImportSummary {
            transactions,
            journal_entry_ids,
            total_debit,
            total_credit,
            bank_import,
        } = summary;

        let transactions = i32::try_from(transactions).unwrap_or(i32::MAX);

        Self {
            bank_import,
            transactions,
            total_debit,
            total_credit,
            journal_entry_ids,
        }
    }
}

#[derive(InputObject)]
pub struct BankImportFilterInput {
    pub company_id: i32,
    pub bank_profile_id: Option<i32>,
    pub status: Option<BankImportStatus>,
}

impl BankMutation {
    async fn set_bank_profile_status(
        &self,
        ctx: &Context<'_>,
        id: i32,
        is_active: bool,
    ) -> FieldResult<BankProfileModel> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let model = bank_profile::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or("Банковият профил не е намерен")?;

        let mut active: BankProfileActiveModel = model.into();
        active.is_active = Set(is_active);
        active.updated_at = Set(Utc::now());

        let record = active.update(db).await?;
        Ok(record)
    }
}

fn decode_bank_document(encoded: &str) -> FieldResult<Vec<u8>> {
    let trimmed = encoded.trim();
    if trimmed.is_empty() {
        return Err("Файлът е празен".into());
    }

    let payload = trimmed
        .split_once(',')
        .map(|(_, data)| data)
        .unwrap_or(trimmed)
        .trim();

    if payload.is_empty() {
        return Err("Файлът е празен".into());
    }

    BASE64
        .decode(payload)
        .map_err(|err| async_graphql::Error::new(format!("Невалидно base64 съдържание: {}", err)))
}
