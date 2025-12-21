use async_graphql::{Context, InputObject, Object, Result, SimpleObject};
use chrono::NaiveDate;
use sea_orm::DatabaseConnection;

use crate::entities::saft::{SafTExportRequest, SafTFileType};
use crate::services::saft_service_v2::SafTServiceV2;

#[derive(Default)]
pub struct SafTMutation;

#[derive(InputObject)]
pub struct SafTExportInput {
    pub company_id: i32,
    pub period_start: i32, // Month 1-12
    pub period_start_year: i32,
    pub period_end: i32, // Month 1-12
    pub period_end_year: i32,
    pub file_type: String,            // "Annual", "Monthly", "OnDemand"
    pub tax_accounting_basis: String, // "A", "P", "BANK", "INSURANCE"
}

#[derive(SimpleObject)]
pub struct SafTExportResult {
    pub success: bool,
    pub file_content: Option<String>,
    pub file_name: String,
    pub error_message: Option<String>,
}

#[Object]
impl SafTMutation {
    /// Export SAF-T file for the specified company and period
    async fn export_saft(
        &self,
        ctx: &Context<'_>,
        input: SafTExportInput,
    ) -> Result<SafTExportResult> {
        let db = ctx.data::<DatabaseConnection>()?;
        let saft_service = SafTServiceV2::new(db.clone());

        let file_type = match input.file_type.as_str() {
            "Annual" => SafTFileType::Annual,
            "Monthly" => SafTFileType::Monthly,
            "OnDemand" => SafTFileType::OnDemand,
            _ => return Err("Invalid file type. Must be Annual, Monthly or OnDemand".into()),
        };

        let request = SafTExportRequest {
            company_id: input.company_id,
            period_start: input.period_start,
            period_start_year: input.period_start_year,
            period_end: input.period_end,
            period_end_year: input.period_end_year,
            file_type,
            tax_accounting_basis: input.tax_accounting_basis.clone(),
        };

        match saft_service.generate_saft(request).await {
            Ok(xml_content) => {
                let file_name = format!(
                    "saft_{}_{}_{}_to_{}_{}.xml",
                    input.company_id,
                    input.period_start_year,
                    input.period_start,
                    input.period_end_year,
                    input.period_end
                );

                Ok(SafTExportResult {
                    success: true,
                    file_content: Some(xml_content),
                    file_name,
                    error_message: None,
                })
            }
            Err(e) => {
                let error_message = format!("SAF-T export failed: {}", e);

                Ok(SafTExportResult {
                    success: false,
                    file_content: None,
                    file_name: "".to_string(),
                    error_message: Some(error_message),
                })
            }
        }
    }
}

#[derive(Default)]
pub struct SafTQuery;

#[derive(SimpleObject)]
pub struct SafTValidationResult {
    pub is_valid: bool,
    pub validation_errors: Vec<String>,
    pub file_size_bytes: i32,
    pub number_of_transactions: i32,
}

#[Object]
impl SafTQuery {
    /// Validate SAF-T export parameters before generating the file
    async fn validate_saft_export(
        &self,
        ctx: &Context<'_>,
        input: SafTExportInput,
    ) -> Result<SafTValidationResult> {
        let db = ctx.data::<DatabaseConnection>()?;
        let mut validation_errors = Vec::new();

        // Basic validation
        if input.period_start < 1 || input.period_start > 12 {
            validation_errors.push("Period start must be between 1 and 12".to_string());
        }

        if input.period_end < 1 || input.period_end > 12 {
            validation_errors.push("Period end must be between 1 and 12".to_string());
        }

        if input.period_start_year > input.period_end_year
            || (input.period_start_year == input.period_end_year
                && input.period_start > input.period_end)
        {
            validation_errors.push("Start period must be before end period".to_string());
        }

        let valid_file_types = ["Annual", "Monthly", "OnDemand"];
        if !valid_file_types.contains(&input.file_type.as_str()) {
            validation_errors
                .push("Invalid file type. Must be Annual, Monthly or OnDemand".to_string());
        }

        let valid_tax_basis = ["A", "P", "BANK", "INSURANCE"];
        if !valid_tax_basis.contains(&input.tax_accounting_basis.as_str()) {
            validation_errors.push("Invalid tax accounting basis".to_string());
        }

        // Check if company exists
        use crate::entities::company::Entity as CompanyEntity;
        let company_exists = CompanyEntity::find_by_id(input.company_id)
            .one(db)
            .await?
            .is_some();

        if !company_exists {
            validation_errors.push("Company not found".to_string());
        }

        // Count transactions in the period (for estimation)
        use crate::entities::journal_entry::Entity as JournalEntryEntity;
        use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

        // Calculate date range for transaction count
        let start_date =
            NaiveDate::from_ymd_opt(input.period_start_year, input.period_start as u32, 1).unwrap();
        let end_date = if input.period_end == 12 {
            NaiveDate::from_ymd_opt(input.period_end_year, 12, 31).unwrap()
        } else {
            NaiveDate::from_ymd_opt(input.period_end_year, (input.period_end + 1) as u32, 1)
                .unwrap()
                .pred_opt()
                .unwrap()
        };

        let transaction_count = JournalEntryEntity::find()
            .filter(crate::entities::journal_entry::Column::CompanyId.eq(input.company_id))
            .filter(
                crate::entities::journal_entry::Column::DocumentDate.between(start_date, end_date),
            )
            .count(db)
            .await? as i32;

        // Estimate file size (rough calculation: ~1KB per transaction)
        let estimated_file_size = transaction_count * 1000;

        Ok(SafTValidationResult {
            is_valid: validation_errors.is_empty(),
            validation_errors,
            file_size_bytes: estimated_file_size,
            number_of_transactions: transaction_count,
        })
    }

    /// Get SAF-T export history for a company
    async fn get_saft_export_history(
        &self,
        _ctx: &Context<'_>,
        _company_id: i32,
        _limit: Option<i32>,
    ) -> Result<Vec<SafTExportHistoryEntry>> {
        // TODO: Implement export history tracking in database
        // For now, return empty list
        Ok(vec![])
    }
}

#[derive(SimpleObject)]
pub struct SafTExportHistoryEntry {
    pub id: i32,
    pub company_id: i32,
    pub export_date: chrono::DateTime<chrono::Utc>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub period_type: String,
    pub file_content_type: String,
    pub file_name: String,
    pub file_size_bytes: i32,
    pub status: String, // "SUCCESS" | "FAILED" | "PROCESSING"
    pub created_by: i32,
}
