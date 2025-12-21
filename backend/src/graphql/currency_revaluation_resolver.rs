use crate::entities::{
    account, company, counterpart, currency, currency_revaluation_settings, entry_line,
    exchange_rate, journal_entry, CurrencyRevaluationSettings, CurrencyRevaluationSettingsModel,
    RevaluationSettingsInput,
};
use async_graphql::{Context, FieldResult, Object, SimpleObject};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(SimpleObject)]
pub struct RevaluationResult {
    pub success: bool,
    pub message: String,
    pub created_entries_count: i32,
    pub total_revaluation_amount: f64,
}

#[derive(SimpleObject, Clone)]
pub struct RevaluationSettingsWithAccounts {
    pub id: i32,
    pub company_id: i32,
    pub expense_account_id: i32,
    pub revenue_account_id: i32,
    #[graphql(skip)]
    pub _expense_account: Option<account::Model>,
    #[graphql(skip)]
    pub _revenue_account: Option<account::Model>,
}

#[derive(SimpleObject)]
pub struct AccountRevaluationData {
    pub account_code: String,
    pub account_name: String,
}

#[derive(SimpleObject, Clone)]
pub struct CurrencyRevaluationPreviewItem {
    pub account_id: i32,
    pub account_code: String,
    pub account_name: String,
    pub counterpart_id: Option<i32>,
    pub counterpart_name: Option<String>,
    pub currency_code: String,
    pub foreign_quantity: f64,             // Салдо: количество във валута
    pub debit_bgn: f64,                    // Дебит в BGN (по среден курс)
    pub credit_bgn: f64,                   // Кредит в BGN (по среден курс)
    pub balance_bgn: f64,                  // Стойност (салдо в BGN по среден курс)
    pub weighted_avg_rate: f64,            // Среден курс на записите
    pub new_rate: f64,                     // Нов курс към датата на преоценка
    pub new_balance_bgn: f64,              // Количество × нов курс
    pub revaluation_difference: f64,       // Разлика за преоценка (нова стойност - стара стойност)
}

#[derive(SimpleObject)]
pub struct CurrencyRevaluationPreview {
    pub items: Vec<CurrencyRevaluationPreviewItem>,
    pub total_items: i32,
    pub total_gain: f64,
    pub total_loss: f64,
    pub net_difference: f64,
}

impl RevaluationSettingsWithAccounts {
    pub fn expense_account(&self) -> Option<AccountRevaluationData> {
        self._expense_account.as_ref().map(|acc| AccountRevaluationData {
            account_code: acc.code.clone(),
            account_name: acc.name.clone(),
        })
    }

    pub fn revenue_account(&self) -> Option<AccountRevaluationData> {
        self._revenue_account.as_ref().map(|acc| AccountRevaluationData {
            account_code: acc.code.clone(),
            account_name: acc.name.clone(),
        })
    }
}

#[derive(Default)]
pub struct CurrencyRevaluationQuery;

#[derive(Default)]
pub struct CurrencyRevaluationMutation;

#[Object]
impl CurrencyRevaluationQuery {
    /// Get revaluation settings for a company
    async fn revaluation_settings(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
    ) -> FieldResult<Option<RevaluationSettingsWithAccounts>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let settings = CurrencyRevaluationSettings::find()
            .filter(currency_revaluation_settings::Column::CompanyId.eq(company_id))
            .one(db)
            .await?;

