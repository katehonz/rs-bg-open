# API Reference

GraphQL API документация за RS-AC-BG системата.

## 🌐 Endpoint

```
POST /graphql
Content-Type: application/json
```

## 📊 Schema Overview

### Query Types
- `accountHierarchy` - Йерархия на сметки
- `journalEntries` - Счетоводни записи
- `counterparts` - Контрагенти
- `importBatches` - Импорт батчове
- `vatReturns` - ДДС декларации

### Mutation Types
- `createJournalEntry` - Създаване на счетоводен запис
- `createImportBatch` - Създаване на импорт батч
- `createVATReturn` - Създаване на ДДС декларация

## 📋 Type Definitions

### JournalEntry

```graphql
type JournalEntry {
  id: Int!
  companyId: Int!
  entryNumber: String!
  
  # Bulgarian triple date system
  documentDate: Date!
  vatDate: Date!
  accountingDate: Date!
  
  # Document info
  documentNumber: String
  description: String
  totalAmount: Decimal!
  totalVatAmount: Decimal!
  
  # Bulgarian VAT codes according to PPZDDS
  vatDocumentType: String
  vatPurchaseOperation: String
  vatSalesOperation: String
  vatAdditionalOperation: String
  vatAdditionalData: String
  
  # Relations
  lines: [EntryLine!]!
  createdBy: User
  
  # Audit fields
  createdAt: DateTime!
  updatedAt: DateTime!
}
```

### EntryLine

```graphql
type EntryLine {
  id: Int!
  journalEntryId: Int!
  accountId: Int!
  counterpartId: Int
  
  debitAmount: Decimal!
  creditAmount: Decimal!
  description: String
  lineOrder: Int!
  
  # Relations  
  journalEntry: JournalEntry!
  account: Account!
  counterpart: Counterpart
}
```

### Account

```graphql
type Account {
  id: Int!
  companyId: Int!
  code: String!
  name: String!
  accountType: AccountType!
  parentId: Int
  isAnalytical: Boolean!
  
  # Relations
  parent: Account
  children: [Account!]!
  entries: [EntryLine!]!
}

enum AccountType {
  ASSETS
  LIABILITIES  
  EQUITY
  REVENUE
  EXPENSES
}
```

### Counterpart

```graphql
type Counterpart {
  id: Int!
  companyId: Int!
  name: String!
  eik: String
  vatNumber: String
  address: String
  city: String
  country: String!
  isVatRegistered: Boolean!
  
  # Relations
  entries: [EntryLine!]!
}
```

### ImportBatch

```graphql
type ImportBatch {
  id: String!
  batchId: String!
  companyId: Int!
  source: ImportSource!
  status: ImportStatus!
  
  # Stats
  documentsCount: Int!
  totalAmount: Decimal!
  
  # Relations
  documents: [ImportDocument!]!
  
  createdAt: DateTime!
  updatedAt: DateTime!
}

enum ImportSource {
  CONTROLISY
  BANK
  UNIVERSAL
}

enum ImportStatus {
  DRAFT
  VALIDATED
  IMPORTED
  FAILED
}
```

## 🔍 Query Examples

### Get Account Hierarchy

```graphql
query GetAccounts($companyId: Int!) {
  accountHierarchy(companyId: $companyId) {
    id
    code
    name
    accountType
    isAnalytical
    children {
      id
      code
      name
      accountType
      isAnalytical
    }
  }
}
```

**Variables:**
```json
{
  "companyId": 1
}
```

**Response:**
```json
{
  "data": {
    "accountHierarchy": [
      {
        "id": 1,
        "code": "1",
        "name": "АКТИВИ",
        "accountType": "ASSETS",
        "isAnalytical": false,
        "children": [
          {
            "id": 10,
            "code": "10",
            "name": "Дълготрайни активи",
            "accountType": "ASSETS",
            "isAnalytical": false
          }
        ]
      }
    ]
  }
}
```

### Get Journal Entries with Filters

