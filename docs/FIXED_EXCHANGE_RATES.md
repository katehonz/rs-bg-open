# Фиксирани валутни курсове

## Общ преглед

Системата поддържа фиксирани валутни курсове, които не се променят от външни API източници. Това е особено важно за:
- **EUR/BGN** - фиксиран currency board курс на България
- **Legacy EU валути** - фиксирани курсове за исторически валути преди влизане в еврозоната

## EUR/BGN Currency Board

България използва currency board механизъм с фиксиран курс:

```
1 EUR = 1.95583 BGN
1 BGN = 0.511292 EUR
```

Този курс е **хардкоднат** в системата и **никога не се променя** от БНБ или ЕЦБ API.

**Забележка:** Системата поддържа общо **32 валути** - BGN (базова), EUR (фиксирана) и 30 валути от БНБ/ECB. За пълен списък вижте [CURRENCY_OPERATIONS.md](./CURRENCY_OPERATIONS.md) или [ECB_EXCHANGE_RATES.md](./ECB_EXCHANGE_RATES.md).

### Имплементация

#### FixedRatesService

Файл: `backend/src/services/fixed_rates_service.rs`

```rust
pub struct FixedRatesService;

impl FixedRatesService {
    /// EUR/BGN is fixed at 1.95583 (Bulgarian currency board)
    pub const EUR_BGN_RATE: &'static str = "1.95583";

    /// Ensure fixed EUR/BGN rate exists in database for a specific date
    pub async fn ensure_eur_bgn_rate(&self, db: &DatabaseConnection, date: NaiveDate) -> Result<()>

    /// Get the fixed EUR/BGN rate (always returns 1.95583)
    pub fn get_eur_bgn_rate() -> Decimal

    /// Check if a currency pair is a fixed rate
    pub fn is_fixed_rate(from_code: &str, to_code: &str) -> bool
}
```

#### Модификации на услугите

**BNB Service** (`backend/src/services/bnb_service.rs`):
- Пропуска EUR при обновяване на курсове
- Log message: "Skipping EUR - fixed currency board rate"

**ECB Service** (`backend/src/services/ecb_service.rs`):
- Пропуска EUR/BGN при обновяване на курсове
- Използва фиксирания курс 1.95583 за конвертиране на други валути към BGN

### GraphQL API

#### Mutations

```graphql
# Задаване на фиксиран EUR/BGN курс за конкретна дата
mutation {
  ensureFixedEurBgnRate(date: "2025-11-05")
}

# Задаване на фиксирани курсове за последните 30 дни
mutation {
  ensureCurrentFixedRates
}
```

## Legacy EU Валути

Системата съдържа фиксирани курсове към EUR за исторически валути:

| Валута | Код | Фиксиран курс към EUR | Дата на въвеждане на EUR |
|--------|-----|----------------------|-------------------------|
| Croatian Kuna | HRK | 7.53450 | 01.01.2023 |
| Lithuanian Litas | LTL | 3.45280 | 01.01.2015 |
| Latvian Lats | LVL | 0.702804 | 01.01.2014 |
| Estonian Kroon | EEK | 15.6466 | 01.01.2011 |
| Slovak Koruna | SKK | 30.1260 | 01.01.2009 |
| Cypriot Pound | CYP | 0.585274 | 01.01.2008 |
| Maltese Lira | MTL | 0.429300 | 01.01.2008 |
| Slovenian Tolar | SIT | 239.640 | 01.01.2007 |

### Получаване на списъка

```rust
let legacy_rates = FixedRatesService::get_legacy_eu_fixed_rates();
// Връща: Vec<(&'static str, &'static str, &'static str)>
//        (Currency Code, Name, Fixed Rate to EUR)
```

## Конфигурация на фирми

Всяка фирма може да избере:

### Основна валута (`base_currency_id`)
- BGN (по подразбиране)
- EUR
- USD
- Други поддържани валути

### Провайдер за курсове (`preferred_rate_provider`)
- **BNB** - Българска Народна Банка (по подразбиране)
- **ECB** - Европейска Централна Банка
- **MANUAL** - Ръчно въвеждане
- **API** - Външно API

**Забележка**: Независимо от избрания провайдер, EUR/BGN винаги остава фиксиран на 1.95583!

## База данни

### Структура на exchange_rates

