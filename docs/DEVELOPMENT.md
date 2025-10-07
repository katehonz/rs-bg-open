# Development Guide

Ръководство за разработчици на RS-AC-BG системата.

## 🚀 Quick Start

### 1. Setup Development Environment

```bash
git clone https://github.com/your-org/rs-ac-bg.git
cd rs-ac-bg

# Setup database
createdb rsacbg
echo "DATABASE_URL=postgresql://localhost:5432/rsacbg" > .env

# Run migrations
cd migration && cargo run && cd ..

# Start backend
cd backend && cargo run &

# Start frontend
cd frontend && npm install && npm run dev
```

### 2. Access Points

- **Frontend**: http://localhost:5173
- **GraphQL Playground**: http://localhost:8080/graphql
- **Database**: postgresql://localhost:5432/rsacbg

## 🏗️ Project Structure

```
rs-ac-bg/
├── backend/           # Rust backend
│   ├── src/
│   │   ├── main.rs           # Application entry
│   │   ├── config.rs         # Configuration
│   │   ├── entities/         # Database models
│   │   ├── graphql/          # GraphQL resolvers
│   │   └── services/         # Business logic
│   ├── Cargo.toml           # Dependencies
│   └── .env                 # Environment variables
├── frontend/          # React frontend  
│   ├── src/
│   │   ├── components/       # React components
│   │   ├── pages/           # Route components
│   │   ├── assets/          # Static assets
│   │   └── App.jsx          # Main app
│   ├── package.json         # Dependencies
│   └── vite.config.js       # Build configuration
├── migration/         # Database migrations
│   ├── src/
│   │   ├── lib.rs           # Migration definitions
│   │   ├── main.rs          # Migration runner
│   │   └── m2024*.rs        # Individual migrations
│   └── Cargo.toml
├── docs/              # Documentation
└── README.md
```

## 🦀 Backend Development

### Tech Stack

- **Rust 1.70+**
- **SeaORM** - Database ORM
- **Async-GraphQL** - GraphQL server
- **Tokio** - Async runtime
- **Axum** - Web framework

### Development Workflow

1. **Start with hot reload**:
```bash
cd backend
cargo install cargo-watch
cargo watch -x run
```

2. **Add new entity**:
```bash
# Create entity file
touch src/entities/new_entity.rs

# Add to entities/mod.rs
pub mod new_entity;
pub use new_entity::Entity as NewEntity;

# Create migration
cd ../migration/src
touch m20240101_000015_create_new_entity.rs
```

3. **Entity Example**:
```rust
// src/entities/new_entity.rs
use sea_orm::entity::prelude::*;
use async_graphql::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, SimpleObject)]
#[sea_orm(table_name = "new_entities")]
#[graphql(concrete(name = "NewEntity", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

4. **GraphQL Resolver**:
```rust
// src/graphql/mutation.rs
use crate::entities::{new_entity, prelude::*};

#[derive(InputObject)]
pub struct CreateNewEntityInput {
    name: String,
}

impl MutationRoot {
    async fn create_new_entity(
        &self,
        ctx: &Context<'_>,
        input: CreateNewEntityInput,
    ) -> FieldResult<new_entity::Model> {
        let db = ctx.data::<DatabaseConnection>()?;
        
        let new_entity = new_entity::ActiveModel {
            name: Set(input.name),
            created_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };
        
        let result = NewEntity::insert(new_entity).exec(db).await?;
        let created = NewEntity::find_by_id(result.last_insert_id)
            .one(db)
            .await?
            .ok_or("Failed to create entity")?;
            
        Ok(created)
    }
}
```

### Database Development

1. **Create new migration**:
```rust
// migration/src/m20240101_000015_example.rs
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NewEntity::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NewEntity::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NewEntity::Name).string().not_null())
                    .col(
                        ColumnDef::new(NewEntity::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NewEntity::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum NewEntity {
    Table,
    Id,
    Name,
    CreatedAt,
}
```

2. **Add to migration list**:
```rust
// migration/src/lib.rs
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // ... existing migrations
            Box::new(m20240101_000015_example::Migration),
        ]
    }
}
```

### Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# With logging
RUST_LOG=debug cargo test -- --nocapture
```

