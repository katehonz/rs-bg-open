use anyhow::{anyhow, Result};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use quick_xml::de::from_str as from_xml_str;
use rust_decimal::Decimal;
use sea_orm::prelude::*;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, Set, TransactionTrait,
};
use serde::Deserialize;
use serde_json::json;
use std::str::FromStr;

use crate::entities::{
    bank_import, entry_line, journal_entry, BankImportFormat, BankImportStatus, BankProfileModel,
};

pub struct BankImportService;

#[derive(Debug, Clone)]
pub struct BankTransaction {
    pub booking_date: NaiveDate,
    pub value_date: Option<NaiveDate>,
    pub amount: Decimal,
    pub currency: String,
    pub is_credit: bool,
    pub description: String,
    pub reference: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImportSummary {
    pub transactions: usize,
    pub journal_entry_ids: Vec<i32>,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub bank_import: bank_import::Model,
}

impl BankImportService {
    pub fn supported_formats() -> Vec<BankImportFormat> {
        vec![
            BankImportFormat::UnicreditMt940,
            BankImportFormat::WiseCamt053,
            BankImportFormat::RevolutCamt053,
            BankImportFormat::PayseraCamt053,
            BankImportFormat::PostbankXml,
            BankImportFormat::ObbXml,
            BankImportFormat::CcbCsv,
        ]
    }

    pub async fn import_statement(
        db: &DatabaseConnection,
        profile: &BankProfileModel,
        file_name: &str,
        file_content: &[u8],
        created_by: Option<i32>,
    ) -> Result<ImportSummary> {
        if profile.import_format.is_empty() {
            return Err(anyhow!(
                "Bank profile {} has no import format configured",
                profile.name
            ));
        }

        let format: BankImportFormat = profile
            .import_format
            .parse()
            .map_err(|e| anyhow!("{}", e))?;

        let content_str = Self::decode_to_string(file_content, format)?;
        let transactions = Self::parse_transactions(&content_str, format, profile)?;

        if transactions.is_empty() {
            return Err(anyhow!("No transactions found in supplied file"));
        }

        let created_by = created_by.unwrap_or(1);

        let txn = db.begin().await?;
        let result =
            Self::persist_transactions(&txn, profile, file_name, &transactions, created_by).await?;
        txn.commit().await?;

        Ok(result)
    }

    fn decode_to_string(content: &[u8], format: BankImportFormat) -> Result<String> {
        match format {
            BankImportFormat::UnicreditMt940 => {
                // Files from banks may be encoded in Windows-1251; try UTF-8 first then fallback
                if let Ok(text) = std::str::from_utf8(content) {
                    Ok(text.replace('\r', ""))
                } else {
                    let (cow, _, _) = encoding_rs::WINDOWS_1251.decode(content);
                    Ok(cow.into_owned().replace('\r', ""))
                }
            }
            BankImportFormat::CcbCsv => {
                if let Ok(text) = std::str::from_utf8(content) {
                    Ok(text.replace('\r', ""))
                } else {
                    let (cow, _, _) = encoding_rs::WINDOWS_1251.decode(content);
                    Ok(cow.into_owned().replace('\r', ""))
                }
            }
            _ => {
                let text = std::str::from_utf8(content)?;
                let text = text.trim();
                let text = text.trim_start_matches('\u{feff}');
                Ok(text.to_string())
            }
        }
    }

    fn parse_transactions(
        content: &str,
        format: BankImportFormat,
        profile: &BankProfileModel,
    ) -> Result<Vec<BankTransaction>> {
        match format {
            BankImportFormat::UnicreditMt940 => Self::parse_mt940(content, &profile.currency_code),
            BankImportFormat::WiseCamt053
            | BankImportFormat::RevolutCamt053
            | BankImportFormat::PayseraCamt053 => Self::parse_camt053(content),
            BankImportFormat::PostbankXml => {
                Self::parse_postbank_xml(content, &profile.currency_code)
            }
            BankImportFormat::ObbXml => Self::parse_obb_xml(content, &profile.currency_code),
            BankImportFormat::CcbCsv => Self::parse_ccb_csv(content, &profile.currency_code),
        }
    }

