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

        for account in accounts {
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
                account_id: account.id,
                account_code: account.code.clone(),
                account_name: account.name.clone(),
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
                        debit_currency_amount: None, // TODO: Add currency support
                        debit_currency_code: None,
                        credit_currency_amount: None,
                        credit_currency_code: None,
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
        format: String, // "XLSX", "PDF" (HTML to PDF)
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
            "PDF" => {
                let content = generate_html_to_pdf_chronological(&chronological_report).await?;
                let encoded_content = base64::prelude::BASE64_STANDARD.encode(&content);
                Ok(ReportExport {
                    format: "PDF".to_string(),
                    content: encoded_content,
                    filename,
                    mime_type: "application/pdf".to_string(),
                })
            }
            _ => Err("Unsupported format. Use XLSX or PDF".into()),
        }
    }

    /// Export monthly statistics in PDF format
    async fn export_monthly_stats(
        &self,
        ctx: &Context<'_>,
        input: MonthlyStatsInput,
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
            "monthly_stats_{}_{}_{}_{}.pdf",
            input.from_year, input.from_month, input.to_year, input.to_month
        );

        let content = generate_monthly_stats_pdf(&stats, &company.name).await?;
        let encoded_content = base64::prelude::BASE64_STANDARD.encode(&content);

        Ok(ReportExport {
            format: "PDF".to_string(),
            content: encoded_content,
            filename,
            mime_type: "application/pdf".to_string(),
        })
    }

    /// Export turnover sheet in specified format
    async fn export_turnover_sheet(
        &self,
        ctx: &Context<'_>,
        input: TurnoverReportInput,
        format: String, // "XLSX", "PDF" (HTML to PDF)
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
            "PDF" => {
                let content = generate_html_to_pdf_turnover(&turnover_sheet).await?;
                let encoded_content = base64::prelude::BASE64_STANDARD.encode(&content);
                Ok(ReportExport {
                    format: "PDF".to_string(),
                    content: encoded_content,
                    filename,
                    mime_type: "application/pdf".to_string(),
                })
            }
            _ => Err("Unsupported format. Use XLSX or PDF".into()),
        }
    }
}

// Helper functions for export generation
async fn generate_html_to_pdf_chronological(
    report: &ChronologicalReport,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use headless_chrome::{Browser, LaunchOptions};

    let html = generate_html_chronological(report);

    // Launch a headless Chrome browser
    let browser = Browser::new(LaunchOptions::default())?;
    let tab = browser.new_tab()?;

    // Navigate to a data URI with our HTML content
    let data_uri = format!(
        "data:text/html;charset=utf-8,{}",
        urlencoding::encode(&html)
    );
    tab.navigate_to(&data_uri)?;

    // Wait for page to load
    tab.wait_for_element("table")?;

    // Generate PDF with landscape orientation
    use headless_chrome::types::PrintToPdfOptions;
    let pdf_options = PrintToPdfOptions {
        landscape: Some(true),
        print_background: Some(true),
        paper_width: Some(11.69), // A4 width in inches
        paper_height: Some(8.27), // A4 height in inches
        margin_top: Some(0.4),
        margin_bottom: Some(0.4),
        margin_left: Some(0.4),
        margin_right: Some(0.4),
        ..Default::default()
    };

    let pdf_data = tab.print_to_pdf(Some(pdf_options))?;
    Ok(pdf_data)
}

fn generate_html_chronological(report: &ChronologicalReport) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Хронологичен регистър</title>
    <style>
        body {{
            font-family: 'Arial', sans-serif;
            margin: 0;
            padding: 20px;
            font-size: 10px;
        }}
        .header {{
            text-align: center;
            margin-bottom: 20px;
        }}
        .title {{
            font-size: 16px;
            font-weight: bold;
            margin-bottom: 5px;
        }}
        .period {{
            font-size: 12px;
            margin-bottom: 15px;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 10px;
            font-size: 9px;
        }}
        th, td {{
            border: 1px solid #333;
            padding: 2px 4px;
            text-align: left;
        }}
        th {{
            background-color: #f0f0f0;
            font-weight: bold;
            text-align: center;
            font-size: 8px;
        }}
        .number {{
            text-align: right;
        }}
        .date {{
            text-align: center;
        }}
        .totals {{
            font-weight: bold;
            background-color: #f5f5f5;
        }}
        .code {{
            width: 8%;
        }}
        .name {{
            width: 15%;
        }}
        .amount {{
            width: 8%;
        }}
        .currency {{
            width: 6%;
        }}
        .description {{
            width: 15%;
        }}
    </style>