### Code Standards

```rust
// Use Result<T, E> for error handling
async fn create_entry() -> Result<Model, AppError> {
    // Implementation
}

// Use proper error types
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    
    #[error("Validation error: {0}")]
    Validation(String),
}

// Document public APIs
/// Creates a new journal entry with validation
/// 
/// # Arguments
/// * `input` - The entry data
/// 
/// # Errors
/// Returns `AppError::Validation` if entry is not balanced
pub async fn create_journal_entry(input: CreateEntryInput) -> Result<Model, AppError> {
    // Implementation
}
```

## ⚛️ Frontend Development

### Tech Stack

- **React 19**
- **Vite** - Build tool
- **Tailwind CSS** - Styling
- **React Router** - Routing

### Development Workflow

1. **Start with hot reload**:
```bash
cd frontend
npm run dev
```

2. **Create new component**:
```jsx
// src/components/NewComponent.jsx
import React, { useState, useEffect } from 'react';

export default function NewComponent({ prop1, prop2 }) {
  const [state, setState] = useState('');
  
  useEffect(() => {
    // Effect logic
  }, []);
  
  return (
    <div className="p-4">
      <h1 className="text-2xl font-bold">New Component</h1>
      {/* Component content */}
    </div>
  );
}
```

3. **Add new page**:
```jsx
// src/pages/NewPage.jsx
import React from 'react';
import NewComponent from '../components/NewComponent';

export default function NewPage() {
  return (
    <div className="container mx-auto px-4">
      <NewComponent prop1="value1" prop2="value2" />
    </div>
  );
}
```

4. **Update routing**:
```jsx
// src/App.jsx
import NewPage from './pages/NewPage';

// In Routes component
<Route path="new-page" element={<NewPage />} />
```

### GraphQL Integration

```jsx
// GraphQL query example
async function fetchJournalEntries(companyId) {
  const query = `
    query GetJournalEntries($companyId: Int!) {
      journalEntries(companyId: $companyId) {
        id
        entryNumber
        documentDate
        totalAmount
        lines {
          id
          debitAmount
          creditAmount
          account {
            code
            name
          }
        }
      }
    }
  `;
  
  const response = await fetch('/graphql', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      query,
      variables: { companyId }
    }),
  });
  
  const data = await response.json();
  return data.data.journalEntries;
}

// Use in component
const [entries, setEntries] = useState([]);

useEffect(() => {
  fetchJournalEntries(1).then(setEntries);
}, []);
```

### State Management Patterns

```jsx
// Local state for form
const [formData, setFormData] = useState({
  documentNumber: '',
  documentDate: '',
  description: '',
  lines: []
});

// Update pattern
const updateFormData = (field, value) => {
  setFormData(prev => ({
    ...prev,
    [field]: value
  }));
};

// Array state updates
const addLine = () => {
  setFormData(prev => ({
    ...prev,
    lines: [...prev.lines, { 
      accountId: '', 
      debitAmount: 0, 
      creditAmount: 0 
    }]
  }));
};

const updateLine = (index, field, value) => {
  setFormData(prev => ({
    ...prev,
    lines: prev.lines.map((line, i) => 
      i === index ? { ...line, [field]: value } : line
    )
  }));
};
```

### Styling Guidelines

```jsx
// Use Tailwind utility classes
<div className="bg-white rounded-lg shadow-md p-6 mb-4">
  <h2 className="text-xl font-semibold mb-4 text-gray-800">
    Title
  </h2>
  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
    <input 
      className="border border-gray-300 rounded-md px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
      placeholder="Enter value"
    />
  </div>
</div>

// Conditional styling
<button 
  className={`
    px-4 py-2 rounded-md font-medium transition-colors
    ${isValid 
      ? 'bg-blue-500 text-white hover:bg-blue-600' 
      : 'bg-gray-300 text-gray-500 cursor-not-allowed'
    }
  `}
  disabled={!isValid}
>
  Submit
</button>
```

