use async_graphql::{Context, FieldResult, InputObject, Object, SimpleObject};
use base64::Engine;
use chrono::{NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::entities::{account, company, counterpart, entry_line, journal_entry};

// Input types for reports
#[derive(InputObject, Deserialize)]
pub struct TurnoverReportInput {
    pub company_id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub account_id: Option<i32>, // Optional: specific account or all accounts
    pub show_zero_balances: Option<bool>, // Show accounts with zero balances
    pub account_code_depth: Option<i32>, // Group accounts by code depth (3, 4, 5 digits, or None for full detail)
}

#[derive(InputObject, Deserialize)]
pub struct TransactionLogInput {
    pub company_id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub account_id: Option<i32>,
}

#[derive(InputObject, Deserialize)]
pub struct GeneralLedgerInput {
    pub company_id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub account_id: Option<i32>,
}

#[derive(InputObject, Deserialize)]
pub struct ChronologicalReportInput {
    pub company_id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub account_id: Option<i32>, // Optional: filter by specific account
}

// Bulgarian variant general ledger input
#[derive(InputObject, Deserialize)]
pub struct BgGeneralLedgerInput {
    pub company_id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub account_id: Option<i32>,
}

// Report result types
#[derive(SimpleObject, Serialize)]
pub struct ChronologicalEntry {
    pub date: NaiveDate,
    pub debit_account_code: String,
    pub debit_account_name: String,
    pub credit_account_code: String,
    pub credit_account_name: String,
    pub amount: Decimal,
    pub debit_currency_amount: Option<Decimal>,
    pub debit_currency_code: Option<String>,
    pub credit_currency_amount: Option<Decimal>,
    pub credit_currency_code: Option<String>,
    pub document_type: Option<String>,
    pub document_date: Option<NaiveDate>,
    pub description: String,
}

#[derive(SimpleObject, Serialize)]
pub struct ChronologicalReport {
    pub company_name: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub entries: Vec<ChronologicalEntry>,
    pub total_amount: Decimal,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

// Bulgarian variant general ledger - simple entry without dates and documents
#[derive(SimpleObject, Serialize, Clone)]
pub struct BgLedgerEntry {
    pub debit_account_code: String,
    pub debit_account_name: String,
    pub credit_account_code: String,
    pub credit_account_name: String,
    pub amount: Decimal,
}

// Grouped by debit account
#[derive(SimpleObject, Serialize)]
pub struct BgLedgerByDebit {
    pub debit_account_code: String,
    pub debit_account_name: String,
    pub entries: Vec<BgLedgerByDebitEntry>,
    pub total_amount: Decimal,
}

#[derive(SimpleObject, Serialize)]
pub struct BgLedgerByDebitEntry {
    pub credit_account_code: String,
    pub credit_account_name: String,
    pub amount: Decimal,
}

// Grouped by credit account
#[derive(SimpleObject, Serialize)]
pub struct BgLedgerByCredit {
    pub credit_account_code: String,
    pub credit_account_name: String,
    pub entries: Vec<BgLedgerByCreditEntry>,
    pub total_amount: Decimal,
}

#[derive(SimpleObject, Serialize)]
pub struct BgLedgerByCreditEntry {
    pub debit_account_code: String,
    pub debit_account_name: String,
    pub amount: Decimal,
}

// Main BG General Ledger report
#[derive(SimpleObject, Serialize)]
pub struct BgGeneralLedger {
    pub company_name: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub by_debit: Vec<BgLedgerByDebit>,
    pub by_credit: Vec<BgLedgerByCredit>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(SimpleObject, Serialize)]
pub struct TurnoverSheetEntry {
    pub account_id: i32,
    pub account_code: String,
    pub account_name: String,
    // Opening balance
    pub opening_debit: Decimal,
    pub opening_credit: Decimal,
    // Turnovers for period
    pub period_debit: Decimal,
    pub period_credit: Decimal,
    // Closing balance
    pub closing_debit: Decimal,
    pub closing_credit: Decimal,
}

#[derive(SimpleObject, Serialize)]
pub struct TurnoverSheet {
    pub company_name: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub entries: Vec<TurnoverSheetEntry>,
    pub totals: TurnoverSheetEntry,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(SimpleObject, Serialize)]
pub struct TransactionLogEntry {
    pub date: NaiveDate,
    pub entry_number: String,
    pub document_number: Option<String>,
    pub description: String,
    pub account_code: String,
    pub account_name: String,
    pub debit_amount: Decimal,
    pub credit_amount: Decimal,
    pub counterpart_name: Option<String>,
}

#[derive(SimpleObject, Serialize)]
pub struct TransactionLog {
    pub company_name: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub entries: Vec<TransactionLogEntry>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(SimpleObject, Serialize)]
pub struct GeneralLedgerEntry {
    pub date: NaiveDate,
    pub entry_number: String,
    pub document_number: Option<String>,
    pub description: String,
    pub debit_amount: Decimal,
    pub credit_amount: Decimal,
    pub balance: Decimal,
    pub counterpart_name: Option<String>,
}

#[derive(SimpleObject, Serialize)]
pub struct GeneralLedgerAccount {
    pub account_id: i32,
    pub account_code: String,
    pub account_name: String,
    pub opening_balance: Decimal,
    pub closing_balance: Decimal,
    pub total_debits: Decimal,
    pub total_credits: Decimal,
    pub entries: Vec<GeneralLedgerEntry>,
}

#[derive(SimpleObject, Serialize)]
pub struct GeneralLedger {
    pub company_name: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub accounts: Vec<GeneralLedgerAccount>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

// Export format options
#[derive(SimpleObject, Serialize)]
pub struct ReportExport {
    pub format: String,  // "XLSX", "PDF" (HTML to PDF)
    pub content: String, // Base64 encoded content
    pub filename: String,
    pub mime_type: String,
}

// Monthly transaction statistics for pricing
#[derive(SimpleObject, Serialize)]
pub struct MonthlyTransactionStats {
    pub year: i32,
    pub month: i32,
    pub month_name: String,
    pub total_entries: i64,        // Total journal entries (documents)
    pub posted_entries: i64,       // Posted journal entries
    pub total_entry_lines: i64,    // Total debit/credit rows
    pub posted_entry_lines: i64,   // Posted debit/credit rows
    pub total_amount: Decimal,     // Total transaction amount
    pub vat_amount: Decimal,       // Total VAT amount
}

// Input for monthly statistics report
#[derive(InputObject, Deserialize, Clone)]
pub struct MonthlyStatsInput {
    pub company_id: i32,
    pub from_year: i32,
    pub from_month: i32,
    pub to_year: i32,
    pub to_month: i32,
}

#[derive(Default)]
pub struct ReportsQuery;

#[Object]
impl ReportsQuery {
    /// Generate turnover sheet (оборотна ведомост) - 6 columns
    async fn turnover_sheet(
        &self,
        ctx: &Context<'_>,
        input: TurnoverReportInput,
    ) -> FieldResult<TurnoverSheet> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Get company info
        let company = company::Entity::find_by_id(input.company_id)
            .one(db)
            .await?
            .ok_or("Company not found")?;

        // Get all accounts (or specific account if provided)
        let mut account_query = account::Entity::find()
            .filter(account::Column::CompanyId.eq(input.company_id))
            .filter(account::Column::IsActive.eq(true));

        if let Some(account_id) = input.account_id {
            account_query = account_query.filter(account::Column::Id.eq(account_id));
        }

        let accounts = account_query
            .order_by_asc(account::Column::Code)
            .all(db)
            .await?;

        let mut entries = Vec::new();
        let mut total_opening_debit = Decimal::ZERO;
        let mut total_opening_credit = Decimal::ZERO;
        let mut total_period_debit = Decimal::ZERO;
        let mut total_period_credit = Decimal::ZERO;
        let mut total_closing_debit = Decimal::ZERO;
        let mut total_closing_credit = Decimal::ZERO;

        // Use HashMap to aggregate by account code (if depth is specified)
        use std::collections::HashMap;
        let mut account_aggregates: HashMap<String, (String, Decimal, Decimal, Decimal, Decimal)> = HashMap::new();

        for account in accounts {
            // Determine the aggregation key based on account_code_depth
            let (agg_code, agg_name) = if let Some(depth) = input.account_code_depth {
                let depth_usize = depth as usize;
                if account.code.len() > depth_usize {
                    let truncated_code = account.code.chars().take(depth_usize).collect::<String>();
                    // Use truncated code and generic name
                    (truncated_code.clone(), format!("Сметки {}", truncated_code))
                } else {
                    (account.code.clone(), account.name.clone())
                }
            } else {
                (account.code.clone(), account.name.clone())
            };

            // Calculate opening balance (before start_date)
            let opening_balance = entry_line::Entity::find()
                .left_join(journal_entry::Entity)
                .filter(entry_line::Column::AccountId.eq(account.id))
                .filter(journal_entry::Column::CompanyId.eq(input.company_id))
                .filter(journal_entry::Column::AccountingDate.lt(input.start_date))
                .filter(journal_entry::Column::IsPosted.eq(true))
                .all(db)
                .await?;

            let mut opening_debit = Decimal::ZERO;
            let mut opening_credit = Decimal::ZERO;

            for line in opening_balance {
                opening_debit += line.debit_amount;
                opening_credit += line.credit_amount;
            }

            // Calculate period turnovers
            let period_lines = entry_line::Entity::find()
                .left_join(journal_entry::Entity)
                .filter(entry_line::Column::AccountId.eq(account.id))
                .filter(journal_entry::Column::CompanyId.eq(input.company_id))
                .filter(journal_entry::Column::AccountingDate.gte(input.start_date))
                .filter(journal_entry::Column::AccountingDate.lte(input.end_date))
                .filter(journal_entry::Column::IsPosted.eq(true))
                .all(db)
                .await?;

            let mut period_debit = Decimal::ZERO;
            let mut period_credit = Decimal::ZERO;

            for line in period_lines {
                period_debit += line.debit_amount;
                period_credit += line.credit_amount;
            }

            // Aggregate into HashMap
            let entry = account_aggregates.entry(agg_code.clone()).or_insert((
                agg_name,
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO,
            ));
            entry.1 += opening_debit;
            entry.2 += opening_credit;
            entry.3 += period_debit;
            entry.4 += period_credit;
        }

        // Convert HashMap to entries
        for (code, (name, opening_debit, opening_credit, period_debit, period_credit)) in account_aggregates {
            // Calculate closing balance
            let closing_debit = opening_debit + period_debit;
            let closing_credit = opening_credit + period_credit;
            let net_closing = closing_debit - closing_credit;

            let (final_closing_debit, final_closing_credit) = if net_closing > Decimal::ZERO {
                (net_closing, Decimal::ZERO)
            } else {
                (Decimal::ZERO, net_closing.abs())
            };

            // Skip zero balance accounts if requested
            let show_zero_balances = input.show_zero_balances.unwrap_or(true);
            if !show_zero_balances
                && opening_debit == Decimal::ZERO
                && opening_credit == Decimal::ZERO
                && period_debit == Decimal::ZERO
                && period_credit == Decimal::ZERO
            {
                continue;
            }

            let entry = TurnoverSheetEntry {
                account_id: 0, // No specific ID for aggregated entries
                account_code: code,
                account_name: name,
                opening_debit,
                opening_credit,
                period_debit,
                period_credit,
                closing_debit: final_closing_debit,
                closing_credit: final_closing_credit,
            };

            // Add to totals
            total_opening_debit += opening_debit;
            total_opening_credit += opening_credit;
            total_period_debit += period_debit;
            total_period_credit += period_credit;
            total_closing_debit += final_closing_debit;
            total_closing_credit += final_closing_credit;

            entries.push(entry);
        }

        // Sort entries by account code
        entries.sort_by(|a, b| a.account_code.cmp(&b.account_code));

        let totals = TurnoverSheetEntry {
            account_id: 0,
            account_code: "ОБЩО".to_string(),
            account_name: "Общо за всички сметки".to_string(),
            opening_debit: total_opening_debit,
            opening_credit: total_opening_credit,
            period_debit: total_period_debit,
            period_credit: total_period_credit,
            closing_debit: total_closing_debit,
            closing_credit: total_closing_credit,
        };

        Ok(TurnoverSheet {
            company_name: company.name,
            period_start: input.start_date,
            period_end: input.end_date,
            entries,
            totals,
            generated_at: Utc::now(),
        })
    }

    /// Generate transaction log (дневник на операциите)
    async fn transaction_log(
        &self,
        ctx: &Context<'_>,
        input: TransactionLogInput,
    ) -> FieldResult<TransactionLog> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Get company info
        let company = company::Entity::find_by_id(input.company_id)
            .one(db)
            .await?
            .ok_or("Company not found")?;

        // Build query for entry lines with joins
        let mut query = entry_line::Entity::find()
            .left_join(journal_entry::Entity)
            .left_join(account::Entity)
            .left_join(counterpart::Entity)
            .filter(journal_entry::Column::CompanyId.eq(input.company_id))
            .filter(journal_entry::Column::AccountingDate.gte(input.start_date))
            .filter(journal_entry::Column::AccountingDate.lte(input.end_date))
            .filter(journal_entry::Column::IsPosted.eq(true));

        if let Some(account_id) = input.account_id {
            query = query.filter(entry_line::Column::AccountId.eq(account_id));
        }

        let lines = query
            .order_by_asc(journal_entry::Column::AccountingDate)
            .order_by_asc(journal_entry::Column::EntryNumber)
            .order_by_asc(entry_line::Column::LineOrder)
            .all(db)
            .await?;

        // TODO: This would need a proper join query implementation
        // For now, we'll do individual queries (not optimal but functional)
        let mut entries = Vec::new();

        for line in lines {
            let je = journal_entry::Entity::find_by_id(line.journal_entry_id)
                .one(db)
                .await?
                .ok_or("Journal entry not found")?;

            let account = account::Entity::find_by_id(line.account_id)
                .one(db)
                .await?
                .ok_or("Account not found")?;

            let counterpart_name = if let Some(counterpart_id) = line.counterpart_id {
                counterpart::Entity::find_by_id(counterpart_id)
                    .one(db)
                    .await?
                    .map(|c| c.name)
            } else {
                None
            };

            // Skip entries with both zero debit and credit amounts
            if line.debit_amount == Decimal::ZERO && line.credit_amount == Decimal::ZERO {
                continue;
            }

            entries.push(TransactionLogEntry {
                date: je.accounting_date,
                entry_number: je.entry_number,
                document_number: je.document_number,
                description: line.description.unwrap_or(je.description.clone()),
                account_code: account.code,
                account_name: account.name,
                debit_amount: line.debit_amount,
                credit_amount: line.credit_amount,
                counterpart_name,
            });
        }

        Ok(TransactionLog {
            company_name: company.name,
            period_start: input.start_date,
            period_end: input.end_date,
            entries,
            generated_at: Utc::now(),
        })
    }

    /// Generate chronological report (хронологичен регистър)
    async fn chronological_report(
        &self,
        ctx: &Context<'_>,
        input: ChronologicalReportInput,
    ) -> FieldResult<ChronologicalReport> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Get company info
        let company = company::Entity::find_by_id(input.company_id)
            .one(db)
            .await?
            .ok_or("Company not found")?;

        // Get all journal entries for the period
        let entry_query = journal_entry::Entity::find()
            .filter(journal_entry::Column::CompanyId.eq(input.company_id))
            .filter(journal_entry::Column::AccountingDate.gte(input.start_date))
            .filter(journal_entry::Column::AccountingDate.lte(input.end_date))
            .filter(journal_entry::Column::IsPosted.eq(true));

        let journal_entries = entry_query
            .order_by_asc(journal_entry::Column::AccountingDate)
            .order_by_asc(journal_entry::Column::EntryNumber)
            .all(db)
            .await?;

        let mut chronological_entries = Vec::new();
        let mut total_amount = Decimal::ZERO;

        for journal_entry in journal_entries {
            // Get all entry lines for this journal entry
            let entry_lines = entry_line::Entity::find()
                .filter(entry_line::Column::JournalEntryId.eq(journal_entry.id))
                .order_by_asc(entry_line::Column::LineOrder)
                .all(db)
                .await?;

            // Group lines by debit and credit
            let mut debit_lines = Vec::new();
            let mut credit_lines = Vec::new();

            for line in entry_lines {
                if line.debit_amount > Decimal::ZERO {
                    debit_lines.push(line);
                } else if line.credit_amount > Decimal::ZERO {
                    credit_lines.push(line);
                }
            }

            // Create chronological entries by pairing debits with credits
            for debit_line in &debit_lines {
                for credit_line in &credit_lines {
                    // Skip if account filter is set and neither line matches
                    if let Some(account_id) = input.account_id {
                        if debit_line.account_id != account_id
                            && credit_line.account_id != account_id
                        {
                            continue;
                        }
                    }

                    // Get account details
                    let debit_account = account::Entity::find_by_id(debit_line.account_id)
                        .one(db)
                        .await?
                        .ok_or("Debit account not found")?;

                    let credit_account = account::Entity::find_by_id(credit_line.account_id)
                        .one(db)
                        .await?
                        .ok_or("Credit account not found")?;

                    // Calculate amount (use the smaller of debit and credit amounts)
                    let amount = debit_line.debit_amount.min(credit_line.credit_amount);
                    total_amount += amount;

                    chronological_entries.push(ChronologicalEntry {
                        date: journal_entry.accounting_date,
                        debit_account_code: debit_account.code.clone(),
                        debit_account_name: debit_account.name.clone(),
                        credit_account_code: credit_account.code.clone(),
                        credit_account_name: credit_account.name.clone(),
                        amount,
                        // Include currency information from entry lines
                        debit_currency_amount: debit_line.currency_amount,
                        debit_currency_code: debit_line.currency_code.clone(),
                        credit_currency_amount: credit_line.currency_amount,
                        credit_currency_code: credit_line.currency_code.clone(),
                        document_type: journal_entry.vat_document_type.clone(),
                        document_date: Some(journal_entry.document_date),
                        description: debit_line
                            .description
                            .clone()
                            .or_else(|| credit_line.description.clone())
                            .unwrap_or_else(|| journal_entry.description.clone()),
                    });
                }
            }
        }

        Ok(ChronologicalReport {
            company_name: company.name,
            period_start: input.start_date,
            period_end: input.end_date,
            entries: chronological_entries,
            total_amount,
            generated_at: Utc::now(),
        })
    }

    /// Generate Bulgarian variant general ledger (BG главна книга)
    /// Shows simple Debit/Credit pairs grouped by debit and credit accounts
    async fn bg_general_ledger(
        &self,
        ctx: &Context<'_>,
        input: BgGeneralLedgerInput,
    ) -> FieldResult<BgGeneralLedger> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Get company info
        let company = company::Entity::find_by_id(input.company_id)
            .one(db)
            .await?
            .ok_or("Company not found")?;

        // Get all journal entries for the period
        let entry_query = journal_entry::Entity::find()
            .filter(journal_entry::Column::CompanyId.eq(input.company_id))
            .filter(journal_entry::Column::AccountingDate.gte(input.start_date))
            .filter(journal_entry::Column::AccountingDate.lte(input.end_date))
            .filter(journal_entry::Column::IsPosted.eq(true));

        let journal_entries = entry_query
            .order_by_asc(journal_entry::Column::AccountingDate)
            .order_by_asc(journal_entry::Column::EntryNumber)
            .all(db)
            .await?;

        // Collect all debit/credit pairs
        use std::collections::HashMap;

        // Map: (debit_account_id, credit_account_id) -> total_amount
        let mut pairs_map: HashMap<(i32, i32), Decimal> = HashMap::new();

        // Map: account_id -> (code, name)
        let mut account_info: HashMap<i32, (String, String)> = HashMap::new();

        for journal_entry in journal_entries {
            // Get all entry lines for this journal entry
            let entry_lines = entry_line::Entity::find()
                .filter(entry_line::Column::JournalEntryId.eq(journal_entry.id))
                .order_by_asc(entry_line::Column::LineOrder)
                .all(db)
                .await?;

            // Group lines by debit and credit
            let mut debit_lines = Vec::new();
            let mut credit_lines = Vec::new();

            for line in entry_lines {
                if line.debit_amount > Decimal::ZERO {
                    debit_lines.push(line);
                } else if line.credit_amount > Decimal::ZERO {
                    credit_lines.push(line);
                }
            }

            // Create pairs by matching debits with credits
            for debit_line in &debit_lines {
                for credit_line in &credit_lines {
                    // Skip if account filter is set and neither line matches
                    if let Some(account_id) = input.account_id {
                        if debit_line.account_id != account_id
                            && credit_line.account_id != account_id
                        {
                            continue;
                        }
                    }

                    // Get or cache account info
                    if !account_info.contains_key(&debit_line.account_id) {
                        let acc = account::Entity::find_by_id(debit_line.account_id)
                            .one(db)
                            .await?
                            .ok_or("Debit account not found")?;
                        account_info.insert(debit_line.account_id, (acc.code, acc.name));
                    }

                    if !account_info.contains_key(&credit_line.account_id) {
                        let acc = account::Entity::find_by_id(credit_line.account_id)
                            .one(db)
                            .await?
                            .ok_or("Credit account not found")?;
                        account_info.insert(credit_line.account_id, (acc.code, acc.name));
                    }

                    // Calculate amount (use the smaller of debit and credit amounts)
                    let amount = debit_line.debit_amount.min(credit_line.credit_amount);

                    // Aggregate amounts
                    let key = (debit_line.account_id, credit_line.account_id);
                    *pairs_map.entry(key).or_insert(Decimal::ZERO) += amount;
                }
            }
        }

        // Group by debit account
        let mut by_debit_map: HashMap<i32, Vec<BgLedgerByDebitEntry>> = HashMap::new();

        for ((debit_id, credit_id), amount) in &pairs_map {
            let credit_info = account_info.get(credit_id).unwrap();
            by_debit_map.entry(*debit_id).or_insert_with(Vec::new).push(
                BgLedgerByDebitEntry {
                    credit_account_code: credit_info.0.clone(),
                    credit_account_name: credit_info.1.clone(),
                    amount: *amount,
                }
            );
        }

        let mut by_debit = Vec::new();
        for (debit_id, entries) in by_debit_map {
            let debit_info = account_info.get(&debit_id).unwrap();
            let total_amount: Decimal = entries.iter().map(|e| e.amount).sum();
            by_debit.push(BgLedgerByDebit {
                debit_account_code: debit_info.0.clone(),
                debit_account_name: debit_info.1.clone(),
                entries,
                total_amount,
            });
        }

        // Sort by debit account code
        by_debit.sort_by(|a, b| a.debit_account_code.cmp(&b.debit_account_code));

        // Group by credit account
        let mut by_credit_map: HashMap<i32, Vec<BgLedgerByCreditEntry>> = HashMap::new();

        for ((debit_id, credit_id), amount) in &pairs_map {
            let debit_info = account_info.get(debit_id).unwrap();
            by_credit_map.entry(*credit_id).or_insert_with(Vec::new).push(
                BgLedgerByCreditEntry {
                    debit_account_code: debit_info.0.clone(),
                    debit_account_name: debit_info.1.clone(),
                    amount: *amount,
                }
            );
        }

        let mut by_credit = Vec::new();
        for (credit_id, entries) in by_credit_map {
            let credit_info = account_info.get(&credit_id).unwrap();
            let total_amount: Decimal = entries.iter().map(|e| e.amount).sum();
            by_credit.push(BgLedgerByCredit {
                credit_account_code: credit_info.0.clone(),
                credit_account_name: credit_info.1.clone(),
                entries,
                total_amount,
            });
        }

        // Sort by credit account code
        by_credit.sort_by(|a, b| a.credit_account_code.cmp(&b.credit_account_code));

        Ok(BgGeneralLedger {
            company_name: company.name,
            period_start: input.start_date,
            period_end: input.end_date,
            by_debit,
            by_credit,
            generated_at: Utc::now(),
        })
    }

    /// Generate monthly transaction statistics for pricing
    async fn monthly_transaction_stats(
        &self,
        ctx: &Context<'_>,
        input: MonthlyStatsInput,
    ) -> FieldResult<Vec<MonthlyTransactionStats>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut stats = Vec::new();

        // Iterate through each month in the range
        let mut current_year = input.from_year;
        let mut current_month = input.from_month;

        while current_year < input.to_year
            || (current_year == input.to_year && current_month <= input.to_month)
        {
            // Calculate start and end dates for the month
            let start_date = NaiveDate::from_ymd_opt(current_year, current_month as u32, 1)
                .ok_or("Invalid start date")?;
            let end_date = if current_month == 12 {
                NaiveDate::from_ymd_opt(current_year, 12, 31).ok_or("Invalid end date")?
            } else {
                NaiveDate::from_ymd_opt(current_year, (current_month + 1) as u32, 1)
                    .ok_or("Invalid end date")?
                    .pred_opt()
                    .ok_or("Invalid end date")?
            };

            // Count total entries for the month
            let total_entries = journal_entry::Entity::find()
                .filter(journal_entry::Column::CompanyId.eq(input.company_id))
                .filter(journal_entry::Column::AccountingDate.gte(start_date))
                .filter(journal_entry::Column::AccountingDate.lte(end_date))
                .count(db)
                .await? as i64;

            // Count posted entries
            let posted_entries = journal_entry::Entity::find()
                .filter(journal_entry::Column::CompanyId.eq(input.company_id))
                .filter(journal_entry::Column::AccountingDate.gte(start_date))
                .filter(journal_entry::Column::AccountingDate.lte(end_date))
                .filter(journal_entry::Column::IsPosted.eq(true))
                .count(db)
                .await? as i64;

            // Count total entry lines
            let total_entry_lines = entry_line::Entity::find()
                .inner_join(journal_entry::Entity)
                .filter(journal_entry::Column::CompanyId.eq(input.company_id))
                .filter(journal_entry::Column::AccountingDate.gte(start_date))
                .filter(journal_entry::Column::AccountingDate.lte(end_date))
                .count(db)
                .await? as i64;

            // Count posted entry lines
            let posted_entry_lines = entry_line::Entity::find()
                .inner_join(journal_entry::Entity)
                .filter(journal_entry::Column::CompanyId.eq(input.company_id))
                .filter(journal_entry::Column::AccountingDate.gte(start_date))
                .filter(journal_entry::Column::AccountingDate.lte(end_date))
                .filter(journal_entry::Column::IsPosted.eq(true))
                .count(db)
                .await? as i64;

            // Calculate total amount and VAT
            let entries = journal_entry::Entity::find()
                .filter(journal_entry::Column::CompanyId.eq(input.company_id))
                .filter(journal_entry::Column::AccountingDate.gte(start_date))
                .filter(journal_entry::Column::AccountingDate.lte(end_date))
                .filter(journal_entry::Column::IsPosted.eq(true))
                .all(db)
                .await?;

            let mut total_amount = Decimal::ZERO;
            let mut vat_amount = Decimal::ZERO;

            for entry in entries {
                total_amount += entry.total_amount;
                vat_amount += entry.total_vat_amount;
            }

            // Month names in Bulgarian
            let month_names = [
                "Януари",
                "Февруари",
                "Март",
                "Април",
                "Май",
                "Юни",
                "Юли",
                "Август",
                "Септември",
                "Октомври",
                "Ноември",
                "Декември",
            ];

            stats.push(MonthlyTransactionStats {
                year: current_year,
                month: current_month,
                month_name: month_names[(current_month - 1) as usize].to_string(),
                total_entries,
                posted_entries,
                total_entry_lines,
                posted_entry_lines,
                total_amount,
                vat_amount,
            });

            // Move to next month
            if current_month == 12 {
                current_year += 1;
                current_month = 1;
            } else {
                current_month += 1;
            }
        }

        Ok(stats)
    }

    /// Generate general ledger (главна книга) - Bulgarian style
    async fn general_ledger(
        &self,
        ctx: &Context<'_>,
        input: GeneralLedgerInput,
    ) -> FieldResult<GeneralLedger> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Get company info
        let company = company::Entity::find_by_id(input.company_id)
            .one(db)
            .await?
            .ok_or("Company not found")?;

        // Get accounts to process - all accounts, not just analytical
        let mut account_query = account::Entity::find()
            .filter(account::Column::CompanyId.eq(input.company_id))
            .filter(account::Column::IsActive.eq(true));
        // Remove analytical filter - we want all accounts that have transactions

        if let Some(account_id) = input.account_id {
            account_query = account_query.filter(account::Column::Id.eq(account_id));
        }

        let accounts = account_query
            .order_by_asc(account::Column::Code)
            .all(db)
            .await?;

        let mut ledger_accounts = Vec::new();

        for account in accounts {
            // Calculate opening balance (before start_date)
            let opening_lines = entry_line::Entity::find()
                .left_join(journal_entry::Entity)
                .filter(entry_line::Column::AccountId.eq(account.id))
                .filter(journal_entry::Column::CompanyId.eq(input.company_id))
                .filter(journal_entry::Column::AccountingDate.lt(input.start_date))
                .filter(journal_entry::Column::IsPosted.eq(true))
                .all(db)
                .await?;

            let mut opening_balance = Decimal::ZERO;
            for line in opening_lines {
                opening_balance += line.debit_amount - line.credit_amount;
            }

            // Get period transactions
            let period_lines = entry_line::Entity::find()
                .left_join(journal_entry::Entity)
                .filter(entry_line::Column::AccountId.eq(account.id))
                .filter(journal_entry::Column::CompanyId.eq(input.company_id))
                .filter(journal_entry::Column::AccountingDate.gte(input.start_date))
                .filter(journal_entry::Column::AccountingDate.lte(input.end_date))
                .filter(journal_entry::Column::IsPosted.eq(true))
                .order_by_asc(journal_entry::Column::AccountingDate)
                .order_by_asc(entry_line::Column::LineOrder)
                .all(db)
                .await?;

            let mut entries = Vec::new();
            let mut running_balance = opening_balance;
            let mut total_debits = Decimal::ZERO;
            let mut total_credits = Decimal::ZERO;

            for line in period_lines {
                // Skip entries with both zero debit and credit amounts
                if line.debit_amount == Decimal::ZERO && line.credit_amount == Decimal::ZERO {
                    continue;
                }

                // Get journal entry details
                let je = journal_entry::Entity::find_by_id(line.journal_entry_id)
                    .one(db)
                    .await?
                    .ok_or("Journal entry not found")?;

                // Get counterpart name
                let counterpart_name = if let Some(counterpart_id) = line.counterpart_id {
                    counterpart::Entity::find_by_id(counterpart_id)
                        .one(db)
                        .await?
                        .map(|c| c.name)
                } else {
                    None
                };

                // Update running balance
                running_balance += line.debit_amount - line.credit_amount;
                total_debits += line.debit_amount;
                total_credits += line.credit_amount;

                entries.push(GeneralLedgerEntry {
                    date: je.accounting_date,
                    entry_number: je.entry_number,
                    document_number: je.document_number,
                    description: line.description.unwrap_or(je.description.clone()),
                    debit_amount: line.debit_amount,
                    credit_amount: line.credit_amount,
                    balance: running_balance,
                    counterpart_name,
                });
            }

            // Only include accounts with opening balance or period activity
            if opening_balance != Decimal::ZERO
                || total_debits != Decimal::ZERO
                || total_credits != Decimal::ZERO
            {
                ledger_accounts.push(GeneralLedgerAccount {
                    account_id: account.id,
                    account_code: account.code,
                    account_name: account.name,
                    opening_balance,
                    closing_balance: running_balance,
                    total_debits,
                    total_credits,
                    entries,
                });
            }
        }

        Ok(GeneralLedger {
            company_name: company.name,
            period_start: input.start_date,
            period_end: input.end_date,
            accounts: ledger_accounts,
            generated_at: Utc::now(),
        })
    }
}

