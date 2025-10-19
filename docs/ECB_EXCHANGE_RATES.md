# ECB (European Central Bank) Exchange Rates Integration

## Общ преглед

ECB сервисът осигурява интеграция с API на Европейската централна банка за получаване на актуални валутни курсове. Това е критична функционалност за подготовка към въвеждането на еврото в България от 2026 година.

## Защо ECB провайдър?

### Преди 2026 (BGN като базова валута)
- **БНБ**: Основен източник за валутни курсове BGN → други валути
- **ECB**: Допълнителен източник за проверка и верификация

### От 2026 нататък (EUR като базова валута)
- **ECB**: Основен източник за валутни курсове EUR → други валути
- **БНБ**: За исторически данни (преди 2026) и за проверка на стари периоди

## Архитектура

### Структура на кода

```
rs-ac-bg/backend/src/
├── services/
│   ├── bnb_service.rs    # БНБ валутни курсове (BGN базирани)
│   └── ecb_service.rs    # ECB валутни курсове (EUR базирани)
├── entities/
│   ├── currency.rs       # Модел на валута
│   └── exchange_rate.rs  # Модел на валутен курс (RateSource::Ecb)
└── graphql/
    └── currency_resolvers.rs # GraphQL mutations/queries
```

### ECB Service API

#### Основни методи

```rust
// Създаване на сервис
let ecb_service = EcbService::new();

// Извличане на курсове за конкретна дата
let rates = ecb_service.fetch_rates_for_date(db, date).await?;

// Обновяване на курсове в базата за конкретна дата
let count = ecb_service.update_rates_for_date(db, date).await?;

// Обновяване на текущите курсове
let count = ecb_service.update_current_rates(db).await?;

// Обновяване за период от дати
let results = ecb_service.update_rates_for_range(db, from_date, to_date).await?;
```

## ECB API Endpoints

### Текущи курсове (последните 90 дни)
```
https://www.ecb.europa.eu/stats/eurofxref/eurofxref-hist-90d.xml
```
- Оптимален за актуални и близки исторически данни
- Малък файл, бърз достъп
- Обновява се всеки работен ден около 16:00 CET

### Пълна история
```
https://www.ecb.europa.eu/stats/eurofxref/eurofxref-hist.xml
```
- Всички курсове от 1999 година
- Голям файл (няколко MB)
- Използва се само когато е нужна стара история

### Формат на XML отговора

```xml
<gesmes:Envelope>
  <Cube>
    <Cube time="2025-05-28">
      <Cube currency="USD" rate="1.0847"/>
      <Cube currency="GBP" rate="0.8532"/>
      <Cube currency="BGN" rate="1.9558"/>
      <!-- ... други валути -->
    </Cube>
  </Cube>
</gesmes:Envelope>
```

## Поддържани валути от ECB

ECB публикува курсове за 31+ валути спрямо EUR:

**Европейски валути:**
- BGN (Български лев) - **ВАЖНО за прехода към EUR**
- GBP (Британска лира)
- CHF (Швейцарски франк)
- CZK (Чешка крона)
- DKK (Датска крона)
- HUF (Унгарски форинт)
- PLN (Полска злота)
- RON (Румънска лея)
- SEK (Шведска крона)
- NOK (Норвежка крона)
- ISK (Исландска крона)
- TRY (Турска лира)

**Световни валути:**
- USD (Американски долар)
- JPY (Японска йена)
- CNY (Китайски юан)
- AUD (Австралийски долар)
- CAD (Канадски долар)
- и други...

Пълен списък: `EcbService::supported_currencies()`

## GraphQL API

### Mutations

#### 1. Обновяване на ECB курсове за конкретна дата

```graphql
mutation {
  updateEcbRatesForDate(date: "2025-05-28")
}
```

**Отговор:** Брой обновени курсове (напр. 31)

#### 2. Обновяване на текущите ECB курсове

```graphql
mutation {
  updateCurrentEcbRates
}
```

**Отговор:** Брой обновени курсове

**Забележка:** Ако днес още няма публикувани курсове (преди 16:00 CET или е почивен ден), ще се използват курсовете от предходния работен ден.

#### 3. Обновяване за период

```graphql
mutation {
  updateEcbRatesForRange(
    fromDate: "2025-01-01",
    toDate: "2025-01-31"
  )
}
```

**Отговор:** Списък с текстови съобщения за всеки ден
```json
[
  "2025-01-02: 31 rates updated",
  "2025-01-03: 31 rates updated",
  ...
]
```