### Testing

```bash
# Run tests
npm test

# Run tests in watch mode
npm run test:watch

# Run tests with coverage
npm run test:coverage
```

### Component Testing Example

```jsx
// src/components/__tests__/NewComponent.test.jsx
import { render, screen, fireEvent } from '@testing-library/react';
import NewComponent from '../NewComponent';

describe('NewComponent', () => {
  it('renders with correct title', () => {
    render(<NewComponent prop1="test" />);
    expect(screen.getByText('New Component')).toBeInTheDocument();
  });
  
  it('handles user interaction', async () => {
    render(<NewComponent prop1="test" />);
    
    const button = screen.getByRole('button', { name: /submit/i });
    fireEvent.click(button);
    
    await screen.findByText('Success message');
    expect(screen.getByText('Success message')).toBeInTheDocument();
  });
});
```

## 📊 Database Development

### Migration Best Practices

```rust
// Always use transactions for multiple operations
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let txn = manager.get_connection().begin().await?;
    
    // Create table
    manager
        .create_table(/* ... */)
        .await?;
        
    // Add indexes
    manager
        .create_index(/* ... */)
        .await?;
        
    txn.commit().await?;
    Ok(())
}

// Always provide proper down migration
async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
        .drop_table(Table::drop().table(MyTable::Table).to_owned())
        .await
}
```

### Seeding Data

```rust
// Create seed data in migration
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let db = manager.get_connection();
    
    // Insert default accounts
    let accounts = vec![
        account::ActiveModel {
            code: Set("1".to_string()),
            name: Set("АКТИВИ".to_string()),
            account_type: Set("ASSETS".to_string()),
            ..Default::default()
        },
        // ... more accounts
    ];
    
    Account::insert_many(accounts).exec(db).await?;
    Ok(())
}
```

## 🔧 Tools & Utilities

### Development Scripts

```bash
# In package.json scripts section
{
  "scripts": {
    "dev": "concurrently \"npm:dev:backend\" \"npm:dev:frontend\"",
    "dev:backend": "cd backend && cargo watch -x run",
    "dev:frontend": "cd frontend && npm run dev",
    "test": "npm run test:backend && npm run test:frontend",
    "test:backend": "cd backend && cargo test",
    "test:frontend": "cd frontend && npm test",
    "migrate": "cd migration && cargo run",
    "reset-db": "dropdb rsacbg && createdb rsacbg && npm run migrate"
  }
}
```

### Useful CLI Commands

```bash
# Database operations
psql -d rsacbg -c "SELECT * FROM journal_entries LIMIT 5;"
psql -d rsacbg -c "TRUNCATE TABLE journal_entries CASCADE;"

# Code formatting
cd backend && cargo fmt
cd frontend && npm run format

# Dependency updates
cd backend && cargo update
cd frontend && npm update

# Build optimization
cd backend && cargo build --release
cd frontend && npm run build

# Bundle analysis
cd frontend && npm run analyze
```

## 🐛 Debugging

### Backend Debugging

```rust
// Add debug prints
dbg!(&variable);
println!("Debug: {:?}", variable);

// Use tracing for structured logging
use tracing::{info, warn, error, debug};

debug!("Processing entry with ID: {}", entry_id);
info!("Entry created successfully: {}", entry.id);
warn!("Entry not balanced: debit={}, credit={}", debit, credit);
error!("Failed to create entry: {}", error);
```

### Frontend Debugging