```sql
CREATE TABLE exchange_rates (
    id SERIAL PRIMARY KEY,
    from_currency_id INTEGER NOT NULL,
    to_currency_id INTEGER NOT NULL,
    rate DECIMAL(15,6) NOT NULL,
    reverse_rate DECIMAL(15,6) NOT NULL,
    valid_date DATE NOT NULL,
    rate_source VARCHAR(10) NOT NULL, -- 'BNB', 'ECB', 'MANUAL', 'API'
    bnb_rate_id VARCHAR,
    notes TEXT,
    ...
);
```

### Пример за фиксиран EUR/BGN запис

```sql
SELECT * FROM exchange_rates
WHERE from_currency_id = (SELECT id FROM currencies WHERE code = 'EUR')
  AND to_currency_id = (SELECT id FROM currencies WHERE code = 'BGN');

-- Result:
-- rate: 1.955830
-- reverse_rate: 0.511292
-- rate_source: MANUAL
-- notes: Fixed EUR/BGN currency board rate
```

## Актуализация на курсове

### Автоматично обновяване

При изпълнение на:
```graphql
mutation {
  updateBnbRatesForDate(date: "2025-11-05")
}
```

или:
```graphql
mutation {
  updateEcbRatesForDate(date: "2025-11-05")
}
```

EUR/BGN курсът **няма да бъде обновен** и ще остане 1.95583.

### Ръчно обновяване

Ако се опитате да създадете/обновите EUR/BGN курс ръчно, системата ще го презапише с фиксирания курс при следващото стартиране на `ensure_eur_bgn_rate()`.

## Миграции

### m20251105_000001_add_preferred_rate_provider.rs

Добавя `preferred_rate_provider` поле в companies таблицата:

```rust
manager.alter_table(
    Table::alter()
        .table(Companies::Table)
        .add_column(
            ColumnDef::new(Companies::PreferredRateProvider)
                .string_len(10)
                .default("BNB")
                .not_null(),
        )
        .to_owned(),
)
```

## Frontend

### Admin Companies форма

Нови полета:
1. **Основна валута** - Dropdown select с всички валути
2. **Провайдер за курсове** - Select с опции:
   - БНБ (Българска Народна Банка)
   - ЕЦБ (Европейска Централна Банка)
   - Ръчно въвеждане
   - Външно API

Файл: `frontend/src/pages/AdminCompanies.jsx`

```jsx
<FormControl fullWidth margin="dense">
  <InputLabel>Провайдер за курсове</InputLabel>
  <Select value={formData.preferredRateProvider}>
    <MenuItem value="BNB">БНБ (Българска Народна Банка)</MenuItem>
    <MenuItem value="ECB">ЕЦБ (Европейска Централна Банка)</MenuItem>
    <MenuItem value="MANUAL">Ръчно въвеждане</MenuItem>
    <MenuItem value="API">Външно API</MenuItem>
  </Select>
</FormControl>
```

## Тестване

### Проверка на фиксирания курс

```sql
-- Проверка дали EUR/BGN е фиксиран
SELECT
    c1.code as from_currency,
    c2.code as to_currency,
    er.rate,
    er.rate_source,
    er.valid_date
FROM exchange_rates er
JOIN currencies c1 ON er.from_currency_id = c1.id
JOIN currencies c2 ON er.to_currency_id = c2.id
WHERE c1.code = 'EUR' AND c2.code = 'BGN'
ORDER BY er.valid_date DESC
LIMIT 5;

-- Expected: rate = 1.955830, rate_source = MANUAL
```

### Проверка на логовете

```bash
docker compose logs accounting-service | grep "EUR"

# Expected output:
# "Skipping EUR - fixed currency board rate"
# "Skipping EUR/BGN - fixed currency board rate"
```

## Важни забележки

1. ⚠️ **Никога не променяйте EUR/BGN курса** в production без официална промяна от БНБ
2. ✅ Системата автоматично поддържа фиксирания курс
3. ✅ Legacy EU валути също имат фиксирани курсове
4. ✅ Независимо от избрания провайдер, фиксираните курсове остават непроменени

## Референции

- [БНБ Currency Board](https://www.bnb.bg/AboutUs/AUCurrencyBoard/index.htm)
- [ECB Exchange Rates](https://www.ecb.europa.eu/stats/policy_and_exchange_rates/)
- [EU Legacy Currencies](https://ec.europa.eu/info/business-economy-euro/euro-area/euro/eu-countries-and-euro_en)
