# Производствен модул (Production Module)

## 📋 Общо описание

Модулът за производство позволява автоматизация на производствения счетоводен процес чрез технологични карти (рецепти) с многоетапно производство. Всеки етап автоматично създава счетоводни операции с дебит и кредит по определени сметки от сметкоплана.

## 🎯 Основни функционалности

### Технологични карти
- Дефиниране на производствени рецепти с множество етапи
- Всеки етап съдържа:
  - Дебитна сметка
  - Кредитна сметка
  - Формула за количество
  - Мерна единица
  - Формула за сума
  - Описание

### Производствени партиди
- Създаване на производствена партида по технологична карта
- Въвеждане на входящо количество
- Автоматично изчисление на етапите по формулите
- Проследяване на статус (чернова, в процес, завършена, отказана)
- **Автоматично създаване** на счетоводни операции за всеки завършен етап

### Справки
- Производствени партиди по период
- Производство по технологични карти
- Счетоводно отражение на производството

## 🏭 Производствен процес - Пример

### Технологична карта: Производство на хляб

**Етап 1:** Изписване на материали
- Дебит: 601 (Разходи за материали)
- Кредит: 301 (Суровини и материали)
- Количество: `{input_quantity}` кг
- Формула сума: `{input_quantity} * 2.50`

**Етап 2:** Производство - полуфабрикат
- Дебит: 611 (Незавършено производство)
- Кредит: 601 (Разходи за материали)
- Количество: `{input_quantity} * 0.95` кг (свиване)
- Формула сума: `{stage_1_amount}`

**Етап 3:** Готова продукция
- Дебит: 341 (Готова продукция)
- Кредит: 611 (Незавършено производство)
- Количество: `{stage_2_quantity}` кг
- Формула сума: `{stage_2_amount}`

## 📊 Database Schema

### Таблица: `technology_cards`
Съхранява технологични карти (рецепти).

```sql
CREATE TABLE technology_cards (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL REFERENCES companies(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    output_unit VARCHAR(50) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by INTEGER REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE
);
```

### Таблица: `technology_card_stages`
Етапи на технологичната карта.

```sql
CREATE TABLE technology_card_stages (
    id SERIAL PRIMARY KEY,
    technology_card_id INTEGER NOT NULL REFERENCES technology_cards(id),
    stage_number INTEGER NOT NULL,
    name VARCHAR(255) NOT NULL,
    debit_account_id INTEGER NOT NULL REFERENCES accounts(id),
    credit_account_id INTEGER NOT NULL REFERENCES accounts(id),
    quantity_formula TEXT,
    unit_of_measure VARCHAR(50),
    amount_formula TEXT,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(technology_card_id, stage_number)
);
```

### Таблица: `production_batches`
Производствени партиди.

```sql
CREATE TABLE production_batches (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL REFERENCES companies(id),
    technology_card_id INTEGER NOT NULL REFERENCES technology_cards(id),
    batch_number VARCHAR(100) NOT NULL,
    input_quantity NUMERIC(15, 4) NOT NULL,
    production_date DATE NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    notes TEXT,
    created_by INTEGER REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(company_id, batch_number)
);
```

**Статуси:**
- `draft` - Чернова
- `in_progress` - В процес
- `completed` - Завършена
- `cancelled` - Отказана

### Таблица: `production_batch_stages`
Етапи на производствената партида.

```sql
CREATE TABLE production_batch_stages (
    id SERIAL PRIMARY KEY,
    production_batch_id INTEGER NOT NULL REFERENCES production_batches(id),
    stage_number INTEGER NOT NULL,
    technology_card_stage_id INTEGER NOT NULL REFERENCES technology_card_stages(id),
    journal_entry_id INTEGER REFERENCES journal_entries(id),
    quantity NUMERIC(15, 4),
    amount NUMERIC(15, 2),
    unit_of_measure VARCHAR(50),
    completed_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    notes TEXT,
    UNIQUE(production_batch_id, stage_number)
);
```

**Статуси на етап:**
- `pending` - Предстои
- `completed` - Завършен
- `cancelled` - Отказан

## 🖥️ Frontend компоненти

### Navigation
Модулът е достъпен от главното меню под **"Производство"** с 3 таба:

