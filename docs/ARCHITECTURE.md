# Архитектура на системата

RS-AC-BG е модерна българска счетоводна система с микросервисна архитектура.

## 🏗️ Общ преглед

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   React App     │    │   Rust Backend  │    │   PostgreSQL    │
│   (Frontend)    │◄──►│   (GraphQL)     │◄──►│   (Database)    │
│   Port 5174     │    │   Port 8080     │    │   Port 5432     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
    ┌─────────┐            ┌─────────────┐        ┌─────────────┐
    │ Vite    │            │ SeaORM      │        │ Migrations  │
    │ Bundler │            │ + GraphQL   │        │ + Indexes   │
    └─────────┘            └─────────────┘        └─────────────┘
```

## 🎯 Frontend архитектура

### Технологичен стек

- **React 19** - Latest stable версия
- **Vite** - Fast build tool и dev server  
- **Tailwind CSS** - Utility-first CSS framework
- **React Router** - Client-side routing

### Структура на проекта

```
frontend/src/
├── components/           # Reusable компоненти
│   ├── layout/          # Layout компоненти
│   │   ├── Layout.jsx   # Main layout wrapper
│   │   ├── Header.jsx   # Top navigation
│   │   ├── Sidebar.jsx  # Side navigation  
│   │   └── Breadcrumbs.jsx
│   └── imports/         # Import specific компоненти
│       ├── ControlisyImport.jsx
│       ├── BankImport.jsx
│       └── UniversalImport.jsx
├── pages/               # Route компоненти  
│   ├── Dashboard.jsx    # Главно табло
│   ├── VATEntry.jsx     # ДДС операции
│   ├── JournalEntry.jsx # Счетоводни записи
│   └── ImportCenter.jsx # Импорт център
└── assets/              # Static assets
```

### State Management

**Local State** - React useState/useEffect за:
- Form data
- UI состояния  
- Temporary data

**No Global State** (за момента):
- Данните се fetch-ват on demand
- GraphQL cache (при внедряване на Apollo)

### Routing Strategy

```javascript
// App.jsx
<Routes>
  <Route path="/" element={<Layout />}>
    <Route index element={<Navigate to="/dashboard" />} />
    <Route path="dashboard" element={<Dashboard />} />
    <Route path="accounting/entries" element={<JournalEntry />} />
    <Route path="accounting/vat-entry" element={<VATEntry />} />
    <Route path="imports" element={<ImportCenter />} />
    <Route path="vat/*" element={<VAT />} />
  </Route>
</Routes>
```

## ⚙️ Backend архитектура  

### Технологичен стек

- **Rust** - System programming language
- **Tokio** - Async runtime
- **SeaORM** - Modern async ORM
- **Async-GraphQL** - GraphQL server
- **PostgreSQL** - Primary database

### Структура на проекта

```
backend/src/
├── entities/            # Database models
│   ├── mod.rs          
│   ├── journal_entry.rs # Счетоводни записи
│   ├── account.rs       # Сметкоплан
│   ├── counterpart.rs   # Контрагенти  
│   └── vat_return.rs    # ДДС декларации
├── graphql/            # GraphQL layer
│   ├── mod.rs
│   ├── query.rs        # Query resolvers
│   ├── mutation.rs     # Mutation resolvers
│   └── accounting_resolvers.rs
├── services/           # Business logic
│   ├── mod.rs
│   └── bnb_service.rs  # БНБ валутни курсове
├── main.rs             # Application entry point
└── config.rs           # Configuration
```

### GraphQL Schema

```graphql
type Query {
  # Accounts
  accountHierarchy(companyId: Int!): [Account!]!
  
  # Journal entries  
  journalEntries(
    companyId: Int!
    filter: JournalEntryFilter
  ): [JournalEntry!]!
  
  # Counterparts
  counterparts(companyId: Int!): [Counterpart!]!
}

type Mutation {
  # Create journal entry with VAT codes
  createJournalEntry(
    input: CreateJournalEntryInput!
  ): JournalEntry!
  
  # Import operations
  createImportBatch(input: ImportBatchInput!): ImportBatch!
}

