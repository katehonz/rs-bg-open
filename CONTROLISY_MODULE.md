# Controlisy Import Module Documentation

## Overview

The Controlisy Import Module provides complete functionality for importing, parsing, reviewing, and processing accounting documents from Controlisy XML exports. It supports both sales (продажби) and purchase (покупки) document types with full VAT handling and accounting integration.

## Architecture

### Backend Components

#### REST API Endpoints (`/backend/src/rest/controlisy_api.rs`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/controlisy/import` | Upload and import XML file |
| GET | `/api/controlisy/imports/{company_id}` | List all imports for company |
| GET | `/api/controlisy/import/{import_id}` | Get specific import with parsed data |
| PUT | `/api/controlisy/import/{import_id}` | Update import metadata |
| DELETE | `/api/controlisy/imports/{import_id}` | Delete import |
| POST | `/api/controlisy/process/{import_id}` | Process import to final accounting |
| PUT | `/api/controlisy/update/{import_id}` | Update staged data before processing |
| POST | `/api/controlisy/review/{import_id}` | Mark import as reviewed |

#### Database Schema (`controlisy_imports` table)

```sql
CREATE TABLE controlisy_imports (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL,
    import_date TIMESTAMP NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    document_type VARCHAR(50) NOT NULL, -- 'sale' or 'purchase'
    raw_xml TEXT NOT NULL,
    parsed_data JSONB,
    status VARCHAR(50) DEFAULT 'staged',
    processed BOOLEAN DEFAULT FALSE,
    error_message TEXT,
    imported_documents INTEGER,
    imported_contractors INTEGER,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

#### Parser Components

**XML Parser** (`/backend/src/services/controlisy_parser.rs`)
- Base64 XML content decoding
- Document type detection (sales vs purchases)
- Contractor extraction and deduplication
- Document parsing with full accounting details
- VAT calculation validation

**Document Types**
- **Sales** (продажби): Keywords `prodaj`, `prodajbi`, `sale`
- **Purchases** (покупки): Keywords `pokupki`, `purchase`, `buy`

### Frontend Components

#### Main Component (`/frontend/src/components/imports/ControlisyImport.jsx`)

**Key Features:**
- File upload with drag-and-drop support
- Import status tracking and display
- Preview/review modal with parsed data
- Document type filtering (sales/purchases)
- Mapping functionality for account adjustments
- Processing workflow management

**Component Structure:**
```jsx
const ControlisyImport = () => {
  // State management
  const [imports, setImports] = useState([]);
  const [selectedFile, setSelectedFile] = useState(null);
  const [uploading, setUploading] = useState(false);
  const [showReviewModal, setShowReviewModal] = useState(false);
  const [reviewData, setReviewData] = useState(null);
  
  // Core functions
  const handleFileUpload = async (file) => { ... }
  const openReviewModal = async (importItem) => { ... }
  const processImport = async (importId) => { ... }
}
```

## Workflow States

1. **Staged** - Initial import, ready for review
2. **Reviewed** - Data verified, ready for processing  
3. **Processing** - Being processed into accounting system
4. **Completed** - Successfully processed
5. **Error** - Processing failed

## Data Structures

### Import Request Format
```json
{
  "company_id": 1,
  "file_name": "prodajbi_2025_07.xml",
  "xml_content": "base64_encoded_xml_content"
}
```

### Parsed Data Structure
```json
{
  "contractors": [
    {
      "ca_contractor_id": "11278",
      "contractor_name": "ЗАГАТО ООД",
      "contractor_eik": "121102130",
      "contractor_vat_number": "BG121102130",
      "contractor_inside_number": "121102130"
    }
  ],
  "documents": [
    {
      "ca_doc_id": "11281",
      "document_number": "0000001895",
      "document_date": "2025-07-01",
      "accounting_month": "2025-07-01",
      "vat_month": "2025-07-01",
      "maturity": "2025-07-01",
      "reason": "Продукция",
      "net_amount_bgn": "1120.00",
      "vat_amount_bgn": "224.00",
      "total_amount_bgn": "1344.00",
      "ca_contractor_id": "11278",
      "accountings": [...],
      "vat_data": [...]
    }
  ]
}
```

### Accounting Detail Structure
```json
{
  "direction": "Debit|Credit",
  "account_number": "411",
  "account_name": "Вземания от клиенти",
  "currency": "",
  "currency_amount": "",
  "unit": "",
  "quantity": "",
  "contractor_name": "ЗАГАТО ООД",
  "contractor_eik": "121102130",
  "contractor_vat_number": "BG121102130",
  "account_item1": "",
  "account_item2": "",
  "account_item3": "",
  "account_item4": ""
}
```

## File Format Support

### Controlisy XML Structure
```xml
<?xml version="1.0" encoding="utf-8"?>
<ExportedData>
  <Contractors>
    <Contractor ca_contractorId="..." contractorName="..." 
                contractorEIK="..." contractorVATNumber="..." />
  </Contractors>
  <Documents>
    <Document accountingMonth="..." vatMonth="..." documentDate="..."
              documentNumber="..." ca_docId="..." reason="..."
              netAmountBGN="..." vatAmountBGN="..." totalAmountBGN="...">
      <Accountings>
        <Accounting amountBGN="..." ca_vatOperationID="...">
          <AccountingDetail direction="..." accountNumber="..."
                           contractorName="..." contractorEIK="..." />
        </Accounting>
        <VAT taxBase="..." vatRate="..." vatAmountBGN="..." />
      </Accountings>
      <VATData vatRegister="..." contractorName="..." contractorVATNumber="...">
        <VAT taxBase="..." vatRate="..." vatAmountBGN="..." />
      </VATData>
    </Document>
  </Documents>
