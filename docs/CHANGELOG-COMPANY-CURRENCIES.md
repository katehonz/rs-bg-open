# Changelog: Company Currencies & Fixed Rates (2025-11-05)

## Промени

### 🎯 Фиксиран EUR/BGN курс

EUR/BGN курсът е сега **хардкоднат на 1.95583** и не се променя от БНБ или ЕЦБ API.

#### Какво беше променено:
- ❌ **Преди**: EUR/BGN се обновяваше от БНБ API с грешни стойности (напр. 1.9572)
- ✅ **Сега**: EUR/BGN е фиксиран на 1.95583 (Bulgarian currency board)

### 🏢 Избор на валута и провайдер за фирми

Всяка фирма сега може да избере:

#### 1. Основна валута (`base_currency_id`)
- BGN (по подразбиране)
- EUR
- USD
- Всички други поддържани валути

#### 2. Провайдер за валутни курсове (`preferred_rate_provider`)
- **BNB** - Българска Народна Банка (по подразбиране)
- **ECB** - Европейска Централна Банка
- **MANUAL** - Ръчно въвеждане
- **API** - Външно API

### 📁 Нови файлове

#### Backend
- `backend/src/services/fixed_rates_service.rs` - Сервис за фиксирани курсове
- `migration/src/m20251105_000001_add_preferred_rate_provider.rs` - Миграция

#### Документация
- `docs/FIXED_EXCHANGE_RATES.md` - Пълна документация за фиксирани курсове
- `docs/CHANGELOG-COMPANY-CURRENCIES.md` - Този файл

### 🔧 Модифицирани файлове

#### Backend
- `backend/src/entities/company.rs`
  - Добавен enum `RateProvider`
  - Добавено поле `preferred_rate_provider: RateProvider`
  - Обновени `CreateCompanyInput` и `UpdateCompanyInput`

- `backend/src/services/bnb_service.rs`
  - Пропуска EUR при обновяване на курсове
  - Log: "Skipping EUR - fixed currency board rate"

- `backend/src/services/ecb_service.rs`
  - Пропуска EUR/BGN при обновяване на курсове
  - Log: "Skipping EUR/BGN - fixed currency board rate"

- `backend/src/services/mod.rs`
  - Добавен `pub mod fixed_rates_service;`

- `backend/src/graphql/admin_resolvers.rs`
  - Добавени `base_currency_id` и `preferred_rate_provider` в Input types
  - Обновени `create_company` и `update_company` mutations

- `backend/src/graphql/currency_resolvers.rs`
  - Добавена функция `ensure_fixed_eur_bgn_rate(date)`
  - Добавена функция `ensure_current_fixed_rates()`

- `migration/src/lib.rs`
  - Добавена нова миграция `m20251105_000001_add_preferred_rate_provider`

#### Frontend
- `frontend/src/pages/AdminCompanies.jsx`
  - Добавен `GET_CURRENCIES` query
  - Обновена `CompanyForm` с две нови полета:
    - Dropdown за избор на основна валута
    - Dropdown за избор на провайдер за курсове
  - Обновени GraphQL mutations за включване на новите полета

#### Database
- Добавена колона `preferred_rate_provider VARCHAR(10) NOT NULL DEFAULT 'BNB'` в `companies` таблица
- Обновен EUR/BGN курс в `exchange_rates` таблица:
  - `rate = 1.955830`
  - `reverse_rate = 0.511292`
  - `rate_source = 'MANUAL'`
  - `notes = 'Fixed EUR/BGN currency board rate'`

## Как да използвате

### 1. Създаване на фирма с избор на валута и провайдер

```jsx
// Frontend - AdminCompanies.jsx
const [formData, setFormData] = useState({
  name: 'Моята фирма',
  eik: '123456789',
  baseCurrencyId: 1, // BGN
  preferredRateProvider: 'BNB',
  // ... други полета
});
```

```graphql
# Backend GraphQL
mutation {
  createCompany(input: {
    name: "Моята фирма"
    eik: "123456789"
    baseCurrencyId: 1
    preferredRateProvider: BNB
  }) {
    id
    name
    baseCurrencyId
    preferredRateProvider
  }
}
```

### 2. Проверка на фиксирания EUR/BGN курс

```sql
SELECT
    c1.code as from_currency,
    c2.code as to_currency,
    er.rate,
    er.rate_source,
    er.valid_date
FROM exchange_rates er
JOIN currencies c1 ON er.from_currency_id = c1.id
JOIN currencies c2 ON er.to_currency_id = c2.id
WHERE c1.code = 'EUR' AND c2.code = 'BGN';

-- Expected: rate = 1.955830, rate_source = MANUAL
```

### 3. Задаване на фиксиран курс програмно

```graphql
# За конкретна дата
mutation {
  ensureFixedEurBgnRate(date: "2025-11-05")
}

# За последните 30 дни
mutation {
  ensureCurrentFixedRates
}
```

## Legacy EU Валути

Системата поддържа фиксирани курсове за исторически валути:

| Валута | Код | Курс към EUR |
|--------|-----|-------------|
| Croatian Kuna | HRK | 7.53450 |
| Lithuanian Litas | LTL | 3.45280 |
| Latvian Lats | LVL | 0.702804 |
| Estonian Kroon | EEK | 15.6466 |
| Slovak Koruna | SKK | 30.1260 |
| Cypriot Pound | CYP | 0.585274 |
| Maltese Lira | MTL | 0.429300 |
| Slovenian Tolar | SIT | 239.640 |

## Migration

Миграцията се прилага автоматично при стартиране на backend:

```bash
docker compose restart accounting-service
```

Или ръчно:

```sql
ALTER TABLE companies
ADD COLUMN preferred_rate_provider VARCHAR(10) NOT NULL DEFAULT 'BNB';
```

## Тестване

1. ✅ Рестартирайте контейнерите
2. ✅ Отворете Admin → Фирми
3. ✅ Създайте/редактирайте фирма
4. ✅ Видими са новите полета за валута и провайдер
5. ✅ EUR/BGN курсът е 1.95583 и не се променя

## Важни забележки

⚠️ **EUR/BGN курсът е фиксиран на 1.95583** и не може да бъде променен от API!

✅ Това отразява реалния currency board на България

✅ Независимо от избрания провайдер, фиксираните курсове остават непроменени

## Автори

Промени направени на: 2025-11-05
