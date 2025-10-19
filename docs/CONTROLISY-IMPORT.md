# Controlisy Import - VAT Code Mapping

## Overview

RS-AC-BG imports accounting data from [Controlisy](https://accounting.controlisy.com/) XML files. With the implementation of **VAT Module v2.0**, the system now uses NAP-compliant text codes (е.g., `пок10`, `про11`) instead of simple numeric codes.

This document explains how Controlisy's numeric VAT operation codes are automatically mapped to the new NAP codes during import.

## Mapping Table

### Purchase Operations (Покупки)

| Controlisy Code | Description (Controlisy) | NAP Code | NAP Description |
|-----------------|-------------------------|----------|-----------------|
| `1` | Получени доставки с право на пълен данъчен кредит | `пок10` | Колона 10: Облагаеми доставки с ПЪЛЕН ДК |
| `2` | Получени доставки с право на частичен данъчен кредит | `пок12` | Колона 12: Облагаеми доставки с ЧАСТИЧЕН ДК |
| `3` (с ДДС > 0) | Получени доставки без право на ДК | `пок09` | Колона 09: Доставки БЕЗ право на ДК |
| `3` (с ДДС = 0) | Получени доставки без ДДС | `0` | Не влиза в дневник |
| `4` | Годишна корекция | `пок14` | Колона 14: Годишна корекция по чл.73, ал.8 ЗДДС |
| `5` | Тристранна операция | `пок15` | Колона 15: Тристранна операция (посредник) |

### Sales Operations (Продажби)

| Controlisy Code | Description (Controlisy) | NAP Code | NAP Description |
|-----------------|-------------------------|----------|-----------------|
| `1` | Облагаеми доставки 20% | `про11` | Колона 11: Облагаеми доставки със ставка 20% |
| `2` | Облагаеми доставки 9% | `про17` | Колона 17: Облагаеми доставки със ставка 9% |
| `3` | Доставки 0% (глава 3) | `про19` | Колона 19: Доставки със ставка 0% по глава 3 ЗДДС |
| `4` | ВОД (вътреобщностна доставка) | `про20` | Колона 20: ВОД (експорт в ЕС) |
| `5` | ВОП (вътреобщностно придобиване) | `про13` | Колона 13: ВОП |
| `8` | Освободени доставки | `про24-1` | Колона 24-1: Освободени доставки в България |
| `9` | Услуги чл.21, ал.2 | `про22` | Колона 22: Услуги чл.21, ал.2 (друга държава ЕС) |

## How It Works

### 1. XML Structure

Controlisy XML files contain VAT operation codes in the `vatOperationIden` attribute:

```xml
<VAT
  taxBase="9744"
  vatRate="20"
  vatAmountBGN="1948.8"
  ca_vatOperationID="11071"
  vatOperationIden="1"
  vatOperationIdentName="Получени доставки с право на пълен данъчен кредит"
  vatOperationName="Покупки 20% ПДКт" />
```

### 2. Automatic Mapping

During import, the system:

1. **Reads** the `vatOperationIden` value (e.g., `"1"`)
2. **Determines** document type (purchase or sale) from:
   - File name (`pokupki` = purchase, `prodajbi` = sale)
   - VAT register field (`1` = purchase, `2` = sale)
   - Accounting entries (account codes 401/402 = purchase, 411/412 = sale)
3. **Maps** the old numeric code to the new NAP code using the mapping table
4. **Stores** the journal entry with the new NAP code in the database

### 3. Code Implementation

The mapping is implemented in `backend/src/services/controlisy.rs`:

```rust
fn map_vat_operation_code(
    old_code: &str,
    document_type: &str,
    vat_amount: Decimal,
) -> Option<String> {
    match document_type {
        "purchase" => {
            match old_code {
                "1" => Some("пок10".to_string()),
                "2" => Some("пок12".to_string()),
                "3" => {
                    if vat_amount > Decimal::ZERO {
                        Some("пок09".to_string())
                    } else {
                        Some("0".to_string())
                    }
                }
                // ... more mappings
            }
        }
        "sale" => {
            match old_code {
                "1" => Some("про11".to_string()),
                "2" => Some("про17".to_string()),
                // ... more mappings
            }
        }
    }
}
```

## Import Workflow

### Step 1: Upload XML File

Upload Controlisy XML file through the REST API:

```bash
POST /api/controlisy/import
Content-Type: multipart/form-data

{
  "company_id": 1,
  "file": pokupki-07.2025_01.xml
}
```

### Step 2: Review Staged Data

The system parses the XML and stages the data for review:

```bash
GET /api/controlisy/staged/{import_id}
```

The parsed data shows:
- Original Controlisy codes (for reference)
- Mapped NAP codes (what will be imported)
- Document details (amounts, dates, counterparts)

### Step 3: Process Import

When you approve the import, the system:

1. Creates/updates counterparts (contractors)
2. Creates journal entries with **NAP codes** (not Controlisy codes)
3. Creates entry lines (debit/credit accounting entries)
4. Links VAT operations for NAP reporting

```bash
POST /api/controlisy/process/{import_id}
```

### Step 4: Verify Results

After import, you can:

- View journal entries with new NAP codes
- Generate NAP export files (DEKLAR.TXT, PRODAGBI.TXT, POKUPKI.TXT)
- Verify that VAT operations match NAP specification

## Examples

### Example 1: Purchase with Full Credit

**Controlisy XML:**
```xml
<Document documentNumber="1000123" vatAmountBGN="200" netAmountBGN="1000">
  <VATData vatRegister="1">
    <VAT vatOperationIden="1" vatOperationIdentName="Получени доставки с право на пълен данъчен кредит" />
  </VATData>
</Document>
```

**Imported as:**
- `vat_purchase_operation`: `пок10`
- `vat_document_type`: `03` (Purchase invoice)
- Tax base: 1000 лв.
- VAT amount: 200 лв.

### Example 2: Sale with 20% VAT

**Controlisy XML:**
```xml
<Document documentNumber="2000456" vatAmountBGN="400" netAmountBGN="2000">
  <VATData vatRegister="2">
    <VAT vatOperationIden="1" vatRate="20" />
  </VATData>
</Document>
```

**Imported as:**
- `vat_sales_operation`: `про11`
- `vat_document_type`: `01` (Sales invoice)
- Tax base: 2000 лв.
- VAT amount: 400 лв.

### Example 3: Purchase without Credit

**Controlisy XML:**
```xml
<Document documentNumber="1000789" vatAmountBGN="50" netAmountBGN="250">
  <VATData vatRegister="1">
    <VAT vatOperationIden="3" vatOperationIdentName="Получени доставки без ДДС" />
  </VATData>
</Document>
```

**Imported as:**
- `vat_purchase_operation`: `пок09` (because VAT amount > 0)
- Tax base: 250 лв.
- VAT amount: 50 лв. (not deductible)

## Default Behavior

If the XML file doesn't contain explicit VAT operation codes, the system uses intelligent defaults:

### Purchase Documents
- If `vat_amount > 0` → `пок10` (full credit)
- If `vat_amount = 0` → `0` (no journal entry)

### Sales Documents
- If `vat_amount > 0` → `про11` (20% taxable)
- If `vat_amount = 0` → `про24-1` (exempt)

## Payment Documents

Documents identified as payments (разплащания) are treated specially:

- **No VAT operations**: `vat_purchase_operation` and `vat_sales_operation` are NULL
- **No VAT document type**: `vat_document_type` is NULL
- **All dates equal**: `document_date` = `vat_date` = `accounting_date`

Payment documents are identified by:
- Empty or "0" value in `ca_vatOperationID`
- "разплащане" in the reason/description field

## Backward Compatibility

### Old Data

Journal entries created **before** VAT Module v2.0 may still have old numeric codes (`"1"`, `"2"`, `"3"`). These will continue to work but:

- ⚠️ They won't appear correctly in NAP export files
- 💡 Recommendation: Re-import from Controlisy or manually update to NAP codes

### Migration Script

To update old journal entries, you can run:

```sql
-- Update purchase operations
UPDATE journal_entries
SET vat_purchase_operation = 'пок10'
WHERE vat_purchase_operation = '1';

UPDATE journal_entries
SET vat_purchase_operation = 'пок12'
WHERE vat_purchase_operation = '2';

UPDATE journal_entries
SET vat_purchase_operation = 'пок09'
WHERE vat_purchase_operation = '3' AND total_vat_amount > 0;

UPDATE journal_entries
SET vat_purchase_operation = '0'
WHERE vat_purchase_operation = '3' AND total_vat_amount = 0;

-- Update sales operations
UPDATE journal_entries
SET vat_sales_operation = 'про11'
WHERE vat_sales_operation = '1';

UPDATE journal_entries
SET vat_sales_operation = 'про24-1'
WHERE vat_sales_operation = '8';
```

## Troubleshooting

### Issue 1: Wrong VAT Code After Import

**Symptom:** Imported document has incorrect VAT operation code

**Solution:**
1. Check the `vatOperationIden` value in the Controlisy XML
2. Verify document type (purchase vs. sale) is detected correctly
3. Check if VAT amount is correct (some operations depend on VAT amount)
4. Review the mapping table and file a bug if mapping is incorrect

### Issue 2: Payment Mistaken as VAT Document

**Symptom:** Payment document has VAT operation code

**Solution:**
- Ensure the document has `ca_vatOperationID = "0"` or empty
- Or include "разплащане" in the reason field
- Check that `vatAmountBGN = 0` for payment documents

### Issue 3: NAP Export Shows Wrong Values

**Symptom:** DEKLAR.TXT or journal files have incorrect values

**Solution:**
1. Verify that journal entries use NAP codes (пок10, про11, etc.)
2. Check that VAT date is correct (especially for purchases)
3. Ensure accounting date is properly set
4. Re-import from Controlisy if using old numeric codes

## References

- [VAT Module v2.0 Documentation](./VAT-MODULE.md)
- [VAT Quick Reference](./VAT-QUICK-REFERENCE.md)
- [Controlisy API Documentation](https://accounting.controlisy.com/docs)
- [NAP PPDDS_2025 Specification](../new-vat-nap/PPDDS_2025_.html)

## Version History

- **2025-10-11**: Initial documentation for VAT code mapping
- **VAT Module v2.0**: Introduced NAP-compliant text codes

---

**Compatibility:** RS-AC-BG v2.0+, Controlisy XML format v2022+

**Last updated:** 11 октомври 2025
