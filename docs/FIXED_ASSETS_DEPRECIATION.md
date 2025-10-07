# Модул за дълготрайни активи и амортизации

Този модул предоставя пълна функционалност за управление на дълготрайни активи (ДМА) и тяхната амортизация в съответствие с българското счетоводно и данъчно законодателство.

## 📋 Съдържание

1. [Преглед](#преглед)
2. [Архитектура](#архитектура)
3. [Функционалности](#функционалности)
4. [API Reference](#api-reference)
5. [Frontend Components](#frontend-components)
6. [Счетоводни правила](#счетоводни-правила)
7. [Примери за използване](#примери-за-използване)

## Преглед

Модулът поддържа:
- **Управление на ДМА**: създаване, редактиране, следене на активи
- **Категории активи**: класификация според ЗКПО
- **Амортизационни изчисления**: счетоводни и данъчни норми
- **Журнал на амортизации**: подробно проследяване
- **Автоматизирано осчетоводяване**: генериране на журнални записи

## Архитектура

### Backend структура

```
backend/src/
├── entities/
│   ├── fixed_asset.rs              # Основен модел за ДМА
│   ├── fixed_asset_category.rs     # Категории активи
│   └── depreciation_journal.rs     # Дневник на амортизации
├── services/
│   └── depreciation_service.rs     # Бизнес логика за амортизации
└── graphql/
    └── fixed_assets_resolvers.rs   # GraphQL API
```

### Frontend структура

```
frontend/src/
├── pages/
│   └── FixedAssets.jsx             # Главна страница
└── components/fixedAssets/
    ├── AssetsList.jsx              # Списък активи
    ├── AssetCategories.jsx         # Управление категории
    ├── DepreciationCalculation.jsx # Изчисляване амортизации
    └── DepreciationJournal.jsx     # Дневник амортизации
```

## Функционалности

### 1. Управление на дълготрайни активи

#### Създаване на актив
```graphql
mutation CreateFixedAsset {
  createFixedAsset(
    companyId: 1,
    input: {
      inventoryNumber: "ДМА-001",
      name: "Служебен автомобил",
      categoryId: 5,
      acquisitionCost: 25000.00,
      acquisitionDate: "2025-01-01",
      accountingUsefulLife: 5,
      accountingDepreciationRate: 20.0,
      taxDepreciationRate: 25.0
    }
  ) {
    id
    inventoryNumber
    name
  }
}
```

#### Полета на актив
- **inventoryNumber**: Инвентарен номер (уникален)
- **name**: Наименование на активa
- **categoryId**: Връзка към категория
- **acquisitionCost**: Придобивна стойност
- **acquisitionDate**: Дата на придобиване
- **putIntoServiceDate**: Дата на въвеждане в експлоатация
- **accountingUsefulLife**: Полезен живот (години)
- **accountingDepreciationRate**: Счетоводна норма (%)
- **accountingDepreciationMethod**: Метод (straight_line, declining_balance)
- **accountingSalvageValue**: Остатъчна стойност
- **taxDepreciationRate**: Данъчна норма (%)
- **location**: Местоположение
- **responsiblePerson**: Отговорно лице

### 2. Категории активи

Предефинирани категории според ЗКПО:

| Код | Наименование | Данъчна група | Макс. норма |
|-----|-------------|---------------|-------------|
| BUILDINGS | Сгради и съоръжения | 1 | 4% |
| MACHINERY | Машини и оборудване | 2 | 15% |
| TRANSPORT | Транспортни средства | 3 | 25% |
| COMPUTERS | Компютри и софтуер | 4 | 50% |
| VEHICLES | Автомобили | 5 | 25% |
| OTHER | Други ДМА | 7 | 15% |

### 3. Амортизационни изчисления

#### Типове амортизация

**Счетоводна амортизация:**
- Според счетоводната политика на фирмата
- Може да се различава от данъчната
- Методи: прави срок, намаляващи остатъци

**Данъчна амортизация:**
- Според максималните норми в ЗКПО
- Винаги прави срок
- За данъчни цели

#### Изчисляване на месечна амортизация
```rust
// Счетоводна амортизация (прави срок)
let annual_amount = (acquisition_cost - salvage_value) * (rate / 100);
let monthly_amount = annual_amount / 12;

// Данъчна амортизация
let annual_amount = acquisition_cost * (tax_rate / 100);
let monthly_amount = annual_amount / 12;
```

### 4. Журнал на амортизации

Всяко амортизационно изчисление се записва в журнала:

| Поле | Описание |
|------|----------|
| fixed_asset_id | ID на активa |
| period | Период (YYYY-MM-01) |
| accounting_depreciation_amount | Счетоводна амортизация |
| accounting_book_value_before/after | Счетоводна балансова стойност |
| tax_depreciation_amount | Данъчна амортизация |
| tax_book_value_before/after | Данъчна балансова стойност |
| is_posted | Дали е осчетоводен |
| journal_entry_id | ID на журналния запис |

## API Reference

### GraphQL Queries

#### Активи
```graphql
# Всички активи
query GetFixedAssets($companyId: Int!, $status: String, $categoryId: Int) {
  fixedAssets(companyId: $companyId, status: $status, categoryId: $categoryId) {
    id
    inventoryNumber
    name
    acquisitionCost
    accountingBookValue
    taxBookValue
    status
    category {
      name
    }
  }
}

# Конкретен актив
query GetFixedAsset($id: Int!) {
  fixedAsset(id: $id) {
    id
    inventoryNumber
    name
    # ... всички полета
    category {
      name
    }
  }
}

# Статистика
query GetFixedAssetsStats($companyId: Int!) {
  fixedAssetsSummary(companyId: $companyId) {
    totalAssets
    totalAcquisitionCost
    totalAccountingBookValue
    totalTaxBookValue
    activeAssets
    disposedAssets
  }
}
```

#### Категории
```graphql
query GetCategories {
  fixedAssetCategories {
    id
    code
    name
    taxCategory
    maxTaxDepreciationRate
  }
}
```

#### Амортизации
```graphql
# Дневник на амортизации
query GetDepreciationJournal($companyId: Int!, $year: Int!, $month: Int) {
  depreciationJournal(companyId: $companyId, year: $year, month: $month) {
    id
    fixedAssetId
    period
    accountingDepreciationAmount
    taxDepreciationAmount
    accountingBookValueAfter
    taxBookValueAfter
    isPosted
    journalEntryId
  }
}
```

### GraphQL Mutations

#### Управление активи
```graphql
# Създаване
mutation CreateFixedAsset($companyId: Int!, $input: CreateFixedAssetInput!) {
  createFixedAsset(companyId: $companyId, input: $input) {
    id
    inventoryNumber
    name
  }
}

# Редактиране
mutation UpdateFixedAsset($input: UpdateFixedAssetInput!) {
  updateFixedAsset(input: $input) {
    id
    name
    status
  }
}

# Изтриване
mutation DeleteFixedAsset($id: Int!) {
  deleteFixedAsset(id: $id)
}
```

#### Амортизационни операции
```graphql
# Изчисляване на амортизации
mutation CalculateDepreciation($input: CalculateDepreciationInput!) {
  calculateDepreciation(input: $input) {
    success
    calculatedCount
    errorCount
    totalAccountingAmount
    totalTaxAmount
    errors
  }
}

# Приключване в дневника
mutation PostDepreciation($input: PostDepreciationInput!, $userId: Int!) {
  postDepreciation(input: $input, userId: $userId) {
    success
    journalEntryId
    totalAmount
    assetsCount
    message
  }
}
```

## Frontend Components

### 1. FixedAssets (главна страница)
Обединява всички табове:
- Активи
- Категории  
- Изчисляване амортизации
- Дневник амортизации

### 2. AssetsList
- Списък с всички активи
- Филтриране по статус и категория
- Създаване и редактиране на активи
- Изтриване на активи

### 3. DepreciationCalculation
- Избор на период за изчисляване
- Стартиране на амортизационни изчисления
- Преглед на резултатите
- Приключване в главния дневник

### 4. DepreciationJournal
- Преглед на всички амортизационни записи
- Филтриране по година и месец
- Статистика по периоди
- Индикатори за статус (приключен/неприключен)

## Счетоводни правила

### Счетоводни сметки
- **241** - Натрупана амортизация на ДМА (кредит)
- **603** - Разходи за амортизация (дебит)

### Журнален запис за амортизация
```
Dt 603 Разходи за амортизация     XXX лв.
    Ct 241 Натрупана амортизация      XXX лв.
```

### Временни разлики
Разликата между счетоводната и данъчната амортизация създава:
- **Отсрочен данъчен актив** (когато данъчната > счетоводната)  
- **Отсрочено данъчно задължение** (когато счетоводната > данъчната)

## Примери за използване

### 1. Създаване на актив и амортизация

```javascript
// 1. Създаване на актив
const asset = await graphqlRequest(`
  mutation CreateFixedAsset($companyId: Int!, $input: CreateFixedAssetInput!) {
    createFixedAsset(companyId: $companyId, input: $input) {
      id
      inventoryNumber
    }
  }
`, {
  companyId: 1,
  input: {
    inventoryNumber: "COMP-001",
    name: "Лаптоп Dell",
    categoryId: 4, // Computers
    acquisitionCost: 2500.00,
    acquisitionDate: "2025-01-01",
    accountingUsefulLife: 3,
    accountingDepreciationRate: 33.33,
    taxDepreciationRate: 50.0
  }
});

// 2. Изчисляване на амортизация за януари
const calculation = await graphqlRequest(`
  mutation CalculateDepreciation($input: CalculateDepreciationInput!) {
    calculateDepreciation(input: $input) {
      success
      calculatedCount
      totalAccountingAmount
      totalTaxAmount
    }
  }
`, {
  input: {
    companyId: 1,
    year: 2025,
    month: 1
  }
});

// 3. Приключване в дневника
const posting = await graphqlRequest(`
  mutation PostDepreciation($input: PostDepreciationInput!, $userId: Int!) {
    postDepreciation(input: $input, userId: $userId) {
      success
      journalEntryId
      totalAmount
    }
  }
`, {
  input: {
    companyId: 1,
    year: 2025,
    month: 1,
    reference: "АМ-2025-01"
  },
  userId: 1
});
```

### 2. Анализ на амортизационни разлики

```javascript
// Вземане на журнала с анализ
const journal = await graphqlRequest(`
  query GetDepreciationAnalysis($companyId: Int!, $year: Int!) {
    depreciationJournal(companyId: $companyId, year: $year) {
      id
      period
      accountingDepreciationAmount  
      taxDepreciationAmount
      fixedAssetId
    }
    
    fixedAssets(companyId: $companyId) {
      id
      name
      inventoryNumber
    }
  }
`, { companyId: 1, year: 2025 });

// Анализ на разликите
journal.depreciationJournal.forEach(entry => {
  const asset = journal.fixedAssets.find(a => a.id === entry.fixedAssetId);
  const difference = parseFloat(entry.taxDepreciationAmount) - 
                    parseFloat(entry.accountingDepreciationAmount);
                    
  console.log(`${asset.name}: разлика ${difference.toFixed(2)} лв.`);
});
```

### 3. Месечен амортизационен процес

```javascript
async function monthlyDepreciationProcess(companyId, year, month) {
  try {
    // 1. Изчисли амортизациите
    const calculation = await calculateDepreciation(companyId, year, month);
    
    if (!calculation.success) {
      console.error('Грешки при изчисление:', calculation.errors);
      return;
    }
    
    console.log(`Изчислени амортизации за ${calculation.calculatedCount} актива`);
    console.log(`Обща сума: ${calculation.totalAccountingAmount} лв.`);
    
    // 2. Приключи в дневника
    const posting = await postDepreciation(companyId, year, month, userId);
    
    if (posting.success) {
      console.log(`Създаден журнален запис #${posting.journalEntryId}`);
    }
    
  } catch (error) {
    console.error('Грешка в амортизационния процес:', error);
  }
}

// Използване
await monthlyDepreciationProcess(1, 2025, 2);
```

## Технически детайли

### База данни

**fixed_assets таблица:**
```sql
CREATE TABLE fixed_assets (
    id SERIAL PRIMARY KEY,
    inventory_number VARCHAR NOT NULL UNIQUE,
    name VARCHAR NOT NULL,
    category_id INTEGER REFERENCES fixed_asset_categories(id),
    company_id INTEGER REFERENCES companies(id),
    acquisition_cost DECIMAL(15,2) NOT NULL,
    acquisition_date DATE NOT NULL,
    put_into_service_date DATE,
    accounting_useful_life INTEGER NOT NULL,
    accounting_depreciation_rate DECIMAL(5,2) NOT NULL,
    accounting_depreciation_method VARCHAR DEFAULT 'straight_line',
    accounting_salvage_value DECIMAL(15,2) DEFAULT 0,
    accounting_book_value DECIMAL(15,2) NOT NULL,
    accounting_accumulated_depreciation DECIMAL(15,2) DEFAULT 0,
    tax_useful_life INTEGER,
    tax_depreciation_rate DECIMAL(5,2) NOT NULL,
    tax_book_value DECIMAL(15,2) NOT NULL,
    tax_accumulated_depreciation DECIMAL(15,2) DEFAULT 0,
    status VARCHAR DEFAULT 'active',
    -- metadata fields
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Валидации

- Инвентарният номер трябва да е уникален
- Придобивната стойност > 0
- Амортизационната норма между 0-100%
- Аналитичните активи трябва да имат родител
- Не може да се изтрият активи с амортизационна история

### Performance съображения

- Индекси на company_id, status, category_id
- Пагинация за списъците с активи
- Кеширане на категории и статистики
- Batch операции за амортизационни изчисления

## Заключение

Модулът за дълготрайни активи и амортизации предоставя пълна функционалност за управление на ДМА в съответствие с българското законодателство. Включва автоматизирани изчисления, подробно проследяване и интеграция с главния дневник.