    async fn persist_transactions(
        txn: &DatabaseTransaction,
        profile: &BankProfileModel,
        file_name: &str,
        transactions: &[BankTransaction],
        created_by: i32,
    ) -> Result<ImportSummary> {
        let mut journal_entry_ids = Vec::with_capacity(transactions.len());
        let mut total_debit = Decimal::ZERO;
        let mut total_credit = Decimal::ZERO;

        for (idx, tx) in transactions.iter().enumerate() {
            let ledger_date = tx.booking_date;
            let value_date = tx.value_date.unwrap_or(tx.booking_date);
            let amount = tx.amount.abs();

            let entry_number = format!(
                "BANK-{}-{}-{:04}",
                profile.id,
                ledger_date.format("%Y%m%d"),
                idx + 1
            );

            let document_number = tx
                .reference
                .clone()
                .or_else(|| Some(format!("{}-{}", file_name, idx + 1)));

            let entry_description =
                format!("Банково извлечение {} — {}", profile.name, tx.description);

            let mut entry = journal_entry::ActiveModel {
                entry_number: Set(entry_number),
                document_date: Set(ledger_date),
                vat_date: Set(Some(value_date)),
                accounting_date: Set(ledger_date),
                document_number: Set(document_number),
                description: Set(entry_description),
                total_amount: Set(amount),
                total_vat_amount: Set(Decimal::ZERO),
                is_posted: Set(false),
                posted_by: Set(None),
                posted_at: Set(None),
                created_by: Set(created_by),
                company_id: Set(profile.company_id),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
                vat_document_type: Set(None),
                vat_purchase_operation: Set(None),
                vat_sales_operation: Set(None),
                vat_additional_operation: Set(None),
                vat_additional_data: Set(None),
                ..Default::default()
            };

            let entry = journal_entry::Entity::insert(entry)
                .exec_with_returning(txn)
                .await?;

            let bank_line_debit = if tx.is_credit { amount } else { Decimal::ZERO };
            let bank_line_credit = if tx.is_credit { Decimal::ZERO } else { amount };

            let buffer_line_debit = if tx.is_credit { Decimal::ZERO } else { amount };
            let buffer_line_credit = if tx.is_credit { amount } else { Decimal::ZERO };

            let description = Some(tx.description.clone());

            let bank_line = entry_line::ActiveModel {
                journal_entry_id: Set(entry.id),
                account_id: Set(profile.account_id),
                debit_amount: Set(bank_line_debit),
                credit_amount: Set(bank_line_credit),
                counterpart_id: Set(None),
                currency_code: Set(Some(tx.currency.clone())),
                currency_amount: Set(Some(amount)),
                exchange_rate: Set(Some(Decimal::ONE)),
                base_amount: Set(amount),
                vat_amount: Set(Decimal::ZERO),
                vat_rate_id: Set(None),
                quantity: Set(None),
                unit_of_measure_code: Set(None),
                description: Set(description.clone()),
                line_order: Set(1),
                created_at: Set(Utc::now()),
                ..Default::default()
            };

            let buffer_line = entry_line::ActiveModel {
                journal_entry_id: Set(entry.id),
                account_id: Set(profile.buffer_account_id),
                debit_amount: Set(buffer_line_debit),
                credit_amount: Set(buffer_line_credit),
                counterpart_id: Set(None),
                currency_code: Set(Some(tx.currency.clone())),
                currency_amount: Set(Some(amount)),
                exchange_rate: Set(Some(Decimal::ONE)),
                base_amount: Set(amount),
                vat_amount: Set(Decimal::ZERO),
                vat_rate_id: Set(None),
                quantity: Set(None),
                unit_of_measure_code: Set(None),
                description: Set(description),
                line_order: Set(2),
                created_at: Set(Utc::now()),
                ..Default::default()
            };

            entry_line::Entity::insert_many([bank_line, buffer_line])
                .exec(txn)
                .await?;

            if tx.is_credit {
                total_debit += amount;
            } else {
                total_credit += amount;
            }

            journal_entry_ids.push(entry.id);
        }

        let bank_import_record = bank_import::ActiveModel {
            bank_profile_id: Set(profile.id),
            company_id: Set(profile.company_id),
            file_name: Set(file_name.to_string()),
            import_format: Set(profile.import_format.clone()),
            imported_at: Set(Utc::now()),
            transactions_count: Set(transactions.len() as i32),
            total_credit: Set(total_credit),
            total_debit: Set(total_debit),
            created_journal_entries: Set(journal_entry_ids.len() as i32),
            journal_entry_ids: Set(Some(json!(journal_entry_ids))),
            status: Set(BankImportStatus::Completed.as_str().to_string()),
            error_message: Set(None),
            created_by: Set(Some(created_by)),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(txn)
        .await?;

        Ok(ImportSummary {
            transactions: transactions.len(),
            journal_entry_ids,
            total_debit,
            total_credit,
            bank_import: bank_import_record,
        })
    }

    fn parse_mt940(content: &str, currency_code: &str) -> Result<Vec<BankTransaction>> {
        let mut transactions = Vec::new();
        let mut current: Option<BankTransaction> = None;

        for raw_line in content.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line == "-" {
                continue;
            }

            if line.starts_with(":61:") {
                if let Some(mut tx) = current.take() {
                    tx.description = tx.description.trim().to_string();
                    transactions.push(tx);
                }

                let data = line.trim_start_matches(":61:");
                let (booking_date, is_credit, amount, reference) =
                    Self::parse_mt940_61_line(data, currency_code)?;

                current = Some(BankTransaction {
                    booking_date,
                    value_date: Some(booking_date),
                    amount,
                    currency: currency_code.to_string(),
                    is_credit,
                    description: String::new(),
                    reference,
                });
            } else if line.starts_with(":86:") {
                if let Some(ref mut tx) = current {
                    let descr = line.trim_start_matches(":86:").trim();
                    Self::append_description(&mut tx.description, descr);
                }
            } else if line.starts_with(":62") {
                // Closing balance, flush current transaction if any
                if let Some(tx) = current.take() {
                    transactions.push(tx);
                }
            } else if !line.starts_with(':') {
                if let Some(ref mut tx) = current {
                    Self::append_description(&mut tx.description, line);
                }
            }
        }

        if let Some(mut tx) = current.take() {
            tx.description = tx.description.trim().to_string();
            transactions.push(tx);
        }

        Ok(transactions)
    }