### Queries

Използват се същите queries като за БНБ, но с филтър по `RateSource`:

```graphql
query {
  exchangeRates(
    filter: {
      rateSource: ECB,
      dateFrom: "2025-05-01",
      dateTo: "2025-05-31"
    }
  ) {
    id
    rate
    validDate
    rateSource
  }
}
```

## Логика на преобразуване на курсове

### Pre-2026: BGN като базова валута

Когато базовата валута е BGN, ECB курсовете (EUR-базирани) се преобразуват към BGN:

```rust
// ECB дава: 1 EUR = 1.0847 USD
// Фиксиран курс: 1 EUR = 1.95583 BGN
// Следователно: 1 USD = 1.95583 / 1.0847 = 1.8034 BGN

if !is_eur_base {
    let fixed_eur_bgn = Decimal::from_str("1.95583").unwrap();
    let bgn_rate = fixed_eur_bgn / eur_to_foreign;
    // Запазва се в БД като: USD -> BGN = 1.8034
}
```

### Post-2026: EUR като базова валута

Когато базовата валута е EUR, ECB курсовете се запазват директно:

```rust
// ECB дава: 1 EUR = 1.0847 USD
// Запазва се директно в БД като: EUR -> USD = 1.0847

if is_eur_base {
    // Директно запазване на курса от ECB
    let rate = eur_to_foreign;
}
```

## База данни

### Таблица `exchange_rates`

Курсовете от ECB се маркират със:
```sql
rate_source = 'ECB'
bnb_rate_id = 'ECB_USD_2025-05-28'  -- Формат: ECB_{currency}_{date}
```

### Примерен запис

```sql
INSERT INTO exchange_rates (
    from_currency_id,     -- EUR (за post-2026) или USD (за pre-2026)
    to_currency_id,       -- USD (за post-2026) или BGN (за pre-2026)
    rate,                 -- 1.0847 (за post-2026) или 1.8034 (за pre-2026)
    reverse_rate,         -- 0.9219 (за post-2026) или 0.5545 (за pre-2026)
    valid_date,           -- 2025-05-28
    rate_source,          -- 'ECB'
    bnb_rate_id,          -- 'ECB_USD_2025-05-28'
    is_active,            -- true
    created_at            -- timestamp
);
```

## Работен поток за 2026

### Сценарий 1: Нова фирма от 2026 (само EUR)

```rust
// 1. Създаване на компания с EUR като базова валута
let company = Company {
    base_currency: "EUR",
    start_date: "2026-01-01",
    ...
};

// 2. Използване на ECB като основен провайдър
let ecb_service = EcbService::new();

// 3. Зареждане на курсове за месеца
ecb_service.update_rates_for_range(
    db,
    NaiveDate::from_ymd(2026, 1, 1),
    NaiveDate::from_ymd(2026, 1, 31)
).await?;

// 4. Работа с курсове
let conversion = convert_currency(
    from: EUR,
    to: USD,
    amount: 1000.00,
    date: "2026-01-15"
);
```

### Сценарий 2: Стара фирма (BGN) - преглед на история

```rust
// Фирма създадена преди 2026, но искаме да разгледаме стари периоди
let company = Company {
    base_currency: "BGN",  // Старата базова валута
    start_date: "2020-01-01",
    ...
};

// За старите периоди използваме БНБ
let bnb_service = BnbService::new();
bnb_service.update_rates_for_range(
    db,
    NaiveDate::from_ymd(2023, 1, 1),
    NaiveDate::from_ymd(2023, 12, 31)
).await?;

// За ревизия и проверка след 2026
let audit_query = "
    SELECT * FROM exchange_rates
    WHERE valid_date >= '2023-01-01'
    AND valid_date <= '2023-12-31'
    AND rate_source = 'BNB'
";
```

## Тестване

### Unit тестове

```bash
cd rs-ac-bg/backend
cargo test ecb_service -- --nocapture
```

### Интеграционни тестове

```rust
#[tokio::test]
async fn test_ecb_fetch_and_save() {
    let db = setup_test_db().await;
    let ecb_service = EcbService::new();

    // Fetch rates for today
    let today = Utc::now().date_naive();
    let count = ecb_service.update_rates_for_date(&db, today).await.unwrap();

    assert!(count > 0);

    // Verify rates are saved
    let rates = exchange_rate::Entity::find()
        .filter(exchange_rate::Column::RateSource.eq(RateSource::Ecb))
        .filter(exchange_rate::Column::ValidDate.eq(today))
        .all(&db)
        .await
        .unwrap();

    assert_eq!(rates.len(), count);
}
```

