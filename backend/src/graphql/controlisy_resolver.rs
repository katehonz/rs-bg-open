use crate::services::controlisy::ControlisyService;
use crate::services::nap_export::NapExportService;
use async_graphql::*;
use chrono::{DateTime, NaiveDateTime, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use serde_json;

#[derive(Default)]
pub struct ControlisyQuery;

#[Object]
impl ControlisyQuery {
    // NOTE: XML parsing moved to REST API at /api/controlisy/parse
    // Use REST API for all XML/JSON file operations

    async fn get_controlisy_import(
        &self,
        ctx: &Context<'_>,
        import_id: i32,
    ) -> Result<ControlisyImport> {
        let db = ctx.data::<DatabaseConnection>()?;

        let result = db.query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT status, imported_documents, imported_contractors FROM controlisy_imports WHERE id = $1",
            vec![import_id.into()],
        ))
        .await
        .map_err(|e| Error::new(format!("Database error: {}", e)))?
        .ok_or_else(|| Error::new("Import not found"))?;

        let status: String = result
            .try_get("", "status")
            .unwrap_or("pending".to_string());
        let imported_documents: i32 = result.try_get("", "imported_documents").unwrap_or(0);
        let imported_contractors: i32 = result.try_get("", "imported_contractors").unwrap_or(0);

        Ok(ControlisyImport {
            id: import_id,
            status,
            imported_documents,
            imported_contractors,
        })
    }

    async fn list_controlisy_imports(
        &self,
        ctx: &Context<'_>,
        _company_id: i32,
    ) -> Result<Vec<ControlisyImportSummary>> {
        let _db = ctx.data::<DatabaseConnection>()?;

        // For now return empty list - we'll implement proper functionality later
        // when we have actual import records in the database
        Ok(vec![])
    }

    async fn get_staged_import_data(&self, ctx: &Context<'_>, import_id: i32) -> Result<String> {
        let db = ctx.data::<DatabaseConnection>()?;

        let result = db.query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT parsed_data FROM controlisy_imports WHERE id = $1 AND status IN ('staged', 'reviewed')",
            vec![import_id.into()],
        ))
        .await
        .map_err(|e| Error::new(format!("Database error: {}", e)))? 
        .ok_or_else(|| Error::new("Import not found or not in staging mode"))?;

        let parsed_data: serde_json::Value = result
            .try_get("", "parsed_data")
            .map_err(|e| Error::new(format!("Failed to get parsed data: {}", e)))?;

        serde_json::to_string_pretty(&parsed_data)
            .map_err(|e| Error::new(format!("Failed to serialize data: {}", e)))
    }
}

#[derive(Default)]
pub struct ControlisyMutation;

#[Object]
impl ControlisyMutation {
    // NOTE: All XML/JSON operations moved to REST API
    // XML import: POST /api/controlisy/import
    // Process import: POST /api/controlisy/process/{import_id}
    // Update staging: PUT /api/controlisy/update/{import_id}
    // Mark reviewed: POST /api/controlisy/review/{import_id}

    async fn generate_vat_files_for_nap(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        year: i32,
        month: i32,
    ) -> Result<VatFilesResult> {
        let db = ctx.data::<DatabaseConnection>()?;

        // Generate VIES format files for NAP
        let vies_files = NapExportService::generate_vies_files(db, company_id, year, month)
            .await
            .map_err(|e| Error::new(format!("Failed to generate VAT files: {}", e)))?;

        use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

        Ok(VatFilesResult {
            success: true,
            deklar_content: BASE64.encode(&vies_files.deklar),
            pokupki_content: BASE64.encode(&vies_files.pokupki),
            prodagbi_content: BASE64.encode(&vies_files.prodagbi),
        })
    }
}

#[derive(SimpleObject)]
pub struct ControlisyImport {
    pub id: i32,
    pub status: String,
    pub imported_documents: i32,
    pub imported_contractors: i32,
}

#[derive(SimpleObject)]
pub struct ControlisyImportSummary {
    pub id: i32,
    pub file_name: String,
    pub document_type: String,
    pub status: String,
    pub import_date: String,
    pub imported_documents: i32,
    pub imported_contractors: i32,
    pub processed: bool,
    pub reviewed_at: Option<String>,
    pub processed_at: Option<String>,
}

#[derive(SimpleObject)]
pub struct ControlisyImportResult {
    pub success: bool,
    pub import_id: i32,
    pub message: String,
    pub documents_count: i32,
    pub contractors_count: i32,
}

#[derive(SimpleObject)]
pub struct ProcessResult {
    pub success: bool,
    pub message: String,
}

#[derive(SimpleObject)]
pub struct VatFilesResult {
    pub success: bool,
    pub deklar_content: String,
    pub pokupki_content: String,
    pub prodagbi_content: String,
}