#[derive(Default)]
pub struct ReportsMutation;

#[Object]
impl ReportsMutation {
    /// Export chronological report in specified format
    async fn export_chronological_report(
        &self,
        ctx: &Context<'_>,
        input: ChronologicalReportInput,
        format: String, // "XLSX", "ODT"
    ) -> FieldResult<ReportExport> {
        let reports_query = ReportsQuery::default();
        let chronological_report = reports_query.chronological_report(ctx, input).await?;

        let filename = format!(
            "chronological_report_{}_{}_{}.{}",
            chronological_report.period_start,
            chronological_report.period_end,
            chronological_report.company_name.replace(" ", "_"),
            format.to_lowercase()
        );

        match format.to_uppercase().as_str() {
            "XLSX" => {
                let content = generate_xlsx_chronological(&chronological_report)?;
                let encoded_content = base64::prelude::BASE64_STANDARD.encode(&content);
                Ok(ReportExport {
                    format: "XLSX".to_string(),
                    content: encoded_content,
                    filename,
                    mime_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                        .to_string(),
                })
            }
            "ODT" => {
                let content = generate_odt_chronological(&chronological_report)?;
                let encoded_content = base64::prelude::BASE64_STANDARD.encode(&content);
                Ok(ReportExport {
                    format: "ODT".to_string(),
                    content: encoded_content,
                    filename,
                    mime_type: "application/vnd.oasis.opendocument.text".to_string(),
                })
            }
            _ => Err("Unsupported format. Use XLSX or ODT".into()),
        }
    }

