# Bank Imports Cleanup - Orphaned References

## Problem Description

When journal entries are deleted from the database, bank import records may still reference them through the `journal_entry_ids` JSON field. This creates "orphaned references" that cause errors when viewing bank import details:

```
Журналният запис с идентификатор 795 не е наличен (вероятно е изтрит).
Журналният запис с идентификатор 796 не е наличен (вероятно е изтрит).
```

## Solution

### Automatic Cleanup Query

Use this SQL query to find and clean up bank imports with deleted journal entries:

```sql
-- Step 1: Find problematic imports
WITH import_entries AS (
    SELECT
        bi.id as import_id,
        bi.file_name,
        bi.created_journal_entries,
        jsonb_array_elements_text(bi.journal_entry_ids::jsonb) as entry_id
    FROM bank_imports bi
    WHERE bi.journal_entry_ids IS NOT NULL
)
SELECT
    ie.import_id,
    ie.file_name,
    ie.created_journal_entries,
    COUNT(*) as total_entries,
    COUNT(je.id) as existing_entries,
    COUNT(*) - COUNT(je.id) as missing_entries,
    ROUND(100.0 * COUNT(je.id) / COUNT(*), 2) as health_percentage
FROM import_entries ie
LEFT JOIN journal_entries je ON je.id = ie.entry_id::integer
GROUP BY ie.import_id, ie.file_name, ie.created_journal_entries
HAVING COUNT(*) - COUNT(je.id) > 0
ORDER BY ie.import_id;
```

**Output Example:**
```
import_id | file_name                                 | created_journal_entries | total_entries | existing_entries | missing_entries | health_percentage
----------|-------------------------------------------|------------------------|---------------|------------------|-----------------|------------------
1         | CustomerReportAccountsTransactions.xml    | 82                     | 82            | 0                | 82              | 0.00
2         | CustomerReportAccountsTransactions.xml    | 82                     | 82            | 0                | 82              | 0.00
```

### Cleanup Options

#### Option 1: Delete Completely Broken Imports (Recommended)

If **all** journal entries are deleted (health_percentage = 0.00):

```sql
-- Delete imports where ALL journal entries are missing
WITH import_entries AS (
    SELECT
        bi.id as import_id,
        jsonb_array_elements_text(bi.journal_entry_ids::jsonb) as entry_id
    FROM bank_imports bi
    WHERE bi.journal_entry_ids IS NOT NULL
),
broken_imports AS (
    SELECT ie.import_id
    FROM import_entries ie
    LEFT JOIN journal_entries je ON je.id = ie.entry_id::integer
    GROUP BY ie.import_id
    HAVING COUNT(je.id) = 0  -- ALL entries are missing
)
DELETE FROM bank_imports
WHERE id IN (SELECT import_id FROM broken_imports)
RETURNING id, file_name, created_journal_entries;
```

#### Option 2: Clean Up Partially Broken Imports

If **some** journal entries exist (0% < health_percentage < 100%):

```sql
-- Update journal_entry_ids to only include existing entries
WITH import_entries AS (
    SELECT
        bi.id as import_id,
        bi.journal_entry_ids,
        jsonb_array_elements_text(bi.journal_entry_ids::jsonb) as entry_id
    FROM bank_imports bi
    WHERE bi.journal_entry_ids IS NOT NULL
),
valid_entries AS (
    SELECT
        ie.import_id,
        jsonb_agg(je.id ORDER BY je.id) as valid_ids,
        COUNT(je.id) as valid_count
    FROM import_entries ie
    INNER JOIN journal_entries je ON je.id = ie.entry_id::integer
    GROUP BY ie.import_id
)
UPDATE bank_imports bi
SET
    journal_entry_ids = ve.valid_ids,
    created_journal_entries = ve.valid_count,
    updated_at = NOW()
FROM valid_entries ve
WHERE bi.id = ve.import_id
AND bi.created_journal_entries != ve.valid_count
RETURNING bi.id, bi.file_name, bi.created_journal_entries as new_count;
```

#### Option 3: Mark as Invalid (Preserve for History)

