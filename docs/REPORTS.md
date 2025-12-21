# Отчети и Справки

RS-AC-BG предоставя пълен набор от стандартни счетоводни отчети за българската практика.

## 📊 Налични отчети

### 1. Оборотна ведомост
**Описание**: Обобщена информация за оборотите и салдата на всички сметки за определен период.

**Функционалности**:
- Филтриране по период (от/до дата)
- Избор на конкретна сметка (опционално)
- Показване/скриване на нулеви салда
- Excel и PDF експорт

**Колони**:
- Сметка (код и име)  
- Начално салдо (дебит/кредит)
- Обороти за периода (дебит/кредит)
- Крайно салдо (дебит/кредит)
- Общи суми (footer row)

### 2. Хронологичен регистър  
**Описание**: Хронологичен списък на всички счетоводни операции според българската счетоводна практика.

**Функционалности**:
- Филтриране по период
- Филтриране по сметка (опционално)  
- Excel и PDF експорт
- Поддръжка за ChronReg.csv формат

**Колони**:
- Дата
- Дебит сметка (код/име)
- Кредит сметка (код/име)
- Сума в лева
- Валутни суми (ако са приложими)
- Валутни кодове
- Тип документ
- Дата на документ
- Описание
- Общи суми (footer)

### 3. Главна книга
**Описание**: Подробна история по всяка сметка с всички движения и текущо салдо.

**Функционалности**:
- Групиране по сметки
- Филтриране по период
- Филтриране по конкретна сметка
- Показване на начални и крайни салда
- Само преглед (без експорт засега)

**Секции за всяка сметка**:
- Header с код, име и салда
- Начално салдо row
- Всички операции хронологично
- Обща сума row за сметката

### 3.1. Главна книга (БГ вариант) 🇧🇬
**Описание**: Агрегиран отчет на кореспондиращите обороти между сметки за определен период, съгласно българската счетоводна практика. Отчетът показва сумарно оборотите между дебитни и кредитни сметки без детайлизация по отделни операции.

**Функционалности**:
- Филтриране по период (от/до дата)
- Два режима на групиране:
  - **По Дебит**: Групирани суми по дебитна сметка с кореспонденции към кредитни
  - **По Кредит**: Групирани суми по кредитна сметка с кореспонденции към дебитни
- **Excel (XLSX) експорт** - два отделни worksheets:
  - "По Дебит" - агрегирани дебитни обороти
  - "По Кредит" - агрегирани кредитни обороти
- **ODT (LibreOffice) експорт** - единен документ с две секции
- Професионално форматиране с header rows, subtotals и styling

**Структура на отчета "По Дебит"**:
```
Дебит сметка: 401 - Доставчици
├── Кредит сметка          │ Стойност
├── 501 - Каса             │ 1,500.00 лв
├── 503 - Разплащателна    │ 25,000.00 лв
└── Общо                   │ 26,500.00 лв
```

**Структура на отчета "По Кредит"**:
```
Кредит сметка: 501 - Каса
├── Дебит сметка           │ Стойност
├── 401 - Доставчици       │ 1,500.00 лв
├── 602 - Разходи          │ 800.00 лв
└── Общо                   │ 2,300.00 лв
```

**GraphQL API**:
```graphql
# Query за генериране на отчета
query GetBgGeneralLedger($input: BgGeneralLedgerInput!) {
  bgGeneralLedger(input: $input) {
    companyName
    periodStart
    periodEnd
    byDebit {
      debitAccountCode
      debitAccountName
      totalAmount
      entries {
        creditAccountCode
        creditAccountName
        amount
      }
    }
    byCredit {
      creditAccountCode
      creditAccountName
      totalAmount
      entries {
        debitAccountCode
        debitAccountName
        amount
      }
    }
  }
}

# Mutation за експорт
mutation ExportBgGeneralLedger($input: BgGeneralLedgerInput!, $format: String!) {
  exportBgGeneralLedger(input: $input, format: $format) {
    format      # "XLSX" или "ODT"
    content     # Base64 encoded
    filename    # bg_general_ledger_YYYY-MM-DD_YYYY-MM-DD_CompanyName.xlsx
    mimeType
  }
}
```

