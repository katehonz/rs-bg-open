# Controlisy Import - VAT Code Mapping Update

## Date: 11 октомври 2025

## Summary

Updated the Controlisy import service to automatically map old numeric VAT operation codes (1, 2, 3, etc.) to new NAP-compliant text codes (пок09, пок10, про11, etc.) introduced in VAT Module v2.0.

## Problem

With the introduction of **VAT Module v2.0**, RS-AC-BG now uses NAP specification codes like:
- `пок09` - Purchase without credit
- `пок10` - Purchase with full credit
- `пок12` - Purchase with partial credit
- `про11` - Sales with 20% VAT
- `про17` - Sales with 9% VAT
- etc.

However, Controlisy XML exports use simple numeric codes:
- `1` - Full credit / 20% VAT
- `2` - Partial credit / 9% VAT
- `3` - No credit / No VAT
- etc.

Without proper mapping, imported documents would have incorrect VAT operation codes, causing NAP export files (DEKLAR.TXT, PRODAGBI.TXT, POKUPKI.TXT) to be invalid.

## Solution

Created an automatic mapping layer in the Controlisy import service that:

1. Reads the numeric code from Controlisy XML (`vatOperationIden` attribute)
2. Determines document type (purchase vs. sale)
3. Maps to the appropriate NAP code
4. Stores journal entry with NAP-compliant code

## Changes

### 1. Backend Code Changes

**File:** `backend/src/services/controlisy.rs`

#### Added Mapping Function (lines 111-168)

```rust
fn map_vat_operation_code(
    old_code: &str,
    document_type: &str,
    vat_amount: Decimal,
) -> Option<String>
```

**Purchase Mappings:**
- `1` → `пок10` (full credit)
- `2` → `пок12` (partial credit)
- `3` with VAT → `пок09` (no credit)
- `3` without VAT → `0` (no journal)
- `4` → `пок14` (annual adjustment)
- `5` → `пок15` (triangular transaction)

**Sales Mappings:**
- `1` → `про11` (20% VAT)
- `2` → `про17` (9% VAT)
- `3` → `про19` (0% chapter 3)
- `4` → `про20` (intra-community supply)
- `5` → `про13` (intra-community acquisition)
- `8` → `про24-1` (exempt supplies)
- `9` → `про22` (services art. 21)

#### Updated Journal Entry Creation (lines 980-1043)

Modified `create_journal_entry_with_type` function to:
- Extract `vatOperationIden` from XML document
- Call mapping function to convert old code to new code
- Use mapped code when creating journal entry

**Before:**
```rust
if document.vat_amount_bgn > Decimal::ZERO {
    (Some("1"), None) // Old numeric code
} else {
    (Some("3"), None)
}
```

**After:**
```rust
let old_code = vat_operation_from_xml; // From XML
let new_code = Self::map_vat_operation_code(
    old_code,
    "purchase",
    document.vat_amount_bgn,
);
(new_code, None) // New NAP code
```

### 2. Documentation

Created comprehensive documentation:

#### A. CONTROLISY-IMPORT.md (New file)

- **Content:** Complete guide to Controlisy VAT code mapping
- **Sections:**
  - Mapping tables (purchase and sales)
  - How it works (XML structure, automatic mapping, code implementation)
  - Import workflow (4 steps)
  - Examples (purchase with credit, sale with VAT, purchase without credit)
  - Default behavior
  - Payment documents handling
  - Backward compatibility
  - Migration script for old data
  - Troubleshooting (3 common issues)

#### B. Updated docs/README.md

Added link to new Controlisy documentation:
```markdown
- [Controlisy Import](./CONTROLISY-IMPORT.md) - Импорт от Controlisy с NAP код mapping
```

## Testing

### Build Status

✅ Backend compiles successfully with no errors
- Warnings: Only unused code warnings (expected)
- Build time: 0.26s (incremental)

### Manual Testing Required

The following scenarios should be tested:

1. **Import Controlisy XML with code "1"**
   - Expected: Purchase → `пок10`, Sale → `про11`