</ExportedData>
```

## Usage Examples

### 1. Import Sales Data
```bash
curl -X POST http://localhost:8080/api/controlisy/import \
  -H "Content-Type: application/json" \
  -d '{
    "company_id": 1,
    "file_name": "prodajbi_july_2025.xml",
    "xml_content": "PD94bWwgdmVyc2lvbj..."
  }'
```

**Response:**
```json
{
  "success": true,
  "import_id": 14,
  "message": "Successfully imported 14 documents and 12 contractors",
  "document_type": "sale",
  "documents_count": 14,
  "contractors_count": 12
}
```

### 2. List Company Imports
```bash
curl -X GET http://localhost:8080/api/controlisy/imports/1
```

**Response:**
```json
[
  {
    "id": 14,
    "file_name": "prodajbi_july_2025.xml",
    "document_type": "sale",
    "status": "staged",
    "import_date": "2025-09-03 19:51:11",
    "imported_documents": 14,
    "imported_contractors": 12
  }
]
```

### 3. Get Import Details for Review
```bash
curl -X GET http://localhost:8080/api/controlisy/import/14
```

**Response:**
```json
{
  "id": 14,
  "file_name": "prodajbi_july_2025.xml",
  "document_type": "sale",
  "status": "staged",
  "import_date": "2025-09-03 19:51:11",
  "imported_documents": 14,
  "imported_contractors": 12,
  "raw_xml": "base64_encoded_original_xml",
  "parsed_data": {
    "contractors": [...],
    "documents": [...]
  }
}
```

## Frontend Integration

### Component Usage
```jsx
import ControlisyImport from './components/imports/ControlisyImport';

function App() {
  return (
    <div>
      <ControlisyImport />
    </div>
  );
}
```

### Key Frontend Functions

#### File Upload Handler
```javascript
const handleFileUpload = async (file) => {
  const formData = new FormData();
  formData.append('file', file);
  
  try {
    const response = await fetch('http://localhost:8080/api/controlisy/import', {
      method: 'POST',
      body: formData
    });
    
    if (response.ok) {
      const result = await response.json();
      console.log('Import successful:', result);
      fetchImports(); // Refresh list
    }
  } catch (error) {
    console.error('Import failed:', error);
  }
};
```

#### Review Modal Handler
```javascript
const openReviewModal = async (importItem) => {
  try {
    const response = await fetch(`http://localhost:8080/api/controlisy/import/${importItem.id}`);
    
    if (response.ok) {
      const importData = await response.json();
      setSelectedImport(importItem);
      setReviewData(importData.parsed_data);
      setShowReviewModal(true);
    }
  } catch (error) {
    console.error('Failed to fetch import data:', error);
  }
};
```

## Configuration

### Document Type Detection
The system automatically detects document type based on filename:
- **Sales**: Contains `prodaj`, `prodajbi`, `sale`
- **Purchases**: Contains `pokupki`, `purchase`, `buy`

### Account Mapping
Default account mappings for Bulgarian accounting:
- **411** - Вземания от клиенти (Customer receivables)
- **501** - Каса лв. (Cash BGN)
- **701** - Приходи от продажби (Sales revenue)
- **4532** - ДДС върху продажбите (VAT on sales)

## Error Handling

### Common Errors
1. **Invalid XML format** - Malformed XML structure
2. **Base64 decode error** - Invalid encoding
3. **Document type detection failed** - Filename doesn't contain recognizable keywords
4. **Missing required fields** - XML missing mandatory elements
5. **VAT calculation mismatch** - Computed VAT doesn't match provided amounts

### Error Response Format
```json
{
  "error": "Failed to decode base64 content: Invalid symbol 36, offset 0."
}
```

## Testing

### Test Data Created
1. **Sales Import**: `test-prodajbi-fixed.xml` - 14 documents, 12 contractors
2. **Purchase Import**: `test-pokupki-data.xml` - 2 documents, 2 contractors

### Test Coverage
- ✅ File upload and parsing
- ✅ Document type detection  
- ✅ Contractor extraction
- ✅ VAT calculations
- ✅ Account mapping
- ✅ Preview/review functionality
- ✅ Status management
- ✅ Error handling

## Production Deployment

### Environment Setup
1. Ensure PostgreSQL database has `controlisy_imports` table
2. Configure CORS for frontend domain
3. Set up proper file upload size limits
4. Configure backup strategy for imported data

### Performance Considerations
- Large XML files (>10MB) may require increased timeouts
- Consider batch processing for high-volume imports
- Index `company_id` and `import_date` columns for performance

### Security Notes
- Validate XML structure before processing
- Sanitize contractor names and document data
- Implement rate limiting for import endpoints
- Log all import activities for audit trail

---

## Recent Bug Fixes

### Fixed Issues (2025-09-04)

1. **SQL Column Name Mismatch** - Fixed `contractor_eik` column reference in SQL query that should use JSON extraction `c->>'contractor_eik'`
2. **Accounts Table Column Error** - Corrected column name from `number` to `code` in accounts table queries
3. **Duplicate Journal Entry Numbers** - Implemented unique entry number generation with:
   - Timestamp with nanosecond precision
   - Retry logic with uniqueness check
   - Up to 10 retry attempts with delays

### Database Column Mappings

| Entity | Correct Column | Previous (Wrong) |
|--------|---------------|------------------|
| counterparts | eik | contractor_eik |
| accounts | code | number |

### Improved Error Handling
- Added duplicate key constraint handling
- Better error messages for import failures
- Validation of required fields before processing

**Module Status**: ✅ Fully Implemented, Tested and Bug-Fixed  
**Last Updated**: 2025-09-04  
**Version**: 1.0.1