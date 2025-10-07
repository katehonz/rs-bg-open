# Импорт Система

Модул за импорт на счетоводни данни от външни източници.

## 📥 Типове импорти

### 1. Controlisy импорт

Автоматичен импорт от PDF документи и XML файлове обработени от Controlisy AI.

#### Поддържани файлове
- `.xml` - Структурирани счетоводни данни  
- `.pdf` - Автоматично обработени фактури

#### XML структура
```xml
<ExportedData>
  <Contractors>
    <Contractor
      contractorName="Име"
      contractorEIK="БУЛСТАТ"
      contractorVATNumber="ДДС номер"
    />
  </Contractors>
  <Documents>
    <Document
      documentDate="YYYY-MM-DD"
      documentNumber="номер"
      reason="описание"
      totalAmountBGN="сума"
    >
      <Accountings>
        <Accounting amountBGN="сума">
          <AccountingDetail 
            direction="Debit|Credit"
            accountNumber="сметка"
            accountName="име"
            contractorName="контрагент"
          />
        </Accounting>
      </Accountings>
      <VATData vatRegister="1">
        <VAT 
          taxBase="основа"
          vatRate="ставка" 
          vatAmountBGN="ддс"
          vatOperationIden="1|2|3|..."
        />
      </VATData>
    </Document>
  </Documents>
</ExportedData>
```

#### ДДС операции
- **Код 1** - Покупки с пълен ДК
- **Код 2** - Покупки с частичен ДК  
- **Код 3** - Покупки без ДДС
- **Код 9001** - ВОП операции
- И други според ППЗДДС

#### Workflow
1. **Upload** - Качване на XML/PDF файлове
2. **Parse** - Автоматично парсване на структурата
3. **Review** - Преглед на данните за импорт
4. **Map** - Мапиране на сметки (Controlisy → Ваши)
5. **Import** - Финален импорт в системата

#### Особености
- Игнорират се аналитичните признаци (accountItem1-4)
- Записи по една сметка се групират автоматично
- Валидация за балансираност на документите

### 2. Банкови извлечения

Импорт на банкови операции от различни формати.

#### Настройка на банкови профили
- Път: **Настройки → Банки** (`/banks`)
- За всяка банка се задават аналитична сметка, буферна сметка, валута и формат на импорта (UniCredit MT940, Wise CAMT.053, Postbank XML и др.).
- IBAN е по избор и се използва за по-лесна идентификация.

> Буферната сметка се използва за временно отразяване на транзакцията и впоследствие може да се разпределя по финални сметки/контрагенти директно от интерфейса за банкови записи.

#### Преглед и разпределение на банкови записи
- Таб **„Банкови записи“** в `/banks` показва всички импортнати извлечения.
- Филтриране по период „От дата / До дата“ (зареждат се само импортите в избрания диапазон).
- Списъкът с импортите е сортиран по дата/час и показва брой записи и обороти.
- Обобщение за периода: брой журнални записи, оборот Дт/Кт и салдо по банковата сметка.
- Щракването върху буферния ред отваря инлайн редактор със търсачка за сметки и контрагенти. След запис се презарежда само съответният журнален запис.
- По-стари извлечения се откриват чрез филтъра по дати.

#### Поддържани формати

##### MT940 (SWIFT стандарт)
```
:20:REFERENCE
:25:ACCOUNT
:28C:STATEMENT
:60F:OPENING
:61:VALUE DATE
:86:INFORMATION
:62F:CLOSING
```

##### CSV формат
```csv
Date,Reference,Description,Debit,Credit,Balance
2024-01-01,REF001,Payment,100.00,,1000.00
```

##### XML банков
```xml
<BankStatement>
  <Account>BG80BNBG96611020345678</Account>
  <Transactions>
    <Transaction>
      <Date>2024-01-01</Date>
      <Amount>-100.00</Amount>
      <Description>Payment</Description>
    </Transaction>
  </Transactions>
</BankStatement>
```

#### Workflow
1. **Format** - Избор на банков формат
2. **Upload** - Качване на файлове
3. **Parse** - Разпознаване на операциите
4. **Match** - Автоматично мапиране към сметки
5. **Import** - Импорт със създаване на записи

### 3. Универсален импорт

Гъвкав импорт от Excel, CSV и други формати.

#### Шаблони

##### Дневник записи
```csv
Date,Document,Description,Account,Debit,Credit,Counterpart
2024-01-01,DOC001,Purchase,601,100.00,0.00,Supplier Ltd
2024-01-01,DOC001,Purchase,401,0.00,100.00,Supplier Ltd
```

##### Сметкоплан
```csv
Code,Name,Type,Parent,IsAnalytical
1,АКТИВИ,ASSETS,,FALSE
10,Дълготрайни активи,ASSETS,1,FALSE
101,Земи и сгради,ASSETS,10,TRUE
```

##### Контрагенти
```csv
Name,EIK,VATNumber,Address,City,Country
Company Ltd,123456789,BG123456789,Address 1,Sofia,BG
```

#### Workflow
1. **Template** - Избор на шаблон или custom мапиране
2. **Upload** - Качване на файлове
3. **Map** - Мапиране на колони към полета
4. **Validate** - Валидация на данните
5. **Import** - Bulk импорт

## 🔧 Техническа архитектура

