# API Structure - RS-AC-BG

## REST API - За XML/JSON файлови операции
**Base URL:** `/api/controlisy`

### Endpoints:

#### 1. Parse XML File (без импорт)
```
POST /api/controlisy/parse
Content-Type: application/json

Body:
{
    "file_name": "prodajbi-07.2025_02-Продажби.xml",
    "xml_content": "<xml>...</xml>"
}

Response:
{
    "success": true,
    "document_type": "sale",
    "documents_count": 15,
    "contractors_count": 8,
    "data": { ... parsed data ... }
}
```

#### 2. Import XML File (със staging)
```
POST /api/controlisy/import
Content-Type: application/json

Body:
{
    "company_id": 1,
    "file_name": "prodajbi-07.2025_02-Продажби.xml", 
    "xml_content": "<xml>...</xml>"
}

Response:
{
    "success": true,
    "import_id": 123,
    "message": "Successfully imported 15 documents and 8 contractors",
    "document_type": "sale",
    "documents_count": 15,
    "contractors_count": 8
}
```

#### 3. List Imports for Company
```
GET /api/controlisy/imports/{company_id}

Response:
[
    {
        "id": 123,
        "file_name": "prodajbi-07.2025.xml",
        "document_type": "sale", 
        "status": "staged",
        "import_date": "2025-09-03T18:00:00Z",
        "documents_count": 15,
        "contractors_count": 8
    }
]
```

#### 4. Update Staged Data
```
PUT /api/controlisy/update/{import_id}
Content-Type: application/json

Body:
{
    "data": "{ ... updated JSON data ... }"
}

Response:
{
    "success": true,
    "message": "Staged data updated successfully"
}
```

#### 5. Mark as Reviewed
```
POST /api/controlisy/review/{import_id}
Content-Type: application/json

Body:
{
    "user_id": 1
}

Response:
{
    "success": true,
    "message": "Import marked as reviewed successfully"
}
```

#### 6. Process Import (Final Step)
```
POST /api/controlisy/process/{import_id}

Response:
{
    "success": true,
    "message": "Import processed successfully"
}
```

---

## GraphQL - За стандартни бази данни операции
**URL:** `/graphql`
**Playground:** `/graphiql`

### Използва се за:
- Отчети и статистики
- CRUD операции върху счетоводни записи
- Компании, потребители, настройки
- NAP файлове генериране
- Стандартни релационни заявки

### Queries:
```graphql
query {
    getControlisyImport(importId: 123) {
        id
        status
        imported_documents
        imported_contractors
    }
    
    listControlisyImports(companyId: 1) {
        id
        fileName
        documentType
        status
        importDate
        processed
    }
    
    getStagedImportData(importId: 123)
}
```

### Mutations:
```graphql
mutation {
    generateVatFilesForNap(companyId: 1, year: 2025, month: 7) {
        success
        deklarContent
        pokupkiContent
        prodagbiContent
    }
}
```

---

## Архитектурно разделение:

**REST API** (/api/controlisy/*):
- ✅ XML парсиране и валидиране
- ✅ Файлови операции 
- ✅ JSON данни манипулиране
- ✅ Staging workflow управление
- ✅ Всички Controlisy операции

**GraphQL** (/graphql):
- ✅ Релационни бази данни заявки
- ✅ Сложни отчети и статистики  
- ✅ CRUD операции
- ✅ Счетоводни операции
- ✅ NAP експорт

Това разделение избягва объркването между двете API-та.