type JournalEntry {
  id: Int!
  entryNumber: String!
  documentDate: Date!
  vatDate: Date!
  accountingDate: Date!
  
  # Bulgarian VAT codes
  vatDocumentType: String
  vatPurchaseOperation: String  
  vatSalesOperation: String
  vatAdditionalOperation: String
  vatAdditionalData: String
}
```

### Database Layer

**SeaORM** предоставя:
- **Type-safe** database операции
- **Async/await** поддръжка  
- **Automatic migrations**
- **Connection pooling**

```rust
// entities/journal_entry.rs
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "journal_entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub entry_number: String,
    pub document_date: Date,
    pub vat_date: Date,
    pub accounting_date: Date,
    
    // Bulgarian VAT codes
    pub vat_document_type: Option<String>,
    pub vat_purchase_operation: Option<String>,
    pub vat_sales_operation: Option<String>,
    pub vat_additional_operation: Option<String>,
    pub vat_additional_data: Option<String>,
}
```

## 🗄️ Database архитектура

### PostgreSQL Schema

```sql
-- Core tables
CREATE TABLE companies (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  eik VARCHAR(13),
  vat_number VARCHAR(15)
);

CREATE TABLE accounts (
  id SERIAL PRIMARY KEY, 
  company_id INT REFERENCES companies(id),
  code VARCHAR(20) NOT NULL,
  name VARCHAR(255) NOT NULL,
  account_type VARCHAR(50),
  is_analytical BOOLEAN DEFAULT FALSE
);

CREATE TABLE journal_entries (
  id SERIAL PRIMARY KEY,
  company_id INT REFERENCES companies(id),
  entry_number VARCHAR NOT NULL,
  
  -- Bulgarian triple date system
  document_date DATE NOT NULL,
  vat_date DATE NOT NULL, 
  accounting_date DATE NOT NULL,
  
  -- Document info
  document_number VARCHAR,
  description TEXT,
  total_amount DECIMAL(15,2),
  total_vat_amount DECIMAL(15,2),
  
  -- Bulgarian VAT codes according to PPZDDS
  vat_document_type VARCHAR(5),
  vat_purchase_operation VARCHAR(5),
  vat_sales_operation VARCHAR(5), 
  vat_additional_operation VARCHAR(5),
  vat_additional_data VARCHAR(5),
  
  -- Audit
  created_by INT REFERENCES users(id),
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE entry_lines (
  id SERIAL PRIMARY KEY,
  journal_entry_id INT REFERENCES journal_entries(id),
  account_id INT REFERENCES accounts(id),
  counterpart_id INT REFERENCES counterparts(id),
  
  debit_amount DECIMAL(15,2) DEFAULT 0,
  credit_amount DECIMAL(15,2) DEFAULT 0,
  
  description TEXT,
  line_order INT
);
```

### Indexes за производительност

```sql  
-- Journal entries indexes
CREATE INDEX idx_journal_entries_company_date 
ON journal_entries(company_id, document_date);

CREATE INDEX idx_journal_entries_vat_codes
ON journal_entries(vat_document_type, vat_purchase_operation, vat_sales_operation);

-- Entry lines indexes  
CREATE INDEX idx_entry_lines_journal_id 
ON entry_lines(journal_entry_id);

CREATE INDEX idx_entry_lines_account_id
ON entry_lines(account_id);
```

## 🔄 Migrations система

### SeaORM Migration framework

```rust
// migration/src/lib.rs
pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000004_create_user_groups::Migration),
            Box::new(m20240101_000001_create_users_table::Migration),
            Box::new(m20240101_000003_create_companies::Migration),
            Box::new(m20240101_000005_create_accounts::Migration),
            Box::new(m20240101_000006_create_journal_entries::Migration),
            Box::new(m20240101_000007_create_entry_lines::Migration),
            Box::new(m20240101_000014_add_vat_codes::Migration), // Последна
        ]
    }
}
```

### Migration пример

```rust
// migration/src/m20240101_000014_add_vat_codes.rs
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(JournalEntries::Table)
                    .add_column(
                        ColumnDef::new(JournalEntries::VatDocumentType)
                            .string()
                            .null()
                            .comment("Bulgarian VAT document type (01-95)")
                    )
                    .add_column(
                        ColumnDef::new(JournalEntries::VatPurchaseOperation)
                            .string()
                            .null()
                            .comment("Purchase operation code (0-6)")
                    )
                    // ... други колони
                    .to_owned(),
            )
            .await
    }
}
```

## 🌐 API архитектура

### GraphQL предимства

- **Single endpoint** - `/graphql`
- **Type safety** - Schema-first approach
- **Flexible queries** - Client определя какво иска
- **Real-time** - Subscription поддръжка (бъдеще)
- **Tooling** - GraphiQL playground

### Request/Response flow

```
1. React компонент прави GraphQL заявка
2. Fetch към /graphql endpoint
3. GraphQL resolver обработва заявката  
4. SeaORM извиква данни от PostgreSQL
5. Резултатът се връща в JSON
6. React компонент обновява UI
```

### Пример заявка

```graphql
# Frontend заявка
mutation CreateVATEntry($input: CreateJournalEntryInput!) {
  createJournalEntry(input: $input) {
    id
    documentNumber
    vatDocumentType
    vatSalesOperation
    lines {
      account {
        code
        name
      }
      debitAmount
      creditAmount
    }
  }
}

