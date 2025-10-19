# Базова валута на компанията - Ръководство

## Общ преглед

От версия 0.1.0+ системата поддържа конфигуриране на базова валута за всяка компания. Това е критична функционалност за подготовка към въвеждането на еврото в България през 2026 година.

## Какво е базова валута?

**Базовата валута** е основната валута, в която компанията води счетоводството си. Всички транзакции, отчети и справки се изчисляват спрямо тази валута.

### Преди 2026
- Базова валута: **BGN (Български лев)**
- Курсове: От БНБ (Българска народна банка)
- Фиксиран курс: 1 EUR = 1.95583 BGN

### След 2026
- Базова валута: **EUR (Евро)**
- Курсове: От ECB (European Central Bank)
- BGN става обикновена валута (не базова)

## Архитектура на системата

### Препоръчителна конфигурация

```
┌─────────────────────────────────────────────┐
│  Инстанция #1: БД за BGN периоди           │
│  ─────────────────────────────────────────  │
│  База данни: accounting_bgn                 │
│  Базова валута: BGN                         │
│  Период: 2020-2025                          │
│  Провайдър курсове: БНБ                     │
│  Предназначение: Архив, проверки, ревизии   │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│  Инстанция #2: БД за EUR периоди           │
│  ─────────────────────────────────────────  │
│  База данни: accounting_eur                 │
│  Базова валута: EUR                         │
│  Период: 2026+                              │
│  Провайдър курсове: ECB                     │
│  Предназначение: Активна работа             │
└─────────────────────────────────────────────┘
```

## База данни

### Таблица `companies`

Новото поле `base_currency_id` съхранява ID на базовата валута:

