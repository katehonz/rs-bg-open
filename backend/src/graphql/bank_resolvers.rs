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
    bank_import, bank_profile, counterpart, entry_line, BankImportModel, BankImportStatus,
    BankProfileActiveModel, BankProfileModel, CreateBankProfileInput, UpdateBankProfileInput,
};
use crate::services::bank_imports::{BankImportService, ImportSummary};
use crate::services::bank_transaction_parser::{BankTransactionParser, ParsedTransactionData};
use crate::services::contragent::ContragentService;

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

        // Note: Date filters are NOT applied to imported_at here
        // They will be applied to journal entries' document_date later
        // This allows showing imports that contain transactions in the date range

        if let Some(limit) = limit {
            query = query.limit(limit as u64);
        }

        if let Some(offset) = offset {
            query = query.offset(offset as u64);
        }

        let imports = query.all(db).await?;

        // Filter out imports where ALL journal entries have been deleted
        // OR where no journal entries match the date range (if date filter is applied)
        use sea_orm::{ConnectionTrait, DbBackend, Statement};
        let mut filtered_imports = Vec::new();

        // Build SQL query once - check if at least one entry EXISTS and optionally matches date range
        let mut sql_query = String::from(
            "SELECT EXISTS(SELECT 1 FROM journal_entries WHERE id = ANY($1::int[])"
        );

        if from_date.is_some() {
            sql_query.push_str(" AND document_date >= $2");
        }

        if to_date.is_some() {
            let param_num = if from_date.is_some() { 3 } else { 2 };
            sql_query.push_str(&format!(" AND document_date <= ${}", param_num));
        }

        sql_query.push(')');

        for import in imports {
            if let Some(entry_ids_value) = &import.journal_entry_ids {
                // Parse journal_entry_ids JSON array
                let entry_ids: Vec<i32> = serde_json::from_value(entry_ids_value.clone())
                    .unwrap_or_default();

                if entry_ids.is_empty() {
                    // No entries referenced - skip import
                    continue;
                }

                // Build params for the query
                let mut params: Vec<sea_orm::Value> = vec![entry_ids.into()];

                if let Some(from) = from_date {
                    params.push(from.into());
                }

                if let Some(to) = to_date {
                    params.push(to.into());
                }

                // Check if at least one entry exists and matches criteria
                let result = db.query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    &sql_query,
                    params
                )).await;

                if let Ok(Some(row)) = result {
                    if let Ok(exists) = row.try_get::<bool>("", "exists") {
                        if exists {
                            // At least one entry exists and matches criteria
                            filtered_imports.push(import);
                        }
                        // else: no entries match - skip import
                    }
                }
                // On error, skip import (don't risk showing bad data)
            }
            // If no journal_entry_ids field, skip import
        }

        Ok(filtered_imports)
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

    /// Парсва описание на банкова транзакция и извлича контрагент с AI
    async fn parse_bank_transaction_description(
        &self,
        ctx: &Context<'_>,
        description: String,
    ) -> FieldResult<Option<ParsedBankTransactionPayload>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let contragent_service = Arc::new(ContragentService::new());
        let parser = BankTransactionParser::new(contragent_service.clone());

        let parsed = parser
            .parse_transaction_description(db, &description)
            .await
            .map_err(|e| async_graphql::Error::new(format!("Грешка при парсване: {}", e)))?;

        if let Some(data) = parsed {
            // Опит да намерим съществуващ контрагент по ЕИК или име
            let mut counterpart_id: Option<i32> = None;

            if let Some(ref eik) = data.eik {
                if !eik.is_empty() {
                    let existing = counterpart::Entity::find()
                        .filter(counterpart::Column::Eik.eq(eik))
                        .one(db)
                        .await?;

                    if let Some(cp) = existing {
                        counterpart_id = Some(cp.id);
                    }
                }
            }

            if counterpart_id.is_none() {
                if let Some(ref name) = data.counterpart_name {
                    if !name.is_empty() {
                        let existing = counterpart::Entity::find()
                            .filter(counterpart::Column::Name.eq(name))
                            .one(db)
                            .await?;

                        if let Some(cp) = existing {
                            counterpart_id = Some(cp.id);
                        }
                    }
                }
            }

            Ok(Some(ParsedBankTransactionPayload {
                counterpart_name: data.counterpart_name,
                eik: data.eik,
                iban: data.iban,
                bic: data.bic,
                city: data.city,
                transaction_type: data.transaction_type,
                confidence: data.confidence,
                found_counterpart_id: counterpart_id,
            }))
        } else {
            Ok(None)
        }
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

/// Резултат от парсване на описание на транзакция
#[derive(SimpleObject)]
pub struct ParsedBankTransactionPayload {
    pub counterpart_name: Option<String>,
    pub eik: Option<String>,
    pub iban: Option<String>,
    pub bic: Option<String>,
    pub city: Option<String>,
    pub transaction_type: Option<String>,
    pub confidence: f64,
    /// ID на намерен контрагент в базата (ако има такъв)
    pub found_counterpart_id: Option<i32>,
}

/// Резултат от автоматично парсване на банков импорт
#[derive(SimpleObject)]
pub struct BankImportParseResult {
    pub total_entries: i32,
    pub parsed_count: i32,
    pub mapped_count: i32,
    pub failed_count: i32,
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
