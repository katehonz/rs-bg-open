use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

use crate::entities::{
    account::{ActiveModel as AccountActiveModel, Entity as AccountEntity, Model as AccountModel},
    counterpart::{ActiveModel as CounterpartActiveModel, Entity as CounterpartEntity},
    vat_rate::{ActiveModel as VatRateActiveModel, Entity as VatRateEntity, Model as VatRateModel},
};

// ============================================================================
// Data Structures matching universal-schema.json
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UniversalData {
    pub version: String,
    pub export_date: DateTime<Utc>,
    pub company_info: CompanyInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_of_accounts: Option<Vec<Account>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_rates: Option<Vec<VatRate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counterparts: Option<Vec<Counterpart>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documents: Option<Vec<Document>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal_entries: Option<Vec<JournalEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_settings: Option<ImportSettings>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompanyInfo {
    pub name: String,
    pub eik: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_class: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_analytical: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_vat_applicable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_direction: Option<String>, // NONE, INPUT, OUTPUT, BOTH
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_quantities: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

impl AccountType {
    pub fn to_db_string(&self) -> String {
        match self {
            AccountType::Asset => "ASSET".to_string(),
            AccountType::Liability => "LIABILITY".to_string(),
            AccountType::Equity => "EQUITY".to_string(),
            AccountType::Revenue => "REVENUE".to_string(),
            AccountType::Expense => "EXPENSE".to_string(),
        }
    }

    pub fn from_db_string(s: &str) -> Result<Self> {
        match s {
            "ASSET" => Ok(AccountType::Asset),
            "LIABILITY" => Ok(AccountType::Liability),
            "EQUITY" => Ok(AccountType::Equity),
            "REVENUE" => Ok(AccountType::Revenue),
            "EXPENSE" => Ok(AccountType::Expense),
            _ => Err(anyhow!("Unknown account type: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VatRate {
    pub code: String,
    pub name: String,
    pub rate: String, // Store as string for JSON, convert to Decimal
    pub vat_direction: String, // NONE, INPUT, OUTPUT, BOTH
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<String>, // ISO date string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<String>, // ISO date string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Counterpart {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eik: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_person: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_vat_registered: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub document_type: String,
    pub transaction_type: String,
    pub document_number: String,
    pub document_date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_event_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_invoice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_rate: Option<f64>,
    pub counterpart: Counterpart,
    pub amounts: DocumentAmounts,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_details: Option<VatDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_items: Option<Vec<LineItem>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentAmounts {
    pub net_amount: f64,
    pub vat_amount: f64,
    pub total_amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VatDetails {
    pub vat_register: i32,
    pub vat_period: String,
    pub operations: Vec<VatOperation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VatOperation {
    pub operation_code: String,
    pub tax_base: f64,
    pub vat_rate: i32,
    pub vat_amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deductible_vat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_deductible_vat: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LineItem {
    pub description: String,
    pub net_amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_number: Option<String>,
    pub entry_date: NaiveDate,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_date: Option<NaiveDate>,
    pub lines: Vec<JournalLine>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JournalLine {
    pub account_code: String,
    pub debit: f64,
    pub credit: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counterpart_eik: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreign_debit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreign_credit: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ImportSettings {
    #[serde(default = "default_true")]
    pub validate_before_import: bool,
    #[serde(default = "default_true")]
    pub auto_create_counterparts: bool,
    #[serde(default = "default_false")]
    pub auto_create_accounts: bool,
    #[serde(default = "default_true")]
    pub skip_duplicates: bool,
    #[serde(default = "default_true")]
    pub skip_existing_accounts: bool,
    #[serde(default = "default_false")]
    pub update_existing: bool,
    #[serde(default = "default_true")]
    pub import_only_active: bool,
    #[serde(default = "default_false")]
    pub delete_existing_accounts: bool,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

// ============================================================================
// Import/Export Summary
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportSummary {
    pub success: bool,
    pub accounts_imported: usize,
    pub vat_rates_imported: usize,
    pub counterparts_imported: usize,
    pub documents_imported: usize,
    pub journal_entries_imported: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

// ============================================================================
// Universal Import/Export Service
// ============================================================================

pub struct UniversalImportExportService;

impl UniversalImportExportService {
    pub fn new() -> Self {
        Self
    }

    // ------------------------------------------------------------------------
    // Import Functions
    // ------------------------------------------------------------------------

    /// Import universal JSON data
    pub async fn import_json(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        json_data: &str,
    ) -> Result<ImportSummary> {
        let data: UniversalData = serde_json::from_str(json_data)?;

        let settings = data.import_settings.unwrap_or_default();

        let mut summary = ImportSummary {
            success: false,
            accounts_imported: 0,
            vat_rates_imported: 0,
            counterparts_imported: 0,
            documents_imported: 0,
            journal_entries_imported: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Import chart of accounts
        if let Some(accounts) = &data.chart_of_accounts {
            match self
                .import_accounts(db, company_id, accounts, &settings)
                .await
            {
                Ok(count) => summary.accounts_imported = count,
                Err(e) => summary.errors.push(format!("Accounts import error: {}", e)),
            }
        }

        // Import VAT rates
        if let Some(vat_rates) = &data.vat_rates {
            match self
                .import_vat_rates(db, company_id, vat_rates, &settings)
                .await
            {
                Ok(count) => summary.vat_rates_imported = count,
                Err(e) => summary.errors.push(format!("VAT rates import error: {}", e)),
            }
        }

        // Import counterparts
        if let Some(counterparts) = &data.counterparts {
            match self
                .import_counterparts(db, company_id, counterparts, &settings)
                .await
            {
                Ok(count) => summary.counterparts_imported = count,
                Err(e) => summary.errors.push(format!("Counterparts import error: {}", e)),
            }
        }

        // TODO: Import documents and journal entries
        if data.documents.is_some() {
            summary
                .warnings
                .push("Document import not yet implemented".to_string());
        }

        if data.journal_entries.is_some() {
            summary
                .warnings
                .push("Journal entries import not yet implemented".to_string());
        }

        summary.success = summary.errors.is_empty();

        Ok(summary)
    }

    /// Import chart of accounts
    pub async fn import_accounts(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        accounts: &[Account],
        settings: &ImportSettings,
    ) -> Result<usize> {
        // Delete all existing accounts if requested
        if settings.delete_existing_accounts {
            // Use raw SQL to disable foreign key checks temporarily
            // This allows us to delete accounts even if they have references
            use sea_orm::Statement;
            use sea_orm::ConnectionTrait;

            // Disable foreign key checks (PostgreSQL specific)
            db.execute(Statement::from_string(
                db.get_database_backend(),
                "SET CONSTRAINTS ALL DEFERRED;".to_owned(),
            ))
            .await?;

            // Delete all accounts for this company
            AccountEntity::delete_many()
                .filter(crate::entities::account::Column::CompanyId.eq(company_id))
                .exec(db)
                .await?;

            // Re-enable foreign key checks
            db.execute(Statement::from_string(
                db.get_database_backend(),
                "SET CONSTRAINTS ALL IMMEDIATE;".to_owned(),
            ))
            .await?;
        }

        let mut count = 0;

        for account in accounts {
            // Skip inactive accounts if configured
            if settings.import_only_active && !account.is_active.unwrap_or(true) {
                continue;
            }

            // Check if account already exists
            let existing = AccountEntity::find()
                .filter(crate::entities::account::Column::Code.eq(&account.code))
                .filter(crate::entities::account::Column::CompanyId.eq(company_id))
                .one(db)
                .await?;

            if existing.is_some() {
                if settings.skip_existing_accounts {
                    continue;
                }

                // Update existing account if update_existing is true
                if settings.update_existing {
                    let existing_model = existing.unwrap();
                    let mut existing_active: AccountActiveModel = existing_model.into();

                    existing_active.name = Set(account.name.clone());
                    existing_active.account_class = Set(account.account_class.unwrap_or(0));
                    existing_active.is_analytical = Set(account.is_analytical.unwrap_or(false));
                    existing_active.is_vat_applicable = Set(account.is_vat_applicable.unwrap_or(false));

                    let vat_direction = match account.vat_direction.as_deref() {
                        Some("NONE") => crate::entities::account::VatDirection::None,
                        Some("INPUT") => crate::entities::account::VatDirection::Input,
                        Some("OUTPUT") => crate::entities::account::VatDirection::Output,
                        Some("BOTH") => crate::entities::account::VatDirection::Both,
                        _ => crate::entities::account::VatDirection::None,
                    };
                    existing_active.vat_direction = Set(vat_direction);

                    existing_active.supports_quantities = Set(account.supports_quantities.unwrap_or(false));
                    existing_active.default_unit = Set(account.default_unit.clone());
                    existing_active.is_active = Set(account.is_active.unwrap_or(true));

                    existing_active.update(db).await?;
                    count += 1;
                    continue;
                }
            }

            // Parse vat_direction from string
            let vat_direction = match account.vat_direction.as_deref() {
                Some("NONE") => crate::entities::account::VatDirection::None,
                Some("INPUT") => crate::entities::account::VatDirection::Input,
                Some("OUTPUT") => crate::entities::account::VatDirection::Output,
                Some("BOTH") => crate::entities::account::VatDirection::Both,
                _ => crate::entities::account::VatDirection::None,
            };

            // Determine account_type enum
            let account_type_enum = match account.account_type {
                AccountType::Asset => crate::entities::account::AccountType::Asset,
                AccountType::Liability => crate::entities::account::AccountType::Liability,
                AccountType::Equity => crate::entities::account::AccountType::Equity,
                AccountType::Revenue => crate::entities::account::AccountType::Revenue,
                AccountType::Expense => crate::entities::account::AccountType::Expense,
            };

            // Create new account
            let new_account = AccountActiveModel {
                company_id: Set(company_id),
                code: Set(account.code.clone()),
                name: Set(account.name.clone()),
                account_type: Set(account_type_enum),
                account_class: Set(account.account_class.unwrap_or(0)),
                parent_id: Set(account.parent_id),
                is_analytical: Set(account.is_analytical.unwrap_or(false)),
                is_vat_applicable: Set(account.is_vat_applicable.unwrap_or(false)),
                vat_direction: Set(vat_direction),
                supports_quantities: Set(account.supports_quantities.unwrap_or(false)),
                default_unit: Set(account.default_unit.clone()),
                is_active: Set(account.is_active.unwrap_or(true)),
                ..Default::default()
            };

            new_account.insert(db).await?;
            count += 1;
        }

        Ok(count)
    }

    /// Import VAT rates
    async fn import_vat_rates(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        vat_rates: &[VatRate],
        settings: &ImportSettings,
    ) -> Result<usize> {
        let mut count = 0;

        for vat_rate in vat_rates {
            // Skip inactive rates if configured
            if settings.import_only_active && !vat_rate.is_active.unwrap_or(true) {
                continue;
            }

            // Check if VAT rate already exists
            let existing = VatRateEntity::find()
                .filter(crate::entities::vat_rate::Column::Code.eq(&vat_rate.code))
                .filter(crate::entities::vat_rate::Column::CompanyId.eq(company_id))
                .one(db)
                .await?;

            if existing.is_some() {
                if settings.skip_duplicates {
                    continue;
                }
                // TODO: Implement update logic
            }

            // Parse rate from string to Decimal
            let rate = Decimal::from_str(&vat_rate.rate)
                .map_err(|e| anyhow!("Invalid rate value '{}': {}", vat_rate.rate, e))?;

            // Parse vat_direction from string
            let vat_direction = match vat_rate.vat_direction.as_str() {
                "NONE" => crate::entities::account::VatDirection::None,
                "INPUT" => crate::entities::account::VatDirection::Input,
                "OUTPUT" => crate::entities::account::VatDirection::Output,
                "BOTH" => crate::entities::account::VatDirection::Both,
                _ => crate::entities::account::VatDirection::None,
            };

            // Parse dates
            let valid_from = if let Some(date_str) = &vat_rate.valid_from {
                NaiveDate::from_str(date_str)
                    .map_err(|e| anyhow!("Invalid valid_from date '{}': {}", date_str, e))?
            } else {
                // Default to today if not provided
                chrono::Utc::now().date_naive()
            };

            let valid_to = if let Some(date_str) = &vat_rate.valid_to {
                Some(NaiveDate::from_str(date_str)
                    .map_err(|e| anyhow!("Invalid valid_to date '{}': {}", date_str, e))?)
            } else {
                None
            };

            // Create new VAT rate
            let new_vat_rate = VatRateActiveModel {
                company_id: Set(company_id),
                code: Set(vat_rate.code.clone()),
                name: Set(vat_rate.name.clone()),
                rate: Set(rate),
                vat_direction: Set(vat_direction),
                valid_from: Set(valid_from),
                valid_to: Set(valid_to),
                is_active: Set(vat_rate.is_active.unwrap_or(true)),
                ..Default::default()
            };

            new_vat_rate.insert(db).await?;
            count += 1;
        }

        Ok(count)
    }

    /// Import counterparts
    async fn import_counterparts(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        counterparts: &[Counterpart],
        settings: &ImportSettings,
    ) -> Result<usize> {
        let mut count = 0;

        for counterpart in counterparts {
            // Skip inactive counterparts if configured
            if settings.import_only_active && !counterpart.is_active.unwrap_or(true) {
                continue;
            }

            // Check if counterpart already exists (by EIK if available)
            if let Some(eik) = &counterpart.eik {
                let existing = CounterpartEntity::find()
                    .filter(crate::entities::counterpart::Column::Eik.eq(eik))
                    .filter(crate::entities::counterpart::Column::CompanyId.eq(company_id))
                    .one(db)
                    .await?;

                if existing.is_some() && settings.skip_duplicates {
                    continue;
                }
            }

            // Create new counterpart
            let new_counterpart = CounterpartActiveModel {
                company_id: Set(company_id),
                name: Set(counterpart.name.clone()),
                eik: Set(counterpart.eik.clone()),
                vat_number: Set(counterpart.vat_number.clone()),
                address: Set(counterpart.address.clone()),
                city: Set(counterpart.city.clone()),
                country: Set(counterpart.country.clone()),
                phone: Set(counterpart.phone.clone()),
                email: Set(counterpart.email.clone()),
                contact_person: Set(counterpart.contact_person.clone()),
                is_vat_registered: Set(counterpart.is_vat_registered.unwrap_or(false)),
                is_active: Set(counterpart.is_active.unwrap_or(true)),
                ..Default::default()
            };

            new_counterpart.insert(db).await?;
            count += 1;
        }

        Ok(count)
    }

    // ------------------------------------------------------------------------
    // Export Functions
    // ------------------------------------------------------------------------

    /// Export all data to universal JSON format
    pub async fn export_all(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        include_documents: bool,
        include_journal_entries: bool,
    ) -> Result<UniversalData> {
        // Get company info
        let company = crate::entities::company::Entity::find_by_id(company_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Company not found"))?;

        let company_info = CompanyInfo {
            name: company.name,
            eik: company.eik,
            vat_number: company.vat_number,
            address: company.address,
            city: company.city,
            country: company.country,
        };

        // Export chart of accounts
        let chart_of_accounts = self.export_accounts(db, company_id).await?;

        // Export VAT rates
        let vat_rates = self.export_vat_rates(db, company_id).await?;

        // Export counterparts
        let counterparts = self.export_counterparts(db, company_id).await?;

        // TODO: Export documents and journal entries if requested

        Ok(UniversalData {
            version: "2.0".to_string(),
            export_date: Utc::now(),
            company_info,
            chart_of_accounts: Some(chart_of_accounts),
            vat_rates: Some(vat_rates),
            counterparts: Some(counterparts),
            documents: None,
            journal_entries: None,
            import_settings: Some(ImportSettings::default()),
        })
    }

    /// Export chart of accounts
    pub async fn export_accounts(&self, db: &DatabaseConnection, company_id: i32) -> Result<Vec<Account>> {
        let accounts = AccountEntity::find()
            .filter(crate::entities::account::Column::CompanyId.eq(company_id))
            .filter(crate::entities::account::Column::IsActive.eq(true))
            .all(db)
            .await?;

        let mut result = Vec::new();
        for acc in accounts {
            // Convert account_type enum to JSON enum
            let account_type = match acc.account_type {
                crate::entities::account::AccountType::Asset => AccountType::Asset,
                crate::entities::account::AccountType::Liability => AccountType::Liability,
                crate::entities::account::AccountType::Equity => AccountType::Equity,
                crate::entities::account::AccountType::Revenue => AccountType::Revenue,
                crate::entities::account::AccountType::Expense => AccountType::Expense,
            };

            // Convert vat_direction enum to string
            let vat_direction = match acc.vat_direction {
                crate::entities::account::VatDirection::None => Some("NONE".to_string()),
                crate::entities::account::VatDirection::Input => Some("INPUT".to_string()),
                crate::entities::account::VatDirection::Output => Some("OUTPUT".to_string()),
                crate::entities::account::VatDirection::Both => Some("BOTH".to_string()),
            };

            result.push(Account {
                code: acc.code,
                name: acc.name,
                account_type,
                account_class: Some(acc.account_class),
                parent_id: acc.parent_id,
                is_analytical: Some(acc.is_analytical),
                is_vat_applicable: Some(acc.is_vat_applicable),
                vat_direction,
                supports_quantities: Some(acc.supports_quantities),
                default_unit: acc.default_unit,
                is_active: Some(acc.is_active),
            });
        }

        Ok(result)
    }

    /// Export VAT rates
    pub async fn export_vat_rates(&self, db: &DatabaseConnection, company_id: i32) -> Result<Vec<VatRate>> {
        let vat_rates = VatRateEntity::find()
            .filter(crate::entities::vat_rate::Column::CompanyId.eq(company_id))
            .filter(crate::entities::vat_rate::Column::IsActive.eq(true))
            .all(db)
            .await?;

        let mut result = Vec::new();
        for rate in vat_rates {
            // Convert vat_direction enum to string
            let vat_direction = match rate.vat_direction {
                crate::entities::account::VatDirection::None => "NONE".to_string(),
                crate::entities::account::VatDirection::Input => "INPUT".to_string(),
                crate::entities::account::VatDirection::Output => "OUTPUT".to_string(),
                crate::entities::account::VatDirection::Both => "BOTH".to_string(),
            };

            result.push(VatRate {
                code: rate.code,
                name: rate.name,
                rate: rate.rate.to_string(),
                vat_direction,
                valid_from: Some(rate.valid_from.to_string()),
                valid_to: rate.valid_to.map(|d| d.to_string()),
                is_active: Some(rate.is_active),
            });
        }

        Ok(result)
    }

    /// Export counterparts
    pub async fn export_counterparts(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
    ) -> Result<Vec<Counterpart>> {
        let counterparts = CounterpartEntity::find()
            .filter(crate::entities::counterpart::Column::CompanyId.eq(company_id))
            .filter(crate::entities::counterpart::Column::IsActive.eq(true))
            .all(db)
            .await?;

        let mut result = Vec::new();
        for cp in counterparts {
            result.push(Counterpart {
                name: cp.name,
                eik: cp.eik,
                vat_number: cp.vat_number,
                address: cp.address,
                city: cp.city,
                country: cp.country,
                phone: cp.phone,
                email: cp.email,
                contact_person: cp.contact_person,
                is_vat_registered: Some(cp.is_vat_registered),
                is_active: Some(cp.is_active),
                account_code: None, // TODO: Add if available in model
            });
        }

        Ok(result)
    }
}