    /// Export monthly statistics in specified format (XLSX or ODT)
    async fn export_monthly_stats(
        &self,
        ctx: &Context<'_>,
        input: MonthlyStatsInput,
        format: String, // "XLSX", "ODT"
    ) -> FieldResult<ReportExport> {
        let reports_query = ReportsQuery::default();
        let stats = reports_query.monthly_transaction_stats(ctx, input.clone()).await?;

        // Get company info
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let company = company::Entity::find_by_id(input.company_id)
            .one(db)
            .await?
            .ok_or("Company not found")?;

        let filename = format!(
            "monthly_stats_{}_{}_{}_{}{}",
            input.from_year, input.from_month, input.to_year, input.to_month,
            format.to_lowercase()
        );

        match format.to_uppercase().as_str() {
            "XLSX" => {
                let content = generate_xlsx_monthly_stats(&stats, &company.name)?;
                let encoded_content = base64::prelude::BASE64_STANDARD.encode(&content);
                Ok(ReportExport {
                    format: "XLSX".to_string(),
                    content: encoded_content,
                    filename: format!("{}.xlsx", filename),
                    mime_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                        .to_string(),
                })
            }
            "ODT" => {
                let content = generate_monthly_stats_odt(&stats, &company.name)?;
                let encoded_content = base64::prelude::BASE64_STANDARD.encode(&content);
                Ok(ReportExport {
                    format: "ODT".to_string(),
                    content: encoded_content,
                    filename: format!("{}.odt", filename),
                    mime_type: "application/vnd.oasis.opendocument.text".to_string(),
                })
            }
            _ => Err("Unsupported format. Use XLSX or ODT".into()),
        }
    }

