# SAF-T v1.0.1 Имплементация

## Преглед

Актуализирана имплементация на SAF-T (Standard Audit File for Tax) според финалната одобрена структура v1.0.1 за България. Тази версия ще влезе в сила през **2026 година** за големи фирми с милиони оборот.

## Промени спрямо предишната версия

### 🔧 Технически промени

#### 1. Нов Namespace
- **Стар**: `urn:StandardAuditFile-Tax:BG_2.0`
- **Нов**: `mf:nra:dgti:dxxxx:declaration:v1`

#### 2. Структурни промени

**Header промени:**
- Добавено: `AuditFileRegion` (BG-01 до BG-28)
- Добавено: `Ownership` структура
- Промяна: `AuditFileDateCreated` от DateTime към Date
- Добавено: `HeaderComment` (A/M/O за Annual/Monthly/OnDemand)
- Добавено: `TaxAccountingBasis` (A/P/BANK/INSURANCE)

**Три типа файлове:**
1. **Annual (A)** - Годишен отчет с активи и собственост
2. **Monthly (M)** - Месечен отчет с главна книга и документи  
3. **OnDemand (O)** - При поискване със складови наличности

**Адресна структура:**
- `Number` вместо `BuildingNumber`
- `AdditionalAddressDetail` вместо `AddressDetail`
- Добавено: `AddressType` (винаги "StreetAddress")

**Контакти:**
- Подробна `ContactPerson` структура с имена и титли
- Множество `OtherTitles`

### 📁 Файлова структура

```
backend/src/
├── entities/saft.rs              # Актуализирани структури v1.0.1
├── services/saft_service.rs      # Стара версия (deprecated)
├── services/saft_service_v2.rs   # Нова версия v1.0.1
└── graphql/saft_resolvers.rs     # Актуализиран GraphQL интерфейс

frontend/src/
└── pages/SafTExport.jsx          # Нов UI за v1.0.1

docs/
├── SAFT_IMPLEMENTATION.md        # Стара документация
└── SAFT_V2_IMPLEMENTATION.md     # Тази документация

file/SAFT_BG/                     # Официални файлове НАП
├── BG_SAFT_Schema_V_1.0.1.xsd    # XSD схема v1.0.1
├── SAF-T_BG_Structure_Definition_V_1.0.1.xlsx
└── VS_SAMPLE_AuditFile_*.xml     # Примерни файлове
```

## GraphQL API

### Нов интерфейс

```graphql
input SafTExportInput {
  companyId: Int!
  periodStart: Int!        # Месец 1-12
  periodStartYear: Int!
  periodEnd: Int!          # Месец 1-12  
  periodEndYear: Int!
  fileType: String!        # "Annual", "Monthly", "OnDemand"
  taxAccountingBasis: String! # "A", "P", "BANK", "INSURANCE"
}

type SafTExportResult {
  success: Boolean!
  fileContent: String
  fileName: String!
  errorMessage: String
}
```

### Примери за използване

**Месечен отчет за ноември-декември 2024:**
```json
{
  "companyId": 1,
  "periodStart": 11,
  "periodStartYear": 2024,
  "periodEnd": 12,
  "periodEndYear": 2024,
  "fileType": "Monthly",
  "taxAccountingBasis": "A"
}
```

## Frontend интерфейс

### Нови полета:
- Избор на начален месец/година
- Избор на краен месец/година  
- Тип файл (Месечен/Годишен/При поискване)
- Тип счетоводство (Търговски/Бюджетни/Банки/Застрахователи)

### Генерирани имена на файлове:
`saft_{company_id}_{start_year}_{start_month}_to_{end_year}_{end_month}.xml`

Пример: `saft_1_2024_11_to_2024_12.xml`

## Обработка на липсващи данни

### Адреси
При липсващи адреси се използват default стойности:
- `street_name`: "неизвестен адрес"
- `city`: "неизвестен град" (или "София" за компании)
- `country`: "BG"

### Контакти
При липсващи данни се създават празни структури според XSD изискванията.

## Валидация

Създаден тест файл `frontend/test_saft_v1_0_1.xml` за проверка срещу официалната XSD схема:

```bash
xmllint --schema ./file/SAFT_BG/BG_SAFT_Schema_V_1.0.1.xsd frontend/test_saft_v1_0_1.xml --noout
```

### Известни проблеми за довършване:
1. Реда на XML елементите в `Contact` структурата
2. Задължителни елементи в `Company` 
3. `Ownership` структура преди `DefaultCurrencyCode`
4. Празни контейнери (`Customers`, `Suppliers`) трябват минимум 1 елемент

## Статус

### ✅ Завършено:
- [x] Актуализирани структури според v1.0.1
- [x] Нова сервизна имплементация
- [x] GraphQL API актуализация
- [x] Frontend интерфейс
- [x] Обработка на липсващи данни
- [x] Основно XML генериране
- [x] Валидация срещу XSD схема

### 🔄 За довършване (преди 2026):
- [ ] Корекция на XML елементите според XSD валидацията
- [ ] Интеграция с реални данни от базата данни
- [ ] Попълване на всички задължителни полета
- [ ] Ownership структура с реални данни
- [ ] Тестване с големи обеми данни
- [ ] Производителност за милионни обороти

## Внедряване за производство

**Таймлайн:** До 2026 г. за фирми с големи обороти

**Следващи стъпки:**
1. Довършване на XML структурата  
2. Тестване с реални данни
3. Производителност тестове
4. Интеграция с НАП системите

---

*Документация създадена: 05.09.2024*  
*Версия: v1.0.1 (финална одобрена структура)*