    fn parse_mt940_61_line(
        data: &str,
        currency_code: &str,
    ) -> Result<(NaiveDate, bool, Decimal, Option<String>)> {
        if data.len() < 7 {
            return Err(anyhow!("Invalid :61: line format"));
        }

        let value_date = &data[0..6];
        let year = 2000 + value_date[0..2].parse::<i32>()?;
        let month = value_date[2..4].parse::<u32>()?;
        let day = value_date[4..6].parse::<u32>()?;
        let booking_date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| anyhow!("Invalid date in :61: line"))?;

        let rest = &data[6..];
        let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
        let (sign_part, remainder) = rest
            .chars()
            .next()
            .map(|c| (c, rest[1..].to_string()))
            .ok_or_else(|| anyhow!("Invalid :61: line payload"))?;

        let is_credit = match sign_part {
            'C' | 'c' => true,
            'D' | 'd' => false,
            _ => return Err(anyhow!("Unknown transaction indicator in :61: line")),
        };

        // Amount is until first letter (N) or sign
        let mut amount_str = String::new();
        for ch in remainder.chars() {
            if ch.is_ascii_digit() || ch == ',' || ch == '.' {
                amount_str.push(if ch == ',' { '.' } else { ch });
            } else {
                break;
            }
        }

        if amount_str.is_empty() {
            return Err(anyhow!("Could not parse amount in :61: line"));
        }

        let amount = Decimal::from_str(&amount_str)?;

        let reference = if let Some(idx) = remainder.find("//") {
            let (_, rest) = remainder.split_at(idx + 2);
            let reference = rest.split_whitespace().next().unwrap_or(rest).trim();
            if reference.is_empty() {
                None
            } else {
                Some(reference.to_string())
            }
        } else {
            None
        };

        Ok((booking_date, is_credit, amount, reference))
    }

    fn append_description(existing: &mut String, fragment: &str) {
        if !existing.is_empty() {
            existing.push(' ');
        }
        existing.push_str(fragment.trim());
    }

    fn parse_amount_field(value: &str) -> Result<Option<Decimal>> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        let normalized = trimmed.replace(' ', "").replace(',', ".");
        if normalized.is_empty() {
            return Ok(None);
        }