        if let Some(settings) = settings {
            // Load expense and revenue accounts
            let expense_account = account::Entity::find_by_id(settings.expense_account_id)
                .one(db)
                .await?;

            let revenue_account = account::Entity::find_by_id(settings.revenue_account_id)
                .one(db)
                .await?;

            Ok(Some(RevaluationSettingsWithAccounts {
                id: settings.id,
                company_id: settings.company_id,
                expense_account_id: settings.expense_account_id,
                revenue_account_id: settings.revenue_account_id,
                _expense_account: expense_account,
                _revenue_account: revenue_account,
            }))
        } else {
            Ok(None)
        }
    }

    /// Preview currency revaluation without creating entries
    async fn currency_revaluation_preview(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        revaluation_date: NaiveDate,
    ) -> FieldResult<CurrencyRevaluationPreview> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Load company to get base currency
        let company = company::Entity::find_by_id(company_id)
            .one(db)
            .await?
            .ok_or("Фирмата не е намерена")?;

        // Get base currency code - default to BGN if not set
        let base_currency = if let Some(currency_id) = company.base_currency_id {
            let currency = currency::Entity::find_by_id(currency_id)
                .one(db)
                .await?;
            currency.map(|c| c.code).unwrap_or("BGN".to_string())
        } else {
            "BGN".to_string()
        };

        // Find all open foreign currency positions (all accounts, not just 401/411)
        let open_positions =
            find_all_currency_balances(db, company_id, &base_currency, revaluation_date).await?;

        // Get exchange rates for revaluation date
        let exchange_rates = get_exchange_rates_map(db, revaluation_date).await?;

        // Calculate preview items
        let mut preview_items = Vec::new();
        let mut total_gain = 0.0;
        let mut total_loss = 0.0;

        for position in open_positions {
            if let Some(&new_rate) = exchange_rates.get(&position.currency_code) {
                let new_rate_f64 = new_rate.to_string().parse::<f64>().unwrap_or(0.0);
                let weighted_avg_rate_f64 = position.weighted_avg_rate.to_string().parse::<f64>().unwrap_or(0.0);
                let foreign_quantity_f64 = position.foreign_amount.to_string().parse::<f64>().unwrap_or(0.0);

                // Calculate values
                let balance_bgn = foreign_quantity_f64 * weighted_avg_rate_f64;
                let new_balance_bgn = foreign_quantity_f64 * new_rate_f64;
                let revaluation_difference = new_balance_bgn - balance_bgn;

                // Track gains and losses
                if revaluation_difference > 0.0 {
                    total_gain += revaluation_difference;
                } else {
                    total_loss += revaluation_difference.abs();
                }

                preview_items.push(CurrencyRevaluationPreviewItem {
                    account_id: position.account_id,
                    account_code: position.account_code.clone(),
                    account_name: position.account_name.clone(),
                    counterpart_id: position.counterpart_id,
                    counterpart_name: position.counterpart_name.clone(),
                    currency_code: position.currency_code.clone(),
                    foreign_quantity: foreign_quantity_f64,
                    debit_bgn: position.total_debit.to_string().parse::<f64>().unwrap_or(0.0),
                    credit_bgn: position.total_credit.to_string().parse::<f64>().unwrap_or(0.0),
                    balance_bgn,
                    weighted_avg_rate: weighted_avg_rate_f64,
                    new_rate: new_rate_f64,
                    new_balance_bgn,
                    revaluation_difference,
                });
            }
        }

        let total_items = preview_items.len() as i32;
        let net_difference = total_gain - total_loss;

        Ok(CurrencyRevaluationPreview {
            items: preview_items,
            total_items,
            total_gain,
            total_loss,
            net_difference,
        })
    }
}

#[Object]
impl CurrencyRevaluationMutation {
    /// Save or update revaluation settings
    async fn save_revaluation_settings(
        &self,
        ctx: &Context<'_>,
        input: RevaluationSettingsInput,
    ) -> FieldResult<CurrencyRevaluationSettingsModel> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Check if settings already exist
        let existing = CurrencyRevaluationSettings::find()
            .filter(currency_revaluation_settings::Column::CompanyId.eq(input.company_id))
            .one(db)
            .await?;

        let model = if let Some(existing) = existing {
            // Update existing
            let mut active: currency_revaluation_settings::ActiveModel = existing.into();
            active.expense_account_id = Set(input.expense_account_id);
            active.revenue_account_id = Set(input.revenue_account_id);
            active.updated_at = Set(chrono::Utc::now());
            active.update(db).await?
        } else {
            // Create new
            let active: currency_revaluation_settings::ActiveModel = input.into();
            active.insert(db).await?
        };

