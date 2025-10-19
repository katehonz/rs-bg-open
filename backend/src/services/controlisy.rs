use anyhow::Result;
use chrono::NaiveDate;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use rust_decimal::Decimal;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlisyData {
    pub contractors: Vec<Contractor>,
    pub documents: Vec<Document>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contractor {
    pub ca_contractor_id: String,
    pub contractor_name: String,
    pub contractor_eik: String,
    pub contractor_vat_number: String,
    pub contractor_inside_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    #[serde(with = "date_format")]
    pub accounting_month: NaiveDate,
    #[serde(with = "date_format")]
    pub vat_month: NaiveDate,
    #[serde(with = "date_format_option")]
    pub maturity: Option<NaiveDate>,
    pub doc_ident: String,
    #[serde(with = "date_format")]
    pub document_date: NaiveDate,
    pub document_number: String,
    pub ca_doc_id: String,
    pub reason: String,
    pub remark: String,
    pub net_amount_bgn: Decimal,
    pub vat_amount_bgn: Decimal,
    pub total_amount_bgn: Decimal,
    pub ca_vat_operation_id: String,
    pub ca_doc_type_id: String,
    pub ca_contractor_id: String,
    pub accountings: Vec<Accounting>,
    pub vat_data: Option<VatData>,
    pub rel_doc: Option<RelDoc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accounting {
    pub amount_bgn: Decimal,
    pub ca_vat_operation_id: String,
    pub accounting_details: Vec<AccountingDetail>,
    pub vat: Option<Vat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingDetail {
    pub direction: String, // Debit or Credit
    pub currency: String,
    pub currency_amount: Option<Decimal>,
    pub unit: String,
    pub quantity: Option<Decimal>,
    pub account_number: String,
    pub account_name: String,
    // Без аналитични признаци (accountItem1-4)
    pub contractor_name: String,
    pub contractor_eik: String,
    pub contractor_vat_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vat {
    pub tax_base: Decimal,
    pub vat_rate: Decimal,
    pub vat_amount_bgn: Decimal,
    pub ca_vat_operation_id: String,
    pub vat_operation_iden: String,
    pub vat_operation_ident_name: String,
    pub vat_operation_name: String,
    pub vat_operation_additional_iden: String,
    pub vat_operation_additional_name: String,
    pub vat_operation_detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VatData {
    pub vat_register: String,
    pub contractor_name: String,
    pub contractor_vat_number: String,
    pub vat: Vec<Vat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelDoc {
    pub ca_rel_document_id: String,
    pub rel_document_number: String,
    #[serde(with = "date_format")]
    pub rel_document_date: NaiveDate,
    pub contractor_name: String,
    pub contractor_eik: String,
    pub contractor_vat_number: String,
}

pub struct ControlisyService;

impl ControlisyService {
    /// Map old Controlisy numeric VAT operation codes to new NAP text codes
    ///
    /// Controlisy uses numeric codes (1, 2, 3) while NAP specification requires
    /// text codes (пок09, пок10, пок12, про11, etc.)
    fn map_vat_operation_code(
        old_code: &str,
        document_type: &str,
        vat_amount: Decimal,
    ) -> Option<String> {
        match document_type {
            "purchase" => {
                // Purchase operations mapping
                match old_code {
                    "1" => Some("пок10".to_string()), // Получени доставки с право на пълен ДК
                    "2" => Some("пок12".to_string()), // Получени доставки с право на частичен ДК
                    "3" => {
                        // No VAT or no credit
                        if vat_amount > Decimal::ZERO {
                            Some("пок09".to_string()) // Доставки БЕЗ право на ДК (но с ДДС)
                        } else {
                            Some("0".to_string()) // Не влиза в дневник (без ДДС)
                        }
                    }
                    "4" => Some("пок14".to_string()), // Годишна корекция
                    "5" => Some("пок15".to_string()), // Тристранна операция
                    _ => {
                        // Default: if has VAT, assume full credit; otherwise no journal entry
                        if vat_amount > Decimal::ZERO {
                            Some("пок10".to_string())
                        } else {
                            Some("0".to_string())
                        }
                    }
                }
            }
            "sale" => {
                // Sales operations mapping
                match old_code {
                    "1" => Some("про11".to_string()), // Облагаеми доставки 20%
                    "2" => Some("про17".to_string()), // Облагаеми доставки 9%
                    "3" => Some("про19".to_string()), // Доставки 0% по глава 3
                    "4" => Some("про20".to_string()), // ВОД (вътреобщностна доставка)
                    "5" => Some("про13".to_string()), // ВОП (вътреобщностно придобиване)
                    "8" => Some("про24-1".to_string()), // Освободени доставки
                    "9" => Some("про22".to_string()),   // Услуги чл.21, ал.2
                    _ => {
                        // Default: if has VAT, assume 20%; otherwise exempt
                        if vat_amount > Decimal::ZERO {
                            Some("про11".to_string())
                        } else {
                            Some("про24-1".to_string())
                        }
                    }
                }
            }
            _ => None, // Unknown document type
        }
    }

    /// Determine document type from file name
    pub fn determine_document_type(file_name: &str) -> String {
        let file_name_lower = file_name.to_lowercase();
        if file_name_lower.contains("pokupki") || file_name_lower.contains("покупки") {
            "purchase".to_string()
        } else if file_name_lower.contains("prodaj")
            || file_name_lower.contains("prodajbi")
            || file_name_lower.contains("продаж")
            || file_name_lower.contains("продажби")
        {
            "sale".to_string()
        } else {
            // Default fallback - try to guess from XML content
            "unknown".to_string()
        }
    }

    /// Determine document type from document's content (VAT data or accounting details)
    fn determine_document_type_from_content(document: &Document) -> String {
        // First check VAT register - this is the most reliable indicator
        if let Some(vat_data) = &document.vat_data {
            match vat_data.vat_register.as_str() {
                "1" => return "purchase".to_string(),
                "2" => return "sale".to_string(),
                _ => {} // Continue to other checks
            }
        }

        // Analyze accounting entries to determine if it's a purchase or sale
        for accounting in &document.accountings {
            for detail in &accounting.accounting_details {
                let account_code = &detail.account_number;

                // Check for typical purchase accounts (suppliers)
                if account_code.starts_with("401")
                    || account_code.starts_with("402")
                    || account_code.starts_with("404")
                    || account_code.starts_with("408")
                {
                    // If supplier account is credited, it's a purchase
                    if detail.direction == "Credit" {
                        return "purchase".to_string();
                    }
                }

                // Check for typical sales accounts (customers)
                if account_code.starts_with("411") || account_code.starts_with("412") {
                    // If customer account is debited, it's a sale
                    if detail.direction == "Debit" {
                        return "sale".to_string();
                    }
                }

                // Check for revenue accounts (7xx) - sales
                if account_code.starts_with("7") {
                    if detail.direction == "Credit" {
                        return "sale".to_string();
                    }
                }

                // Check for expense/material accounts (2xx, 3xx, 6xx) - purchases
                if account_code.starts_with("2")
                    || account_code.starts_with("3")
                    || account_code.starts_with("6")
                {
                    if detail.direction == "Debit" {
                        return "purchase".to_string();
                    }
                }
            }
        }

        // Fallback: if we can't determine from accounts, check VAT direction
        // Higher VAT receivable (debit on 4531) suggests purchase
        // Higher VAT payable (credit on 4532) suggests sale
        for accounting in &document.accountings {
            for detail in &accounting.accounting_details {
                if detail.account_number == "4531" && detail.direction == "Debit" {
                    return "purchase".to_string();
                }
                if detail.account_number == "4532" && detail.direction == "Credit" {
                    return "sale".to_string();
                }
            }
        }

        // If still can't determine, default to purchase
        "purchase".to_string()
    }

    pub fn parse_xml(xml_content: &str) -> Result<ControlisyData> {
        // Pre-process XML to fix quote issues in Bulgarian company names from commercial register
        let cleaned_xml = xml_content
            .replace(
                "\"ЖЕЛЕЗОПЪТНА ИНФРАСТРУКТУРА\"",
                "&quot;ЖЕЛЕЗОПЪТНА ИНФРАСТРУКТУРА&quot;",
            )
            .replace("\"МЕТРО\" АД", "&quot;МЕТРО&quot; АД")
            .replace("\"БИГ\"", "&quot;БИГ&quot;");

        let mut reader = Reader::from_str(&cleaned_xml);
        reader.config_mut().trim_text(true);

        let mut contractors = Vec::new();
        let mut documents = Vec::new();

        let mut buf = Vec::new();
        let mut in_contractors = false;
        let mut in_documents = false;
        let mut current_document: Option<Document> = None;
        let mut current_accounting: Option<Accounting> = None;
        let mut current_accounting_details = Vec::new();
        let mut current_vat: Option<Vat> = None;
        let mut current_vat_data: Option<VatData> = None;
        let mut current_rel_doc: Option<RelDoc> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    let name = e.name();
                    let tag_name = String::from_utf8_lossy(name.as_ref());

                    match tag_name.as_ref() {
                        "Contractors" => in_contractors = true,
                        "Documents" => {
                            in_contractors = false;
                            in_documents = true;
                        }
                        "Contractor" if in_contractors => {
                            let mut contractor = Contractor {
                                ca_contractor_id: String::new(),
                                contractor_name: String::new(),
                                contractor_eik: String::new(),
                                contractor_vat_number: String::new(),
                                contractor_inside_number: String::new(),
                            };

                            for attr in e.attributes() {
                                let attr = attr?;
                                let key = String::from_utf8_lossy(attr.key.as_ref());
                                let value = String::from_utf8_lossy(&attr.value);

                                match key.as_ref() {
                                    "ca_contractorId" => {
                                        contractor.ca_contractor_id = value.to_string()
                                    }
                                    "contractorName" => {
                                        contractor.contractor_name = value.to_string()
                                    }
                                    "contractorEIK" => {
                                        contractor.contractor_eik = value.to_string()
                                    }
                                    "contractorVATNumber" => {
                                        contractor.contractor_vat_number = value.to_string()
                                    }
                                    "contractorInsideNumber" => {
                                        contractor.contractor_inside_number = value.to_string()
                                    }
                                    _ => {}
                                }
                            }

                            contractors.push(contractor);
                        }
                        "Document" if in_documents => {
                            let mut document = Document {
                                accounting_month: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                                vat_month: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                                maturity: None,
                                doc_ident: String::new(),
                                document_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                                document_number: String::new(),
                                ca_doc_id: String::new(),
                                reason: String::new(),
                                remark: String::new(),
                                net_amount_bgn: Decimal::ZERO,
                                vat_amount_bgn: Decimal::ZERO,
                                total_amount_bgn: Decimal::ZERO,
                                ca_vat_operation_id: String::new(),
                                ca_doc_type_id: String::new(),
                                ca_contractor_id: String::new(),
                                accountings: Vec::new(),
                                vat_data: None,
                                rel_doc: None,
                            };

                            for attr in e.attributes() {
                                let attr = attr?;
                                let key = String::from_utf8_lossy(attr.key.as_ref());
                                let value = String::from_utf8_lossy(&attr.value);

                                match key.as_ref() {
                                    "accountingMonth" => {
                                        if let Ok(date) =
                                            NaiveDate::parse_from_str(&value, "%Y-%m-%d")
                                        {
                                            document.accounting_month = date;
                                        }
                                    }
                                    "vatMonth" => {
                                        if let Ok(date) =
                                            NaiveDate::parse_from_str(&value, "%Y-%m-%d")
                                        {
                                            document.vat_month = date;
                                        }
                                    }
                                    "maturity" => {
                                        if !value.is_empty() {
                                            if let Ok(date) =
                                                NaiveDate::parse_from_str(&value, "%Y-%m-%d")
                                            {
                                                document.maturity = Some(date);
                                            }
                                        }
                                    }
                                    "docIdent" => document.doc_ident = value.to_string(),
                                    "documentDate" => {
                                        if let Ok(date) =
                                            NaiveDate::parse_from_str(&value, "%Y-%m-%d")
                                        {
                                            document.document_date = date;
                                        }
                                    }
                                    "documentNumber" => {
                                        document.document_number = value.to_string()
                                    }
                                    "ca_docId" => document.ca_doc_id = value.to_string(),
                                    "reason" => document.reason = value.to_string(),
                                    "remark" => document.remark = value.to_string(),
                                    "netAmountBGN" => {
                                        if let Ok(amount) = Decimal::from_str(&value) {
                                            document.net_amount_bgn = amount;
                                        }
                                    }
                                    "vatAmountBGN" => {
                                        if let Ok(amount) = Decimal::from_str(&value) {
                                            document.vat_amount_bgn = amount;
                                        }
                                    }
                                    "totalAmountBGN" => {
                                        if let Ok(amount) = Decimal::from_str(&value) {
                                            document.total_amount_bgn = amount;
                                        }
                                    }
                                    "ca_vatOperationID" => {
                                        document.ca_vat_operation_id = value.to_string()
                                    }
                                    "ca_docTypeID" => document.ca_doc_type_id = value.to_string(),
                                    "ca_contractorId" => {
                                        document.ca_contractor_id = value.to_string()
                                    }
                                    _ => {}
                                }
                            }

                            current_document = Some(document);
                        }
                        "Accounting" if current_document.is_some() => {
                            let mut accounting = Accounting {
                                amount_bgn: Decimal::ZERO,
                                ca_vat_operation_id: String::new(),
                                accounting_details: Vec::new(),
                                vat: None,
                            };

                            for attr in e.attributes() {
                                let attr = attr?;
                                let key = String::from_utf8_lossy(attr.key.as_ref());
                                let value = String::from_utf8_lossy(&attr.value);

                                match key.as_ref() {
                                    "amountBGN" => {
                                        if let Ok(amount) = Decimal::from_str(&value) {
                                            accounting.amount_bgn = amount;
                                        }
                                    }
                                    "ca_vatOperationID" => {
                                        accounting.ca_vat_operation_id = value.to_string()
                                    }
                                    _ => {}
                                }
                            }

                            current_accounting = Some(accounting);
                            current_accounting_details.clear();
                        }
                        "AccountingDetail" if current_accounting.is_some() => {
                            let mut detail = AccountingDetail {
                                direction: String::new(),
                                currency: String::new(),
                                currency_amount: None,
                                unit: String::new(),
                                quantity: None,
                                account_number: String::new(),
                                account_name: String::new(),
                                contractor_name: String::new(),
                                contractor_eik: String::new(),
                                contractor_vat_number: String::new(),
                            };

                            for attr in e.attributes() {
                                let attr = attr?;
                                let key = String::from_utf8_lossy(attr.key.as_ref());
                                let value = String::from_utf8_lossy(&attr.value);

                                match key.as_ref() {
                                    "direction" => detail.direction = value.to_string(),
                                    "currency" => detail.currency = value.to_string(),
                                    "currencyAmount" => {
                                        if !value.is_empty() {
                                            if let Ok(amount) = Decimal::from_str(&value) {
                                                detail.currency_amount = Some(amount);
                                            }
                                        }
                                    }
                                    "unit" => detail.unit = value.to_string(),
                                    "quantity" => {
                                        if !value.is_empty() && value != "0" {
                                            if let Ok(qty) = Decimal::from_str(&value) {
                                                detail.quantity = Some(qty);
                                            }
                                        }
                                    }
                                    "accountNumber" => detail.account_number = value.to_string(),
                                    "accountName" => detail.account_name = value.to_string(),
                                    "contractorName" => detail.contractor_name = value.to_string(),
                                    "contractorEIK" => detail.contractor_eik = value.to_string(),
                                    "contractorVATNumber" => {
                                        detail.contractor_vat_number = value.to_string()
                                    }
                                    // Игнорираме accountItem1-4
                                    _ => {}
                                }
                            }

                            current_accounting_details.push(detail);
                        }
                        "VATData" if current_document.is_some() => {
                            let mut vat_data = VatData {
                                vat_register: String::new(),
                                contractor_name: String::new(),
                                contractor_vat_number: String::new(),
                                vat: Vec::new(),
                            };

                            for attr in e.attributes() {
                                let attr = attr?;
                                let key = String::from_utf8_lossy(attr.key.as_ref());
                                let value = String::from_utf8_lossy(&attr.value);

                                match key.as_ref() {
                                    "vatRegister" => vat_data.vat_register = value.to_string(),
                                    "contractorName" => {
                                        vat_data.contractor_name = value.to_string()
                                    }
                                    "contractorVATNumber" => {
                                        vat_data.contractor_vat_number = value.to_string()
                                    }
                                    _ => {}
                                }
                            }

                            current_vat_data = Some(vat_data);
                        }
                        "VAT" if current_vat_data.is_some() => {
                            let mut vat = Vat {
                                tax_base: Decimal::ZERO,
                                vat_rate: Decimal::ZERO,
                                vat_amount_bgn: Decimal::ZERO,
                                ca_vat_operation_id: String::new(),
                                vat_operation_iden: String::new(),
                                vat_operation_ident_name: String::new(),
                                vat_operation_name: String::new(),
                                vat_operation_additional_iden: String::new(),
                                vat_operation_additional_name: String::new(),
                                vat_operation_detail: String::new(),
                            };

                            for attr in e.attributes() {
                                let attr = attr?;
                                let key = String::from_utf8_lossy(attr.key.as_ref());
                                let value = String::from_utf8_lossy(&attr.value);

                                match key.as_ref() {
                                    "taxBase" => {
                                        if let Ok(amount) = Decimal::from_str(&value) {
                                            vat.tax_base = amount;
                                        }
                                    }
                                    "vatRate" => {
                                        if let Ok(rate) = Decimal::from_str(&value) {
                                            vat.vat_rate = rate;
                                        }
                                    }
                                    "vatAmountBGN" => {
                                        if let Ok(amount) = Decimal::from_str(&value) {
                                            vat.vat_amount_bgn = amount;
                                        }
                                    }
                                    "ca_vatOperationID" => {
                                        vat.ca_vat_operation_id = value.to_string()
                                    }
                                    "vatOperationIden" => {
                                        vat.vat_operation_iden = value.to_string()
                                    }
                                    "vatOperationIdenName" => {
                                        vat.vat_operation_ident_name = value.to_string()
                                    }
                                    "vatOperationName" => {
                                        vat.vat_operation_name = value.to_string()
                                    }
                                    "vatOperationAdditionalIden" => {
                                        vat.vat_operation_additional_iden = value.to_string()
                                    }
                                    "vatOperationAdditionalName" => {
                                        vat.vat_operation_additional_name = value.to_string()
                                    }
                                    "vatOperationDetail" => {
                                        vat.vat_operation_detail = value.to_string()
                                    }
                                    _ => {}
                                }
                            }

                            if let Some(ref mut vat_data) = current_vat_data {
                                vat_data.vat.push(vat);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(e)) => {
                    let name = e.name();
                    let tag_name = String::from_utf8_lossy(name.as_ref());

                    match tag_name.as_ref() {
                        "Accounting" => {
                            if let Some(mut accounting) = current_accounting.take() {
                                accounting.accounting_details = current_accounting_details.clone();
                                if let Some(vat) = current_vat.take() {
                                    accounting.vat = Some(vat);
                                }
                                if let Some(ref mut doc) = current_document {
                                    doc.accountings.push(accounting);
                                }
                            }
                        }
                        "Document" => {
                            if let Some(mut doc) = current_document.take() {
                                if let Some(vat_data) = current_vat_data.take() {
                                    doc.vat_data = Some(vat_data);
                                }
                                if let Some(rel_doc) = current_rel_doc.take() {
                                    doc.rel_doc = Some(rel_doc);
                                }
                                documents.push(doc);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }

        Ok(ControlisyData {
            contractors,
            documents,
        })
    }

    pub async fn import_to_database(
        db: &DatabaseConnection,
        company_id: i32,
        file_name: &str,
        document_type: &str, // "purchase" or "sale"
        xml_content: &str,
    ) -> Result<i32> {
        let parsed_data = Self::parse_xml(xml_content)?;

        // Serialize parsed data to JSON
        let parsed_json = serde_json::to_value(&parsed_data)?;

        // Create import record using raw SQL
        let import_result = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
            INSERT INTO controlisy_imports (
                company_id, import_date, file_name, document_type, 
                raw_xml, parsed_data, status, processed, 
                imported_documents, imported_contractors,
                created_at, updated_at
            ) VALUES ($1, NOW(), $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
            RETURNING id
            "#,
                vec![
                    company_id.into(),
                    file_name.into(),
                    document_type.into(),
                    xml_content.into(),
                    parsed_json.into(),
                    "staged".into(),
                    false.into(),
                    (parsed_data.documents.len() as i32).into(),
                    (parsed_data.contractors.len() as i32).into(),
                ],
            ))
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to get import ID"))?;

        // Get the inserted ID
        let import_id: i32 = import_result.try_get("", "id")?;

        Ok(import_id)
    }

    pub async fn update_staged_data(
        db: &DatabaseConnection,
        import_id: i32,
        updated_data: &str,
    ) -> Result<()> {
        // Update staged data
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE controlisy_imports SET parsed_data = $1, updated_at = NOW() WHERE id = $2 AND status = 'staged'",
            vec![updated_data.into(), import_id.into()],
        )).await?;

        Ok(())
    }

    pub async fn mark_as_reviewed(
        db: &DatabaseConnection,
        import_id: i32,
        user_id: i32,
    ) -> Result<()> {
        // Mark as reviewed and ready for processing
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE controlisy_imports SET status = 'reviewed', reviewed_at = NOW(), reviewed_by = $1, updated_at = NOW() WHERE id = $2",
            vec![user_id.into(), import_id.into()],
        )).await?;

        Ok(())
    }

    pub async fn process_import(db: &DatabaseConnection, import_id: i32) -> Result<()> {
        // Update status to processing (from staged or reviewed status)
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE controlisy_imports SET status = $1, updated_at = NOW() WHERE id = $2 AND status IN ('staged', 'reviewed')",
            vec!["processing".into(), import_id.into()],
        )).await?;

        // Get import data
        let import_data = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT company_id, parsed_data FROM controlisy_imports WHERE id = $1",
                vec![import_id.into()],
            ))
            .await?
            .ok_or(anyhow::anyhow!("Import not found"))?;

        let company_id: i32 = import_data.try_get("", "company_id")?;
        let parsed_data: serde_json::Value = import_data.try_get("", "parsed_data")?;
        let controlisy_data: ControlisyData = serde_json::from_value(parsed_data)?;

        // Process contractors
        for contractor in &controlisy_data.contractors {
            Self::create_or_update_contractor(db, company_id, contractor).await?;
        }

        // Process documents - continue processing even if some fail
        let mut successful_documents = 0;
        let mut failed_documents = 0;
        let mut error_messages = Vec::new();

        println!(
            "🚀 Starting to process {} documents from Controlisy import",
            controlisy_data.documents.len()
        );

        for (doc_index, document) in controlisy_data.documents.iter().enumerate() {
            println!(
                "📄 Processing document {}/{}: {}",
                doc_index + 1,
                controlisy_data.documents.len(),
                document.document_number
            );

            match Self::process_document(db, company_id, import_id, document).await {
                Ok(_) => {
                    successful_documents += 1;
                    println!(
                        "✅ Document {}/{} processed successfully",
                        doc_index + 1,
                        controlisy_data.documents.len()
                    );
                }
                Err(e) => {
                    failed_documents += 1;
                    let error_msg = format!(
                        "Failed to process document {}: {}",
                        document.document_number, e
                    );
                    error_messages.push(error_msg.clone());
                    eprintln!("❌ {}", error_msg);
                    println!(
                        "❌ Document {}/{} failed: {}",
                        doc_index + 1,
                        controlisy_data.documents.len(),
                        e
                    );
                }
            }
        }

        // Update status based on results
        let final_status = if failed_documents == 0 {
            "completed"
        } else if successful_documents > 0 {
            "partially_completed"
        } else {
            "failed"
        };

        let error_message = if !error_messages.is_empty() {
            Some(error_messages.join("\n"))
        } else {
            None
        };

        println!(
            "Controlisy import {} completed: {} successful, {} failed documents",
            import_id, successful_documents, failed_documents
        );

        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE controlisy_imports SET status = $1, processed = $2, error_message = $3, processed_at = NOW(), updated_at = NOW() WHERE id = $4",
            vec![
                final_status.into(),
                (successful_documents > 0).into(),
                error_message.into(),
                import_id.into()
            ],
        )).await?;

        Ok(())
    }

    async fn create_or_update_contractor(
        db: &DatabaseConnection,
        company_id: i32,
        contractor: &Contractor,
    ) -> Result<i32> {
        // Check if contractor exists
        let existing = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT id FROM counterparts WHERE company_id = $1 AND eik = $2",
                vec![company_id.into(), contractor.contractor_eik.clone().into()],
            ))
            .await?;

        if let Some(row) = existing {
            // Update existing
            let id: i32 = row.try_get("", "id")?;
            db.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "UPDATE counterparts SET name = $1, vat_number = $2, updated_at = NOW() WHERE id = $3",
                vec![
                    contractor.contractor_name.clone().into(),
                    contractor.contractor_vat_number.clone().into(),
                    id.into(),
                ],
            )).await?;
            Ok(id)
        } else {
            // Create new
            let result = db
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                INSERT INTO counterparts (
                    company_id, name, eik, vat_number, counterpart_type,
                    country, is_vat_registered, is_active, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
                RETURNING id
                "#,
                    vec![
                        company_id.into(),
                        contractor.contractor_name.clone().into(),
                        contractor.contractor_eik.clone().into(),
                        contractor.contractor_vat_number.clone().into(),
                        "SUPPLIER".into(), // counterpart_type
                        "BG".into(),       // country
                        true.into(),       // is_vat_registered (assume true if has VAT number)
                        true.into(),       // is_active
                    ],
                ))
                .await?
                .ok_or_else(|| anyhow::anyhow!("Failed to create contractor"))?;

            Ok(result.try_get("", "id")?)
        }
    }

    async fn process_document(
        db: &DatabaseConnection,
        company_id: i32,
        import_id: i32,
        document: &Document,
    ) -> Result<()> {
        println!(
            "🔄 Processing document: {} - {}",
            document.document_number, document.reason
        );

        // Determine document type from the document's content (VAT register or accounting details)
        let document_type = Self::determine_document_type_from_content(document);
        println!("📋 Document type determined as: {}", document_type);

        // Get contractor ID
        let contractor_id = if !document.ca_contractor_id.is_empty() {
            println!(
                "🔍 Looking for contractor with ca_contractor_id: {}",
                document.ca_contractor_id
            );
            let result = db
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "SELECT id FROM counterparts WHERE company_id = $1 AND eik IN (
                    SELECT c->>'contractor_eik' FROM controlisy_imports
                    CROSS JOIN LATERAL json_array_elements(parsed_data->'contractors') AS c
                    WHERE controlisy_imports.id = $2
                    AND c->>'ca_contractor_id' = $3
                )",
                    vec![
                        company_id.into(),
                        import_id.into(),
                        document.ca_contractor_id.clone().into(),
                    ],
                ))
                .await?;

            let contractor_id = result.map(|r| r.try_get::<i32>("", "id").ok()).flatten();
            match contractor_id {
                Some(id) => println!("✅ Found contractor ID: {}", id),
                None => println!(
                    "⚠️  Contractor not found for ca_contractor_id: {}",
                    document.ca_contractor_id
                ),
            }
            contractor_id
        } else {
            println!("⚠️  Document has empty ca_contractor_id");
            None
        };

        // Create journal entry with proper document type
        println!(
            "🏗️  Creating journal entry for document {}",
            document.document_number
        );
        let journal_entry_id = Self::create_journal_entry_with_type(
            db,
            company_id,
            document,
            contractor_id,
            &document_type,
        )
        .await?;

        // Log successful document processing
        println!(
            "✅ Successfully processed Controlisy document {}: {} as {} with journal entry ID {}",
            document.document_number, document.reason, document_type, journal_entry_id
        );

        Ok(())
    }

    async fn create_journal_entry_with_type(
        db: &DatabaseConnection,
        company_id: i32,
        document: &Document,
        contractor_id: Option<i32>,
        import_document_type: &str,
    ) -> Result<i32> {
        // Check if this is a payment document (empty or "0" VAT operation ID)
        let is_payment_document = document.ca_vat_operation_id.is_empty()
            || document.ca_vat_operation_id == "0"
            || document.reason.to_lowercase().contains("разплащане");

        // Set VAT document type based on import type (None for payment documents)
        let vat_document_type = if is_payment_document {
            None // No VAT document type for payment documents
        } else {
            match import_document_type {
                "purchase" => Some("03"), // Фактура за покупка
                "sale" => Some("01"),     // Фактура за продажба
                _ => Some("03"),          // Default to purchase
            }
        };

        // Determine VAT operation codes based on import type and document data
        // Try to get the VAT operation code from the document's VAT data first
        let vat_operation_from_xml = document
            .vat_data
            .as_ref()
            .and_then(|vd| vd.vat.first())
            .map(|v| v.vat_operation_iden.as_str())
            .unwrap_or("");

        let (vat_purchase_op, vat_sales_op) = if is_payment_document {
            // Payment documents have no VAT operations
            (None, None)
        } else {
            match import_document_type {
                "purchase" => {
                    // Map the old numeric code to new NAP code
                    let old_code = if !vat_operation_from_xml.is_empty() {
                        vat_operation_from_xml
                    } else if document.vat_amount_bgn > Decimal::ZERO {
                        "1" // Default: full credit
                    } else {
                        "3" // Default: no VAT
                    };

                    let new_code = Self::map_vat_operation_code(
                        old_code,
                        "purchase",
                        document.vat_amount_bgn,
                    );
                    (new_code, None)
                }
                "sale" => {
                    // Map the old numeric code to new NAP code
                    let old_code = if !vat_operation_from_xml.is_empty() {
                        vat_operation_from_xml
                    } else if document.vat_amount_bgn > Decimal::ZERO {
                        "1" // Default: 20% taxable
                    } else {
                        "8" // Default: exempt
                    };

                    let new_code = Self::map_vat_operation_code(
                        old_code,
                        "sale",
                        document.vat_amount_bgn,
                    );
                    (None, new_code)
                }
                _ => {
                    // Default to purchase with mapping
                    let old_code = if document.vat_amount_bgn > Decimal::ZERO {
                        "1"
                    } else {
                        "3"
                    };
                    let new_code = Self::map_vat_operation_code(
                        old_code,
                        "purchase",
                        document.vat_amount_bgn,
                    );
                    (new_code, None)
                }
            }
        };

        // Generate unique entry number with retry logic
        let mut retry_count = 0;
        let entry_number =
            loop {
                let candidate = format!(
                    "CTRL-{}-{}-{}",
                    document.document_number,
                    chrono::Utc::now().format("%Y%m%d%H%M%S"),
                    chrono::Utc::now().timestamp_micros() % 1000000
                );

                println!("🔢 Trying entry number: {}", candidate);

                // Check if this number already exists
                let existing = db.query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT id FROM journal_entries WHERE company_id = $1 AND entry_number = $2",
                vec![company_id.into(), candidate.clone().into()],
            )).await?;

                if existing.is_none() {
                    println!("✅ Entry number is unique: {}", candidate);
                    break candidate; // Number is unique
                }

                retry_count += 1;
                if retry_count > 10 {
                    return Err(anyhow::anyhow!(
                        "Failed to generate unique journal entry number"
                    ));
                }

                println!(
                    "⚠️  Entry number already exists, retrying... (attempt {})",
                    retry_count
                );
                // Small delay before retry
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            };

        // Create journal entry with proper VAT fields (or without for payment documents)
        println!("📝 Creating journal entry with number: {}", entry_number);
        let entry_result = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
            INSERT INTO journal_entries (
                company_id, entry_number, document_number, 
                document_date, vat_date, accounting_date,
                description, total_amount, total_vat_amount,
                vat_document_type, vat_purchase_operation, vat_sales_operation,
                created_by, is_posted,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW(), NOW())
            RETURNING id
            "#,
                vec![
                    company_id.into(),
                    entry_number.into(),
                    document.document_number.clone().into(),
                    document.document_date.into(), // Document date always stays the same
                    if is_payment_document {
                        document.document_date.into() // Use document_date for payment documents
                    } else {
                        // For VAT documents: different logic for purchases vs sales
                        match import_document_type {
                            "purchase" => document.vat_month.into(), // Purchases: VAT date = period (YYYY-MM-01)
                            "sale" => document.document_date.into(), // Sales: VAT date = document date
                            _ => document.vat_month.into(),          // Default to period date
                        }
                    },
                    // Accounting date logic
                    if is_payment_document {
                        document.document_date.into() // Payment documents: all dates = document date
                    } else {
                        // For VAT documents: purchases use document date, sales use document date
                        match import_document_type {
                            "purchase" => document.document_date.into(), // Purchases: accounting date = document date
                            "sale" => document.document_date.into(), // Sales: accounting date = document date
                            _ => document.accounting_month.into(),   // Fallback to accounting_month
                        }
                    },
                    document.reason.clone().into(),
                    document.total_amount_bgn.into(),
                    if is_payment_document {
                        Decimal::ZERO.into() // No VAT for payment documents
                    } else {
                        document.vat_amount_bgn.into()
                    },
                    vat_document_type.into(),
                    vat_purchase_op.into(),
                    vat_sales_op.into(),
                    1.into(),     // Default user ID - should be passed as parameter
                    false.into(), // Not posted by default
                ],
            ))
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to create journal entry"))?;

        let journal_entry_id: i32 = entry_result.try_get("", "id")?;

        // Create entry lines for each accounting detail
        let mut line_order = 1;
        for accounting in &document.accountings {
            for detail in &accounting.accounting_details {
                let account_id = Self::get_or_create_account(
                    db,
                    company_id,
                    &detail.account_number,
                    &detail.account_name,
                )
                .await?;

                let (debit_amount, credit_amount) = if detail.direction == "Debit" {
                    (accounting.amount_bgn, Decimal::ZERO)
                } else {
                    (Decimal::ZERO, accounting.amount_bgn)
                };

                db.execute(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                    INSERT INTO entry_lines (
                        journal_entry_id, account_id, counterpart_id,
                        debit_amount, credit_amount, base_amount, vat_amount,
                        description, line_order, currency_code, exchange_rate,
                        created_at
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
                    "#,
                    vec![
                        journal_entry_id.into(),
                        account_id.into(),
                        contractor_id.into(),
                        debit_amount.into(),
                        credit_amount.into(),
                        accounting.amount_bgn.into(), // base_amount
                        Decimal::ZERO.into(), // VAT amount per line - will be calculated later
                        document.reason.clone().into(),
                        line_order.into(),
                        "BGN".into(),
                        Decimal::ONE.into(), // exchange_rate for BGN
                    ],
                ))
                .await?;

                line_order += 1;
            }
        }

        Ok(journal_entry_id)
    }

    async fn get_or_create_account(
        db: &DatabaseConnection,
        company_id: i32,
        account_number: &str,
        account_name: &str,
    ) -> Result<i32> {
        // Check if account exists
        let existing = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT id FROM accounts WHERE company_id = $1 AND code = $2",
                vec![company_id.into(), account_number.into()],
            ))
            .await?;

        if let Some(row) = existing {
            Ok(row.try_get("", "id")?)
        } else {
            // Create new account
            let result = db
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                INSERT INTO accounts (
                    company_id, code, name, account_type, account_class, level,
                    is_vat_applicable, vat_direction, is_active, is_analytical,
                    created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
                RETURNING id
                "#,
                    vec![
                        company_id.into(),
                        account_number.into(),
                        account_name.into(),
                        "ASSET".into(), // account_type
                        1.into(),       // account_class
                        3.into(),       // level
                        false.into(),   // is_vat_applicable
                        "NONE".into(),  // vat_direction
                        true.into(),    // is_active
                        true.into(),    // is_analytical
                    ],
                ))
                .await?
                .ok_or_else(|| anyhow::anyhow!("Failed to create account"))?;

            Ok(result.try_get("", "id")?)
        }
    }
}

// Date formatting modules for serde
mod date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d";

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

mod date_format_option {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d";

    pub fn serialize<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(d) => {
                let s = format!("{}", d.format(FORMAT));
                serializer.serialize_str(&s)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        match opt {
            Some(s) => {
                if s.is_empty() {
                    Ok(None)
                } else {
                    NaiveDate::parse_from_str(&s, FORMAT)
                        .map(Some)
                        .map_err(serde::de::Error::custom)
                }
            }
            None => Ok(None),
        }
    }
}