1. **Технологични карти** (`/production/technology-cards`)
   - Списък с технологични карти
   - Създаване/редакция на карти
   - Дефиниране на етапи
   - Избор на дебитни/кредитни сметки

2. **Производствени партиди** (`/production/batches`)
   - Създаване на нова партида
   - Избор на технологична карта
   - Въвеждане на входящо количество и дата
   - Проследяване на статус

3. **Справки** (`/production/reports`)
   - Справка за производствени партиди по период
   - Производство по технологични карти
   - Счетоводни операции от производство

### Файлова структура
```
frontend/src/pages/production/
├── TechnologyCards.jsx      # Управление на технологични карти
├── ProductionBatches.jsx    # Управление на производствени партиди
└── ProductionReports.jsx    # Справки за производство
```

## 🔧 Backend имплементация (TODO)

### GraphQL Schema

```graphql
# Technology Cards
type TechnologyCard {
  id: ID!
  companyId: Int!
  name: String!
  description: String
  outputUnit: String!
  isActive: Boolean!
  stages: [TechnologyCardStage!]!
  createdAt: DateTime!
  updatedAt: DateTime!
}

type TechnologyCardStage {
  id: ID!
  technologyCardId: Int!
  stageNumber: Int!
  name: String!
  debitAccount: Account!
  creditAccount: Account!
  quantityFormula: String
  unitOfMeasure: String
  amountFormula: String
  description: String
}

# Production Batches
type ProductionBatch {
  id: ID!
  companyId: Int!
  technologyCard: TechnologyCard!
  batchNumber: String!
  inputQuantity: Decimal!
  productionDate: Date!
  status: ProductionBatchStatus!
  stages: [ProductionBatchStage!]!
  notes: String
  createdAt: DateTime!
  updatedAt: DateTime!
}

type ProductionBatchStage {
  id: ID!
  productionBatchId: Int!
  stageNumber: Int!
  technologyCardStage: TechnologyCardStage!
  journalEntry: JournalEntry
  quantity: Decimal
  amount: Decimal
  unitOfMeasure: String
  completedAt: DateTime
  status: StageStatus!
  notes: String
}

enum ProductionBatchStatus {
  DRAFT
  IN_PROGRESS
  COMPLETED
  CANCELLED
}

enum StageStatus {
  PENDING
  COMPLETED
  CANCELLED
}

# Mutations
type Mutation {
  createTechnologyCard(input: CreateTechnologyCardInput!): TechnologyCard!
  updateTechnologyCard(id: ID!, input: UpdateTechnologyCardInput!): TechnologyCard!
  deleteTechnologyCard(id: ID!): Boolean!

  createProductionBatch(input: CreateProductionBatchInput!): ProductionBatch!
  updateProductionBatch(id: ID!, input: UpdateProductionBatchInput!): ProductionBatch!
  completeProductionStage(batchId: ID!, stageNumber: Int!): ProductionBatchStage!
  cancelProductionBatch(id: ID!): ProductionBatch!
}

# Queries
type Query {
  technologyCards(companyId: Int!, isActive: Boolean): [TechnologyCard!]!
  technologyCard(id: ID!): TechnologyCard

  productionBatches(companyId: Int!, filters: ProductionBatchFilters): [ProductionBatch!]!
  productionBatch(id: ID!): ProductionBatch

  productionReport(companyId: Int!, filters: ProductionReportFilters!): ProductionReport!
}
```

### Backend задачи

#### 1. Rust Entities (SeaORM)
- [ ] `technology_card.rs` - Entity за технологични карти
- [ ] `technology_card_stage.rs` - Entity за етапи
- [ ] `production_batch.rs` - Entity за партиди
- [ ] `production_batch_stage.rs` - Entity за етапи на партиди

#### 2. GraphQL Resolvers
- [ ] `technology_card_resolver.rs` - CRUD операции за карти
- [ ] `production_batch_resolver.rs` - CRUD операции за партиди
- [ ] `production_reports_resolver.rs` - Справки

#### 3. Логика за формули
- [ ] Formula evaluator - Изчисление на формули от текст
- [ ] Променливи: `{input_quantity}`, `{stage_N_quantity}`, `{stage_N_amount}`
- [ ] Математически операции: `+`, `-`, `*`, `/`, `()`

