# Валутна преоценка - Backend имплементация

## 1. Database Migration (SQL)

Създайте migration файл в `backend/migrations/`:

```sql
-- Create currency_revaluation_settings table
CREATE TABLE IF NOT EXISTS currency_revaluation_settings (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    expense_account_id INTEGER NOT NULL REFERENCES accounts(id),
    revenue_account_id INTEGER NOT NULL REFERENCES accounts(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(company_id)
);

CREATE INDEX idx_currency_revaluation_settings_company
ON currency_revaluation_settings(company_id);
```

## 2. Rust Entity (`backend/src/entities/currency_revaluation_settings.rs`)

```rust
use async_graphql::{InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "currency_revaluation_settings")]
#[graphql(concrete(name = "CurrencyRevaluationSettings", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub company_id: i32,
    pub expense_account_id: i32,
    pub revenue_account_id: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::CompanyId",
        to = "super::company::Column::Id"
    )]
    Company,

    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::ExpenseAccountId",
        to = "super::account::Column::Id"
    )]
    ExpenseAccount,

    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::RevenueAccountId",
        to = "super::account::Column::Id"
    )]
    RevenueAccount,
}

#[derive(InputObject, Deserialize)]
pub struct RevaluationSettingsInput {
    pub company_id: i32,
    pub expense_account_id: i32,
    pub revenue_account_id: i32,
}

impl ActiveModelBehavior for ActiveModel {}
```

## 3. GraphQL Types (`backend/src/graphql/currency_revaluation_types.rs`)

```rust
use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct RevaluationResult {
    pub success: bool,
    pub message: String,
    pub created_entries_count: i32,
    pub total_revaluation_amount: f64,
}

#[derive(SimpleObject)]
pub struct RevaluationSettingsWithAccounts {
    pub id: i32,
    pub company_id: i32,
    pub expense_account_id: i32,
    pub revenue_account_id: i32,
    pub expense_account: Option<crate::entities::account::Model>,
    pub revenue_account: Option<crate::entities::account::Model>,
}
```

## 4. GraphQL Resolver (`backend/src/graphql/currency_revaluation_resolver.rs`)

Трябва да съдържа:

### Queries:
- `revaluationSettings(companyId: Int!)` - Връща настройките за фирмата

### Mutations:
- `saveRevaluationSettings(input: RevaluationSettingsInput!)` - Записва настройките
- `runCurrencyRevaluation(companyId: Int!, revaluationDate: NaiveDate!)` - Стартира преоценка

### Ключова логика за `runCurrencyRevaluation`:

```rust
async fn run_currency_revaluation(
    ctx: &Context<'_>,
    company_id: i32,
    revaluation_date: NaiveDate,
) -> FieldResult<RevaluationResult> {
    let db = ctx.data::<Arc<DatabaseConnection>>()?;

    // 1. Load settings
    let settings = load_revaluation_settings(db, company_id).await?;

    // 2. Find all open positions in foreign currencies
    let open_positions = find_open_foreign_currency_positions(db, company_id, revaluation_date).await?;

    // 3. Get exchange rates for revaluation date
    let exchange_rates = get_exchange_rates_for_date(db, revaluation_date).await?;

    // 4. Calculate revaluation differences
    let mut total_difference = Decimal::ZERO;
    let mut entries_count = 0;

    for position in open_positions {
        // Calculate difference between current rate and original rate
        let original_amount_bgn = position.amount * position.original_rate;
        let current_amount_bgn = position.amount * current_rate;
        let difference = current_amount_bgn - original_amount_bgn;

        if difference.abs() > Decimal::new(1, 2) { // > 0.01 BGN
            // Create journal entry for revaluation
            create_revaluation_entry(
                db,
                company_id,
                revaluation_date,
                &position,
                difference,
                &settings,
            ).await?;

            total_difference += difference;
            entries_count += 1;
        }
    }

    Ok(RevaluationResult {
        success: true,
        message: format!("Преоценени {} позиции", entries_count),
        created_entries_count: entries_count,
        total_revaluation_amount: total_difference.to_f64().unwrap_or(0.0),
    })
}
```

## 5. Помощни функции

### `find_open_foreign_currency_positions()`
Намира всички:
- Незатворени фактури на клиенти (дебити)
- Незатворени фактури на доставчици (кредити)
- В чужда валута (currency_code != 'BGN')
- За определена дата

SQL концепция:
```sql
SELECT
    el.id,
    el.counterpart_id,
    el.currency_code,
    el.currency_amount,
    el.exchange_rate as original_rate,
    SUM(el.debit_amount - el.credit_amount) as balance_bgn
FROM entry_lines el
JOIN journal_entries je ON je.id = el.journal_entry_id
WHERE je.company_id = $company_id
  AND je.accounting_date <= $revaluation_date
  AND el.currency_code IS NOT NULL
  AND el.currency_code != 'BGN'
  AND el.counterpart_id IS NOT NULL
GROUP BY el.counterpart_id, el.currency_code
HAVING SUM(el.debit_amount - el.credit_amount) != 0
```

### `create_revaluation_entry()`
Създава журнален запис:
- Document number: "REVAL-{date}-{counter}"
- Description: "Валутна преоценка - {counterpart} - {currency}"
- Lines:
  * Дебит/Кредит сметка на контрагента (сума 0, но с валутна разлика)
  * Кредит/Дебит сметка 624 или 724 (валутната разлика в BGN)

## 6. Регистриране на модула

В `backend/src/graphql/mod.rs` добавете:
```rust
pub mod currency_revaluation_resolver;
pub mod currency_revaluation_types;
```

В `backend/src/entities/mod.rs`:
```rust
pub mod currency_revaluation_settings;
```

В главния GraphQL schema добавете resolver-ите към `QueryRoot` и `MutationRoot`.

## 7. Тестване

След имплементация:
1. Стартирайте backend: `cargo run`
2. Rebuild frontend: `npm run build`
3. Restart frontend container
4. Отворете: http://localhost/accounting/currency-revaluation
5. Тествайте:
   - Настройка на сметки 624 и 724
   - Преглед на валутни курсове
   - Изпълнение на преоценка

## Бележки

- Преоценката е **read-only операция** докато не се натисне бутонът
- Всяка преоценка създава нови записи, не модифицира съществуващи
- Валутните разлики се записват като **допълнителна валутна разлика** в entry_lines
- Документите за преоценка имат нулева стойност но променят валутната равностойност