</head>
<body>
    <div class="header">
        <div class="title">Хронологичен регистър - {}</div>
        <div class="period">от {} до {}</div>
    </div>
    
    <table>
        <thead>
            <tr>
                <th>Дата</th>
                <th class="code">Дебит</th>
                <th class="name">Дебит име</th>
                <th class="code">Кредит</th>
                <th class="name">Кредит име</th>
                <th class="amount">Сума</th>
                <th class="currency">Дебит валутна сума</th>
                <th>Дебит валута</th>
                <th class="currency">Кредит валутна сума</th>
                <th>Кредит валута</th>
                <th>Док. вид</th>
                <th>Док. дата</th>
                <th class="description">Описание</th>
            </tr>
        </thead>
        <tbody>
{}
            <tr class="totals">
                <td></td>
                <td></td>
                <td></td>
                <td></td>
                <td class="number">Общо</td>
                <td class="number">{}</td>
                <td></td>
                <td></td>
                <td></td>
                <td></td>
                <td></td>
                <td></td>
                <td></td>
            </tr>
        </tbody>
    </table>
</body>
</html>"#,
        report.company_name,
        report.period_start.format("%d.%m.%Y"),
        report.period_end.format("%d.%m.%Y"),
        report
            .entries
            .iter()
            .map(|entry| format!(
                "            <tr>
                <td class=\"date\">{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td class=\"number\">{}</td>
                <td class=\"number\">{}</td>
                <td>{}</td>
                <td class=\"number\">{}</td>
                <td>{}</td>
                <td>{}</td>
                <td class=\"date\">{}</td>
                <td>{}</td>
            </tr>",
                entry.date.format("%d.%m.%Y"),
                entry.debit_account_code,
                entry.debit_account_name,
                entry.credit_account_code,
                entry.credit_account_name,
                entry.amount,
                entry
                    .debit_currency_amount
                    .map_or("".to_string(), |a| a.to_string()),
                entry.debit_currency_code.as_deref().unwrap_or(""),
                entry
                    .credit_currency_amount
                    .map_or("".to_string(), |a| a.to_string()),
                entry.credit_currency_code.as_deref().unwrap_or(""),
                entry.document_type.as_deref().unwrap_or(""),
                entry
                    .document_date
                    .map_or("".to_string(), |d| d.format("%d.%m.%Y").to_string()),
                entry.description
            ))
            .collect::<Vec<_>>()
            .join("\n"),
        report.total_amount
    )
}

async fn generate_html_to_pdf_turnover(
    sheet: &TurnoverSheet,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use headless_chrome::{Browser, LaunchOptions};

    let html = generate_html_turnover(sheet);

    // Launch a headless Chrome browser
    let browser = Browser::new(LaunchOptions::default())?;
    let tab = browser.new_tab()?;

    // Navigate to a data URI with our HTML content
    let data_uri = format!(
        "data:text/html;charset=utf-8,{}",
        urlencoding::encode(&html)
    );
    tab.navigate_to(&data_uri)?;

    // Wait for page to load
    tab.wait_for_element("table")?;

    // Generate PDF with landscape orientation
    use headless_chrome::types::PrintToPdfOptions;
    let pdf_options = PrintToPdfOptions {
        landscape: Some(true),
        print_background: Some(true),
        paper_width: Some(11.69), // A4 width in inches
        paper_height: Some(8.27), // A4 height in inches
        margin_top: Some(0.4),
        margin_bottom: Some(0.4),
        margin_left: Some(0.4),
        margin_right: Some(0.4),
        ..Default::default()
    };

    let pdf_data = tab.print_to_pdf(Some(pdf_options))?;
    Ok(pdf_data)
}