#### 4. Автоматично създаване на операции
При завършване на етап (`completeProductionStage`):
- [ ] Изчисление на количество по формула
- [ ] Изчисление на сума по формула
- [ ] Създаване на `journal_entry` с:
  - Дебит от `debit_account_id`
  - Кредит от `credit_account_id`
  - Сума от изчислението
  - Описание с информация за партида и етап
- [ ] Запазване на `journal_entry_id` в `production_batch_stages`

#### 5. Валидации
- [ ] Проверка че сметките съществуват
- [ ] Проверка че формулите са валидни
- [ ] Проверка че етапите са в правилна последователност
- [ ] Проверка че партидата е активна преди завършване на етап

#### 6. Справки
- [ ] Агрегация на партиди по период
- [ ] Агрегация на производство по технологични карти
- [ ] Списък на автоматично създадени операции

## 📝 Пример workflow

### 1. Създаване на технологична карта
```graphql
mutation {
  createTechnologyCard(input: {
    companyId: 1
    name: "Производство на хляб"
    description: "Стандартна рецепта за бял хляб"
    outputUnit: "кг"
    stages: [
      {
        stageNumber: 1
        name: "Изписване на материали"
        debitAccountId: 601
        creditAccountId: 301
        quantityFormula: "{input_quantity}"
        unitOfMeasure: "кг"
        amountFormula: "{input_quantity} * 2.50"
      },
      {
        stageNumber: 2
        name: "Незавършено производство"
        debitAccountId: 611
        creditAccountId: 601
        quantityFormula: "{stage_1_quantity} * 0.95"
        unitOfMeasure: "кг"
        amountFormula: "{stage_1_amount}"
      },
      {
        stageNumber: 3
        name: "Готова продукция"
        debitAccountId: 341
        creditAccountId: 611
        quantityFormula: "{stage_2_quantity}"
        unitOfMeasure: "кг"
        amountFormula: "{stage_2_amount}"
      }
    ]
  }) {
    id
    name
    stages {
      id
      stageNumber
      name
    }
  }
}
```

### 2. Създаване на производствена партида
```graphql
mutation {
  createProductionBatch(input: {
    companyId: 1
    technologyCardId: 123
    inputQuantity: 100.00
    productionDate: "2025-11-08"
    notes: "Партида №001"
  }) {
    id
    batchNumber
    inputQuantity
    stages {
      stageNumber
      status
    }
  }
}
```

### 3. Завършване на етап
```graphql
mutation {
  completeProductionStage(
    batchId: 456
    stageNumber: 1
  ) {
    id
    quantity
    amount
    journalEntry {
      id
      entryNumber
      debitLine {
        account {
          code
          name
        }
        amount
      }
      creditLine {
        account {
          code
          name
        }
        amount
      }
    }
  }
}
```

Автоматично се създава операция:
- Дебит: 601 - 250.00 лв (100 кг * 2.50)
- Кредит: 301 - 250.00 лв
- Описание: "Производство - Партида №001 - Етап 1: Изписване на материали"

## 🚀 Deployment

### Database Migration
```bash
# Migration вече е приложена
psql -U postgres -d accounting -f migration/production_module.sql
```

### Frontend
```bash
# Компонентите са създадени в:
frontend/src/pages/production/
```

### Backend (TODO)
```bash
# След имплементация на GraphQL resolvers
cargo build --release
```

## 🔍 Troubleshooting

### Грешка: "Cannot find debit/credit account"
**Причина:** Избраните сметки не съществуват в сметкоплана
**Решение:** Проверете че сметките са създадени в Chart of Accounts

### Грешка: "Invalid formula syntax"
**Причина:** Формулата съдържа невалиден синтаксис
**Решение:** Използвайте само валидни променливи и оператори

### Партидата не създава операции
**Причина:** Етапът не е маркиран като "завършен"
**Решение:** Използвайте `completeProductionStage` mutation

## 📚 Свързани документи

- [Архитектура](./ARCHITECTURE.md) - Техническа архитектура
- [API Reference](./API.md) - GraphQL API
- [Счетоводни записи](../README.md#accounting) - Journal entries

## 📅 История на промените

### v1.0.0 - 2025-11-08
- ✅ Създадена database schema
- ✅ Създадени frontend компоненти
- ✅ Добавено меню в навигацията
- ⏳ Backend GraphQL API (TODO)
- ⏳ Formula evaluator (TODO)
- ⏳ Автоматично създаване на операции (TODO)