    /// Export turnover sheet in specified format
    async fn export_turnover_sheet(
        &self,
        ctx: &Context<'_>,
        input: TurnoverReportInput,
        format: String, // "XLSX", "ODT"
    ) -> FieldResult<ReportExport> {
        let reports_query = ReportsQuery::default();
        let turnover_sheet = reports_query.turnover_sheet(ctx, input).await?;

        let filename = format!(
            "turnover_sheet_{}_{}_{}.{}",
            turnover_sheet.period_start,
            turnover_sheet.period_end,
            turnover_sheet.company_name.replace(" ", "_"),
            format.to_lowercase()
        );

        match format.to_uppercase().as_str() {
            "XLSX" => {
                let content = generate_xlsx_turnover(&turnover_sheet)?;
                let encoded_content = base64::prelude::BASE64_STANDARD.encode(&content);
                Ok(ReportExport {
                    format: "XLSX".to_string(),
                    content: encoded_content,
                    filename,
                    mime_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                        .to_string(),
                })
            }
            "ODT" => {
                let content = generate_odt_turnover(&turnover_sheet)?;
                let encoded_content = base64::prelude::BASE64_STANDARD.encode(&content);
                Ok(ReportExport {
                    format: "ODT".to_string(),
                    content: encoded_content,
                    filename,
                    mime_type: "application/vnd.oasis.opendocument.text".to_string(),
                })
            }
            _ => Err("Unsupported format. Use XLSX or ODT".into()),
        }
    }

    /// Export General Ledger in specified format
    async fn export_general_ledger(
        &self,
        ctx: &Context<'_>,
        input: GeneralLedgerInput,
        format: String, // "XLSX", "ODT"
    ) -> FieldResult<ReportExport> {
        let reports_query = ReportsQuery::default();
        let general_ledger = reports_query.general_ledger(ctx, input).await?;

        let filename = format!(
            "general_ledger_{}_{}_{}.{}",
            general_ledger.period_start,
            general_ledger.period_end,
            general_ledger.company_name.replace(" ", "_"),
            format.to_lowercase()
        );

        match format.to_uppercase().as_str() {
            "XLSX" => {
                let content = generate_xlsx_general_ledger(&general_ledger)?;
                let encoded_content = base64::prelude::BASE64_STANDARD.encode(&content);
                Ok(ReportExport {
                    format: "XLSX".to_string(),
                    content: encoded_content,
                    filename,
                    mime_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                        .to_string(),
                })
            }
            "ODT" => {
                let content = generate_odt_general_ledger(&general_ledger)?;
                let encoded_content = base64::prelude::BASE64_STANDARD.encode(&content);
                Ok(ReportExport {
                    format: "ODT".to_string(),
                    content: encoded_content,
                    filename,
                    mime_type: "application/vnd.oasis.opendocument.text".to_string(),
                })
            }
            _ => Err("Unsupported format. Use XLSX or ODT".into()),
        }
    }

    /// Export BG General Ledger in specified format
    async fn export_bg_general_ledger(
        &self,
        ctx: &Context<'_>,
        input: BgGeneralLedgerInput,
        format: String, // "XLSX", "ODT"
    ) -> FieldResult<ReportExport> {
        let reports_query = ReportsQuery::default();
        let bg_general_ledger = reports_query.bg_general_ledger(ctx, input).await?;

        let filename = format!(
            "bg_general_ledger_{}_{}_{}.{}",
            bg_general_ledger.period_start,
            bg_general_ledger.period_end,
            bg_general_ledger.company_name.replace(" ", "_"),
            format.to_lowercase()
        );

        match format.to_uppercase().as_str() {
            "XLSX" => {
                let content = generate_xlsx_bg_general_ledger(&bg_general_ledger)?;
                let encoded_content = base64::prelude::BASE64_STANDARD.encode(&content);
                Ok(ReportExport {
                    format: "XLSX".to_string(),
                    content: encoded_content,
                    filename,
                    mime_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                        .to_string(),
                })
            }
            "ODT" => {
                let content = generate_odt_bg_general_ledger(&bg_general_ledger)?;
                let encoded_content = base64::prelude::BASE64_STANDARD.encode(&content);
                Ok(ReportExport {
                    format: "ODT".to_string(),
                    content: encoded_content,
                    filename,
                    mime_type: "application/vnd.oasis.opendocument.text".to_string(),
                })
            }
            _ => Err("Unsupported format. Use XLSX or ODT".into()),
        }
    }
}

// Helper functions for export generation - ODT format
fn generate_odt_chronological(
    report: &ChronologicalReport,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use std::io::Write;
    use zip::write::{FileOptions, ZipWriter};

    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));

    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    // mimetype (uncompressed)
    zip.start_file("mimetype", FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored))?;
    zip.write_all(b"application/vnd.oasis.opendocument.text")?;

    // content.xml - the actual document content
    let content_xml = generate_odt_content_chronological(report);
    zip.start_file("content.xml", options)?;
    zip.write_all(content_xml.as_bytes())?;

    // manifest
    zip.start_file("META-INF/manifest.xml", options)?;
    zip.write_all(ODT_MANIFEST.as_bytes())?;

    // styles
    zip.start_file("styles.xml", options)?;
    zip.write_all(ODT_STYLES.as_bytes())?;

    zip.finish()?;
    Ok(buffer)
}

// ODT Constants
const ODT_MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0">
  <manifest:file-entry manifest:media-type="application/vnd.oasis.opendocument.text" manifest:full-path="/"/>
  <manifest:file-entry manifest:media-type="text/xml" manifest:full-path="content.xml"/>
  <manifest:file-entry manifest:media-type="text/xml" manifest:full-path="styles.xml"/>
</manifest:manifest>"#;

const ODT_STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-styles xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
 xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0">
  <office:styles>
    <style:default-style style:family="paragraph"/>
    <style:default-style style:family="table"/>
  </office:styles>
</office:document-styles>"#;

fn generate_odt_content_chronological(report: &ChronologicalReport) -> String {
    let mut rows = String::new();
    for entry in &report.entries {
        rows.push_str(&format!(
            r#"
        <table:table-row>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
        </table:table-row>"#,
            entry.date.format("%d.%m.%Y"),
            entry.debit_account_code,
            entry.debit_account_name,
            entry.credit_account_code,
            entry.credit_account_name,
            entry.amount,
            entry.description
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
 xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0">
  <office:body>
    <office:text>
      <text:p><text:span text:style-name="Title">Хронологичен регистър - {}</text:span></text:p>
      <text:p>от {} до {}</text:p>
      <table:table table:name="ChronologicalReport">
        <table:table-column table:number-columns-repeated="7"/>
        <table:table-row>
          <table:table-cell><text:p>Дата</text:p></table:table-cell>
          <table:table-cell><text:p>Дебит</text:p></table:table-cell>
          <table:table-cell><text:p>Дебит име</text:p></table:table-cell>
          <table:table-cell><text:p>Кредит</text:p></table:table-cell>
          <table:table-cell><text:p>Кредит име</text:p></table:table-cell>
          <table:table-cell><text:p>Сума</text:p></table:table-cell>
          <table:table-cell><text:p>Описание</text:p></table:table-cell>
        </table:table-row>
        {}
        <table:table-row>
          <table:table-cell table:number-columns-spanned="5"><text:p>Общо</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p></text:p></table:table-cell>
        </table:table-row>
      </table:table>
    </office:text>
  </office:body>
</office:document-content>"#,
        report.company_name,
        report.period_start.format("%d.%m.%Y"),
        report.period_end.format("%d.%m.%Y"),
        rows,
        report.total_amount
    )
}

fn generate_odt_turnover(
    sheet: &TurnoverSheet,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use std::io::Write;
    use zip::write::{FileOptions, ZipWriter};

    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));

    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("mimetype", FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored))?;
    zip.write_all(b"application/vnd.oasis.opendocument.text")?;

    let content_xml = generate_odt_content_turnover(sheet);
    zip.start_file("content.xml", options)?;
    zip.write_all(content_xml.as_bytes())?;

    zip.start_file("META-INF/manifest.xml", options)?;
    zip.write_all(ODT_MANIFEST.as_bytes())?;

    zip.start_file("styles.xml", options)?;
    zip.write_all(ODT_STYLES.as_bytes())?;

    zip.finish()?;
    Ok(buffer)
}