fn generate_html_turnover(sheet: &TurnoverSheet) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Оборотна ведомост</title>
    <style>
        body {{
            font-family: 'Arial', sans-serif;
            margin: 0;
            padding: 20px;
            font-size: 11px;
        }}
        .header {{
            text-align: center;
            margin-bottom: 20px;
        }}
        .title {{
            font-size: 16px;
            font-weight: bold;
            margin-bottom: 5px;
        }}
        .period {{
            font-size: 12px;
            margin-bottom: 15px;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 10px;
        }}
        th, td {{
            border: 1px solid #333;
            padding: 4px 6px;
            text-align: left;
        }}
        th {{
            background-color: #f0f0f0;
            font-weight: bold;
            text-align: center;
            font-size: 10px;
        }}
        .number {{
            text-align: right;
        }}
        .totals {{
            font-weight: bold;
            background-color: #f5f5f5;
        }}
        .code {{
            width: 8%;
        }}
        .name {{
            width: 30%;
        }}
        .amount {{
            width: 10.33%;
        }}
    </style>
</head>
<body>
    <div class="header">
        <div class="title">Оборотна ведомост - {}</div>
        <div class="period">Период: {} - {}</div>
    </div>
    
    <table>
        <thead>
            <tr>
                <th class="code">Код сметка</th>
                <th class="name">Име сметка</th>
                <th class="amount">Начално салдо Дт</th>
                <th class="amount">Начално салдо Кт</th>
                <th class="amount">Обороти Дт</th>
                <th class="amount">Обороти Кт</th>
                <th class="amount">Крайно салдо Дт</th>
                <th class="amount">Крайно салдо Кт</th>
            </tr>
        </thead>
        <tbody>
{}
            <tr class="totals">
                <td>{}</td>
                <td>{}</td>
                <td class="number">{}</td>
                <td class="number">{}</td>
                <td class="number">{}</td>
                <td class="number">{}</td>
                <td class="number">{}</td>
                <td class="number">{}</td>
            </tr>
        </tbody>
    </table>
</body>
</html>"#,
        sheet.company_name,
        sheet.period_start,
        sheet.period_end,
        sheet
            .entries
            .iter()
            .map(|entry| format!(
                "            <tr>
                <td>{}</td>
                <td>{}</td>
                <td class=\"number\">{}</td>
                <td class=\"number\">{}</td>
                <td class=\"number\">{}</td>
                <td class=\"number\">{}</td>
                <td class=\"number\">{}</td>
                <td class=\"number\">{}</td>
            </tr>",
                entry.account_code,
                entry.account_name,
                entry.opening_debit,
                entry.opening_credit,
                entry.period_debit,
                entry.period_credit,
                entry.closing_debit,
                entry.closing_credit
            ))
            .collect::<Vec<_>>()
            .join("\n"),
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

// Generate PDF for monthly transaction statistics
async fn generate_monthly_stats_pdf(
    stats: &[MonthlyTransactionStats],
    company_name: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use headless_chrome::{Browser, LaunchOptions};

    let html = generate_html_monthly_stats(stats, company_name);

    let browser = Browser::new(LaunchOptions::default())?;
    let tab = browser.new_tab()?;

    let data_uri = format!(
        "data:text/html;charset=utf-8,{}",
        urlencoding::encode(&html)
    );
    tab.navigate_to(&data_uri)?;
    tab.wait_for_element("table")?;

    use headless_chrome::types::PrintToPdfOptions;
    let pdf_options = PrintToPdfOptions {
        landscape: Some(true),
        print_background: Some(true),
        paper_width: Some(11.69),
        paper_height: Some(8.27),
        margin_top: Some(0.4),
        margin_bottom: Some(0.4),
        margin_left: Some(0.4),
        margin_right: Some(0.4),
        ..Default::default()
    };

    let pdf_data = tab.print_to_pdf(Some(pdf_options))?;
    Ok(pdf_data)
}