```sql
CREATE TABLE companies (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    eik VARCHAR NOT NULL,
    base_currency_id INTEGER REFERENCES currencies(id),
    -- ... други полета
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Таблица `currencies`

```sql
CREATE TABLE currencies (
    id SERIAL PRIMARY KEY,
    code VARCHAR(3) NOT NULL,        -- ISO код: BGN, EUR, USD
    name VARCHAR NOT NULL,
    name_bg VARCHAR NOT NULL,
    symbol VARCHAR(10),              -- Не се използва! Само ISO кодове
    decimal_places INTEGER NOT NULL DEFAULT 2,
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_base_currency BOOLEAN NOT NULL DEFAULT false,
    bnb_code VARCHAR(3),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**ВАЖНО:** Полето `is_base_currency` показва дали валутата може да бъде базова (BGN, EUR), но конкретната базова валута за компанията се определя от `companies.base_currency_id`.

## Миграция на данни

### Създаване на нова компания за 2026

```sql
-- 1. Намери EUR валута
SELECT id FROM currencies WHERE code = 'EUR';
-- Резултат: 2

-- 2. Създай нова компания с EUR като базова валута
INSERT INTO companies (
    name,
    eik,
    base_currency_id,
    created_at
) VALUES (
    'Моята Фирма ЕООД (2026)',
    '123456789',
    2,  -- EUR
    '2026-01-01'
);
```

### Затваряне на стара BGN компания

```sql
-- Деактивирай старата компания
UPDATE companies
SET is_active = false,
    updated_at = NOW()
WHERE id = 1 AND base_currency_id = (
    SELECT id FROM currencies WHERE code = 'BGN'
);
```

## GraphQL API

### Създаване на компания с базова валута

```graphql
mutation CreateCompany {
  createCompany(input: {
    name: "Моята Фирма ЕООД"
    eik: "123456789"
    baseCurrencyId: 2  # EUR
    address: "София, ул. Витоша 1"
    email: "office@company.bg"
  }) {
    id
    name
    baseCurrencyId
  }
}
```

### Промяна на базова валута (не препоръчително!)

```graphql
mutation UpdateCompanyBaseCurrency {
  updateCompany(
    id: 1
    input: {
      baseCurrencyId: 2  # Промяна от BGN на EUR
    }
  ) {
    id
    baseCurrencyId
  }
}
```

⚠️ **ВНИМАНИЕ:** Смяната на базова валута на съществуваща компания **не е препоръчително**! Вместо това създайте нова компания и прехвърлете салдата ръчно.

### Четене на валути

```graphql
query GetCurrencies {
  currencies(filter: { isActive: true }) {
    id
    code
    nameBg
    isBaseCurrency
  }
}
```

## Frontend интерфейс

### Settings страница

Локация: `http://localhost:5173/settings/system`

```jsx
<select value={baseCurrencyId} onChange={handleCurrencyChange}>
  <option value={1}>BGN - Български лев</option>
  <option value={2}>EUR - Евро</option>
  <option value={3}>USD - Американски долар</option>
</select>
```

**Важно:**
- ❌ НЕ използвайте символи: `лв.`, `€`, `$`
- ✅ Използвайте ISO кодове: `BGN`, `EUR`, `USD`

### Currencies страница

Локация: `http://localhost:5173/currencies`

- Избор на провайдър: БНБ или ECB
- Автоматично обновяване на курсове
- Показване само на ISO кодове

## Валутни курсове

### Автоматично определяне на провайдър

Системата автоматично избира правилния провайдър според базовата валута:

```rust
let provider = match company.base_currency_id {
    1 => "BNB",  // BGN → Използвай БНБ
    2 => "ECB",  // EUR → Използвай ECB
    _ => "BNB",  // По подразбиране БНБ
};
```

### БНБ Курсове (за BGN)

```graphql
mutation UpdateBnbRates {
  updateCurrentBnbRates
}
```

- Източник: https://www.bnb.bg/Statistics
- Валути: EUR, USD, GBP, CHF, JPY и др.
- Базова валута: BGN
- Формат: 1 EUR = 1.95583 BGN

### ECB Курсове (за EUR)

```graphql
mutation UpdateEcbRates {
  updateCurrentEcbRates
}
```

- Източник: https://www.ecb.europa.eu/stats/eurofxref/
- Валути: USD, GBP, CHF, BGN, JPY и др.
- Базова валута: EUR
- Формат: 1 EUR = 1.0847 USD

## Сценарии за 2026

### Сценарий 1: Нова фирма (чиста EUR)

**Ситуация:** Започвате нов бизнес през януари 2026.

**Стъпки:**
1. Създайте нова инстанция на приложението
2. Нова база данни: `accounting_eur`
3. Изберете EUR като базова валута
4. Активирайте ECB като провайдър
5. Започнете счетоводството на EUR

**Предимства:**
- Чисто начало, няма миграция
- Пряка работа с евро
- По-лесно съответствие с EU стандарти

### Сценарий 2: Стара фирма (миграция BGN → EUR)

**Ситуация:** Съществуваща фирма, която преминава към EUR през 2026.

**Стъпки:**
1. Затворете финансовата година 2025 в BGN базата
2. Създайте нова компания (или нова БД) за 2026
3. Изберете EUR като базова валута
4. Прехвърлете начални салда **ръчно** към 01.01.2026
5. Продължете работа в EUR

**Важно:**
- Старата BGN база остава за архив и проверки
- Не мигрирайте автоматично - рискувате неточности
- НАП и ревизори ще искат достъп до стари периоди

### Сценарий 3: Работа с два периода (хибриден)

**Ситуация:** Необходимост от едновременен достъп до BGN и EUR периоди.

**Решение:**
```
┌────────────────────────────┐     ┌────────────────────────────┐
│  rs-ac-bg Instance #1      │     │  rs-ac-bg Instance #2      │
│  Port: 8080                │     │  Port: 8081                │
│  DB: accounting_bgn        │     │  DB: accounting_eur        │
│  Base: BGN                 │     │  Base: EUR                 │
│  URL: localhost:5173       │     │  URL: localhost:5174       │
└────────────────────────────┘     └────────────────────────────┘
```

**Конфигурация:**

`.env` за BGN инстанция:
```bash
DATABASE_URL=postgresql://user:pass@localhost/accounting_bgn
PORT=8080
BASE_CURRENCY=BGN
```

`.env` за EUR инстанция:
```bash
DATABASE_URL=postgresql://user:pass@localhost/accounting_eur
PORT=8081
BASE_CURRENCY=EUR
```

## Best Practices

### ✅ Добри практики

1. **Една базова валута = Една база данни**
   - Не смесвайте BGN и EUR в една БД
   - Всяка компания има само една базова валута

2. **Използвайте ISO кодове**
   - BGN, EUR, USD (не лв., €, $)
   - Избягвайте символи и emoji

3. **Архивирайте стари периоди**
   - Пазете BGN база за минали години
   - Backup редовно

4. **Планирайте миграцията**
   - Затворете финансова година преди смяна
   - Прехвърлете салда на 01.01.2026

5. **Тествайте предварително**
   - Създайте тестова EUR компания през Q4 2025
   - Проверете всички отчети

### ❌ Лоши практики

1. **НЕ сменяйте базова валута на активна компания**
   - Риск от загуба на данни
   - Проблеми с исторически отчети

2. **НЕ използвайте символи**
   - `лв.`, `€`, `$` са визуални
   - Използвайте ISO кодове в БД

3. **НЕ правете автоматична конвертация**
   - Не конвертирайте всички записи от BGN → EUR
   - Ръчно прехвърляне на салда е по-безопасно

4. **НЕ забравяйте стари данни**
   - НАП може да поиска данни от 2023
   - Пазете BGN базата минимум 5 години

## Миграция скрипт (пример)

```sql
-- 1. Създай нова EUR компания
INSERT INTO companies (name, eik, base_currency_id)
SELECT
    name || ' (EUR 2026)',
    eik,
    (SELECT id FROM currencies WHERE code = 'EUR')
FROM companies
WHERE id = 1;

-- 2. Копирай сметкоплан
INSERT INTO accounts (company_id, code, name, account_type)
SELECT
    (SELECT MAX(id) FROM companies),  -- Новата компания
    code,
    name,
    account_type
FROM accounts
WHERE company_id = 1;

-- 3. Добави начални салда на 01.01.2026 (РЪЧНО!)
-- НЕ правете автоматично - проверявайте всяко салдо!

INSERT INTO journal_entries (company_id, entry_date, description)
VALUES (
    (SELECT MAX(id) FROM companies),
    '2026-01-01',
    'Начални салда 2026 (конвертирани от BGN)'
);

-- ... продължете с entry_lines
```

## Troubleshooting

### Проблем: Няма базова валута

**Симптом:** `base_currency_id` е NULL

**Решение:**
```sql
-- Задай BGN като базова валута по подразбиране
UPDATE companies
SET base_currency_id = (SELECT id FROM currencies WHERE code = 'BGN')
WHERE base_currency_id IS NULL;
```

### Проблем: Грешни курсове

**Симптом:** Курсовете са различни от очакваните

**Проверка:**
```sql
-- Провери кой провайдър е използван
SELECT er.*, c.code, er.rate_source
FROM exchange_rates er
JOIN currencies c ON c.id = er.from_currency_id
WHERE er.valid_date = CURRENT_DATE
ORDER BY er.rate_source, c.code;
```

**Решение:**
- За BGN компании → Използвай БНБ
- За EUR компании → Използвай ECB

### Проблем: Не мога да променя базова валута

**Симптом:** UI не позволява смяна

**Обяснение:** Това е **feature**, не bug! Смяната на базова валута е опасна операция.

**Решение:** Създайте нова компания с новата валута.

## Референции

- [ECB Exchange Rates API](https://www.ecb.europa.eu/stats/eurofxref/)
- [БНБ Валутни Курсове](https://www.bnb.bg/Statistics/StExternalSector/StExchangeRates/)
- [ECB Integration Guide](./ECB_EXCHANGE_RATES.md)
- [ISO 4217 Currency Codes](https://www.iso.org/iso-4217-currency-codes.html)

## Контакти

За въпроси относно базова валута:
- GitHub Issues: [rs-ac-bg/issues](https://github.com/yourusername/rs-ac-bg/issues)
- Email: support@yourcompany.bg

---

**Последна актуализация:** Октомври 2025
**Версия:** 1.0.0
**Автори:** Development Team
**Статус:** Production Ready
