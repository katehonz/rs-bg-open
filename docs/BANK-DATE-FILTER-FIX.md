# Bank Import Date Filter Fix

## Problem

When filtering bank imports by date range (e.g., 01.07.2025 - 10.07.2025), the system showed **zero results** even though journal entries existed in that period.

### Root Cause

The date filter was being applied to `bank_imports.imported_at` (when the file was imported), **NOT** to `journal_entries.document_date` (the actual transaction dates).

**Example:**
- Import #3 was created on **2025-10-04** (`imported_at`)
- But it contains journal entries with `document_date` from **2025-07-01 to 2025-07-31**
- When filtering by 01.07.2025 - 10.07.2025, backend checked if `imported_at` was in that range
- Since 2025-10-04 is NOT in the range 01.07-10.07, it returned no results ❌

## Solution

### Backend Fix (`bank_resolvers.rs`)

**Changed date filter logic:**
1. **Removed** filter on `imported_at`
2. **Added** filter on `journal_entries.document_date`
3. Show import if **at least one** journal entry falls in the date range

**Before:**
```rust
if let Some(from_date) = from_date {
    query = query.filter(bank_import::Column::ImportedAt.gte(start));
}
if let Some(to_date) = to_date {
    query = query.filter(bank_import::Column::ImportedAt.lte(end));
}
```

**After:**
```rust
// Note: Date filters are NOT applied to imported_at here
// They will be applied to journal entries' document_date later

// ... later in the code ...

// Check if at least one journal entry exists AND matches date range
let mut sql_query = String::from("SELECT EXISTS(SELECT 1 FROM journal_entries WHERE id = ANY($1::int[])");

if let Some(from) = from_date {
    sql_query.push_str(" AND document_date >= $2");
    params.push(from.into());
}

if let Some(to) = to_date {
    let param_num = if from_date.is_some() { 3 } else { 2 };
    sql_query.push_str(&format!(" AND document_date <= ${}", param_num));
    params.push(to.into());
}
```

### Frontend Fix (`BankStatementsReview.jsx`)

**Added client-side filtering of displayed entries:**

Even though backend returns imports with at least one matching entry, frontend was showing **all** journal entries from that import (including those outside the date range).

**Solution:** Filter `entryDetails` based on `documentDate`:

```javascript
const entryDetails = useMemo(() => {
    const allDetails = entryIds.map((id) => entriesCache[id] ?? { missing: true, id });

    // Filter by date range if date filter is active
    if (!dateFilter.from && !dateFilter.to) {
      return allDetails; // No date filter, return all
    }

    return allDetails.filter((detail) => {
      if (!detail || detail.missing || !detail.entry) {
        return false; // Don't show missing entries when date filter is active
      }

      const docDate = detail.entry.documentDate;
      if (!docDate) {
        return false; // No document date, skip
      }

      // Check if document date is within range
      if (dateFilter.from && docDate < dateFilter.from) {
        return false; // Before start date
      }
      if (dateFilter.to && docDate > dateFilter.to) {
        return false; // After end date
      }

      return true; // Within range
    });
  }, [entryIds, entriesCache, dateFilter.from, dateFilter.to]);
```

## Behavior

### Before Fix
- Filter: 01.07.2025 - 10.07.2025
- Result: 0 imports shown ❌
- Reason: Import #3 was created on 2025-10-04, outside the filter range

### After Fix
- Filter: 01.07.2025 - 10.07.2025
- Result: Import #3 shown ✅
- Entries shown: **Only 35 entries** with `document_date` in 01.07-10.07 range
- Entries hidden: 47 entries with `document_date` outside the range (11.07-31.07)

### No Filter
- Result: All imports shown
- Entries shown: All journal entries (82 entries in import #3)

## Testing

### SQL Test Query

Test if import #3 has entries in date range:

```sql
-- Check if import has any entries in date range
WITH import_3_entries AS (
    SELECT jsonb_array_elements_text(journal_entry_ids::jsonb)::int as entry_id
    FROM bank_imports
    WHERE id = 3
)
SELECT EXISTS(
    SELECT 1
    FROM journal_entries je
    WHERE je.id IN (SELECT entry_id FROM import_3_entries)
      AND je.document_date >= '2025-07-01'
      AND je.document_date <= '2025-07-10'
) as has_entries_in_range;

-- Result: t (true) ✅
```

### Count Entries in Range

```sql
SELECT
    COUNT(*) as entries_in_range,
    MIN(document_date) as first_date,
    MAX(document_date) as last_date
FROM journal_entries
WHERE id >= 877 AND id <= 958  -- Import #3 entry IDs
  AND document_date >= '2025-07-01'
  AND document_date <= '2025-07-10';

-- Result:
-- entries_in_range: 35
-- first_date: 2025-07-01
-- last_date: 2025-07-10
```

## Files Changed

1. **`backend/src/graphql/bank_resolvers.rs:78-165`**
   - Removed `imported_at` date filtering
   - Added `document_date` filtering in SQL query
   - Filter imports with at least one matching journal entry

2. **`frontend/src/components/banks/BankStatementsReview.jsx:558-586`**
   - Added date range filtering to `entryDetails`
   - Only show journal entries within the date range
   - Hide entries outside the range when filter is active

## Impact

✅ **Positive:**
- Date filter now works correctly for bank imports
- Users can find transactions by transaction date, not import date
- More intuitive behavior (filter by when transaction happened, not when it was imported)
- Correct totals shown (only includes filtered entries)

⚠️ **Behavioral Change:**
- Previously: date filter applied to import date (when file was uploaded)
- Now: date filter applies to transaction date (when bank transaction occurred)
- This is the **correct** and **expected** behavior

🔧 **Performance:**
- Minimal impact: one additional SQL EXISTS query per import
- Query is fast (uses index on journal_entries.id and document_date)

## Related Issues

- [Bank Imports Cleanup](./BANK-IMPORTS-CLEANUP.md) - Related work on filtering deleted entries

## Future Improvements

**Possible enhancements:**
1. Add UI indicator showing how many entries are filtered out (e.g., "Showing 35 of 82 entries")
2. Add option to choose filter field (document_date vs vat_date vs accounting_date)
3. Add date range histogram showing distribution of transactions
4. Cache filtered results to avoid re-filtering on every render

---

**Date Fixed:** 11 октомври 2025
**Issue:** Date filter on bank imports not working
**Resolution:** Changed filter from `imported_at` to `document_date`
**Status:** ✅ Resolved