### Frontend компоненти
```
pages/ImportCenter.jsx          # Основна страница с табове
├── components/imports/
│   ├── ControlisyImport.jsx    # Controlisy импорт
│   ├── BankImport.jsx          # Банкови извлечения  
│   └── UniversalImport.jsx     # Универсален импорт
└── components/imports/
    └── ControlisyImportModal.jsx # 4-стъпков wizard
```

### Backend endpoints
```rust
// GraphQL mutations
createImportBatch(input: ImportBatchInput!) -> ImportBatch
validateImportBatch(batchId: String!) -> ValidationResult  
importBatch(batchId: String!) -> ImportResult

// Resolvers
import_resolvers.rs             # Импорт логика
controlisy_parser.rs            # Controlisy XML парсър
bank_parser.rs                  # Банкови формати
universal_parser.rs             # Универсален парсър
```

### База данни
```sql
-- Импорт батчове
CREATE TABLE import_batches (
  id UUID PRIMARY KEY,
  batch_id VARCHAR UNIQUE,
  source VARCHAR(50),          -- 'controlisy', 'bank', 'universal'
  status VARCHAR(20),          -- 'draft', 'validated', 'imported'
  company_id UUID
);

-- Импорт документи
CREATE TABLE import_documents (
  id UUID PRIMARY KEY,
  batch_id UUID REFERENCES import_batches(id),
  doc_number VARCHAR,
  doc_date DATE,
  total_amount DECIMAL,
  is_balanced BOOLEAN
);

-- Импорт записи
CREATE TABLE import_entries (
  id UUID PRIMARY KEY,
  document_id UUID REFERENCES import_documents(id),
  account VARCHAR,
  debit DECIMAL,
  credit DECIMAL
);
```

## 🚀 Използване

### 1. Достъп до импорти
```
Меню → Счетоводство → Импорти
URL: /imports
```

### 2. Избор на тип импорт
- **Controlisy** - PDF/XML от Controlisy система
- **Банкови извлечения** - MT940, CSV, XML  
- **Универсален** - Excel, CSV с шаблони

### 3. Качване на файлове
- Drag & drop поддръжка
- Multiple file selection
- Real-time валидация на форматите

### 4. Преглед и валидация
- Автоматично парсване
- Показване на статистики
- Preview на данните за импорт

### 5. Мапиране (при нужда)
- Автоматични предложения
- Ръчно мапиране на сметки
- Запазване на мапинга за бъдещо

### 6. Финален импорт
- Batch създаване на записи
- Real-time progress
- Детайлен резултат с грешки

## 🔍 Валидация

### Общи проверки
- ✅ Файлов формат
- ✅ Размер на файла  
- ✅ Encoding (UTF-8)
- ✅ Валидна структура

### Счетоводни проверки
- ✅ Балансираност (дебит = кредит)
- ✅ Валидни сметки от сметкоплана
- ✅ Валидни контрагенти
- ✅ Правилни дати

### ДДС проверки
- ✅ Валидни ДДС кодове
- ✅ Правилни ставки (0%, 9%, 20%)
- ✅ Точни изчисления
- ✅ ДДС регистрация на контрагенти

## 📊 История и статистики

### История на импорти
- Списък със всички импорти
- Статус и резултати
- Възможност за rollback
- Детайлни логове

### Статистики
- Общо импорти
- Успешни vs грешки
- Най-често използвани източници
- Performance метрики

## 🔧 Конфигурация

### Настройки за импорт
```json
{
  "maxFileSize": "50MB",
  "allowedFormats": [".xml", ".pdf", ".csv", ".xlsx"],
  "autoValidation": true,
  "defaultAccountMapping": {
    "601": "6010",
    "401": "4010"
  }
}
```

### Controlisy специфични
```json
{
  "ignoreAnalyticalItems": true,
  "groupEntriesByAccount": true,
  "validateVATCodes": true,
  "defaultVATRegister": "1"
}
```

### Банкови специфични  
```json
{
  "mt940Encoding": "UTF-8",
  "csvDelimiter": ",",
  "dateFormat": "YYYY-MM-DD",
  "defaultBankAccount": "501"
}
```

## 🚨 Грешки и troubleshooting

### Чести грешки

#### "Файлът не може да се парсне"
- Проверете формата на файла
- Уверете се че е валиден XML/CSV
- Проверете encoding (трябва UTF-8)

#### "Документът не е балансиран"
- Дебит сумата не е равна на кредит
- Проверете изчисленията в оригиналния файл
- Възможно е грешка при парсването

#### "Невалидна сметка"
- Сметката не съществува в сметкоплана
- Използвайте мапинг за коригиране
- Добавете сметката в сметкоплана

#### "ДДС кодът не е валиден"
- Проверете ППЗДДС кодовете (01-95)
- Уверете се за правилна ДДС операция
- Консултирайте се с ППЗДДС документацията

### Debug режим
```javascript
localStorage.setItem('debug-imports', 'true');
// Ще покаже подробни логове в console
```

## 📈 Бъдещи подобрения

- [ ] **OCR интеграция** за PDF файлове
- [ ] **AI мапиране** на сметки
- [ ] **Real-time синхронизация** с банки
- [ ] **Bulk edit** на импорт данни
- [ ] **Advanced филтри** в историята
- [ ] **Export на резултати** в различни формати
- [ ] **Email нотификации** за завършени импорти
- [ ] **API за програмен импорт**
