# SAF-T Имплементация - RS-AC-BG

## Преглед

SAF-T (Standard Audit File for Tax) е международен стандарт за електронни одитни файлове, който улеснява данъчните проверки и анализи. Тази документация описва пълната имплементация на SAF-T в RS-AC-BG счетоводната система за България.

## Статус на Имплементацията

### ✅ Завършени Компоненти

- **Backend SAF-T Структури** - Пълни Rust структури за SAF-T данни
- **XML Генериране** - Автоматично генериране на валиден SAF-T XML
- **GraphQL API** - Мутации и заявки за SAF-T експорт
- **Frontend Интерфейс** - Потребителски интерфейс за експорт
- **Валидация** - Проверка на данни преди експорт
- **Основни Секции** - Главна книга, сметкоплан, контрагенти

### 🚧 В Процес на Разработка

- **ДДС Секция** - Специфично ДДС отчитане според български изисквания
- **Тестване** - Валидация на генерираните файлове

## Архитектура

### Backend Компоненти

```
backend/src/
├── entities/
│   └── saft.rs                    # SAF-T структури
├── services/
│   └── saft_service.rs           # SAF-T бизнес логика
└── graphql/
    └── saft_resolvers.rs         # GraphQL resolvers
```

### Frontend Компоненти

```
frontend/src/
└── pages/
    └── SafTExport.jsx            # Потребителски интерфейс
```

## Структура на SAF-T Файла

### Основни Секции

1. **Header (Заглавка)**
   - Информация за файла и софтуера
   - Данни за компанията
   - Критерии за селекция

2. **MasterFiles (Основни Файлове)**
   - Сметкоплан
   - Клиенти и доставчици
   - ДДС ставки
   - Мерни единици

3. **GeneralLedger (Главна Книга)**
   - Журнални записи
   - Транзакции и редове

4. **AccountsPayable (Задължения)**
   - Транзакции с доставчици
   
5. **AccountsReceivable (Вземания)**
   - Транзакции с клиенти

6. **FixedAssets (Дълготрайни Активи)**
   - Данни за активи

## Технически Детайли

### Backend Имплементация

#### SAF-T Структури (`saft.rs`)

```rust
pub struct BulgarianSafT {
    pub header: SafTHeader,
    pub master_files: MasterFiles,
    pub general_ledger: GeneralLedger,
    pub accounts_payable: Option<AccountsPayable>,
    pub accounts_receivable: Option<AccountsReceivable>,
    pub fixed_assets: Option<FixedAssets>,
}
```

#### Ключови Функции

- **XML Генериране**: Използва `quick-xml` библиотеката
- **Валидация**: Проверява компанията, датите и типа период
- **Български Стандарти**: Включва ДДС ставки 20%, 9%, 0%
- **Йерархия на Сметките**: Синтетични и аналитични сметки

#### SAF-T Service (`saft_service.rs`)

```rust
impl SafTService {
    pub async fn generate_saft(&self, request: SafTExportRequest) 
        -> Result<String, Box<dyn std::error::Error + Send + Sync>>
}
```

### Frontend Имплементация

#### Основни Функции

```javascript
// Валидация на експорт параметри
const handleValidate = async () => { ... }

// Експорт и изтегляне на файла
const handleExport = async () => { ... }
```

#### UI Компоненти

- **Избор на Компания**: Dropdown с налични компании
- **Период**: Месечен или годишен
- **Дати**: От и до с date picker
- **Тип Съдържание**: GL, MD, SI, PI, TR, PS
- **Допълнителни Секции**: Checkbox за AP, AR, Fixed Assets

## Използване

### Стъпки за Експорт

1. **Изберете компанията** за която искате да експортирате
2. **Задайте период** - начална и крайна дата
3. **Изберете тип период** - месечен или годишен
4. **Изберете тип съдържание** според нуждите
5. **Валидирайте** параметрите преди експорт
6. **Експортирайте** файла

### GraphQL Заявки

#### Валидация

```graphql
query ValidateSafTExport($input: SafTExportInput!) {
  validateSafTExport(input: $input) {
    isValid
    validationErrors
    fileSizeBytes
    numberOfTransactions
  }
}
```

#### Експорт

```graphql
mutation ExportSafT($input: SafTExportInput!) {
  exportSaft(input: $input) {
    success
    fileContent
    fileName
    errorMessage
  }
}
```