```graphql
query GetJournalEntries(
  $companyId: Int!,
  $filter: JournalEntryFilter
) {
  journalEntries(companyId: $companyId, filter: $filter) {
    id
    entryNumber
    documentDate
    vatDate
    accountingDate
    documentNumber
    description
    totalAmount
    totalVatAmount
    
    # VAT codes
    vatDocumentType
    vatPurchaseOperation
    vatSalesOperation
    
    lines {
      id
      debitAmount
      creditAmount
      description
      account {
        code
        name
      }
      counterpart {
        name
        eik
      }
    }
  }
}
```

**Filter Input:**
```graphql
input JournalEntryFilter {
  dateFrom: Date
  dateTo: Date
  documentNumber: String
  accountCodes: [String!]
  counterpartIds: [Int!]
  vatDocumentTypes: [String!]
  minAmount: Decimal
  maxAmount: Decimal
}
```

### Get Import Batches

```graphql
query GetImportBatches($companyId: Int!) {
  importBatches(companyId: $companyId) {
    id
    batchId
    source
    status
    documentsCount
    totalAmount
    createdAt
    
    documents {
      id
      documentNumber
      documentDate
      totalAmount
      isBalanced
    }
  }
}
```

## ✏️ Mutation Examples

### Create Journal Entry

```graphql
mutation CreateJournalEntry($input: CreateJournalEntryInput!) {
  createJournalEntry(input: $input) {
    id
    entryNumber
    documentNumber
    totalAmount
    totalVatAmount
    
    # VAT codes
    vatDocumentType
    vatPurchaseOperation
    vatSalesOperation
    
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
```

**Input:**
```graphql
input CreateJournalEntryInput {
  companyId: Int!
  entryNumber: String!
  
  # Bulgarian triple dates
  documentDate: Date!
  vatDate: Date!
  accountingDate: Date!
  
  # Document info
  documentNumber: String
  description: String
  
  # Bulgarian VAT codes
  vatDocumentType: String
  vatPurchaseOperation: String
  vatSalesOperation: String
  vatAdditionalOperation: String
  vatAdditionalData: String
  
  # Entry lines
  lines: [CreateEntryLineInput!]!
}

input CreateEntryLineInput {
  accountId: Int!
  counterpartId: Int
  debitAmount: Decimal!
  creditAmount: Decimal!
  description: String
  lineOrder: Int!
}
```

**Variables:**
```json
{
  "input": {
    "companyId": 1,
    "entryNumber": "001/2024",
    "documentDate": "2024-01-01",
    "vatDate": "2024-01-01",
    "accountingDate": "2024-01-01",
    "documentNumber": "INV-001",
    "description": "Покупка стоки",
    "vatDocumentType": "01",
    "vatPurchaseOperation": "1",
    "lines": [
      {
        "accountId": 601,
        "counterpartId": 1,
        "debitAmount": 120.00,
        "creditAmount": 0.00,
        "description": "Стоки",
        "lineOrder": 1
      },
      {
        "accountId": 401,
        "counterpartId": 1,
        "debitAmount": 0.00,
        "creditAmount": 120.00,
        "description": "Задължение към доставчик",
        "lineOrder": 2
      }
    ]
  }
}
```

### Create Import Batch

```graphql
mutation CreateImportBatch($input: CreateImportBatchInput!) {
  createImportBatch(input: $input) {
    id
    batchId
    source
    status
    documentsCount
    totalAmount
  }
}
```

**Input:**
```graphql
input CreateImportBatchInput {
  companyId: Int!
  source: ImportSource!
  fileContent: String!
  fileName: String!
  options: ImportOptionsInput
}

input ImportOptionsInput {
  # Controlisy options
  ignoreAnalyticalItems: Boolean
  groupByAccount: Boolean
  validateVATCodes: Boolean
  
  # Bank options
  encoding: String
  dateFormat: String
  delimiter: String
  
  # Universal options
  templateId: String
  columnMapping: JSON
}
```

## 🔍 Subscription Examples (Planned)

### Real-time Import Progress