fn generate_html_monthly_stats(stats: &[MonthlyTransactionStats], company_name: &str) -> String {
    let mut total_entries: i64 = 0;
    let mut total_posted_entries: i64 = 0;
    let mut total_lines: i64 = 0;
    let mut total_posted_lines: i64 = 0;
    let mut total_amount = Decimal::ZERO;
    let mut total_vat = Decimal::ZERO;

    let rows: Vec<String> = stats
        .iter()
        .map(|s| {
            total_entries += s.total_entries;
            total_posted_entries += s.posted_entries;
            total_lines += s.total_entry_lines;
            total_posted_lines += s.posted_entry_lines;
            total_amount += s.total_amount;
            total_vat += s.vat_amount;

            format!(
                r#"<tr>
                    <td>{} {}</td>
                    <td class=\"number\">{}</td>
                    <td class=\"number\">{}</td>
                    <td class=\"number\">{}</td>
                    <td class=\"number\">{}</td>
                    <td class=\"number\">{:.2}</td>
                    <td class=\"number\">{:.2}</td>
                </tr>"#,
                s.month_name,
                s.year,
                s.total_entries,
                s.posted_entries,
                s.total_entry_lines,
                s.posted_entry_lines,
                s.total_amount,
                s.vat_amount
            )
        })
        .collect();

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset=\"UTF-8\">
    <title>Месечна статистика за ценообразуване</title>
    <style>
        body {{
            font-family: 'Arial', sans-serif;
            margin: 0;
            padding: 20px;
            font-size: 11px;
        }}
        .header {{
            text-align: center;
            margin-bottom: 20px;
        }}
        .title {{
            font-size: 18px;
            font-weight: bold;
            margin-bottom: 5px;
        }}
        .subtitle {{
            font-size: 14px;
            color: #666;
            margin-bottom: 20px;
        }}
        .pricing-note {{
            background-color: #f0f8ff;
            border-left: 4px solid #4a90e2;
            padding: 12px;
            margin-bottom: 20px;
            font-size: 12px;
        }}
        .pricing-note strong {{
            color: #2c5aa0;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 10px;
        }}
        th, td {{
            border: 1px solid #333;
            padding: 8px 6px;
            text-align: left;
        }}
        th {{
            background-color: #4a90e2;
            color: white;
            font-weight: bold;
            text-align: center;
            font-size: 10px;
        }}
        .number {{
            text-align: right;
            font-family: 'Courier New', monospace;
        }}
        .totals {{
            font-weight: bold;
            background-color: #e6f2ff;
            border-top: 3px solid #4a90e2;
        }}
        .footer {{
            margin-top: 30px;
            font-size: 10px;
            color: #666;
            text-align: center;
        }}
    </style>
</head>
<body>
    <div class=\"header\">
        <div class=\"title\">Месечна статистика на транзакции</div>
        <div class=\"subtitle\">{}</div>
    </div>

    <div class=\"pricing-note\">
        <strong>Модел на ценообразуване:</strong> Базова цена (ангажимент) + допълнително заплащане по брой транзакции и счетоводни редове (Дт/Кт).
        <br><strong>Важно:</strong> Отчитат се само приключени (posted) записи.
    </div>

    <table>
        <thead>
            <tr>
                <th>Период</th>
                <th>Документи<br>(общо)</th>
                <th>Документи<br>(приключени)</th>
                <th>Редове Дт/Кт<br>(общо)</th>
                <th>Редове Дт/Кт<br>(приключени)</th>
                <th>Оборот<br>(лв.)</th>
                <th>ДДС<br>(лв.)</th>
            </tr>
        </thead>
        <tbody>
{}
            <tr class=\"totals\">
                <td>ОБЩО</td>
                <td class=\"number\">{}</td>
                <td class=\"number\">{}</td>
                <td class=\"number\">{}</td>
                <td class=\"number\">{}</td>
                <td class=\"number\">{:.2}</td>
                <td class=\"number\">{:.2}</td>
            </tr>
        </tbody>
    </table>

    <div class=\"footer\">
        Генериран на {} • Система: rs-ac-bg
    </div>
</body>
</html>"#,
        company_name,
        rows.join("\n"),
        total_entries,
        total_posted_entries,
        total_lines,
        total_posted_lines,
        total_amount,
        total_vat,
        chrono::Utc::now().format("%d.%m.%Y %H:%M")
    )
}