        let amount = Decimal::from_str(&normalized)?;
        Ok(Some(amount))
    }

    fn parse_camt053(content: &str) -> Result<Vec<BankTransaction>> {
        let document: CamtDocument = from_xml_str(content)?;
        let mut transactions = Vec::new();

        let statements = match &document.bk_to_cstmr_stmt {
            CamtStatementContainer::Single(stmt) => vec![stmt.clone()],
            CamtStatementContainer::Multiple(list) => list.clone(),
        };

        for statement in statements {
            for entry in statement.entries.unwrap_or_default() {
                if let Some(amount) = entry.amount {
                    let currency = amount.currency.unwrap_or_else(|| "EUR".to_string());
                    let amount_val = Decimal::from_str(&amount.value.replace(',', "."))?;
                    let is_credit = entry
                        .credit_debit_indicator
                        .as_deref()
                        .map(|v| v.eq_ignore_ascii_case("CRDT"))
                        .unwrap_or(false);

                    let booking_date = entry
                        .booking_date
                        .as_ref()
                        .and_then(|d| d.to_naive_date())
                        .ok_or_else(|| anyhow!("Missing booking date in CAMT entry"))?;
                    let value_date = entry.value_date.as_ref().and_then(|d| d.to_naive_date());

                    let mut description = String::new();
                    if let Some(dtls) = entry.details {
                        for detail in dtls.transactions {
                            if let Some(remit) = detail.remittance_info {
                                for line in remit.unstructured {
                                    Self::append_description(&mut description, &line);
                                }
                            }
                            if let Some(extra) = detail.additional_info {
                                Self::append_description(&mut description, &extra);
                            }
                            if let Some(card_tx) = detail.card_transaction {
                                if let Some(info) = card_tx.additional_info {
                                    Self::append_description(&mut description, &info);
                                }
                            }
                        }
                    }

                    if description.is_empty() {
                        if let Some(ref ref_text) = entry.reference {
                            description = ref_text.clone();
                        }
                    }

                    let description = description.trim().to_string();

                    transactions.push(BankTransaction {
                        booking_date,
                        value_date,
                        amount: amount_val,
                        currency,
                        is_credit,
                        description,
                        reference: entry.reference.clone(),
                    });
                }
            }
        }

        Ok(transactions)
    }

    fn parse_ccb_csv(content: &str, currency_code: &str) -> Result<Vec<BankTransaction>> {
        let mut transactions = Vec::new();

        for raw_line in content.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }

            if !line
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                continue;
            }

            let mut parts = line.splitn(6, ';');
            let booking_date_str = parts.next().unwrap_or("").trim();
            let value_date_str = parts.next().unwrap_or("").trim();
            let credit_str = parts.next().unwrap_or("").trim();
            let debit_str = parts.next().unwrap_or("").trim();
            let reference_str = parts.next().unwrap_or("").trim();
            let description_str = parts.next().unwrap_or("").trim();

            let booking_date = match NaiveDate::parse_from_str(booking_date_str, "%d.%m.%Y") {
                Ok(date) => date,
                Err(_) => continue,
            };

            let value_date = NaiveDate::parse_from_str(value_date_str, "%d.%m.%Y").ok();

            let credit_amount = Self::parse_amount_field(credit_str)?;
            let debit_amount = Self::parse_amount_field(debit_str)?;

            let (amount, is_credit) = match (credit_amount, debit_amount) {
                (Some(val), _) if !val.is_zero() => (val, true),
                (_, Some(val)) if !val.is_zero() => (val, false),
                _ => continue,
            };

            let mut description = description_str.replace('\r', "");
            if description.is_empty() {
                description = reference_str.to_string();
            }

            transactions.push(BankTransaction {
                booking_date,
                value_date,
                amount,
                currency: currency_code.to_string(),
                is_credit,
                description: description.trim().to_string(),
                reference: if reference_str.is_empty() {
                    None
                } else {
                    Some(reference_str.to_string())
                },
            });
        }

        Ok(transactions)
    }

    fn parse_postbank_xml(content: &str, currency_code: &str) -> Result<Vec<BankTransaction>> {
        let document: PostbankDocument = from_xml_str(content)?;
        let mut transactions = Vec::new();

        for item in document.items {
            let booking_date = item
                .booking_date
                .as_ref()
                .and_then(|d| d.to_naive_date())
                .ok_or_else(|| anyhow!("Missing booking date in Postbank item"))?;

            let value_date = item.value_date.as_ref().and_then(|d| d.to_naive_date());

            let amount_value = item
                .transaction_amount
                .amount
                .unwrap_or(item.transaction_amount.value);
            let amount = Decimal::from_str(&amount_value.to_string())?;
            let flow_direction = item
                .flow_direction
                .unwrap_or_else(|| "Outbound".to_string());
            let is_credit = flow_direction.eq_ignore_ascii_case("Inbound");

            let mut description = String::new();
            if let Some(info) = item.remittance_information {
                Self::append_description(&mut description, &info);
            }
            if let Some(counterparty) = item.counterparty {
                Self::append_description(&mut description, &counterparty);
            }
            if let Some(op_type) = item.operation_type {
                Self::append_description(&mut description, &op_type);
            }

            let description = description.trim().to_string();

            transactions.push(BankTransaction {
                booking_date,
                value_date,
                amount,
                currency: currency_code.to_string(),
                is_credit,
                description,
                reference: item.bordero_number.clone(),
            });
        }

        Ok(transactions)
    }

    fn parse_obb_xml(content: &str, currency_code: &str) -> Result<Vec<BankTransaction>> {
        let document: ObbStatement = from_xml_str(content)?;
        let mut transactions = Vec::new();

        for item in document.transactions {
            let ObbTransaction {
                post_date,
                date_val,
                to_date,
                amount_debit,
                amount_credit,
                transaction_name,
                rem_i,
                rem_ii,
                name_r,
                reference,
                more1,
                more2,
                bic_r,
                iban_r,
                bulstat,
                time,
            } = item;

            let booking_date = post_date
                .as_deref()
                .and_then(Self::parse_obb_date)
                .ok_or_else(|| anyhow!("Missing booking date in OBB transaction"))?;

            let value_date = date_val
                .as_deref()
                .and_then(Self::parse_obb_date)
                .or_else(|| to_date.as_deref().and_then(Self::parse_obb_to_date));

            let credit_amount = amount_credit
                .as_ref()
                .map(|v| Self::parse_obb_amount(v))
                .transpose()?;
            let debit_amount = amount_debit
                .as_ref()
                .map(|v| Self::parse_obb_amount(v))
                .transpose()?;

            let (amount, is_credit) = match (credit_amount, debit_amount) {
                (Some(val), _) => (val, true),
                (_, Some(val)) => (val, false),
                _ => continue,
            };

            let mut description = String::new();
            if let Some(ref name) = transaction_name {
                Self::append_description(&mut description, name);
            }
            if let Some(ref val) = time {
                Self::append_description(&mut description, &format!("Час {}", val));
            }
            if let Some(ref val) = rem_i {
                Self::append_description(&mut description, val);
            }
            if let Some(ref val) = rem_ii {
                Self::append_description(&mut description, val);
            }
            if let Some(ref val) = name_r {
                Self::append_description(&mut description, val);
            }
            if let Some(ref val) = more1 {
                Self::append_description(&mut description, val);
            }
            if let Some(ref val) = more2 {
                Self::append_description(&mut description, val);
            }
            if let Some(ref val) = bic_r {
                Self::append_description(&mut description, val);
            }
            if let Some(ref val) = iban_r {
                Self::append_description(&mut description, val);
            }
            if let Some(ref val) = bulstat {
                Self::append_description(&mut description, &format!("Булстат {}", val));
            }

            let description = if description.is_empty() {
                transaction_name
                    .clone()
                    .unwrap_or_else(|| "OBB транзакция".to_string())
            } else {
                description
            };

            let reference = reference.and_then(|r| {
                let trimmed = r.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            });

            transactions.push(BankTransaction {
                booking_date,
                value_date,
                amount,
                currency: currency_code.to_string(),
                is_credit,
                description: description.trim().to_string(),
                reference,
            });
        }

        Ok(transactions)
    }

    fn parse_obb_amount(value: &str) -> Result<Decimal> {
        let mut normalized: String = value.chars().filter(|c| !c.is_whitespace()).collect();
        if normalized.is_empty() {
            return Err(anyhow!("Empty amount value"));
        }

        let last_dot = normalized.rfind('.');
        let last_comma = normalized.rfind(',');

        normalized = if let (Some(dot), Some(comma)) = (last_dot, last_comma) {
            if dot > comma {
                normalized.replace(',', "")
            } else {
                normalized.replace('.', "").replace(',', ".")
            }
        } else if last_comma.is_some() {
            normalized.replace(',', ".")
        } else {
            normalized
        };

        Decimal::from_str(&normalized).map_err(|e| anyhow!("Failed to parse amount '{value}': {e}"))
    }

    fn parse_obb_date(value: &str) -> Option<NaiveDate> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return None;
        }

        NaiveDate::parse_from_str(trimmed, "%d/%m/%Y").ok()
    }

    fn parse_obb_to_date(value: &str) -> Option<NaiveDate> {
        let trimmed = value.trim();
        if trimmed.len() != 8 {
            return None;
        }

        NaiveDate::parse_from_str(trimmed, "%Y%m%d").ok()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct CamtDocument {
    #[serde(rename = "BkToCstmrStmt")]
    pub bk_to_cstmr_stmt: CamtStatementContainer,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum CamtStatementContainer {
    Single(CamtStatement),
    Multiple(Vec<CamtStatement>),
}

#[derive(Debug, Clone, Deserialize)]
struct CamtStatement {
    #[serde(rename = "Ntry", default)]
    pub entries: Option<Vec<CamtEntry>>,
}

#[derive(Debug, Clone, Deserialize)]
struct CamtEntry {
    #[serde(rename = "Amt", default)]
    pub amount: Option<CamtAmount>,
    #[serde(rename = "CdtDbtInd", default)]
    pub credit_debit_indicator: Option<String>,
    #[serde(rename = "BookgDt", default)]
    pub booking_date: Option<CamtDate>,
    #[serde(rename = "ValDt", default)]
    pub value_date: Option<CamtDate>,
    #[serde(rename = "AcctSvcrRef", default)]
    pub reference: Option<String>,
    #[serde(rename = "NtryDtls", default)]
    pub details: Option<CamtEntryDetails>,
}

#[derive(Debug, Clone, Deserialize)]
struct CamtAmount {
    #[serde(rename = "@Ccy", default)]
    pub currency: Option<String>,
    #[serde(rename = "$value", default)]
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CamtDate {
    #[serde(rename = "Dt", default)]
    pub date: Option<String>,
    #[serde(rename = "DtTm", default)]
    pub datetime: Option<String>,
}

impl CamtDate {
    pub fn to_naive_date(&self) -> Option<NaiveDate> {
        if let Some(ref d) = self.date {
            NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()
        } else if let Some(ref dt) = self.datetime {
            NaiveDateTime::parse_from_str(dt, "%Y-%m-%dT%H:%M:%S%.f")
                .map(|dt| dt.date())
                .ok()
                .or_else(|| NaiveDate::parse_from_str(&dt[..10], "%Y-%m-%d").ok())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct CamtEntryDetails {
    #[serde(rename = "TxDtls", default)]
    pub transactions: Vec<CamtTransactionDetails>,
}

#[derive(Debug, Clone, Deserialize)]
struct CamtTransactionDetails {
    #[serde(rename = "RmtInf", default)]
    pub remittance_info: Option<CamtRemittance>,
    #[serde(rename = "AddtlTxInf", default)]
    pub additional_info: Option<String>,
    #[serde(rename = "CardTx", default)]
    pub card_transaction: Option<CamtCardTransaction>,
}

#[derive(Debug, Clone, Deserialize)]
struct CamtRemittance {
    #[serde(rename = "Ustrd", default)]
    pub unstructured: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CamtCardTransaction {
    #[serde(rename = "AddtlTxInf", default)]
    pub additional_info: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct PostbankDocument {
    #[serde(rename = "MultipleAccountTransactionItemAPIOutputModel", default)]
    pub items: Vec<PostbankItem>,
}

#[derive(Debug, Clone, Deserialize)]
struct PostbankItem {
    #[serde(rename = "TransactionAmount")]
    pub transaction_amount: PostbankAmount,
    #[serde(rename = "FlowDirection", default)]
    pub flow_direction: Option<String>,
    #[serde(rename = "RemittanceInformation", default)]
    pub remittance_information: Option<String>,
    #[serde(rename = "Counterparty", default)]
    pub counterparty: Option<String>,
    #[serde(rename = "TypeOperation", default)]
    pub operation_type: Option<String>,
    #[serde(rename = "SystemData", default)]
    pub system_data: Option<PostbankDateTime>,
    #[serde(rename = "BookingDate", default)]
    pub booking_date: Option<PostbankDateTime>,
    #[serde(rename = "ValueDate", default)]
    pub value_date: Option<PostbankDateTime>,
    #[serde(rename = "BorderoNumber", default)]
    pub bordero_number: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct PostbankAmount {
    #[serde(rename = "Value")]
    pub value: f64,
    #[serde(rename = "Amount", default)]
    pub amount: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct ObbStatement {
    #[serde(rename = "TRANSACTION", default)]
    pub transactions: Vec<ObbTransaction>,
}

#[derive(Debug, Clone, Deserialize)]
struct ObbTransaction {
    #[serde(rename = "POST_DATE", default)]
    pub post_date: Option<String>,
    #[serde(rename = "DATEVAL", default)]
    pub date_val: Option<String>,
    #[serde(rename = "TODATE", default)]
    pub to_date: Option<String>,
    #[serde(rename = "TIME", default)]
    pub time: Option<String>,
    #[serde(rename = "AMOUNT_D", default)]
    pub amount_debit: Option<String>,
    #[serde(rename = "AMOUNT_C", default)]
    pub amount_credit: Option<String>,
    #[serde(rename = "TR_NAME", default)]
    pub transaction_name: Option<String>,
    #[serde(rename = "REM_I", default)]
    pub rem_i: Option<String>,
    #[serde(rename = "REM_II", default)]
    pub rem_ii: Option<String>,
    #[serde(rename = "NAME_R", default)]
    pub name_r: Option<String>,
    #[serde(rename = "REFERENCE", default)]
    pub reference: Option<String>,
    #[serde(rename = "MORE1", default)]
    pub more1: Option<String>,
    #[serde(rename = "MORE2", default)]
    pub more2: Option<String>,
    #[serde(rename = "BIC_R", default)]
    pub bic_r: Option<String>,
    #[serde(rename = "IBAN_R", default)]
    pub iban_r: Option<String>,
    #[serde(rename = "BULSTAT", default)]
    pub bulstat: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct PostbankDateTime {
    #[serde(rename = "DateTimeValue", default)]
    pub datetime: Option<String>,
}

impl PostbankDateTime {
    pub fn to_naive_date(&self) -> Option<NaiveDate> {
        self.datetime
            .as_ref()
            .and_then(|dt| NaiveDateTime::parse_from_str(dt, "%Y-%m-%dT%H:%M:%S").ok())
            .map(|dt| dt.date())
            .or_else(|| {
                self.datetime
                    .as_ref()
                    .and_then(|dt| NaiveDate::parse_from_str(&dt[..10], "%Y-%m-%d").ok())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{BankImportFormat, BankImportService};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn parse_ccb_csv_sample_extracts_transactions() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../banki/ccbank/ccb--statements_BG43CECB979010H7074200.csv");
        let bytes = std::fs::read(&path).expect("failed to read CCB sample file");
        let content = BankImportService::decode_to_string(&bytes, BankImportFormat::CcbCsv)
            .expect("failed to decode CCB CSV");

        let transactions =
            BankImportService::parse_ccb_csv(&content, "BGN").expect("failed to parse CCB CSV");

        assert!(
            !transactions.is_empty(),
            "expected at least one transaction"
        );

        let first = &transactions[0];
        assert_eq!(
            first.booking_date,
            NaiveDate::from_ymd_opt(2025, 9, 1).unwrap()
        );
        assert_eq!(
            first.amount,
            Decimal::from_str("49.93").expect("decimal parse")
        );
        assert!(!first.is_credit, "first transaction should be an expense");

        assert!(
            transactions
                .iter()
                .any(|tx| tx.is_credit && tx.amount > Decimal::ZERO),
            "expected at least one inbound transaction"
        );
    }

    #[test]
    fn parse_obb_xml_sample_extracts_transactions() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../banki/OBB/01-2025-xml_OBB.xml");
        let bytes = std::fs::read(&path).expect("failed to read OBB sample file");
        let content = BankImportService::decode_to_string(&bytes, BankImportFormat::ObbXml)
            .expect("failed to decode OBB XML");

        let transactions =
            BankImportService::parse_obb_xml(&content, "BGN").expect("failed to parse OBB XML");

        assert!(
            !transactions.is_empty(),
            "expected at least one OBB transaction"
        );

        let first = &transactions[0];
        assert_eq!(
            first.booking_date,
            NaiveDate::from_ymd_opt(2025, 2, 3).unwrap()
        );
        assert_eq!(
            first.amount,
            Decimal::from_str("13.90").expect("decimal parse")
        );
        assert!(
            transactions
                .iter()
                .any(|tx| tx.is_credit && tx.amount > Decimal::ZERO),
            "expected at least one inbound transaction"
        );
    }
}