### Ръчно тестване чрез GraphQL Playground

```graphql
# 1. Обнови курсовете за днес
mutation {
  updateCurrentEcbRates
}

# 2. Виж какво е записано
query {
  exchangeRates(
    filter: { rateSource: ECB }
    limit: 10
  ) {
    id
    fromCurrencyId
    toCurrencyId
    rate
    validDate
    rateSource
  }
}

# 3. Направи конвертация
query {
  convertCurrency(
    fromCurrencyId: 2,  # EUR
    toCurrencyId: 3,    # USD
    amount: 1000,
    date: "2025-05-28"
  ) {
    fromCurrency
    toCurrency
    fromAmount
    toAmount
    exchangeRate
    rateSource
  }
}
```

## Производителност и оптимизация

### Кеширане

- ECB API връща статични XML файлове
- Можете да кеширате резултатите за деня до 16:00 CET
- За исторически данни кеширането е безопасно

### Rate Limiting

ECB няма строги ограничения, но:
- Използвайте 90-дневния endpoint когато е възможно
- Добавяйте малки забавяния между заявките (300ms в кода)
- Не обновявайте курсове по-често от веднъж на ден

### Batch операции

```rust
// ПРАВИЛНО: Зареди период наведнъж
ecb_service.update_rates_for_range(db, from, to).await?;

// ГРЕШНО: Зареди ден по ден в цикъл без забавяне
for date in dates {
    ecb_service.update_rates_for_date(db, date).await?; // Без delay!
}
```

## Миграция за 2026

### Контролен списък за преход

- [ ] **Q4 2025**: Добавете EUR като валута (ако вече не е)
- [ ] **Q4 2025**: Тествайте ECB интеграцията
- [ ] **Jan 2026**:
  - [ ] Затворете старата компания (BGN базирана)
  - [ ] Създайте нова компания (EUR базирана)
  - [ ] Прехвърлете салда ръчно
  - [ ] Активирайте ECB като primary source
- [ ] **Post Jan 2026**:
  - [ ] БНБ остава активна за исторически справки
  - [ ] Добавете BGN като обикновена валута (не базова)

### Скрипт за миграция (пример)

```sql
-- 1. Създай нова компания за 2026
INSERT INTO companies (name, base_currency_id, start_date)
VALUES ('My Company SAS', (SELECT id FROM currencies WHERE code = 'EUR'), '2026-01-01');

-- 2. Деактивирай старите BGN курсове за новата компания
-- (Те остават за старата компания за справки)

-- 3. Зареди EUR курсове от ECB
-- (Чрез GraphQL mutation или service)
```

## Troubleshooting

### Проблем: Няма курсове за днес

**Причина:** ECB публикува курсове около 16:00 CET. Ако е по-рано или е почивен ден, няма да има нови данни.

**Решение:** Методът `update_current_rates()` автоматично fallback-ва към вчерашния ден.

### Проблем: XML parse error

**Причина:** ECB понякога променя структурата на XML или API-то е недостъпно.

**Решение:**
```rust
match ecb_service.fetch_rates_for_date(db, date).await {
    Ok(rates) => { /* use rates */ },
    Err(e) => {
        tracing::error!("ECB fetch failed: {}", e);
        // Fallback to BNB or manual entry
    }
}
```

### Проблем: BGN курс не се преобразува правилно

**Причина:** Фиксираният курс EUR/BGN = 1.95583 може да се промени при официално въвеждане на еврото.

**Решение:** Проверете актуалния официален курс от БНБ и ако е необходимо, актуализирайте константата в кода.

## Полезни връзки

- [ECB Exchange Rate API Docs](https://www.ecb.europa.eu/stats/eurofxref/)
- [ECB Statistical Data Warehouse](https://sdw.ecb.europa.eu/)
- [БНБ - Въвеждане на еврото](https://www.bnb.bg/AboutUs/AUEuroIntroduction)

## Контакти и поддръжка

За въпроси и проблеми:
- GitHub Issues: [вашия repo]
- Email: [support email]

---

**Последна актуализация:** Октомври 2025
**Версия:** 1.0.0
**Съвместимост:** rs-ac-bg v0.1.0+