        Ok(model)
    }

    /// Run currency revaluation for receivables (411) and payables (401)
    async fn run_currency_revaluation(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        revaluation_date: NaiveDate,
    ) -> FieldResult<RevaluationResult> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // 1. Load settings
        let settings = CurrencyRevaluationSettings::find()
            .filter(currency_revaluation_settings::Column::CompanyId.eq(company_id))
            .one(db)
            .await?;

        let settings = settings.ok_or("Настройките за валутна преоценка не са конфигурирани")?;

        // 2. Load company to get base currency
        let company = company::Entity::find_by_id(company_id)
            .one(db)
            .await?
            .ok_or("Фирмата не е намерена")?;

        // Get base currency code - default to BGN if not set
        let base_currency = if let Some(currency_id) = company.base_currency_id {
            let currency = currency::Entity::find_by_id(currency_id)
                .one(db)
                .await?;
            currency.map(|c| c.code).unwrap_or("BGN".to_string())
        } else {
            "BGN".to_string()
        };

        // 3. Find all currency balances (universal - all accounts)
        let open_positions =
            find_all_currency_balances(db, company_id, &base_currency, revaluation_date)
                .await?;

        if open_positions.is_empty() {
            return Ok(RevaluationResult {
                success: true,
                message: "Няма отворени позиции в чужда валута за преоценка".to_string(),
                created_entries_count: 0,
                total_revaluation_amount: 0.0,
            });
        }

        // 4. Get exchange rates for revaluation date
        let exchange_rates = get_exchange_rates_map(db, revaluation_date).await?;

        // 5. Create revaluation entries
        let mut total_difference = Decimal::ZERO;
        let mut entries_count = 0;

        for position in open_positions {
            if let Some(&current_rate) = exchange_rates.get(&position.currency_code) {
                // Calculate revaluation difference
                let original_bgn = position.foreign_amount * position.weighted_avg_rate;
                let current_bgn = position.foreign_amount * current_rate;
                let difference = current_bgn - original_bgn;

                // Only create entry if difference is significant (> 0.01 BGN)
                if difference.abs() > Decimal::new(1, 2) {
                    create_revaluation_journal_entry(
                        db,
                        company_id,
                        revaluation_date,
                        &position,
                        difference,
                        current_rate,
                        &settings,
                    )
                    .await?;

                    total_difference += difference;
                    entries_count += 1;
                }
            }
        }

        Ok(RevaluationResult {
            success: true,
            message: format!(
                "Преоценката завърши успешно! Създадени {} записа.",
                entries_count
            ),
            created_entries_count: entries_count,
            total_revaluation_amount: total_difference.to_string().parse::<f64>().unwrap_or(0.0),
        })
    }
}

#[derive(Debug)]
struct OpenForeignPosition {
    account_id: i32,
    account_code: String,
    account_name: String,
    counterpart_id: Option<i32>,
    counterpart_name: Option<String>,
    currency_code: String,
    foreign_amount: Decimal,        // Amount in foreign currency (салдо)
    total_debit: Decimal,           // Total debit in BGN
    total_credit: Decimal,          // Total credit in BGN
    weighted_avg_rate: Decimal,     // Weighted average exchange rate
    current_bgn_equivalent: Decimal, // Current equivalent in base currency
}

/// Find all open foreign currency positions for accounts 401 (payables) and 411 (receivables)
async fn find_open_foreign_currency_positions(
    db: &DatabaseConnection,
    company_id: i32,
    base_currency: &str,
    as_of_date: NaiveDate,
) -> FieldResult<Vec<OpenForeignPosition>> {
    use sea_orm::sea_query::{Expr, Func};
    use sea_orm::{QueryOrder, QueryTrait, Statement};

    // Raw SQL query for complex aggregation
    let sql = format!(
        r#"
        SELECT
            el.account_id,
            acc.code as account_code,
            acc.name as account_name,
            el.counterpart_id,
            cp.name as counterpart_name,
            el.currency_code,
            SUM(el.currency_amount) as total_foreign_amount,
            SUM(el.debit_amount) as total_debit,
            SUM(el.credit_amount) as total_credit,
            CASE
                WHEN SUM(el.currency_amount) != 0
                THEN SUM(el.debit_amount - el.credit_amount) / SUM(el.currency_amount)
                ELSE 1
            END as weighted_avg_rate,
            SUM(el.debit_amount - el.credit_amount) as bgn_equivalent
        FROM entry_lines el
        INNER JOIN journal_entries je ON je.id = el.journal_entry_id
        INNER JOIN accounts acc ON acc.id = el.account_id
        INNER JOIN counterparts cp ON cp.id = el.counterpart_id
        WHERE je.company_id = {}
          AND je.accounting_date <= '{}'
          AND je.is_posted = true
          AND el.counterpart_id IS NOT NULL
          AND el.currency_code IS NOT NULL
          AND el.currency_code != '{}'
          AND (acc.code LIKE '401%' OR acc.code LIKE '411%')
        GROUP BY el.account_id, acc.code, acc.name, el.counterpart_id, cp.name, el.currency_code
        HAVING ABS(SUM(el.currency_amount)) > 0.01
        "#,
        company_id, as_of_date, base_currency
    );

    use sea_orm::DbBackend;
    let stmt = Statement::from_string(DbBackend::Postgres, sql);

    let query_result = db.query_all(stmt).await?;

    let mut positions = Vec::new();
    for row in query_result {
        positions.push(OpenForeignPosition {
            account_id: row.try_get("", "account_id")?,
            account_code: row.try_get("", "account_code")?,
            account_name: row.try_get("", "account_name")?,
            counterpart_id: Some(row.try_get("", "counterpart_id")?),
            counterpart_name: Some(row.try_get("", "counterpart_name")?),
            currency_code: row.try_get("", "currency_code")?,
            foreign_amount: row.try_get("", "total_foreign_amount")?,
            total_debit: row.try_get("", "total_debit")?,
            total_credit: row.try_get("", "total_credit")?,
            weighted_avg_rate: row.try_get("", "weighted_avg_rate")?,
            current_bgn_equivalent: row.try_get("", "bgn_equivalent")?,
        });
    }

    Ok(positions)
}