```jsx
// Console debugging
console.log('State:', state);
console.warn('Validation error:', error);
console.error('API call failed:', error);

// React DevTools debugging
const Component = () => {
  const [state, setState] = useState('');
  
  // Debug effect
  useEffect(() => {
    console.log('Effect triggered with state:', state);
  }, [state]);
  
  return <div>Component</div>;
};

// Network debugging
const apiCall = async () => {
  try {
    const response = await fetch('/graphql', {
      method: 'POST',
      body: JSON.stringify({ query })
    });
    
    console.log('Response status:', response.status);
    const data = await response.json();
    console.log('Response data:', data);
    
    return data;
  } catch (error) {
    console.error('API Error:', error);
    throw error;
  }
};
```

### Database Debugging

```sql
-- Check recent entries
SELECT * FROM journal_entries ORDER BY created_at DESC LIMIT 10;

-- Check balances
SELECT 
  id,
  entry_number,
  (SELECT SUM(debit_amount) FROM entry_lines WHERE journal_entry_id = je.id) as total_debit,
  (SELECT SUM(credit_amount) FROM entry_lines WHERE journal_entry_id = je.id) as total_credit
FROM journal_entries je;

-- Check VAT codes
SELECT vat_document_type, COUNT(*) 
FROM journal_entries 
WHERE vat_document_type IS NOT NULL 
GROUP BY vat_document_type;
```

## 📋 Code Review Guidelines

### Backend Review Checklist

- [ ] Error handling with proper Result types
- [ ] Database queries are efficient (avoid N+1)
- [ ] Input validation on GraphQL resolvers  
- [ ] Proper logging with tracing
- [ ] Tests cover happy path and error cases
- [ ] Database transactions where needed
- [ ] No hardcoded values, use config

### Frontend Review Checklist

- [ ] Components are properly decomposed
- [ ] State updates are immutable
- [ ] Effects have proper dependencies
- [ ] Error states are handled
- [ ] Loading states are shown
- [ ] Accessibility attributes (aria-*)
- [ ] Responsive design with Tailwind
- [ ] No console.log in production code

## 🚀 Deployment

### Development Deployment

```bash
# Quick deployment script
#!/bin/bash
set -e

echo "Building backend..."
cd backend && cargo build --release && cd ..

echo "Building frontend..."
cd frontend && npm run build && cd ..

echo "Running migrations..."
cd migration && cargo run && cd ..

echo "Starting services..."
systemctl restart rsacbg-backend
systemctl restart nginx

echo "Deployment complete!"
```

### Production Checklist

- [ ] Environment variables configured
- [ ] Database backups scheduled
- [ ] SSL certificates installed
- [ ] Firewall rules configured
- [ ] Monitoring and logging setup
- [ ] Health checks implemented
- [ ] Error tracking (Sentry) configured

## 📚 Resources

### Documentation

- [Rust Book](https://doc.rust-lang.org/book/)
- [SeaORM Documentation](https://sea-ql.org/SeaORM/)
- [Async-GraphQL](https://async-graphql.github.io/async-graphql/)
- [React Documentation](https://react.dev/)
- [Tailwind CSS](https://tailwindcss.com/)

### Bulgarian Accounting

- [ППЗДДС](https://nap.bg) - НАП официални правила
- [БНБ API](https://bnb.bg) - Валутни курсове
- [БСС](https://www.minfin.bg) - Български счетоводни стандарти

### Community

- [Rust Discord](https://discord.gg/rust-lang)
- [React Community](https://react.dev/community)
- [PostgreSQL Slack](https://postgres-slack.herokuapp.com/)

## 🤝 Contributing

1. Fork the repository
2. Create feature branch: `git checkout -b feature/new-feature`
3. Make changes with proper tests
4. Run quality checks: `cargo fmt && cargo clippy`
5. Commit with descriptive message
6. Push branch and create Pull Request
7. Address review feedback

### Commit Message Format

```
feat: add VAT code validation for journal entries

- Implement PPZDDS code validation
- Add error messages in Bulgarian
- Update tests for new validation rules

Fixes #123
```