# Variables
{
  "input": {
    "documentNumber": "INV-001",
    "documentDate": "2024-01-01",
    "vatDate": "2024-01-01", 
    "accountingDate": "2024-01-01",
    "vatDocumentType": "01",
    "vatSalesOperation": "1",
    "lines": [
      {
        "accountId": 1,
        "debitAmount": 100.00
      }
    ]
  }
}
```

## 🔐 Security архитектура

### Authentication (планирано)

- **JWT tokens** за session management
- **Role-based access** (Admin, Accountant, Viewer)
- **Company isolation** - users са ограничени до своята компания

### Authorization

```rust
// Context за GraphQL
pub struct GraphQLContext {
    pub user_id: Option<i32>,
    pub company_id: Option<i32>,
    pub db: DatabaseConnection,
}

// В resolver
async fn journal_entries(
    ctx: &Context<'_>,
    company_id: i32
) -> FieldResult<Vec<journal_entry::Model>> {
    let context = ctx.data::<GraphQLContext>()?;
    
    // Check user access to company
    if context.company_id != Some(company_id) {
        return Err("Unauthorized".into());
    }
    
    // Proceed with query...
}
```

### Data Protection

- **Input validation** на GraphQL ниво
- **SQL injection protection** чрез SeaORM
- **XSS protection** чрез React escaping
- **CORS configuration** за production

## 📦 Deployment архитектура

### Development

```bash
# Backend
cd backend && cargo run

# Frontend  
cd frontend && npm run dev

# Database
docker run -d postgres:15
```

### Production (планирано)

```yaml
# docker-compose.yml
version: '3.8'
services:
  backend:
    build: ./backend
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://user:pass@db:5432/rsacbg
    depends_on:
      - db
      
  frontend:
    build: ./frontend
    ports:
      - "80:80"
    depends_on:
      - backend
      
  db:
    image: postgres:15
    environment:
      - POSTGRES_DB=rsacbg
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=pass
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

## 🔧 Configuration

### Environment Variables

```bash
# Backend (.env)
DATABASE_URL=postgresql://username:password@localhost:5432/rsacbg
RUST_LOG=debug
BNB_API_URL=https://www.bnb.bg/Statistics/StExternalSector/StExchangeRates/StERForeignCurrencies/

# Frontend (.env)
VITE_API_URL=http://localhost:8080
VITE_APP_TITLE=RS-AC-BG
```

### Rust конфигурация

```rust
// src/config.rs
#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub bnb_api_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            server_host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("PORT")?.parse()?,
            bnb_api_url: env::var("BNB_API_URL")?,
        })
    }
}
```

## 📊 Monitoring и Logging

### Rust logging