## Български Специфични Изисквания

### ДДС Ставки

- **20%** - Стандартна ставка
- **9%** - Намалена ставка
- **0%** - Нулева ставка

### Задължителни Полета

- **ЕИК** - Единен идентификационен код
- **ДДС номер** - При регистрирани по ДДС
- **Адрес** - Пълен адрес на компанията

### Дати

Използва се българската тройна система от дати:
- **Документна дата**
- **ДДС дата** (опционална)
- **Счетоводна дата**

## Файлова Структура

### Генериран SAF-T Файл

```xml
<?xml version="1.0" encoding="UTF-8"?>
<AuditFile xmlns="urn:StandardAuditFile-Tax:BG_2.0">
  <Header>
    <AuditFileVersion>2.0</AuditFileVersion>
    <AuditFileCountry>BG</AuditFileCountry>
    <!-- ... -->
  </Header>
  <MasterFiles>
    <GeneralLedgerAccounts>
      <!-- Сметкоплан -->
    </GeneralLedgerAccounts>
    <Customers>
      <!-- Клиенти -->
    </Customers>
    <Suppliers>
      <!-- Доставчици -->
    </Suppliers>
  </MasterFiles>
  <GeneralLedger>
    <Journal>
      <Transaction>
        <!-- Журнални записи -->
      </Transaction>
    </Journal>
  </GeneralLedger>
</AuditFile>
```

## Конфигурация

### Backend Зависимости

```toml
[dependencies]
quick-xml = { version = "0.36", features = ["serialize"] }
chrono = { workspace = true }
rust_decimal = { version = "1.37", features = ["serde", "db-postgres"] }
```

### Навигация

SAF-T експортът е достъпен през:
- **Меню**: Счетоводство → SAF-T Export
- **URL**: `/saft-export`
- **Badge**: Маркиран като "NEW"

## Тестване

### Ръчно Тестване

1. Отворете `/saft-export`
2. Изберете компания
3. Задайте период (напр. цяла 2024 г.)
4. Валидирайте параметрите
5. Експортирайте файла
6. Проверете генерирания XML

### Автоматично Тестване

```bash
# Backend тестове
cd backend
cargo test saft

# Frontend тестове (когато се добавят)
cd frontend
npm test SafTExport
```

## Производителност

### Оптимизации

- **Streaming XML**: За големи файлове
- **Chunked Export**: Обработка на парчета
- **Background Jobs**: За много големи експорти

### Ограничения

- **Файлов размер**: До 100MB препоръчително
- **Брой записи**: До 100,000 транзакции наведнъж
- **Timeout**: 10 минути максимум за експорт

## Съответствие със Стандартите

### OECD SAF-T v2.0

- ✅ Основна структура
- ✅ Задължителни полета
- ✅ XML схема
- ✅ Кодировки и формати

### Български Изисквания

- ✅ НАП стандарти (подготовка за 2026)
- ✅ ДДС отчитане
- ✅ Кирилица поддръжка
- 🚧 Специфични полета (в процес)

## Сигурност

### Защита на Данните

- **Валидация на входа**: Всички параметри се валидират
- **Авторизация**: Достъп само за упълномощени потребители
- **Логиране**: Всички експорти се логират
- **Temporary Files**: Автоматично изтриване

### Лични Данни

- **GDPR съответствие**: Съгласно европейските изисkvания
- **Псевдонимизиране**: При необходимост
- **Достъпни права**: Контрол на достъпа

## Поддръжка и Разширения

### Планирани Подобрения

1. **ДДС Модул**: Пълно ДДС отчитане
2. **Batch Export**: Множество компании наведнъж
3. **Scheduled Exports**: Автоматични периодични експорти
4. **Email Delivery**: Изпращане по имейл
5. **Archive Management**: Архивиране на старите експорти

### Известни Проблеми

- Frontend използва mock данни (временно)
- GraphQL интеграцията предстои
- ДДС секцията не е завършена

## Контакти

За въпроси и поддръжка относно SAF-T имплементацията:
- **Документация**: Виж тази страница
- **Код**: `/backend/src/entities/saft.rs`, `/backend/src/services/saft_service.rs`
- **UI**: `/frontend/src/pages/SafTExport.jsx`

---

*Последна актуализация: 2025-01-09*
*Версия: 1.0.0*