/// Find ALL currency balances (universal for all accounts with foreign currency)
/// Supports: 501 (cash), 503 (bank), 401-409 (payables), 411-419 (receivables), loans, etc.
async fn find_all_currency_balances(
    db: &DatabaseConnection,
    company_id: i32,
    base_currency: &str,
    as_of_date: NaiveDate,
) -> FieldResult<Vec<OpenForeignPosition>> {
    use sea_orm::DbBackend;
    use sea_orm::Statement;

    // Universal SQL query - finds all accounts with currency balances
    // Groups by account and optionally by counterpart (if exists)
    let sql = format!(
        r#"
        SELECT
            el.account_id,
            acc.code as account_code,
            acc.name as account_name,
            el.counterpart_id,
            cp.name as counterpart_name,
            el.currency_code,
            SUM(COALESCE(el.quantity, 0)) as total_foreign_amount,
            SUM(el.debit_amount) as total_debit,
            SUM(el.credit_amount) as total_credit,
            CASE
                WHEN SUM(COALESCE(el.quantity, 0)) != 0
                THEN SUM(el.debit_amount - el.credit_amount) / SUM(COALESCE(el.quantity, 0))
                ELSE 1
            END as weighted_avg_rate,
            SUM(el.debit_amount - el.credit_amount) as bgn_equivalent
        FROM entry_lines el
        INNER JOIN journal_entries je ON je.id = el.journal_entry_id
        INNER JOIN accounts acc ON acc.id = el.account_id
        LEFT JOIN counterparts cp ON cp.id = el.counterpart_id
        WHERE je.company_id = {}
          AND je.accounting_date <= '{}'
          AND je.is_posted = true
          AND el.currency_code IS NOT NULL
          AND el.currency_code != '{}'
        GROUP BY el.account_id, acc.code, acc.name, el.counterpart_id, cp.name, el.currency_code
        HAVING ABS(SUM(COALESCE(el.quantity, 0))) > 0.01
        ORDER BY acc.code, el.currency_code
        "#,
        company_id, as_of_date, base_currency
    );

    let stmt = Statement::from_string(DbBackend::Postgres, sql);
    let query_result = db.query_all(stmt).await?;

    let mut positions = Vec::new();
    for row in query_result {
        let counterpart_id: Option<i32> = row.try_get("", "counterpart_id").ok();
        let counterpart_name: Option<String> = row.try_get("", "counterpart_name").ok();

        positions.push(OpenForeignPosition {
            account_id: row.try_get("", "account_id")?,
            account_code: row.try_get("", "account_code")?,
            account_name: row.try_get("", "account_name")?,
            counterpart_id,
            counterpart_name,
            currency_code: row.try_get("", "currency_code")?,
            foreign_amount: row.try_get("", "total_foreign_amount")?,
            total_debit: row.try_get("", "total_debit")?,
            total_credit: row.try_get("", "total_credit")?,
            weighted_avg_rate: row.try_get("", "weighted_avg_rate")?,
            current_bgn_equivalent: row.try_get("", "bgn_equivalent")?,
        });
    }

    Ok(positions)
}

/// Get exchange rates for a specific date as a HashMap
async fn get_exchange_rates_map(
    db: &DatabaseConnection,
    date: NaiveDate,
) -> FieldResult<HashMap<String, Decimal>> {
    let rates = exchange_rate::Entity::find()
        .filter(exchange_rate::Column::ValidDate.lte(date))
        .filter(exchange_rate::Column::IsActive.eq(true))
        .filter(exchange_rate::Column::ToCurrencyId.eq(1)) // Assuming BGN is ID 1
        .all(db)
        .await?;

    // Get currency codes
    let currency_ids: Vec<i32> = rates.iter().map(|r| r.from_currency_id).collect();
    let currencies = currency::Entity::find()
        .filter(currency::Column::Id.is_in(currency_ids))
        .all(db)
        .await?;

    let currency_map: HashMap<i32, String> = currencies
        .into_iter()
        .map(|c| (c.id, c.code))
        .collect();

    // Group rates by currency and take the latest
    let mut rate_map: HashMap<String, (NaiveDate, Decimal)> = HashMap::new();
    for rate in rates {
        if let Some(code) = currency_map.get(&rate.from_currency_id) {
            rate_map
                .entry(code.clone())
                .and_modify(|(existing_date, existing_rate)| {
                    if rate.valid_date > *existing_date {
                        *existing_date = rate.valid_date;
                        *existing_rate = rate.rate;
                    }
                })
                .or_insert((rate.valid_date, rate.rate));
        }
    }

    Ok(rate_map.into_iter().map(|(k, (_, v))| (k, v)).collect())
}