fn generate_odt_content_turnover(sheet: &TurnoverSheet) -> String {
    let mut rows = String::new();
    for entry in &sheet.entries {
        rows.push_str(&format!(
            r#"
        <table:table-row>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
        </table:table-row>"#,
            entry.account_code,
            entry.account_name,
            entry.opening_debit,
            entry.opening_credit,
            entry.period_debit,
            entry.period_credit,
            entry.closing_debit,
            entry.closing_credit
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
 xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0">
  <office:body>
    <office:text>
      <text:p><text:span text:style-name="Title">Оборотна ведомост - {}</text:span></text:p>
      <text:p>Период: {} - {}</text:p>
      <table:table table:name="TurnoverSheet">
        <table:table-column table:number-columns-repeated="8"/>
        <table:table-row>
          <table:table-cell><text:p>Код сметка</text:p></table:table-cell>
          <table:table-cell><text:p>Име сметка</text:p></table:table-cell>
          <table:table-cell><text:p>Начално салдо Дт</text:p></table:table-cell>
          <table:table-cell><text:p>Начално салдо Кт</text:p></table:table-cell>
          <table:table-cell><text:p>Обороти Дт</text:p></table:table-cell>
          <table:table-cell><text:p>Обороти Кт</text:p></table:table-cell>
          <table:table-cell><text:p>Крайно салдо Дт</text:p></table:table-cell>
          <table:table-cell><text:p>Крайно салдо Кт</text:p></table:table-cell>
        </table:table-row>
        {}
        <table:table-row>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
        </table:table-row>
      </table:table>
    </office:text>
  </office:body>
</office:document-content>"#,
        sheet.company_name,
        sheet.period_start,
        sheet.period_end,
        rows,
        sheet.totals.account_code,
        sheet.totals.account_name,
        sheet.totals.opening_debit,
        sheet.totals.opening_credit,
        sheet.totals.period_debit,
        sheet.totals.period_credit,
        sheet.totals.closing_debit,
        sheet.totals.closing_credit
    )
}

fn generate_monthly_stats_odt(
    stats: &[MonthlyTransactionStats],
    company_name: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use std::io::Write;
    use zip::write::{FileOptions, ZipWriter};

    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));

    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("mimetype", FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored))?;
    zip.write_all(b"application/vnd.oasis.opendocument.text")?;

    let content_xml = generate_odt_content_monthly_stats(stats, company_name);
    zip.start_file("content.xml", options)?;
    zip.write_all(content_xml.as_bytes())?;

    zip.start_file("META-INF/manifest.xml", options)?;
    zip.write_all(ODT_MANIFEST.as_bytes())?;

    zip.start_file("styles.xml", options)?;
    zip.write_all(ODT_STYLES.as_bytes())?;

    zip.finish()?;
    Ok(buffer)
}

fn generate_odt_content_monthly_stats(stats: &[MonthlyTransactionStats], company_name: &str) -> String {
    let mut rows = String::new();
    let mut total_entries: i64 = 0;
    let mut total_posted_entries: i64 = 0;
    let mut total_lines: i64 = 0;
    let mut total_posted_lines: i64 = 0;
    let mut total_amount = Decimal::ZERO;
    let mut total_vat = Decimal::ZERO;

    for s in stats {
        total_entries += s.total_entries;
        total_posted_entries += s.posted_entries;
        total_lines += s.total_entry_lines;
        total_posted_lines += s.posted_entry_lines;
        total_amount += s.total_amount;
        total_vat += s.vat_amount;

        rows.push_str(&format!(
            r#"
        <table:table-row>
          <table:table-cell><text:p>{} {}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{:.2}</text:p></table:table-cell>
          <table:table-cell><text:p>{:.2}</text:p></table:table-cell>
        </table:table-row>"#,
            s.month_name, s.year,
            s.total_entries,
            s.posted_entries,
            s.total_entry_lines,
            s.posted_entry_lines,
            s.total_amount,
            s.vat_amount
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
 xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0">
  <office:body>
    <office:text>
      <text:p><text:span text:style-name="Title">Месечна статистика на транзакции</text:span></text:p>
      <text:p>{}</text:p>
      <table:table table:name="MonthlyStats">
        <table:table-column table:number-columns-repeated="7"/>
        <table:table-row>
          <table:table-cell><text:p>Период</text:p></table:table-cell>
          <table:table-cell><text:p>Документи (общо)</text:p></table:table-cell>
          <table:table-cell><text:p>Документи (приключени)</text:p></table:table-cell>
          <table:table-cell><text:p>Редове Дт/Кт (общо)</text:p></table:table-cell>
          <table:table-cell><text:p>Редове Дт/Кт (приключени)</text:p></table:table-cell>
          <table:table-cell><text:p>Оборот (лв.)</text:p></table:table-cell>
          <table:table-cell><text:p>ДДС (лв.)</text:p></table:table-cell>
        </table:table-row>
        {}
        <table:table-row>
          <table:table-cell><text:p>ОБЩО</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{}</text:p></table:table-cell>
          <table:table-cell><text:p>{:.2}</text:p></table:table-cell>
          <table:table-cell><text:p>{:.2}</text:p></table:table-cell>
        </table:table-row>
      </table:table>
    </office:text>
  </office:body>
</office:document-content>"#,
        company_name,
        rows,
        total_entries,
        total_posted_entries,
        total_lines,
        total_posted_lines,
        total_amount,
        total_vat
    )
}

fn generate_xlsx_monthly_stats(
    stats: &[MonthlyTransactionStats],
    company_name: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use rust_xlsxwriter::*;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Set column widths
    worksheet.set_column_width(0, 18.0)?; // Period
    worksheet.set_column_width(1, 15.0)?; // Total entries
    worksheet.set_column_width(2, 15.0)?; // Posted entries
    worksheet.set_column_width(3, 18.0)?; // Total entry lines
    worksheet.set_column_width(4, 18.0)?; // Posted entry lines
    worksheet.set_column_width(5, 15.0)?; // Total amount
    worksheet.set_column_width(6, 15.0)?; // VAT amount

    // Title and header formats
    let title_format = Format::new()
        .set_font_size(16)
        .set_bold()
        .set_align(FormatAlign::Center);

    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xE8E8E8))
        .set_border(FormatBorder::Thin);

    let number_format = Format::new()
        .set_num_format("0.00")
        .set_border(FormatBorder::Thin);

    let integer_format = Format::new()
        .set_num_format("0")
        .set_border(FormatBorder::Thin);

    let text_format = Format::new().set_border(FormatBorder::Thin);

    let totals_format = Format::new()
        .set_bold()
        .set_num_format("0.00")
        .set_background_color(Color::RGB(0xD0E4F7))
        .set_border(FormatBorder::Thin);

    let totals_integer_format = Format::new()
        .set_bold()
        .set_num_format("0")
        .set_background_color(Color::RGB(0xD0E4F7))
        .set_border(FormatBorder::Thin);

    let mut row = 0;

    // Title
    worksheet.merge_range(
        row,
        0,
        row,
        6,
        &format!("Месечна статистика на транзакции - {}", company_name),
        &title_format,
    )?;
    row += 2;

    // Headers
    worksheet.write_string_with_format(row, 0, "Период", &header_format)?;
    worksheet.write_string_with_format(row, 1, "Документи (общо)", &header_format)?;
    worksheet.write_string_with_format(row, 2, "Документи (приключени)", &header_format)?;
    worksheet.write_string_with_format(row, 3, "Редове Дт/Кт (общо)", &header_format)?;
    worksheet.write_string_with_format(row, 4, "Редове Дт/Кт (приключени)", &header_format)?;
    worksheet.write_string_with_format(row, 5, "Оборот (лв.)", &header_format)?;
    worksheet.write_string_with_format(row, 6, "ДДС (лв.)", &header_format)?;
    row += 1;

    // Calculate totals
    let mut total_entries: i64 = 0;
    let mut total_posted_entries: i64 = 0;
    let mut total_lines: i64 = 0;
    let mut total_posted_lines: i64 = 0;
    let mut total_amount = Decimal::ZERO;
    let mut total_vat = Decimal::ZERO;

    // Data rows
    for stat in stats {
        total_entries += stat.total_entries;
        total_posted_entries += stat.posted_entries;
        total_lines += stat.total_entry_lines;
        total_posted_lines += stat.posted_entry_lines;
        total_amount += stat.total_amount;
        total_vat += stat.vat_amount;

        worksheet.write_string_with_format(
            row,
            0,
            &format!("{} {}", stat.month_name, stat.year),
            &text_format,
        )?;
        worksheet.write_number_with_format(row, 1, stat.total_entries as f64, &integer_format)?;
        worksheet.write_number_with_format(row, 2, stat.posted_entries as f64, &integer_format)?;
        worksheet.write_number_with_format(row, 3, stat.total_entry_lines as f64, &integer_format)?;
        worksheet.write_number_with_format(row, 4, stat.posted_entry_lines as f64, &integer_format)?;
        worksheet.write_number_with_format(
            row,
            5,
            stat.total_amount.to_f64().unwrap_or(0.0),
            &number_format,
        )?;
        worksheet.write_number_with_format(
            row,
            6,
            stat.vat_amount.to_f64().unwrap_or(0.0),
            &number_format,
        )?;
        row += 1;
    }

    // Totals row
    worksheet.write_string_with_format(row, 0, "ОБЩО", &totals_format)?;
    worksheet.write_number_with_format(row, 1, total_entries as f64, &totals_integer_format)?;
    worksheet.write_number_with_format(row, 2, total_posted_entries as f64, &totals_integer_format)?;
    worksheet.write_number_with_format(row, 3, total_lines as f64, &totals_integer_format)?;
    worksheet.write_number_with_format(row, 4, total_posted_lines as f64, &totals_integer_format)?;
    worksheet.write_number_with_format(row, 5, total_amount.to_f64().unwrap_or(0.0), &totals_format)?;
    worksheet.write_number_with_format(row, 6, total_vat.to_f64().unwrap_or(0.0), &totals_format)?;

    // Save to buffer
    let buffer = workbook.save_to_buffer()?;
    Ok(buffer)
}

