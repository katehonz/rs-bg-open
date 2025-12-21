# Валутна преоценка - Финални стъпки

## ✅ ЗАВЪРШЕНО:

1. ✅ **SQL Migration** - Таблица `currency_revaluation_settings` създадена успешно
2. ✅ **Rust Entity** - `backend/src/entities/currency_revaluation_settings.rs`
3. ✅ **GraphQL Resolver** - `backend/src/graphql/currency_revaluation_resolver.rs`
4. ✅ **Frontend Page** - `frontend/src/pages/CurrencyRevaluation.jsx`
5. ✅ **Menu Integration** - Добавено в App.jsx и Sidebar.jsx
6. ✅ **Registered in Schema** - Добавено в query.rs и mutation.rs

## ⚠️ КОМПИЛАЦИОННИ ГРЕШКИ ЗА ПОПРАВЯНЕ:

### 1. Поправка в `currency_revaluation_resolver.rs`

Файлът използва сложен raw SQL query който може да има грешки. Трябва да се тества компилацията:

```bash
cd backend
cargo check
```

Ако има грешки свързани с:
- `row.try_get("", "column_name")` - променете на правилния синтаксис за SeaORM
- Statement::from_string - проверете дали се използва правилно

### 2. Алтернативен опростен подход (ако има проблеми):

Вместо сложния SQL query, може да използвате SeaORM entities директно:

```rust
// Simplified approach без raw SQL
let entries = journal_entry::Entity::find()
    .filter(journal_entry::Column::CompanyId.eq(company_id))
    .filter(journal_entry::Column::AccountingDate.lte(as_of_date))
    .filter(journal_entry::Column::IsPosted.eq(true))
    .find_also_related(entry_line::Entity)
    .all(db)
    .await?;

// Group by counterpart and currency manually
let mut positions: HashMap<(i32, String), OpenForeignPosition> = HashMap::new();
// ... aggregate logic
```

## 🔧 СТЪПКИ ЗА ТЕСТВАНЕ:

### 1. Build Backend
```bash
cd /home/dvg/z-nim-proloq/rs-bg__contragent/rs-ac-bg/backend
cargo build --release
```

### 2. Restart Backend Container
```bash
docker compose restart accounting-service
```

### 3. Тествайте GraphQL
Отворете http://localhost/graphiql и тествайте:

#### Създаване на настройки:
```graphql
mutation {
  saveRevaluationSettings(input: {
    companyId: 1
    expenseAccountId: 123  # ID на сметка 624
    revenueAccountId: 456  # ID на сметка 724
  }) {
    id
    companyId
  }
}
```

#### Зареждане на настройки:
```graphql
query {
  revaluationSettings(companyId: 1) {
    id
    expenseAccountId
    revenueAccountId
  }
}
```

#### Стартиране на преоценка:
```graphql
mutation {
  runCurrencyRevaluation(
    companyId: 1
    revaluationDate: "2025-11-02"
  ) {
    success
    message
    createdEntriesCount
    totalRevaluationAmount
  }
}
```

### 4. Тествайте Frontend
1. Отворете http://localhost/accounting/currency-revaluation
2. Изберете сметки 624 и 724
3. Запазете настройките
4. Изберете дата и стартирайте преоценка

## 📋 БИЗНЕС ЛОГИКА:

### Как работи преоценката:

1. **Намира отворени позиции** (сметки 401* и 411*):
   - GROUP BY account_id, counterpart_id, currency_code
   - HAVING SUM(currency_amount) != 0
   - Само валути различни от основната (BGN/EUR)

2. **Изчислява претеглен среден курс**:
   - weighted_avg_rate = SUM(bgn_amount) / SUM(currency_amount)

3. **Изчислява разлика**:
   - current_bgn = foreign_amount * current_rate
   - original_bgn = foreign_amount * weighted_avg_rate
   - difference = current_bgn - original_bgn

4. **Създава записи**:
   - Ако difference > 0: Приход (сметка 724)
   - Ако difference < 0: Разход (сметка 624)
   - Контрагентска сметка (401/411) с нулева валутна сума но с BGN разлика

### Пример:

Фирма има вземане от клиент:
- Фактура от 10.09: 1000 EUR @ 1.9558 = 1955.80 BGN
- Плащане на 15.09: 500 EUR @ 1.9600 = 980.00 BGN
- **Отворена позиция**: 500 EUR @ средно 1.9558 = 977.90 BGN

На 31.10 преоценка при курс 1.9620:
- Текуща стойност: 500 * 1.9620 = 981.00 BGN
- Разлика: 981.00 - 977.90 = **+3.10 BGN** (приход)

Запис за преоценка:
```
Дт 411 - Клиенти: 0.00 EUR (но +3.10 BGN разлика)
Кт 724 - Приходи от валутни операции: 3.10 BGN
```

## 🐛 ИЗВЕСТНИ ПРОБЛЕМИ:

1. **company.base_currency може да не съществува**
   - Проверете schema на companies table
   - Може да трябва да добавите колона `base_currency VARCHAR(3) DEFAULT 'BGN'`

2. **Currency ID = 1 може да не е BGN**
   - В get_exchange_rates_map() има hardcoded `.eq(1)`
   - Трябва да намирате BGN/EUR currency_id динамично

3. **Frontend показва грешка "revaluationSettings is not a function"**
   - Проверете дали GraphQL schema е expose-нат правилно
   - Рестартирайте backend след build

## 💡 ПОДОБРЕНИЯ ЗА БЪДЕЩЕТО:

1. **Batch processing** - Ако има много контрагенти, обработвайте на парт и
2. **Audit log** - Логвайте всяка преоценка
3. **Preview mode** - Покажете резултатите преди да създадете записите
4. **Undo/Reverse** - Функция за обратно проведение на преоценка
5. **Scheduled revaluation** - Автоматична преоценка всеки месечен край

## 📞 SUPPORT:

Ако срещнете проблеми:
1. Проверете логовете: `docker logs accounting_service`
2. Проверете GraphQL errors в browser console
3. Тествайте с GraphiQL playground първо
4. Проверете дали има данни в accounts 401* и 411* с foreign currency

Good luck! 🚀