/// Create a journal entry for currency revaluation
async fn create_revaluation_journal_entry(
    db: &DatabaseConnection,
    company_id: i32,
    revaluation_date: NaiveDate,
    position: &OpenForeignPosition,
    difference: Decimal, // Positive = gain (revenue 724), Negative = loss (expense 624)
    new_rate: Decimal,
    settings: &CurrencyRevaluationSettingsModel,
) -> FieldResult<()> {
    // Determine if this is a gain or loss
    let is_gain = difference > Decimal::ZERO;

    // Generate entry number with timestamp to ensure uniqueness
    let timestamp = chrono::Utc::now().format("%H%M%S").to_string();
    let entry_number = format!(
        "REVAL-{}-{}-{}",
        revaluation_date.format("%Y%m%d"),
        timestamp,
        position.counterpart_id.unwrap_or(position.account_id)
    );

    // Create journal entry
    let entry = journal_entry::ActiveModel {
        company_id: Set(company_id),
        entry_number: Set(entry_number),
        document_date: Set(revaluation_date),
        accounting_date: Set(revaluation_date),
        vat_date: Set(Some(revaluation_date)),
        document_number: Set(Some(format!(
            "REVAL-{}-{}",
            revaluation_date.format("%Y%m%d"),
            position.counterpart_id.unwrap_or(position.account_id)
        ))),
        description: Set(format!(
            "Валутна преоценка {} {} - {} ({}) - курс {}",
            position.account_code,
            position.counterpart_name.as_ref().map(|n| format!("- {}", n)).unwrap_or_default(),
            position.currency_code,
            revaluation_date.format("%d.%m.%Y"),
            new_rate
        )),
        total_amount: Set(difference.abs()), // Total revaluation amount
        total_vat_amount: Set(Decimal::ZERO), // No VAT on revaluation
        is_posted: Set(false), // Leave unposted for review
        created_by: Set(1), // System/Admin user
        ..Default::default()
    };

    let entry = entry.insert(db).await?;

    // Line 1: Account with currency balance - zero amount but with currency difference
    let counterpart_line = entry_line::ActiveModel {
        journal_entry_id: Set(entry.id),
        account_id: Set(position.account_id),
        counterpart_id: Set(position.counterpart_id),
        currency_code: Set(Some(position.currency_code.clone())),
        currency_amount: Set(Some(Decimal::ZERO)), // Zero in foreign currency
        quantity: Set(Some(Decimal::ZERO)), // Zero quantity
        exchange_rate: Set(Some(new_rate)),
        debit_amount: Set(if is_gain { Decimal::ZERO } else { difference.abs() }),
        credit_amount: Set(if is_gain { difference.abs() } else { Decimal::ZERO }),
        base_amount: Set(Decimal::ZERO),
        vat_amount: Set(Decimal::ZERO),
        description: Set(Some(format!(
            "Преоценка {} към {}",
            position.currency_code,
            revaluation_date.format("%d.%m.%Y")
        ))),
        line_order: Set(1),
        ..Default::default()
    };

    counterpart_line.insert(db).await?;

    // Line 2: Revenue (724) or Expense (624) account
    let result_account_id = if is_gain {
        settings.revenue_account_id // 724 - Revenue from currency operations
    } else {
        settings.expense_account_id // 624 - Expense from currency operations
    };

    let result_line = entry_line::ActiveModel {
        journal_entry_id: Set(entry.id),
        account_id: Set(result_account_id),
        counterpart_id: Set(None),
        currency_code: Set(Some("BGN".to_string())),
        currency_amount: Set(Some(difference.abs())),
        exchange_rate: Set(Some(Decimal::ONE)),
        debit_amount: Set(if is_gain { difference.abs() } else { Decimal::ZERO }),
        credit_amount: Set(if is_gain { Decimal::ZERO } else { difference.abs() }),
        base_amount: Set(Decimal::ZERO),
        vat_amount: Set(Decimal::ZERO),
        description: Set(Some(format!(
            "{} от валутна преоценка {}",
            if is_gain { "Приход" } else { "Разход" },
            position.currency_code
        ))),
        line_order: Set(2),
        ..Default::default()
    };

    result_line.insert(db).await?;

    Ok(())
}