// Helper functions for export generation - XLSX format
fn generate_xlsx_chronological(
    report: &ChronologicalReport,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use rust_xlsxwriter::*;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Set column widths
    worksheet.set_column_width(0, 12.0)?; // Date
    worksheet.set_column_width(1, 15.0)?; // Debit code
    worksheet.set_column_width(2, 25.0)?; // Debit name
    worksheet.set_column_width(3, 15.0)?; // Credit code
    worksheet.set_column_width(4, 25.0)?; // Credit name
    worksheet.set_column_width(5, 15.0)?; // Amount
    worksheet.set_column_width(6, 15.0)?; // Currency amounts
    worksheet.set_column_width(7, 10.0)?; // Currency
    worksheet.set_column_width(8, 15.0)?; // Currency amounts
    worksheet.set_column_width(9, 10.0)?; // Currency
    worksheet.set_column_width(10, 15.0)?; // Doc type
    worksheet.set_column_width(11, 12.0)?; // Doc date
    worksheet.set_column_width(12, 30.0)?; // Description

    // Title and header formats
    let title_format = Format::new()
        .set_font_size(16)
        .set_bold()
        .set_align(FormatAlign::Center);

    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xE8E8E8))
        .set_border(FormatBorder::Thin);

    let number_format = Format::new()
        .set_num_format("0.00")
        .set_border(FormatBorder::Thin);

    let text_format = Format::new().set_border(FormatBorder::Thin);

    let date_format = Format::new()
        .set_num_format("dd.mm.yyyy")
        .set_border(FormatBorder::Thin);

    let totals_format = Format::new()
        .set_bold()
        .set_num_format("0.00")
        .set_background_color(Color::RGB(0xF0F0F0))
        .set_border(FormatBorder::Thin);

    let mut row = 0;

    // Title
    worksheet.merge_range(
        row,
        0,
        row,
        12,
        &format!("Хронологичен регистър - {}", report.company_name),
        &title_format,
    )?;
    row += 1;

    // Period info
    worksheet.merge_range(
        row,
        0,
        row,
        12,
        &format!("от {} до {}", report.period_start, report.period_end),
        &title_format,
    )?;
    row += 2;

    // Headers
    worksheet.write_string_with_format(row, 0, "Дата", &header_format)?;
    worksheet.write_string_with_format(row, 1, "Дебит", &header_format)?;
    worksheet.write_string_with_format(row, 2, "Дебит име", &header_format)?;
    worksheet.write_string_with_format(row, 3, "Кредит", &header_format)?;
    worksheet.write_string_with_format(row, 4, "Кредит име", &header_format)?;
    worksheet.write_string_with_format(row, 5, "Сума", &header_format)?;
    worksheet.write_string_with_format(row, 6, "Дебит валутна сума", &header_format)?;
    worksheet.write_string_with_format(row, 7, "Дебит валута", &header_format)?;
    worksheet.write_string_with_format(row, 8, "Кредит валутна сума", &header_format)?;
    worksheet.write_string_with_format(row, 9, "Кредит валута", &header_format)?;
    worksheet.write_string_with_format(row, 10, "Док. вид", &header_format)?;
    worksheet.write_string_with_format(row, 11, "Док. дата", &header_format)?;
    worksheet.write_string_with_format(row, 12, "Описание", &header_format)?;
    row += 1;

    // Data rows
    for entry in &report.entries {
        worksheet.write_string_with_format(
            row,
            0,
            &entry.date.format("%d.%m.%Y").to_string(),
            &date_format,
        )?;
        worksheet.write_string_with_format(row, 1, &entry.debit_account_code, &text_format)?;
        worksheet.write_string_with_format(row, 2, &entry.debit_account_name, &text_format)?;
        worksheet.write_string_with_format(row, 3, &entry.credit_account_code, &text_format)?;
        worksheet.write_string_with_format(row, 4, &entry.credit_account_name, &text_format)?;
        worksheet.write_number_with_format(
            row,
            5,
            entry.amount.to_f64().unwrap_or(0.0),
            &number_format,
        )?;

        // Currency amounts
        if let Some(amt) = entry.debit_currency_amount {
            worksheet.write_number_with_format(
                row,
                6,
                amt.to_f64().unwrap_or(0.0),
                &number_format,
            )?;
        } else {
            worksheet.write_string_with_format(row, 6, "", &text_format)?;
        }
        worksheet.write_string_with_format(
            row,
            7,
            entry.debit_currency_code.as_deref().unwrap_or(""),
            &text_format,
        )?;

        if let Some(amt) = entry.credit_currency_amount {
            worksheet.write_number_with_format(
                row,
                8,
                amt.to_f64().unwrap_or(0.0),
                &number_format,
            )?;
        } else {
            worksheet.write_string_with_format(row, 8, "", &text_format)?;
        }
        worksheet.write_string_with_format(
            row,
            9,
            entry.credit_currency_code.as_deref().unwrap_or(""),
            &text_format,
        )?;

        worksheet.write_string_with_format(
            row,
            10,
            entry.document_type.as_deref().unwrap_or(""),
            &text_format,
        )?;

        if let Some(doc_date) = entry.document_date {
            worksheet.write_string_with_format(
                row,
                11,
                &doc_date.format("%d.%m.%Y").to_string(),
                &text_format,
            )?;
        } else {
            worksheet.write_string_with_format(row, 11, "", &text_format)?;
        }

        worksheet.write_string_with_format(row, 12, &entry.description, &text_format)?;
        row += 1;
    }

    // Empty row
    row += 1;

    // Totals
    worksheet.write_string_with_format(row, 0, "", &totals_format)?;
    worksheet.write_string_with_format(row, 1, "", &totals_format)?;
    worksheet.write_string_with_format(row, 2, "", &totals_format)?;
    worksheet.write_string_with_format(row, 3, "", &totals_format)?;
    worksheet.write_string_with_format(row, 4, "Общо", &totals_format)?;
    worksheet.write_number_with_format(
        row,
        5,
        report.total_amount.to_f64().unwrap_or(0.0),
        &totals_format,
    )?;
    for col in 6..13 {
        worksheet.write_string_with_format(row, col, "", &totals_format)?;
    }

    let buffer = workbook.save_to_buffer()?;
    Ok(buffer)
}

fn generate_xlsx_turnover(
    sheet: &TurnoverSheet,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use rust_xlsxwriter::*;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Set column widths
    worksheet.set_column_width(0, 12.0)?; // Account code
    worksheet.set_column_width(1, 30.0)?; // Account name
    worksheet.set_column_width(2, 15.0)?; // Opening debit
    worksheet.set_column_width(3, 15.0)?; // Opening credit
    worksheet.set_column_width(4, 15.0)?; // Period debit
    worksheet.set_column_width(5, 15.0)?; // Period credit
    worksheet.set_column_width(6, 15.0)?; // Closing debit
    worksheet.set_column_width(7, 15.0)?; // Closing credit

    // Title and header formats
    let title_format = Format::new()
        .set_font_size(16)
        .set_bold()
        .set_align(FormatAlign::Center);

    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xE8E8E8))
        .set_border(FormatBorder::Thin);

    let number_format = Format::new()
        .set_num_format("0.00")
        .set_border(FormatBorder::Thin);

    let text_format = Format::new().set_border(FormatBorder::Thin);

    let totals_format = Format::new()
        .set_bold()
        .set_num_format("0.00")
        .set_background_color(Color::RGB(0xF0F0F0))
        .set_border(FormatBorder::Thin);

    let mut row = 0;

    // Title
    worksheet.merge_range(
        row,
        0,
        row,
        7,
        &format!("Оборотна ведомост - {}", sheet.company_name),
        &title_format,
    )?;
    row += 1;

    // Period info
    worksheet.merge_range(
        row,
        0,
        row,
        7,
        &format!("Период: {} - {}", sheet.period_start, sheet.period_end),
        &title_format,
    )?;
    row += 2;

    // Headers
    worksheet.write_string_with_format(row, 0, "Код сметка", &header_format)?;
    worksheet.write_string_with_format(row, 1, "Име сметка", &header_format)?;
    worksheet.write_string_with_format(row, 2, "Начално салдо Дт", &header_format)?;
    worksheet.write_string_with_format(row, 3, "Начално салдо Кт", &header_format)?;
    worksheet.write_string_with_format(row, 4, "Обороти Дт", &header_format)?;
    worksheet.write_string_with_format(row, 5, "Обороти Кт", &header_format)?;
    worksheet.write_string_with_format(row, 6, "Крайно салдо Дт", &header_format)?;
    worksheet.write_string_with_format(row, 7, "Крайно салдо Кт", &header_format)?;
    row += 1;

    // Data rows
    for entry in &sheet.entries {
        worksheet.write_string_with_format(row, 0, &entry.account_code, &text_format)?;
        worksheet.write_string_with_format(row, 1, &entry.account_name, &text_format)?;
        worksheet.write_number_with_format(
            row,
            2,
            entry.opening_debit.to_f64().unwrap_or(0.0),
            &number_format,
        )?;
        worksheet.write_number_with_format(
            row,
            3,
            entry.opening_credit.to_f64().unwrap_or(0.0),
            &number_format,
        )?;
        worksheet.write_number_with_format(
            row,
            4,
            entry.period_debit.to_f64().unwrap_or(0.0),
            &number_format,
        )?;
        worksheet.write_number_with_format(
            row,
            5,
            entry.period_credit.to_f64().unwrap_or(0.0),
            &number_format,
        )?;
        worksheet.write_number_with_format(
            row,
            6,
            entry.closing_debit.to_f64().unwrap_or(0.0),
            &number_format,
        )?;
        worksheet.write_number_with_format(
            row,
            7,
            entry.closing_credit.to_f64().unwrap_or(0.0),
            &number_format,
        )?;
        row += 1;
    }

    // Empty row
    row += 1;

    // Totals
    worksheet.write_string_with_format(row, 0, &sheet.totals.account_code, &totals_format)?;
    worksheet.write_string_with_format(row, 1, &sheet.totals.account_name, &totals_format)?;
    worksheet.write_number_with_format(
        row,
        2,
        sheet.totals.opening_debit.to_f64().unwrap_or(0.0),
        &totals_format,
    )?;
    worksheet.write_number_with_format(
        row,
        3,
        sheet.totals.opening_credit.to_f64().unwrap_or(0.0),
        &totals_format,
    )?;
    worksheet.write_number_with_format(
        row,
        4,
        sheet.totals.period_debit.to_f64().unwrap_or(0.0),
        &totals_format,
    )?;
    worksheet.write_number_with_format(
        row,
        5,
        sheet.totals.period_credit.to_f64().unwrap_or(0.0),
        &totals_format,
    )?;
    worksheet.write_number_with_format(
        row,
        6,
        sheet.totals.closing_debit.to_f64().unwrap_or(0.0),
        &totals_format,
    )?;
    worksheet.write_number_with_format(
        row,
        7,
        sheet.totals.closing_credit.to_f64().unwrap_or(0.0),
        &totals_format,
    )?;

    let buffer = workbook.save_to_buffer()?;
    Ok(buffer)
}