2. **Import Controlisy XML with code "3"**
   - Expected: Purchase with VAT → `пок09`, without VAT → `0`

3. **Generate NAP files after import**
   - Expected: DEKLAR.TXT, PRODAGBI.TXT, POKUPKI.TXT use NAP codes

4. **Import payment document**
   - Expected: No VAT operation codes assigned

## Migration Path

### For New Imports

✅ **No action required** - Mapping is automatic

### For Old Data (Pre-v2.0)

Journal entries created before this update may have old numeric codes:

**Option 1: Re-import from Controlisy** (Recommended)
- Delete old journal entries
- Re-import XML files
- New entries will have correct NAP codes

**Option 2: SQL Migration Script**

```sql
-- Update purchase operations
UPDATE journal_entries SET vat_purchase_operation = 'пок10'
WHERE vat_purchase_operation = '1';

UPDATE journal_entries SET vat_purchase_operation = 'пок12'
WHERE vat_purchase_operation = '2';

UPDATE journal_entries SET vat_purchase_operation = 'пок09'
WHERE vat_purchase_operation = '3' AND total_vat_amount > 0;

UPDATE journal_entries SET vat_purchase_operation = '0'
WHERE vat_purchase_operation = '3' AND total_vat_amount = 0;

-- Update sales operations
UPDATE journal_entries SET vat_sales_operation = 'про11'
WHERE vat_sales_operation = '1';

UPDATE journal_entries SET vat_sales_operation = 'про17'
WHERE vat_sales_operation = '2';

UPDATE journal_entries SET vat_sales_operation = 'про24-1'
WHERE vat_sales_operation = '8';
```

## Impact Analysis

### 🟢 Positive Impact

1. **NAP Compliance**: Imports now generate valid NAP export files
2. **Data Quality**: VAT operations match official specification
3. **Automation**: No manual code conversion needed
4. **Documentation**: Clear mapping tables for reference
5. **Backward Compatible**: Old data continues to work (with migration path)

### 🟡 Considerations

1. **Old Data**: Existing journal entries need migration (see above)
2. **Testing**: Manual testing required for all VAT operation types
3. **Controlisy Updates**: If Controlisy changes their codes, mapping needs update

### ⚪ No Impact

1. **Performance**: Mapping is done once per document during import
2. **Database Schema**: No schema changes required
3. **Frontend**: No UI changes needed
4. **API**: GraphQL/REST endpoints unchanged

## Files Changed

1. `backend/src/services/controlisy.rs` - Added mapping function and updated import logic
2. `docs/CONTROLISY-IMPORT.md` - New documentation file (27KB)
3. `docs/README.md` - Added link to Controlisy documentation
4. `docs/CHANGELOG-CONTROLISY-MAPPING.md` - This file

## Related Issues

- **Root Cause**: VAT Module v2.0 introduced NAP codes without updating Controlisy import
- **User Report**: "при промяна ДДС схемата може да имаме проблема с ипорта от c https://accounting.controlisy.com/"
- **Fix Duration**: ~1 hour (analysis + implementation + documentation)

## Next Steps

1. ✅ Code implementation - DONE
2. ✅ Documentation - DONE
3. ✅ Build verification - DONE
4. ⏳ Manual testing with sample XML files
5. ⏳ Test NAP export after import
6. ⏳ Migrate old data (if any)
7. ⏳ Deploy to production

## References

- [VAT Module v2.0 Documentation](./VAT-MODULE.md)
- [VAT Quick Reference](./VAT-QUICK-REFERENCE.md)
- [Controlisy Import Guide](./CONTROLISY-IMPORT.md)
- [NAP PPDDS_2025 Specification](../new-vat-nap/PPDDS_2025_.html)

## Version

- **Module**: Controlisy Import Service
- **Version**: 2.0.1 (compatible with VAT Module v2.0)
- **Date**: 11 октомври 2025
- **Author**: Claude (Anthropic) + DVG
- **Status**: Ready for testing

---

**Note**: This update ensures that RS-AC-BG can import Controlisy data while maintaining full NAP compliance for Bulgarian tax reporting requirements.
