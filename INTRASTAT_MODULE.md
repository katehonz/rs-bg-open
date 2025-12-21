# INTRASTAT Модул - Техническа документация

## 🌍 Общ преглед

INTRASTAT модулът е интегрирана част от RS-AC-BG счетоводната система, предназначена за управление на декларации за вътреобщностна търговия съгласно изискванията на НАП и ЕС регламентите.

## 📋 Съдържание

- [Функционалности](#функционалности)
- [Архитектура](#архитектура)
- [База данни](#база-данни)
- [Backend API](#backend-api)
- [Frontend](#frontend)
- [Инсталация](#инсталация)
- [Конфигурация](#конфигурация)
- [XML експорт](#xml-експорт)
- [Тестване](#тестване)

## 🔧 Функционалности

### Основни възможности:
- ✅ **Автоматично проследяване на прагове** - 400,000 лв. за входящи/изходящи
- ✅ **Свързване на сметки с номенклатура** - опционално при надвишени прагове
- ✅ **Автоматично генериране на декларации** от журналните записи
- ✅ **XML експорт** според формата на НАП (версия 2022)
- ✅ **Управление на CN номенклатура** с импорт от CSV
- ✅ **Детайлни справки** по страни, периоди и кодове
- ✅ **Валидация на данни** преди подаване
- ✅ **Многоезична поддръжка** (БГ/EN)

### Типове декларации:
- **Пристигания (ARRIVAL)** - внос от други ЕС страни
- **Изпращания (DISPATCH)** - износ към други ЕС страни

## 🏗️ Архитектура

### Backend (Rust + SeaORM)
```
backend/src/
├── entities/
│   ├── intrastat_nomenclature.rs      # CN номенклатура
│   ├── intrastat_account_mapping.rs   # Свързване сметки-номенклатура
│   ├── intrastat_declaration.rs       # Декларации
│   ├── intrastat_declaration_item.rs  # Артикули в декларации
│   └── intrastat_settings.rs          # Настройки на компанията
├── services/
│   ├── intrastat_service.rs           # Бизнес логика
│   └── intrastat_xml_export.rs        # XML експорт
└── graphql/
    └── intrastat_resolver.rs           # GraphQL API
```

### Frontend (React)
```
frontend/src/
├── components/Intrastat/
│   ├── IntrastatDashboard.jsx         # Основно табло
│   ├── IntrastatReports.jsx           # Справки
│   ├── IntrastatSettings.jsx          # Настройки
│   └── index.jsx                      # Експорти
└── pages/
    └── Intrastat.jsx                  # Главна страница с табове
```

## 🗄️ База данни

### Основни таблици:

#### `intrastat_nomenclature`
```sql
CREATE TABLE intrastat_nomenclature (
    id SERIAL PRIMARY KEY,
    cn_code VARCHAR(10) NOT NULL UNIQUE,        -- CN код (8 цифри)
    description_bg TEXT NOT NULL,               -- Описание на български
    description_en TEXT,                        -- Описание на английски
    unit_of_measure VARCHAR(20) NOT NULL,       -- Мерна единица
    unit_description VARCHAR(100) NOT NULL,     -- Описание на мерната единица
    parent_code VARCHAR(10),                    -- Родителски код
    level INTEGER NOT NULL DEFAULT 0,           -- Ниво в йерархията
    is_active BOOLEAN NOT NULL DEFAULT true,    -- Активен запис
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### `intrastat_account_mapping`
```sql
CREATE TABLE intrastat_account_mapping (
    id SERIAL PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    nomenclature_id INTEGER NOT NULL REFERENCES intrastat_nomenclature(id),
    flow_direction VARCHAR(10) NOT NULL CHECK (flow_direction IN ('ARRIVAL', 'DISPATCH')),
    transaction_nature_code VARCHAR(5) NOT NULL,    -- Вид сделка (11, 12, 21, ...)
    is_quantity_tracked BOOLEAN NOT NULL DEFAULT true,
    default_country_code VARCHAR(2),                -- Страна по подразбиране
    default_transport_mode INTEGER,                 -- Транспорт по подразбиране
    is_optional BOOLEAN NOT NULL DEFAULT false,     -- Опционално свързване
    min_threshold_bgn DECIMAL(15, 2),              -- Минимален праг в лв.
    company_id INTEGER NOT NULL REFERENCES companies(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### `intrastat_declaration`
```sql
CREATE TABLE intrastat_declaration (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL REFERENCES companies(id),
    declaration_type VARCHAR(10) NOT NULL CHECK (declaration_type IN ('ARRIVAL', 'DISPATCH')),
    reference_period VARCHAR(6) NOT NULL,           -- YYYYMM формат
    year INTEGER NOT NULL,
    month INTEGER NOT NULL CHECK (month >= 1 AND month <= 12),
    declaration_number VARCHAR(50),                 -- Номер от НАП
    declarant_eik VARCHAR(20) NOT NULL,            -- ЕИК на декларатора
    declarant_name VARCHAR(200) NOT NULL,          -- Име на декларатора
    contact_person VARCHAR(200) NOT NULL,          -- Лице за контакт
    contact_phone VARCHAR(50) NOT NULL,            -- Телефон
    contact_email VARCHAR(200) NOT NULL,           -- Имейл
    total_items INTEGER NOT NULL DEFAULT 0,         -- Общ брой артикули
    total_statistical_value DECIMAL(15, 2) NOT NULL DEFAULT 0.00,
    total_invoice_value DECIMAL(15, 2) NOT NULL DEFAULT 0.00,
    status VARCHAR(15) NOT NULL DEFAULT 'DRAFT',   -- DRAFT/SUBMITTED/ACCEPTED/REJECTED
    submission_date TIMESTAMP WITH TIME ZONE,      -- Дата на подаване
    xml_file_path TEXT,                            -- Път до XML файл
    created_by INTEGER NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### `intrastat_settings`
```sql
CREATE TABLE intrastat_settings (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL REFERENCES companies(id),
    is_enabled BOOLEAN NOT NULL DEFAULT false,
    arrival_threshold_bgn DECIMAL(15, 2) NOT NULL DEFAULT 400000.00,     -- Праг за входящи
    dispatch_threshold_bgn DECIMAL(15, 2) NOT NULL DEFAULT 400000.00,    -- Праг за изходящи
    current_arrival_threshold_bgn DECIMAL(15, 2) NOT NULL DEFAULT 0.00,  -- Текущо за входящи
    current_dispatch_threshold_bgn DECIMAL(15, 2) NOT NULL DEFAULT 0.00, -- Текущо за изходящи
    auto_generate_declarations BOOLEAN NOT NULL DEFAULT false,           -- Авто генериране
    default_transport_mode INTEGER,                                     -- Транспорт по подр.
    default_delivery_terms VARCHAR(10),                                 -- Условия доставка
    default_transaction_nature VARCHAR(5),                              -- Вид сделка по подр.
    responsible_person_name VARCHAR(200),                               -- Отговорно лице
    responsible_person_phone VARCHAR(50),
    responsible_person_email VARCHAR(200),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## 🔌 Backend API

### GraphQL Queries

```graphql
# Настройки на компанията
query GetIntrastatSettings($companyId: Int!) {
  intrastatSettings(companyId: $companyId) {
    id
    isEnabled
    arrivalThresholdBgn
    dispatchThresholdBgn
    currentArrivalThresholdBgn
    currentDispatchThresholdBgn
  }
}

# Номенклатура с търсене
query GetNomenclatures($search: String, $limit: Int) {
  intrastatNomenclatures(search: $search, limit: $limit) {
    id
    cnCode
    descriptionBg
    unitOfMeasure
  }
}

# Декларации по период
query GetDeclarations($companyId: Int!, $year: Int, $month: Int) {
  intrastatDeclarations(companyId: $companyId, year: $year, month: $month) {
    id
    declarationType
    referencePeriod
    totalItems
    totalInvoiceValue
    status
  }
}

# Проверка на прагове
query CheckThreshold($companyId: Int!) {
  checkIntrastatThreshold(companyId: $companyId) {
    arrivalExceeded
    dispatchExceeded
  }
}
```

### GraphQL Mutations

```graphql
# Създаване на декларация
mutation CreateDeclaration($input: CreateIntrastatDeclarationInput!) {
  createIntrastatDeclaration(input: $input) {
    id
    declarationType
    year
    month
  }
}

# Импорт на номенклатура
mutation ImportNomenclature($csvData: String!) {
  importIntrastatNomenclature(csvData: $csvData) {
    success
    importedCount
    message
  }
}

# Експорт в XML
mutation ExportXml($declarationId: Int!) {
  exportIntrastatXml(declarationId: $declarationId)
}

# Валидация на декларация
mutation ValidateDeclaration($declarationId: Int!) {
  validateIntrastatDeclaration(declarationId: $declarationId) {
    isValid
    errors
  }
}
```

## 🎨 Frontend

### Навигация
Модулът е достъпен от главното меню:
```
Счетоводство > 🌍 INTRASTAT
```

### Табове:
1. **📊 Табло** - Преглед на статус, прагове и декларации
2. **📈 Справки** - Детайлни анализи и експорти
3. **⚙️ Настройки** - Конфигурация и свързвания

### Компоненти:
- `IntrastatDashboard` - Основно табло с overview
- `IntrastatReports` - Справки по страни, CN кодове, периоди
- `IntrastatSettings` - Настройки, импорт номенклатура, свързвания

## 🚀 Инсталация

### 1. Backend зависимости
```toml
# Cargo.toml
[dependencies]
csv = "1.3"                    # За импорт на номенклатура
quick-xml = { version = "0.36", features = ["serialize"] }
rust_decimal = { version = "1.37", features = ["serde", "db-postgres"] }
```

### 2. База данни миграция
```bash
# Изпълнете SQL скрипта
psql -d your_database -f migration/intrastat_schema.sql
```

### 3. Frontend зависимости
Няма допълнителни зависимости - използва се Tailwind CSS.

## ⚙️ Конфигурация

### 1. Активиране на модула
```javascript
// В настройките на компанията
{
  "isEnabled": true,
  "arrivalThresholdBgn": 400000.00,
  "dispatchThresholdBgn": 400000.00,
  "responsiblePersonName": "Иван Иванов",
  "responsiblePersonPhone": "+359 2 123 4567",
  "responsiblePersonEmail": "ivan@company.bg"
}
```

### 2. Импорт на номенклатура
CSV файл с колони:
```csv
CN_код,Описание,Мерна_единица,Описание_единица
84832000,Лагери с търкалящи се елементи,p/st,бройки
85011000,Електрически двигатели,p/st,бройки
```

### 3. Свързване на сметки
```javascript
// Пример за свързване
{
  "accountId": 304,           // Материали
  "nomenclatureId": 123,      // CN код 84832000
  "flowDirection": "ARRIVAL", // Входящи
  "transactionNatureCode": "11", // Незабавна продажба
  "isOptional": true         // Опционално свързване
}
```

## 📤 XML Експорт

### Формат според НАП (2022)
```xml
<?xml version="1.0" encoding="UTF-8"?>
<INSTAT>
  <Envelope>
    <envelopeId>BG123456789202411</envelopeId>
    <DateTime>2024-11-30T10:30:00</DateTime>
    <Party>
      <partyType>PSI</partyType>
      <partyId>123456789</partyId>
      <partyName>Моята фирма ООД</partyName>
    </Party>
    <Declaration>
      <declarationId>1_202411</declarationId>
      <referencePeriod>202411</referencePeriod>
      <Function>O+A</Function>
      <declarationTypeCode>19</declarationTypeCode>
      <flowCode>A</flowCode>
      <currencyCode>BGN</currencyCode>
      <totalInvoicedAmount>25000.00</totalInvoicedAmount>
      <totalStatisticalValue>25000.00</totalStatisticalValue>
      <totalItems>15</totalItems>
    </Declaration>
  </Envelope>
  
  <INSTAT_A>
    <ITEM_A>
      <itemNumber>1</itemNumber>
      <CN8>
        <CN8Code>84832000</CN8Code>
      </CN8>
      <MSConsDestCode>DE</MSConsDestCode>
      <countryOfOriginCode>DE</countryOfOriginCode>
      <netMass>15.500</netMass>
      <invoicedAmount>2500.00</invoicedAmount>
      <statisticalValue>2500.00</statisticalValue>
      <NatureOfTransaction>
        <natureOfTransactionACode>1</natureOfTransactionACode>
        <natureOfTransactionBCode>1</natureOfTransactionBCode>
      </NatureOfTransaction>
      <modeOfTransportCode>3</modeOfTransportCode>
      <deliveryTerms>EXW</deliveryTerms>
    </ITEM_A>
  </INSTAT_A>
</INSTAT>
```

## 🧪 Тестване

### 1. Тест на компилация
```bash
cd backend && cargo build
```

### 2. Тест на frontend
```bash
cd frontend && npm run build
```

### 3. Тест на API
```bash
# GraphQL playground на http://localhost:8000/graphql
```

### 4. Тест на XML експорт
```bash
# Създайте тестова декларация и експортирайте XML
```

## 🔄 Workflow

### 1. Настройка (еднократно)
- Активиране на модула в настройките
- Импорт на CN номенклатура от CSV
- Задаване на отговорно лице

### 2. Месечна работа
- Система автоматично проследява оборота
- При надвишаване на прага - предупреждение
- Генериране на декларация от журналните записи
- Редактиране и валидация на данните
- XML експорт за подаване в НАП

### 3. Опционални дейности
- Свързване на количествени сметки с номенклатура
- Детайлни справки за анализ
- Експорт на данни за архивиране

## 📊 Статистики и мониторинг

### Key Performance Indicators (KPIs)
- Общ оборот по месеци (входящи/изходящи)
- Процент надвишаване на прагове
- Брой подадени декларации
- Най-често използвани CN кодове
- Топ търговски партньори по страни

### Справки
- Месечни обороти по страни
- Детайли по CN кодове
- Анализ на транспортните режими
- История на подаванията

## 🚨 Известни ограничения

1. **Автоматично свързване** - не покрива 100% от случаите
2. **Валидация** - основни проверки, не заменя човешкия преглед
3. **Исторически данни** - миграция от стари системи не е автоматична
4. **Многовалутност** - работи основно с BGN

## 🔮 Планирани подобрения

- [ ] AI-базирано предложение на CN кодове
- [ ] Автоматично разпознаване на стоките от описанието
- [ ] Интеграция с митническите декларации
- [ ] Mobile app за бърза проверка на статус
- [ ] Advanced аналитика с графики и dashboard

## 📞 Поддръжка

За техническа поддръжка или въпроси относно INTRASTAT модула:
- 📧 Email: support@rs-ac-bg.com
- 📱 Телефон: +359 2 XXX XXXX
- 💬 Вътрешен чат: #intrastat-support

---

**Документация създадена на:** 11.09.2025  
**Версия:** 1.0.0  
**Автор:** Claude Code Assistant  
**Статус:** ✅ Готов за продукция