// Generate ODT for General Ledger
fn generate_odt_general_ledger(
    report: &GeneralLedger,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use std::io::Write;
    use zip::write::{FileOptions, ZipWriter};

    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));

    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    // mimetype (uncompressed)
    zip.start_file("mimetype", FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored))?;
    zip.write_all(b"application/vnd.oasis.opendocument.text")?;

    // content.xml
    zip.start_file("content.xml", options)?;

    let mut content = String::from(r##"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
  xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
  xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0"
  xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
  xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0">
  <office:automatic-styles>
    <style:style style:name="Table1" style:family="table">
      <style:table-properties style:width="17cm"/>
    </style:style>
    <style:style style:name="Table1.Col" style:family="table-column">
      <style:table-column-properties style:column-width="2cm"/>
    </style:style>
    <style:style style:name="HeaderRow" style:family="table-cell">
      <style:table-cell-properties fo:background-color="#4A5568" fo:border="0.05pt solid #000000"/>
      <style:text-properties fo:font-weight="bold" fo:color="#FFFFFF"/>
    </style:style>
    <style:style style:name="DataCell" style:family="table-cell">
      <style:table-cell-properties fo:border="0.05pt solid #CCCCCC"/>
    </style:style>
    <style:style style:name="AccountHeader" style:family="table-cell">
      <style:table-cell-properties fo:background-color="#667EEA" fo:border="0.05pt solid #000000"/>
      <style:text-properties fo:font-weight="bold" fo:color="#FFFFFF"/>
    </style:style>
    <style:style style:name="TotalCell" style:family="table-cell">
      <style:table-cell-properties fo:background-color="#F7FAFC" fo:border="0.05pt solid #000000"/>
      <style:text-properties fo:font-weight="bold"/>
    </style:style>
  </office:automatic-styles>
  <office:body>
    <office:text>
      <text:h text:outline-level="1">Главна книга - "##);

    content.push_str(&report.company_name);
    content.push_str("</text:h>\n");
    content.push_str(&format!(
        "      <text:p>Период: {} - {}</text:p>\n",
        report.period_start, report.period_end
    ));

    // Iterate through accounts
    for account in &report.accounts {
        content.push_str(&format!(r#"
      <text:h text:outline-level="2">{} - {}</text:h>
      <text:p>Начално салдо: {} лв. | Крайно салдо: {} лв.</text:p>
      <table:table table:name="AccountTable" table:style-name="Table1">
        <table:table-column table:style-name="Table1.Col"/>
        <table:table-column table:style-name="Table1.Col"/>
        <table:table-column table:style-name="Table1.Col"/>
        <table:table-column table:style-name="Table1.Col"/>
        <table:table-column table:style-name="Table1.Col"/>
        <table:table-column table:style-name="Table1.Col"/>
        <table:table-column table:style-name="Table1.Col"/>
        <table:table-row>
          <table:table-cell table:style-name="HeaderRow">
            <text:p>Дата</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="HeaderRow">
            <text:p>Документ</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="HeaderRow">
            <text:p>Описание</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="HeaderRow">
            <text:p>Дебит</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="HeaderRow">
            <text:p>Кредит</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="HeaderRow">
            <text:p>Салдо</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="HeaderRow">
            <text:p>Контрагент</text:p>
          </table:table-cell>
        </table:table-row>
"#,
            account.account_code,
            account.account_name,
            account.opening_balance,
            account.closing_balance
        ));

        // Opening balance row
        if account.opening_balance != rust_decimal::Decimal::ZERO {
            content.push_str(&format!(r#"
        <table:table-row>
          <table:table-cell table:style-name="DataCell">
            <text:p>{}</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="DataCell">
            <text:p>Начално салдо</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="DataCell">
            <text:p>Салдо към началото на периода</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="DataCell">
            <text:p>{}</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="DataCell">
            <text:p>{}</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="DataCell">
            <text:p>{}</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="DataCell">
            <text:p>-</text:p>
          </table:table-cell>
        </table:table-row>
"#,
                report.period_start,
                if account.opening_balance > rust_decimal::Decimal::ZERO { account.opening_balance.to_string() } else { "-".to_string() },
                if account.opening_balance < rust_decimal::Decimal::ZERO { account.opening_balance.abs().to_string() } else { "-".to_string() },
                account.opening_balance
            ));
        }

        // Transaction entries
        for entry in &account.entries {
            let doc_info = if let Some(ref doc_num) = entry.document_number {
                format!("{} (№ {})", entry.entry_number, doc_num)
            } else {
                entry.entry_number.clone()
            };

            content.push_str(&format!(r#"
        <table:table-row>
          <table:table-cell table:style-name="DataCell">
            <text:p>{}</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="DataCell">
            <text:p>{}</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="DataCell">
            <text:p>{}</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="DataCell">
            <text:p>{}</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="DataCell">
            <text:p>{}</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="DataCell">
            <text:p>{}</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="DataCell">
            <text:p>{}</text:p>
          </table:table-cell>
        </table:table-row>
"#,
                entry.date,
                doc_info,
                entry.description,
                if entry.debit_amount > rust_decimal::Decimal::ZERO { entry.debit_amount.to_string() } else { "-".to_string() },
                if entry.credit_amount > rust_decimal::Decimal::ZERO { entry.credit_amount.to_string() } else { "-".to_string() },
                entry.balance,
                entry.counterpart_name.as_deref().unwrap_or("-")
            ));
        }

        // Summary row
        content.push_str(&format!(r#"
        <table:table-row>
          <table:table-cell table:style-name="TotalCell" table:number-columns-spanned="3">
            <text:p>Общо за сметката:</text:p>
          </table:table-cell>
          <table:covered-table-cell/>
          <table:covered-table-cell/>
          <table:table-cell table:style-name="TotalCell">
            <text:p>{}</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="TotalCell">
            <text:p>{}</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="TotalCell">
            <text:p>{}</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="TotalCell">
            <text:p></text:p>
          </table:table-cell>
        </table:table-row>
      </table:table>
"#,
            account.total_debits,
            account.total_credits,
            account.closing_balance
        ));
    }

    content.push_str(r#"
    </office:text>
  </office:body>
</office:document-content>
"#);

    zip.write_all(content.as_bytes())?;

    // manifest
    zip.start_file("META-INF/manifest.xml", options)?;
    zip.write_all(ODT_MANIFEST.as_bytes())?;

    // styles
    zip.start_file("styles.xml", options)?;
    zip.write_all(ODT_STYLES.as_bytes())?;

    zip.finish()?;
    Ok(buffer)
}

// Generate XLSX for General Ledger
fn generate_xlsx_general_ledger(
    report: &GeneralLedger,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use rust_xlsxwriter::*;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Formats
    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x4A5568))
        .set_font_color(Color::White)
        .set_border(FormatBorder::Thin);

    let account_header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x667EEA))
        .set_font_color(Color::White)
        .set_border(FormatBorder::Thin);

    let data_format = Format::new()
        .set_border(FormatBorder::Thin);

    let number_format = Format::new()
        .set_num_format("0.00")
        .set_border(FormatBorder::Thin);

    let totals_format = Format::new()
        .set_bold()
        .set_num_format("0.00")
        .set_background_color(Color::RGB(0xF7FAFC))
        .set_border(FormatBorder::Thin);

    let mut row: u32 = 0;

    // Title
    worksheet.write_string_with_format(row, 0, &format!("Главна книга - {}", report.company_name), &header_format)?;
    row += 1;
    worksheet.write_string_with_format(row, 0, &format!("Период: {} - {}", report.period_start, report.period_end), &data_format)?;
    row += 2;

    // Iterate through accounts
    for account in &report.accounts {
        // Account header
        worksheet.write_string_with_format(row, 0, &format!("{} - {}", account.account_code, account.account_name), &account_header_format)?;
        worksheet.write_string_with_format(row, 1, &format!("Начално: {} лв.", account.opening_balance), &account_header_format)?;
        worksheet.write_string_with_format(row, 2, &format!("Крайно: {} лв.", account.closing_balance), &account_header_format)?;
        row += 1;

        // Column headers
        let headers = vec!["Дата", "Документ", "Описание", "Дебит", "Кредит", "Салдо", "Контрагент"];
        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(row, col as u16, *header, &header_format)?;
        }
        row += 1;

        // Opening balance
        if account.opening_balance != rust_decimal::Decimal::ZERO {
            worksheet.write_string_with_format(row, 0, &report.period_start.to_string(), &data_format)?;
            worksheet.write_string_with_format(row, 1, "Начално салдо", &data_format)?;
            worksheet.write_string_with_format(row, 2, "Салдо към началото на периода", &data_format)?;
            worksheet.write_number_with_format(row, 3, if account.opening_balance > rust_decimal::Decimal::ZERO { account.opening_balance.to_f64().unwrap_or(0.0) } else { 0.0 }, &data_format)?;
            worksheet.write_number_with_format(row, 4, if account.opening_balance < rust_decimal::Decimal::ZERO { account.opening_balance.abs().to_f64().unwrap_or(0.0) } else { 0.0 }, &data_format)?;
            worksheet.write_number_with_format(row, 5, account.opening_balance.to_f64().unwrap_or(0.0), &data_format)?;
            worksheet.write_string_with_format(row, 6, "-", &data_format)?;
            row += 1;
        }

        // Entries
        for entry in &account.entries {
            worksheet.write_string_with_format(row, 0, &entry.date.to_string(), &data_format)?;
            let doc_info = if let Some(ref doc_num) = entry.document_number {
                format!("{} (№ {})", entry.entry_number, doc_num)
            } else {
                entry.entry_number.clone()
            };
            worksheet.write_string_with_format(row, 1, &doc_info, &data_format)?;
            worksheet.write_string_with_format(row, 2, &entry.description, &data_format)?;
            worksheet.write_number_with_format(row, 3, if entry.debit_amount > rust_decimal::Decimal::ZERO { entry.debit_amount.to_f64().unwrap_or(0.0) } else { 0.0 }, &data_format)?;
            worksheet.write_number_with_format(row, 4, if entry.credit_amount > rust_decimal::Decimal::ZERO { entry.credit_amount.to_f64().unwrap_or(0.0) } else { 0.0 }, &data_format)?;
            worksheet.write_number_with_format(row, 5, entry.balance.to_f64().unwrap_or(0.0), &data_format)?;
            worksheet.write_string_with_format(row, 6, entry.counterpart_name.as_deref().unwrap_or("-"), &data_format)?;
            row += 1;
        }

        // Totals
        worksheet.write_string_with_format(row, 0, "Общо за сметката:", &totals_format)?;
        worksheet.write_string_with_format(row, 1, "", &totals_format)?;
        worksheet.write_string_with_format(row, 2, "", &totals_format)?;
        worksheet.write_number_with_format(row, 3, account.total_debits.to_f64().unwrap_or(0.0), &totals_format)?;
        worksheet.write_number_with_format(row, 4, account.total_credits.to_f64().unwrap_or(0.0), &totals_format)?;
        worksheet.write_number_with_format(row, 5, account.closing_balance.to_f64().unwrap_or(0.0), &totals_format)?;
        worksheet.write_string_with_format(row, 6, "", &totals_format)?;
        row += 2; // Extra space between accounts
    }

    let buffer = workbook.save_to_buffer()?;
    Ok(buffer)
}