**Excel формат особености**:
- Два worksheets в един файл
- Форматиране:
  - Title row: 16pt, bold, centered, merged cells
  - Section headers: 13pt, bold, blue background (#4472C4), white text
  - Column headers: Bold, gray background (#E8E8E8), border
  - Data rows: Thin borders, currency format "0.00"
  - Total rows: Bold, light blue background (#D9E2F3), border
- Автоматична ширина на колони
- Пълна поддръжка на кирилица (UTF-8)

**ODT формат особености**:
- Единен документ с две основни секции
- OpenDocument Text спецификация
- Table стилизиране с borders и backgrounds
- Отваря се директно в LibreOffice Writer
- Пълна съвместимост с кирилица

**Технически детайли**:
- **Backend имплементация**: `backend/src/graphql/reports_resolvers.rs`
  - `bg_general_ledger()` - query resolver
  - `export_bg_general_ledger()` - export mutation
  - `generate_xlsx_bg_general_ledger()` - XLSX generation
  - `generate_odt_bg_general_ledger()` - ODT generation (използва ZIP архивиране)
- **Използвани библиотеки**:
  - `rust_xlsxwriter` - Excel generation
  - `zip` crate - ODT packaging
- **Raw string literals fix**: Използва `r##"..."##` синтаксис за XML съдържание с `#` символи (hex цветове)

### 4. Дневник на операциите
**Описание**: Детайлен списък на всички счетоводни записи с техните редове.

**Функционалности**:
- Филтриране по период
- Филтриране по сметка
- Показване на номер на запис
- Номер на документ
- Само преглед (без експорт засега)

**Колони**:
- Дата
- Номер на запис
- Номер на документ
- Сметка (код/име)  
- Описание
- Дебит/Кредит суми
- Контрагент

## 🔄 Експорт възможности

### Excel (.xlsx)
- **Библиотека**: `rust_xlsxwriter`
- **Формат**: Структурирани работни листове
- **Функции**: 
  - Автоматично оразмеряване на колони
  - Header форматиране
  - Числови формати за суми
  - UTF-8 поддръжка за кирилица

### PDF  
- **Технология**: HTML5-to-PDF с `headless_chrome`
- **Предимства**:
  - Отлично кирилично рендиране  
  - CSS стилизиране
  - Никакви проблеми с транслитерация
  - Професионален вид

**HTML Template функции**:
```html
<!DOCTYPE html>
<html lang="bg">
<head>
    <meta charset="UTF-8">
    <title>{report_title}</title>
    <style>
        body { font-family: Arial, sans-serif; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; }
        .header { background-color: #f5f5f5; }
    </style>
</head>
...
```

## 📋 GraphQL API

### Queries

```graphql
# Оборотна ведомост
query GetTurnoverSheet($input: TurnoverReportInput!) {
  turnoverSheet(input: $input) {
    companyName
    periodStart  
    periodEnd
    entries { ... }
    totals { ... }
    generatedAt
  }
}

# Хронологичен регистър
query GetChronologicalReport($input: ChronologicalReportInput!) {
  chronologicalReport(input: $input) {
    companyName
    entries { ... }
    totalAmount
    generatedAt
  }
}
```

### Mutations за експорт

```graphql
# Excel/PDF експорт
mutation ExportTurnoverSheet($input: TurnoverReportInput!, $format: String!) {
  exportTurnoverSheet(input: $input, format: $format) {
    format
    content    # Base64 encoded
    filename
    mimeType
  }
}
```

## 🔧 Техническа имплементация

### Backend (Rust)
```
backend/src/graphql/reports_resolvers.rs
├── TurnoverSheet queries & mutations  
├── ChronologicalReport queries & mutations
├── TransactionLog & GeneralLedger queries
├── Excel generation functions
└── HTML-to-PDF generation functions
```

### Frontend (React)
```
frontend/src/pages/Reports.jsx
├── Report type selection dropdown
├── Date range pickers  
├── Account filter (optional)
├── Generate & Export buttons
└── Report display tables
```

## 🎯 Планирани подобрения

### V0.3.0
- [ ] PDF експорт за Главна книга и Дневник
- [ ] Пакетен експорт на всички отчети  
- [ ] Email изпращане на отчети
- [ ] Scheduled отчети (daily/monthly)
- [ ] Отчети с лого и header информация

### V0.4.0  
- [ ] Аналитични отчети
- [ ] Графики и визуализации
- [ ] Сравнителни анализи по периоди
- [ ] Custom report builder
- [ ] Интеграция с BI инструменти

## 🇧🇬 Българска специфика

- **Формат на датите**: DD.MM.YYYY (българска локализация)
- **Валута**: BGN с българска локализация (2 дес. знака) 
- **Хронологичен регистър**: Според ChronReg.csv стандарт
- **Нумерация**: Българска последователност на записи
- **Кирилица**: Пълна поддръжка във всички отчети

## 🚫 Недостъпни (засега)

**Годишни отчети** - ще бъдат добавени когато има достатъчно данни:
- [ ] Отчет за приходите и разходите (ОПР)
- [ ] Баланс  
- [ ] Отчет за паричните потоци
- [ ] Приложения към годишния отчет

Тези отчети изискват комплексна година данни и специални изчисления които засега не са приоритет.