```graphql
subscription ImportProgress($batchId: String!) {
  importProgress(batchId: $batchId) {
    batchId
    status
    processedDocuments
    totalDocuments
    currentDocument {
      documentNumber
      status
      errors
    }
  }
}
```

### Live Journal Entry Updates

```graphql
subscription JournalEntryUpdates($companyId: Int!) {
  journalEntryUpdates(companyId: $companyId) {
    mutation # CREATED, UPDATED, DELETED
    entry {
      id
      entryNumber
      documentNumber
      totalAmount
    }
  }
}
```

## 🚨 Error Handling

### Standard Error Format

```json
{
  "errors": [
    {
      "message": "Journal entry is not balanced",
      "extensions": {
        "code": "VALIDATION_ERROR",
        "field": "lines",
        "details": {
          "debitTotal": 100.00,
          "creditTotal": 120.00,
          "difference": -20.00
        }
      }
    }
  ]
}
```

### Common Error Codes

- `VALIDATION_ERROR` - Валидационна грешка
- `NOT_FOUND` - Записът не е намерен
- `UNAUTHORIZED` - Няма права за достъп
- `BUSINESS_LOGIC_ERROR` - Бизнес логика грешка
- `DATABASE_ERROR` - Грешка в базата данни

### VAT Validation Errors

```json
{
  "errors": [
    {
      "message": "Invalid VAT document type",
      "extensions": {
        "code": "VAT_VALIDATION_ERROR",
        "field": "vatDocumentType",
        "details": {
          "provided": "99",
          "allowedValues": ["01", "02", "03", "..."]
        }
      }
    }
  ]
}
```

### Balance Validation

```json
{
  "errors": [
    {
      "message": "Entry lines are not balanced",
      "extensions": {
        "code": "BALANCE_ERROR", 
        "details": {
          "debitTotal": "100.00",
          "creditTotal": "95.00",
          "difference": "5.00"
        }
      }
    }
  ]
}
```

## 🔐 Authentication (Planned)

### JWT Authorization Header

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Context Information

```graphql
type User {
  id: Int!
  username: String!
  email: String!
  role: UserRole!
  companyId: Int!
  permissions: [Permission!]!
}

enum UserRole {
  ADMIN
  ACCOUNTANT
  VIEWER
}
```

## 📊 Pagination

### Cursor-based Pagination

```graphql
type JournalEntryConnection {
  edges: [JournalEntryEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type JournalEntryEdge {
  node: JournalEntry!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}
```

### Usage

```graphql
query GetJournalEntries(
  $companyId: Int!,
  $first: Int,
  $after: String
) {
  journalEntries(
    companyId: $companyId,
    first: $first,
    after: $after
  ) {
    edges {
      node {
        id
        entryNumber
        documentDate
        totalAmount
      }
      cursor
    }
    pageInfo {
      hasNextPage
      endCursor
    }
    totalCount
  }
}
```

## 📈 Performance Tips

### Efficient Queries

**Good:**
```graphql
query GetJournalEntriesSummary($companyId: Int!) {
  journalEntries(companyId: $companyId) {
    id
    entryNumber
    totalAmount
    # Only essential fields
  }
}
```

**Avoid:**
```graphql
query GetAllData($companyId: Int!) {
  journalEntries(companyId: $companyId) {
    id
    # ... all fields
    lines {
      # ... all line fields
      account {
        # ... all account fields
        children {
          # ... nested children - expensive!
        }
      }
    }
  }
}
```

### Use Fragments

```graphql
fragment JournalEntryBasic on JournalEntry {
  id
  entryNumber
  documentDate
  totalAmount
  documentNumber
}

query GetEntries($companyId: Int!) {
  journalEntries(companyId: $companyId) {
    ...JournalEntryBasic
  }
}
```

## 🧪 Testing

### GraphQL Playground

Development: `http://localhost:8080/graphql`

### Example Test Query

```graphql
{
  accountHierarchy(companyId: 1) {
    code
    name
    children {
      code
      name
    }
  }
}
```