fn generate_odt_bg_general_ledger(
    report: &BgGeneralLedger,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use std::io::Write;
    use zip::write::{FileOptions, ZipWriter};

    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));

    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    // mimetype (uncompressed)
    zip.start_file("mimetype", FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored))?;
    zip.write_all(b"application/vnd.oasis.opendocument.text")?;

    // content.xml
    zip.start_file("content.xml", options)?;

    let mut content = String::from(r##"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
  xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
  xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0"
  xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
  xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0">
  <office:automatic-styles>
    <style:style style:name="Table1" style:family="table">
      <style:table-properties style:width="17cm"/>
    </style:style>
    <style:style style:name="Table1.A" style:family="table-column">
      <style:table-column-properties style:column-width="4cm"/>
    </style:style>
    <style:style style:name="Table1.B" style:family="table-column">
      <style:table-column-properties style:column-width="3cm"/>
    </style:style>
    <style:style style:name="HeaderRow" style:family="table-cell">
      <style:table-cell-properties fo:background-color="#E0E0E0" fo:border="0.05pt solid #000000"/>
      <style:text-properties fo:font-weight="bold"/>
    </style:style>
    <style:style style:name="DataCell" style:family="table-cell">
      <style:table-cell-properties fo:border="0.05pt solid #CCCCCC"/>
    </style:style>
    <style:style style:name="TotalCell" style:family="table-cell">
      <style:table-cell-properties fo:background-color="#F0F0F0" fo:border="0.05pt solid #000000"/>
      <style:text-properties fo:font-weight="bold"/>
    </style:style>
  </office:automatic-styles>
  <office:body>
    <office:text>
      <text:h text:outline-level="1">Главна книга (БГ вариант) - "##);

    content.push_str(&report.company_name);
    content.push_str("</text:h>\n");
    content.push_str(&format!(
        "      <text:p>Период: {} - {}</text:p>\n",
        report.period_start, report.period_end
    ));

    // Section 1: By Debit
    content.push_str(r#"
      <text:h text:outline-level="2">Главна книга по Дебит</text:h>
"#);

    for debit_group in &report.by_debit {
        content.push_str(&format!(r#"
      <text:h text:outline-level="3">{} - {}</text:h>
      <text:p>Общо: {} лв.</text:p>
      <table:table table:name="DebitTable" table:style-name="Table1">
        <table:table-column table:style-name="Table1.A"/>
        <table:table-column table:style-name="Table1.B"/>
        <table:table-row>
          <table:table-cell table:style-name="HeaderRow">
            <text:p>Кредит сметка</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="HeaderRow">
            <text:p>Стойност</text:p>
          </table:table-cell>
        </table:table-row>
"#,
            debit_group.debit_account_code,
            debit_group.debit_account_name,
            debit_group.total_amount
        ));

        for entry in &debit_group.entries {
            content.push_str(&format!(r#"
        <table:table-row>
          <table:table-cell table:style-name="DataCell">
            <text:p>{} - {}</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="DataCell">
            <text:p>{}</text:p>
          </table:table-cell>
        </table:table-row>
"#,
                entry.credit_account_code,
                entry.credit_account_name,
                entry.amount
            ));
        }

        content.push_str("      </table:table>\n");
    }

    // Section 2: By Credit
    content.push_str(r#"
      <text:h text:outline-level="2">Главна книга по Кредит</text:h>
"#);

    for credit_group in &report.by_credit {
        content.push_str(&format!(r#"
      <text:h text:outline-level="3">{} - {}</text:h>
      <text:p>Общо: {} лв.</text:p>
      <table:table table:name="CreditTable" table:style-name="Table1">
        <table:table-column table:style-name="Table1.A"/>
        <table:table-column table:style-name="Table1.B"/>
        <table:table-row>
          <table:table-cell table:style-name="HeaderRow">
            <text:p>Дебит сметка</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="HeaderRow">
            <text:p>Стойност</text:p>
          </table:table-cell>
        </table:table-row>
"#,
            credit_group.credit_account_code,
            credit_group.credit_account_name,
            credit_group.total_amount
        ));

        for entry in &credit_group.entries {
            content.push_str(&format!(r#"
        <table:table-row>
          <table:table-cell table:style-name="DataCell">
            <text:p>{} - {}</text:p>
          </table:table-cell>
          <table:table-cell table:style-name="DataCell">
            <text:p>{}</text:p>
          </table:table-cell>
        </table:table-row>
"#,
                entry.debit_account_code,
                entry.debit_account_name,
                entry.amount
            ));
        }

        content.push_str("      </table:table>\n");
    }

    content.push_str(r#"
    </office:text>
  </office:body>
</office:document-content>"#);

    zip.write_all(content.as_bytes())?;

    // META-INF/manifest.xml
    zip.start_file("META-INF/manifest.xml", options)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0">
  <manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.text"/>
  <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
</manifest:manifest>"#,
    )?;

    zip.finish()?;
    Ok(buffer)
}
fn generate_xlsx_bg_general_ledger(
    report: &BgGeneralLedger,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use rust_xlsxwriter::*;

    let mut workbook = Workbook::new();

    // Create worksheet for debit section
    let debit_sheet = workbook.add_worksheet().set_name("По Дебит")?;

    // Set column widths
    debit_sheet.set_column_width(0, 35.0)?; // Account
    debit_sheet.set_column_width(1, 15.0)?; // Amount

    // Title and header formats
    let title_format = Format::new()
        .set_font_size(16)
        .set_bold()
        .set_align(FormatAlign::Center);

    let section_header_format = Format::new()
        .set_font_size(13)
        .set_bold()
        .set_background_color(Color::RGB(0x4472C4))
        .set_font_color(Color::White);

    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xE8E8E8))
        .set_border(FormatBorder::Thin);

    let number_format = Format::new()
        .set_num_format("0.00")
        .set_border(FormatBorder::Thin);

    let text_format = Format::new().set_border(FormatBorder::Thin);

    let total_format = Format::new()
        .set_bold()
        .set_num_format("0.00")
        .set_background_color(Color::RGB(0xD9E2F3))
        .set_border(FormatBorder::Thin);

    let mut row = 0;

    // Title for debit section
    debit_sheet.merge_range(
        row,
        0,
        row,
        1,
        &format!("Главна книга (БГ вариант) - {} - По Дебит", report.company_name),
        &title_format,
    )?;
    row += 1;

    // Period info
    debit_sheet.merge_range(
        row,
        0,
        row,
        1,
        &format!("Период: {} - {}", report.period_start, report.period_end),
        &text_format,
    )?;
    row += 2;

    // Process debit groups
    for debit_group in &report.by_debit {
        // Group header
        debit_sheet.merge_range(
            row,
            0,
            row,
            1,
            &format!("{} - {}", debit_group.debit_account_code, debit_group.debit_account_name),
            &section_header_format,
        )?;
        row += 1;

        // Column headers
        debit_sheet.write_string_with_format(row, 0, "Кредит сметка", &header_format)?;
        debit_sheet.write_string_with_format(row, 1, "Стойност", &header_format)?;
        row += 1;

        // Entries
        for entry in &debit_group.entries {
            debit_sheet.write_string_with_format(
                row,
                0,
                &format!("{} - {}", entry.credit_account_code, entry.credit_account_name),
                &text_format,
            )?;
            debit_sheet.write_number_with_format(
                row,
                1,
                entry.amount.to_f64().unwrap_or(0.0),
                &number_format,
            )?;
            row += 1;
        }

        // Subtotal
        debit_sheet.write_string_with_format(row, 0, "Общо", &total_format)?;
        debit_sheet.write_number_with_format(
            row,
            1,
            debit_group.total_amount.to_f64().unwrap_or(0.0),
            &total_format,
        )?;
        row += 2; // Extra space between groups
    }

    // Create worksheet for credit section
    let credit_sheet = workbook.add_worksheet().set_name("По Кредит")?;

    // Set column widths
    credit_sheet.set_column_width(0, 35.0)?;
    credit_sheet.set_column_width(1, 15.0)?;

    let mut row = 0;

    // Title for credit section
    credit_sheet.merge_range(
        row,
        0,
        row,
        1,
        &format!("Главна книга (БГ вариант) - {} - По Кредит", report.company_name),
        &title_format,
    )?;
    row += 1;

    // Period info
    credit_sheet.merge_range(
        row,
        0,
        row,
        1,
        &format!("Период: {} - {}", report.period_start, report.period_end),
        &text_format,
    )?;
    row += 2;

    // Process credit groups
    for credit_group in &report.by_credit {
        // Group header
        credit_sheet.merge_range(
            row,
            0,
            row,
            1,
            &format!("{} - {}", credit_group.credit_account_code, credit_group.credit_account_name),
            &section_header_format,
        )?;
        row += 1;

        // Column headers
        credit_sheet.write_string_with_format(row, 0, "Дебит сметка", &header_format)?;
        credit_sheet.write_string_with_format(row, 1, "Стойност", &header_format)?;
        row += 1;

        // Entries
        for entry in &credit_group.entries {
            credit_sheet.write_string_with_format(
                row,
                0,
                &format!("{} - {}", entry.debit_account_code, entry.debit_account_name),
                &text_format,
            )?;
            credit_sheet.write_number_with_format(
                row,
                1,
                entry.amount.to_f64().unwrap_or(0.0),
                &number_format,
            )?;
            row += 1;
        }

        // Subtotal
        credit_sheet.write_string_with_format(row, 0, "Общо", &total_format)?;
        credit_sheet.write_number_with_format(
            row,
            1,
            credit_group.total_amount.to_f64().unwrap_or(0.0),
            &total_format,
        )?;
        row += 2; // Extra space between groups
    }

    let buffer = workbook.save_to_buffer()?;
    Ok(buffer)
}