```rust
// Cargo.toml
tracing = "0.1"
tracing-subscriber = "0.3"

// src/main.rs
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::init();
    
    info!("Starting RS-AC-BG server");
    
    // Application logic
    
    info!("Server running on http://{}:{}", config.host, config.port);
    Ok(())
}
```

### Error handling

```rust
// Custom error types
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Business logic error: {0}")]
    Business(String),
}

// GraphQL error mapping
impl From<AppError> for async_graphql::Error {
    fn from(err: AppError) -> Self {
        async_graphql::Error::new(err.to_string())
    }
}
```

## 🚀 Performance

### Database оптимизации

- **Connection pooling** чрез SeaORM
- **Prepared statements** автоматично  
- **Selective queries** - само нужните полета
- **Pagination** за големи datasets

### Frontend оптимизации

- **Code splitting** чрез Vite
- **Lazy loading** на компонентите
- **Memoization** с React.memo
- **Virtual scrolling** за големи списъци

### Caching strategy

```rust
// Backend caching (планирано)
use moka::future::Cache;

pub struct CacheService {
    accounts_cache: Cache<i32, Vec<account::Model>>,
}

// Frontend caching
// При внедряване на Apollo Client
const GET_ACCOUNTS = gql`
  query GetAccounts($companyId: Int!) {
    accountHierarchy(companyId: $companyId) @cached(ttl: 300) {
      id
      code  
      name
    }
  }
`;
```

## 📈 Scalability планове

### Horizontal scaling

- **Load balancer** пред множество backend инстанции
- **Database read replicas** за queries
- **CDN** за static assets

### Microservices разделяне

```
Current: Monolith
┌─────────────────────────────────┐
│       RS-AC-BG Backend          │
│ ┌─────────┬─────────┬─────────┐ │
│ │ Accounting │ VAT │ Imports │ │  
│ └─────────┴─────────┴─────────┘ │
└─────────────────────────────────┘

Future: Microservices  
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│ Accounting  │  │    VAT      │  │   Imports   │
│  Service    │  │   Service   │  │   Service   │
└─────────────┘  └─────────────┘  └─────────────┘
       │                 │                 │
       └─────────────────┼─────────────────┘
                         │
              ┌─────────────────┐
              │   API Gateway   │
              └─────────────────┘
```

## 🧪 Testing стратегия

### Backend тестове

```rust
// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_journal_entry() {
        let db = setup_test_db().await;
        let input = CreateJournalEntryInput { /* ... */ };
        
        let result = create_journal_entry(&db, input).await;
        
        assert!(result.is_ok());
    }
}

// Integration tests
#[tokio::test]  
async fn test_graphql_mutation() {
    let schema = build_schema().await;
    let query = r#"
        mutation {
            createJournalEntry(input: { /* ... */ }) {
                id
            }
        }
    "#;
    
    let result = schema.execute(query).await;
    assert!(result.errors.is_empty());
}
```

### Frontend тестове

```javascript
// Unit tests с Jest
import { render, screen } from '@testing-library/react';
import VATEntry from './VATEntry';

test('renders VAT entry form', () => {
  render(<VATEntry />);
  expect(screen.getByText('Нова ДДС операция')).toBeInTheDocument();
});

// E2E tests с Playwright
test('create VAT entry flow', async ({ page }) => {
  await page.goto('/accounting/vat-entry');
  await page.fill('[data-testid=document-number]', 'INV-001');  
  await page.click('[data-testid=save-button]');
  await expect(page.locator('[data-testid=success-message]')).toBeVisible();
});
```

## 🔮 Бъдещи разширения

### Planned features

- [ ] **Real-time collaboration** с WebSockets
- [ ] **Mobile приложение** за контролери  
- [ ] **AI асистент** за счетоводни въпроси
- [ ] **Blockchain интеграция** за audit trail
- [ ] **Multi-tenant SaaS** архитектура
- [ ] **Advanced analytics** с machine learning

### Technical debt

- [ ] Migrate to **Apollo Client** за по-добър caching
- [ ] Въведи **comprehensive testing** 
- [ ] Setup **CI/CD pipeline**
- [ ] Implement **proper authentication**
- [ ] Add **API rate limiting**
- [ ] Setup **monitoring и alerting**