```sql
-- Add status column if not exists (run once)
ALTER TABLE bank_imports ADD COLUMN IF NOT EXISTS status VARCHAR DEFAULT 'completed';

-- Mark broken imports as invalid
WITH import_entries AS (
    SELECT
        bi.id as import_id,
        jsonb_array_elements_text(bi.journal_entry_ids::jsonb) as entry_id
    FROM bank_imports bi
    WHERE bi.journal_entry_ids IS NOT NULL
)
UPDATE bank_imports bi
SET
    status = 'invalid',
    error_message = 'Journal entries were deleted',
    updated_at = NOW()
WHERE bi.id IN (
    SELECT ie.import_id
    FROM import_entries ie
    LEFT JOIN journal_entries je ON je.id = ie.entry_id::integer
    GROUP BY ie.import_id
    HAVING COUNT(*) - COUNT(je.id) > 0
)
RETURNING id, file_name, status;
```

## Prevention

### Add Foreign Key Constraint (Recommended)

Currently, `bank_imports.journal_entry_ids` is a JSON field without referential integrity. To prevent this issue:

**Option A:** Use a junction table (best practice):

```sql
-- Create bank_import_journal_entries table
CREATE TABLE bank_import_journal_entries (
    id SERIAL PRIMARY KEY,
    bank_import_id INTEGER NOT NULL REFERENCES bank_imports(id) ON DELETE CASCADE,
    journal_entry_id INTEGER NOT NULL REFERENCES journal_entries(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(bank_import_id, journal_entry_id)
);

CREATE INDEX idx_bank_import_journal_entries_import ON bank_import_journal_entries(bank_import_id);
CREATE INDEX idx_bank_import_journal_entries_entry ON bank_import_journal_entries(journal_entry_id);
```

**Option B:** Add a trigger to validate JSON references:

```sql
-- Trigger function to validate journal_entry_ids
CREATE OR REPLACE FUNCTION validate_journal_entry_ids()
RETURNS TRIGGER AS $$
DECLARE
    entry_id INTEGER;
    missing_ids INTEGER[] := ARRAY[]::INTEGER[];
BEGIN
    -- Check each journal entry ID exists
    FOR entry_id IN
        SELECT jsonb_array_elements_text(NEW.journal_entry_ids::jsonb)::integer
    LOOP
        IF NOT EXISTS (SELECT 1 FROM journal_entries WHERE id = entry_id) THEN
            missing_ids := array_append(missing_ids, entry_id);
        END IF;
    END LOOP;

    -- If any IDs are missing, raise warning
    IF array_length(missing_ids, 1) > 0 THEN
        RAISE WARNING 'Bank import references missing journal entries: %', missing_ids;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply trigger
CREATE TRIGGER check_journal_entry_ids
    BEFORE INSERT OR UPDATE ON bank_imports
    FOR EACH ROW
    WHEN (NEW.journal_entry_ids IS NOT NULL)
    EXECUTE FUNCTION validate_journal_entry_ids();
```

### Soft Delete for Journal Entries

Instead of hard deleting journal entries, mark them as deleted:

```sql
-- Add deleted flag to journal_entries
ALTER TABLE journal_entries ADD COLUMN IF NOT EXISTS is_deleted BOOLEAN DEFAULT FALSE;
ALTER TABLE journal_entries ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE journal_entries ADD COLUMN IF NOT EXISTS deleted_by INTEGER REFERENCES users(id);

-- Create index for filtering
CREATE INDEX idx_journal_entries_is_deleted ON journal_entries(is_deleted) WHERE is_deleted = false;

-- Update queries to exclude deleted entries
-- Example:
-- SELECT * FROM journal_entries WHERE is_deleted = FALSE;
```

## Maintenance Script

Create a monthly maintenance job:

