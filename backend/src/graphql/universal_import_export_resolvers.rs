use async_graphql::{Context, FieldResult, InputObject, Object, SimpleObject};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::services::universal_import_export::{
    ImportSummary as ServiceImportSummary, UniversalImportExportService,
};

// ============================================================================
// GraphQL Types
// ============================================================================

#[derive(SimpleObject, Serialize, Deserialize)]
pub struct ImportSummary {
    pub success: bool,
    pub accounts_imported: i32,
    pub vat_rates_imported: i32,
    pub counterparts_imported: i32,
    pub documents_imported: i32,
    pub journal_entries_imported: i32,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl From<ServiceImportSummary> for ImportSummary {
    fn from(summary: ServiceImportSummary) -> Self {
        Self {
            success: summary.success,
            accounts_imported: summary.accounts_imported as i32,
            vat_rates_imported: summary.vat_rates_imported as i32,
            counterparts_imported: summary.counterparts_imported as i32,
            documents_imported: summary.documents_imported as i32,
            journal_entries_imported: summary.journal_entries_imported as i32,
            errors: summary.errors,
            warnings: summary.warnings,
        }
    }
}

#[derive(InputObject)]
pub struct UniversalImportInput {
    pub company_id: i32,
    pub json_data: String,
}

#[derive(InputObject)]
pub struct UniversalExportInput {
    pub company_id: i32,
    pub include_documents: Option<bool>,
    pub include_journal_entries: Option<bool>,
}

// ============================================================================
// Query
// ============================================================================

#[derive(Default)]
pub struct UniversalImportExportQuery;

#[Object]
impl UniversalImportExportQuery {
    /// Get export schema documentation
    async fn export_schema_version(&self) -> &str {
        "2.0"
    }

    /// Get supported import formats
    async fn supported_import_formats(&self) -> Vec<String> {
        vec!["JSON".to_string()]
    }
}

// ============================================================================
// Mutation
// ============================================================================

#[derive(Default)]
pub struct UniversalImportExportMutation;

#[Object]
impl UniversalImportExportMutation {
    /// Import universal JSON data (chart of accounts, VAT rates, counterparts)
    async fn import_universal_json(
        &self,
        ctx: &Context<'_>,
        input: UniversalImportInput,
    ) -> FieldResult<ImportSummary> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let service = UniversalImportExportService::new();

        let summary = service
            .import_json(db, input.company_id, &input.json_data)
            .await
            .map_err(|e| format!("Import failed: {}", e))?;

        Ok(ImportSummary::from(summary))
    }

    /// Export all data to universal JSON format
    async fn export_universal_json(
        &self,
        ctx: &Context<'_>,
        input: UniversalExportInput,
    ) -> FieldResult<String> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let service = UniversalImportExportService::new();

        let data = service
            .export_all(
                db,
                input.company_id,
                input.include_documents.unwrap_or(false),
                input.include_journal_entries.unwrap_or(false),
            )
            .await
            .map_err(|e| format!("Export failed: {}", e))?;

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("JSON serialization failed: {}", e))?;

        Ok(json)
    }

    /// Export chart of accounts only
    async fn export_chart_of_accounts(&self, ctx: &Context<'_>, company_id: i32) -> FieldResult<String> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let service = UniversalImportExportService::new();

        let accounts = service
            .export_accounts(db, company_id)
            .await
            .map_err(|e| format!("Export failed: {}", e))?;

        let json = serde_json::to_string_pretty(&accounts)
            .map_err(|e| format!("JSON serialization failed: {}", e))?;

        Ok(json)
    }

    /// Export VAT rates only
    async fn export_vat_rates(&self, ctx: &Context<'_>, company_id: i32) -> FieldResult<String> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let service = UniversalImportExportService::new();

        let vat_rates = service
            .export_vat_rates(db, company_id)
            .await
            .map_err(|e| format!("Export failed: {}", e))?;

        let json = serde_json::to_string_pretty(&vat_rates)
            .map_err(|e| format!("JSON serialization failed: {}", e))?;

        Ok(json)
    }

    /// Export counterparts only
    async fn export_counterparts(&self, ctx: &Context<'_>, company_id: i32) -> FieldResult<String> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let service = UniversalImportExportService::new();

        let counterparts = service
            .export_counterparts(db, company_id)
            .await
            .map_err(|e| format!("Export failed: {}", e))?;

        let json = serde_json::to_string_pretty(&counterparts)
            .map_err(|e| format!("JSON serialization failed: {}", e))?;

        Ok(json)
    }

    /// Import chart of accounts only from JSON array
    async fn import_chart_of_accounts(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        json_data: String,
    ) -> FieldResult<ImportSummary> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let service = UniversalImportExportService::new();

        // Parse the JSON as array of accounts
        let accounts: Vec<crate::services::universal_import_export::Account> =
            serde_json::from_str(&json_data)
                .map_err(|e| format!("Invalid JSON format: {}", e))?;

        // Enable delete_existing_accounts for chart import
        let settings = crate::services::universal_import_export::ImportSettings {
            validate_before_import: true,
            auto_create_counterparts: false,
            auto_create_accounts: false,
            skip_duplicates: true,
            skip_existing_accounts: false,
            update_existing: false,
            import_only_active: true,
            delete_existing_accounts: true, // Delete existing accounts before import
        };

        let count = service
            .import_accounts(db, company_id, &accounts, &settings)
            .await
            .map_err(|e| format!("Import failed: {}", e))?;

        Ok(ImportSummary {
            success: true,
            accounts_imported: count as i32,
            vat_rates_imported: 0,
            counterparts_imported: 0,
            documents_imported: 0,
            journal_entries_imported: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}