```sql
-- Maintenance procedure
CREATE OR REPLACE FUNCTION cleanup_orphaned_bank_imports()
RETURNS TABLE(
    action TEXT,
    import_id INTEGER,
    file_name VARCHAR,
    details TEXT
) AS $$
BEGIN
    -- Delete completely broken imports
    RETURN QUERY
    WITH import_entries AS (
        SELECT
            bi.id as imp_id,
            bi.file_name as fname,
            jsonb_array_elements_text(bi.journal_entry_ids::jsonb) as entry_id
        FROM bank_imports bi
        WHERE bi.journal_entry_ids IS NOT NULL
    ),
    broken_imports AS (
        SELECT ie.imp_id, ie.fname, COUNT(*) as total, COUNT(je.id) as existing
        FROM import_entries ie
        LEFT JOIN journal_entries je ON je.id = ie.entry_id::integer
        GROUP BY ie.imp_id, ie.fname
        HAVING COUNT(je.id) = 0
    )
    DELETE FROM bank_imports bi
    WHERE bi.id IN (SELECT imp_id FROM broken_imports)
    RETURNING
        'DELETED'::TEXT,
        bi.id,
        bi.file_name,
        format('%s journal entries missing', bi.created_journal_entries)::TEXT;

    -- Report partially broken imports
    RETURN QUERY
    WITH import_entries AS (
        SELECT
            bi.id as imp_id,
            bi.file_name as fname,
            jsonb_array_elements_text(bi.journal_entry_ids::jsonb) as entry_id
        FROM bank_imports bi
        WHERE bi.journal_entry_ids IS NOT NULL
    )
    SELECT
        'WARNING'::TEXT,
        ie.imp_id,
        ie.fname,
        format('%s of %s entries missing',
            COUNT(*) - COUNT(je.id),
            COUNT(*)
        )::TEXT
    FROM import_entries ie
    LEFT JOIN journal_entries je ON je.id = ie.entry_id::integer
    GROUP BY ie.imp_id, ie.fname
    HAVING COUNT(*) - COUNT(je.id) > 0
    AND COUNT(je.id) > 0;
END;
$$ LANGUAGE plpgsql;

-- Run maintenance
SELECT * FROM cleanup_orphaned_bank_imports();
```

## Issue Fixed: 2025-10-11

### Problem
- Bank import #1: All 82 journal entries (IDs 712-793) were deleted
- Bank import #2: All 82 journal entries (IDs 795-876) were deleted
- UI showed error messages for each missing entry

### Solution Applied

**Step 1: Clean up existing broken imports (SQL)**
```sql
DELETE FROM bank_imports WHERE id IN (1, 2);
```

**Step 2: Prevent future issues (Backend filter)**

Updated `backend/src/graphql/bank_resolvers.rs` to automatically filter out imports where ALL journal entries have been deleted:

```rust
async fn bank_imports(...) -> FieldResult<Vec<BankImportModel>> {
    // ... fetch imports ...

    // Filter out imports where ALL journal entries have been deleted
    for import in imports {
        if let Some(entry_ids_value) = &import.journal_entry_ids {
            let entry_ids: Vec<i32> = serde_json::from_value(entry_ids_value.clone())
                .unwrap_or_default();

            // Check if at least one journal entry exists
            let exists = db.query_one(...).await;

            if exists {
                filtered_imports.push(import);  // Keep import
            }
            // else: skip import (all entries deleted)
        }
    }

    Ok(filtered_imports)
}
```

**How it works:**
- For each import, check if ANY journal entry from `journal_entry_ids` still exists
- If at least ONE entry exists → show import
- If ALL entries are deleted → hide import automatically
- If no entries referenced → show import (empty import)

### Result
- 2 broken imports removed manually
- 1 healthy import remains (import #3 with 82 valid entries)
- **Backend now automatically filters out broken imports**
- UI no longer shows imports with completely deleted journal entries
- No more error messages for missing entries

## Monitoring Query

Add this to your monitoring dashboard:

```sql
-- Check health of all bank imports
WITH import_entries AS (
    SELECT
        bi.id as import_id,
        bi.file_name,
        bi.imported_at,
        bi.created_journal_entries,
        jsonb_array_elements_text(bi.journal_entry_ids::jsonb) as entry_id
    FROM bank_imports bi
    WHERE bi.journal_entry_ids IS NOT NULL
)
SELECT
    ie.import_id,
    ie.file_name,
    ie.imported_at,
    ie.created_journal_entries,
    COUNT(je.id) as existing_entries,
    COUNT(*) - COUNT(je.id) as missing_entries,
    CASE
        WHEN COUNT(je.id) = COUNT(*) THEN '✅ Healthy'
        WHEN COUNT(je.id) = 0 THEN '❌ Broken'
        ELSE '⚠️  Partial'
    END as status
FROM import_entries ie
LEFT JOIN journal_entries je ON je.id = ie.entry_id::integer
GROUP BY ie.import_id, ie.file_name, ie.imported_at, ie.created_journal_entries
ORDER BY ie.imported_at DESC;
```

## See Also

- [Bank Module Documentation](./BANK-MODULE.md)
- [Database Maintenance](./DATABASE-MAINTENANCE.md)
- [Data Integrity Guidelines](./DATA-INTEGRITY.md)

---

**Last Updated:** 11 октомври 2025
**Status:** Resolved
**Affected Imports:** 2 (deleted)
**Healthy Imports:** 1 (verified)
