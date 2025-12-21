# S3 Implementation - Quick Reference Guide

## File Paths

| Component | Location |
|-----------|----------|
| Main S3 Implementation | `backend/src/services/maintenance.rs` |
| Configuration Structures | `backend/src/config.rs` |
| GraphQL API | `backend/src/graphql/maintenance_resolver.rs` |
| Settings Entity | `backend/src/entities/contragent_setting.rs` |
| Config File | `configdb.json` or `configdb.example.json` |
| Dependencies | `backend/Cargo.toml` (line 30) |

---

## Key Concepts

### 1. Architecture
- **S3 Library**: AWS SDK for Rust v1.37.0 with rustls
- **Provider Type**: Any S3-compatible object storage
- **Use Case**: PostgreSQL database backup management
- **Configuration Storage**: Dual-layer (config file + database)

### 2. Supported Providers
- AWS S3
- Hetzner Object Storage
- MinIO
- Any S3-compatible API endpoint

**NOT Supported:**
- Mega.nz (would require different SDK)
- Google Cloud Storage (different API)
- Azure Blob Storage (different API)

### 3. Configuration Keys (Database Storage)
```
maintenance.object_storage.enabled
maintenance.object_storage.endpoint
maintenance.object_storage.access_key
maintenance.object_storage.secret_key
maintenance.object_storage.region
maintenance.object_storage.bucket
maintenance.object_storage.prefix
maintenance.object_storage.force_path_style
```

---

## Configuration Setup

### Step 1: Static Configuration (configdb.json)
```json
"object_storage": {
  "enabled": false,
  "endpoint": "https://endpoint-url",
  "access_key": "your-key",
  "secret_key": "your-secret",
  "region": "eu-central",
  "bucket": "bucket-name",
  "prefix": "rs-ac-bg/backups",
  "force_path_style": true
}
```

### Step 2: Runtime Configuration (via GraphQL)
Use mutation `updateObjectStorageSettings` to update via API.

### Step 3: Load in Application
- Loaded from `configdb.json` on startup
- Overridable via database (runtime changes)
- Validated before use (empty check, enabled flag)

---

## S3 API Usage

### Upload (PUT)
```rust
client.put_object()
    .bucket(&bucket_name)
    .key(&object_key)
    .body(ByteStream::from_path(file_path))
    .content_type("application/octet-stream")
    .send()
    .await?
```

### Download (GET)
```rust
client.get_object()
    .bucket(&bucket_name)
    .key(&object_key)
    .send()
    .await?
```

### List (LIST)
```rust
client.list_objects_v2()
    .bucket(&bucket_name)
    .prefix(&prefix)
    .max_keys(50)
    .send()
    .await?
```

---

## Data Flow

### Create Backup
```
GraphQL: createDatabaseBackup
  ↓
pg_dump → backups/{db}_{timestamp}.dump
  ↓
Upload to S3 (if enabled)
  ↓
Delete local file (if upload succeeds)
  ↓
Return backup metadata
```

### Restore Database
```
GraphQL: restoreDatabase
  ↓
Download from S3 (if remote storage)
  ↓
Save to /tmp/ with UUID name
  ↓
pg_restore from temp file
  ↓
Delete temp file
  ↓
Return restore summary
```

### List Backups
```
GraphQL: databaseMaintenanceStatus
  ↓
List local backups from backups/
  ↓
List remote backups from S3 (if enabled)
  ↓
Merge, sort, return top 5 most recent
```

---

## Error Handling

### Graceful Degradation
| Condition | Behavior |
|-----------|----------|
| Storage disabled | Skip upload, return success |
| Missing credentials | Skip upload, return success |
| Empty bucket | Skip upload, return success |
| Network error | Propagate error |
| File not found | Propagate error |

### Error Context Messages (Bulgarian)
- "Неуспешно четене на архивния файл" = Failed to read archive file
- "Object storage не е конфигуриран" = Object storage not configured
- "Не е зададен bucket" = Bucket not configured

---

## GraphQL Operations

### Query: Get Status
```graphql
query {
  databaseMaintenanceStatus {
    databaseName
    databaseSizePretty
    objectStorageEnabled
    recentBackups {
      fileName
      sizePretty
      storageType
      objectKey
    }
  }
}
```

### Query: Get Settings
```graphql
query {
  objectStorageSettings {
    enabled
    endpoint
    accessKey
    hasSecretKey
    region
    bucket
    prefix
    forcePathStyle
  }
}
```

### Mutation: Create Backup
```graphql
mutation {
  createDatabaseBackup {
    backup {
      fileName
      fullPath
      storageType
    }
    durationMs
    remoteObjectKey
  }
}
```

### Mutation: Update Settings
```graphql
mutation UpdateObjectStorage($input: UpdateObjectStorageSettingsInput!) {
  updateObjectStorageSettings(input: $input) {
    enabled
    endpoint
    accessKey
    region
    bucket
    prefix
    forcePathStyle
  }
}
```

### Mutation: Restore
```graphql
mutation Restore($input: RestoreDatabaseInput!) {
  restoreDatabase(input: $input) {
    source
    storageType
    durationMs
    startedAt
  }
}
```

---

## Testing Configurations

### AWS S3
```json
{
  "enabled": true,
  "endpoint": null,
  "access_key": "AKIAIOSFODNN7EXAMPLE",
  "secret_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
  "region": "us-east-1",
  "bucket": "my-bucket",
  "prefix": "backups",
  "force_path_style": false
}
```

### Hetzner Object Storage (Актуализирано 2025-11-05)
```json
{
  "enabled": true,
  "endpoint": "https://nbg1.your-objectstorage.com",
  "access_key": "YOUR_HETZNER_KEY",
  "secret_key": "YOUR_HETZNER_SECRET",
  "region": "eu-central",
  "bucket": "rsacbackup",
  "prefix": "backup",
  "force_path_style": false
}
```

**ВАЖНО:** Hetzner използва virtual-hosted-style URLs, затова:
- `force_path_style` трябва да е `false` (не `true`!)
- `endpoint` трябва да е базовият URL (не bucket.endpoint!)
- Генериран URL: `https://{bucket}.{endpoint}/{prefix}/{file}`
- Пример: `https://rsacbackup.nbg1.your-objectstorage.com/backup/accounting_20251105_232958.dump`

**Hetzner региони:**
- `nbg1.your-objectstorage.com` - Nuremberg
- `fsn1.your-objectstorage.com` - Falkenstein
- `hel1.your-objectstorage.com` - Helsinki

### MinIO Local
```json
{
  "enabled": true,
  "endpoint": "http://minio:9000",
  "access_key": "minioadmin",
  "secret_key": "minioadmin",
  "region": "us-east-1",
  "bucket": "backups",
  "prefix": "postgresql",
  "force_path_style": true
}
```

---

## Key Implementation Details

### Backup File Format
- Format: PostgreSQL custom format (pg_dump -F c)
- Extension: `.dump`
- Naming: `{database}_{YYYYMMDD_HHMMSS}.dump`

### S3 Object Keys
- Pattern: `{prefix}/{file_name}`
- Example: `rs-ac-bg/backups/rs_ac_bg_20240101_143022.dump`

### Temporary Files
- Location: System temp directory `/tmp/`
- Pattern: `restore-{YYYYMMDD_HHMMSS}-{UUID}.dump`
- Cleanup: Automatic after restore completes

### Metadata Tracking
- File name
- File size (human readable format)
- Creation/modification date (UTC)
- Storage type (Local/ObjectStorage enum)
- S3 object key

---

## Performance Characteristics

### Upload
- Streaming upload via ByteStream
- No size limits enforced (depends on S3 provider)
- Single PUT request per backup
- Content-Type: application/octet-stream

### Download
- Full file download to memory
- Written to temp file with fsync
- Single GET request per restore

### Listing
- Max 50 objects per LIST request
- Prefix filter applied server-side
- .dump extension filter applied client-side

### Concurrent Operations
- No connection pooling
- S3 clients built per-operation
- No rate limiting enforced

---

## Security Model

### Credentials
- Stored in `contragent_settings` table
- Secret key marked as `encrypted = true`
- Credentials never logged
- Validated before use

### Access Control
- Settings update requires database access
- No per-backup access control
- Settings shared across all companies
- GraphQL context validation only

### Secrets Exposure
- Secret key hidden in API responses (`has_secret_key` boolean only)
- Error messages don't include credentials
- Temporary files use UUID-based names

---

## Database Schema (Relevant Parts)

### Table: `contragent_settings`
```sql
CREATE TABLE contragent_settings (
  id BIGSERIAL PRIMARY KEY,
  key VARCHAR(255) NOT NULL,
  value TEXT,
  description TEXT,
  encrypted BOOLEAN NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);
```

### Settings Entries
Each setting is a row with:
- `key`: maintenance.object_storage.*
- `value`: Configuration value
- `encrypted`: true for secret_key, false for others
- `created_at`, `updated_at`: Timestamps

---

## Environment Variables (Optional)

From `config.rs`, can override via env:
- `DATABASE_URL`: Override database connection
- `HOST`: Server host (default: 127.0.0.1)
- `PORT`: Server port (default: 8080)

Object storage settings must come from config file or database.

---

## Dependencies Tree

```
aws-sdk-s3 (v1.37.0)
├── aws-smithy-runtime (async HTTP client)
├── aws-smithy-types (AWS type definitions)
├── tokio (async runtime)
└── rustls (TLS provider)

Supporting crates:
├── sea-orm (database ORM)
├── async-graphql (GraphQL framework)
├── chrono (date/time handling)
├── uuid (unique identifiers)
└── anyhow (error handling)
```

---

## Troubleshooting Checklist

| Issue | Solution |
|-------|----------|
| Upload fails | Check credentials, bucket name, endpoint, network |
| Download fails | Check object exists, credentials, network |
| List returns empty | Check bucket, prefix, .dump filter |
| Settings not persist | Verify database connection |
| Missing secret key on read | Expected behavior (only has_secret_key returned) |
| Connection timeout | Check endpoint URL, network firewall |
| Authentication error | Validate access_key and secret_key |
| **"dispatch failure" error** | **Липсваща bucket конфигурация - виж раздела по-долу** |

### Често срещана грешка: "dispatch failure" при зареждане на S3 настройки

**Симптоми:**
- Frontend показва: `GraphQL request failed: Error: dispatch failure`
- Грешка: `Грешка при зареждане на информацията за базата данни: dispatch failure`
- Не можете да видите/редактирате S3 настройките в интерфейса

**Причина:**
Липсва bucket конфигурацията в базата данни. S3 изисква **всички 8 настройки**:
1. `enabled`
2. `endpoint`
3. `access_key`
4. `secret_key`
5. `region`
6. `bucket` ← Обикновено това липсва!
7. `prefix`
8. `force_path_style`

**Решение:**
```sql
-- Проверете колко settings има (трябва да са 8):
SELECT COUNT(*) FROM contragent_settings
WHERE key LIKE 'maintenance.object_storage%';

-- Проверете кои settings липсват:
SELECT key, value FROM contragent_settings
WHERE key LIKE 'maintenance.object_storage%'
ORDER BY key;

-- Добавете липсващия bucket:
INSERT INTO contragent_settings (key, value, description, encrypted, created_at, updated_at)
VALUES ('maintenance.object_storage.bucket', 'rsacbackup', 'Hetzner S3 bucket', false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = CURRENT_TIMESTAMP;

-- Рестартирайте backend-а:
docker compose restart accounting-service
```

**Проверка след fix:**
```bash
# Трябва да върне 8 реда:
docker compose exec db psql -U app -d accounting -c "SELECT key FROM contragent_settings WHERE key LIKE 'maintenance.object_storage%' ORDER BY key;"
```

---

## Quick Start

1. **Enable object storage** in configdb.json:
   ```json
   "object_storage": {
     "enabled": true,
     "endpoint": "https://your-s3-endpoint",
     "access_key": "your-key",
     "secret_key": "your-secret",
     "region": "your-region",
     "bucket": "your-bucket"
   }
   ```

2. **Create a backup**:
   ```graphql
   mutation { createDatabaseBackup { ... } }
   ```

3. **Verify it uploaded**:
   ```graphql
   query { databaseMaintenanceStatus { recentBackups { storageType objectKey } } }
   ```

4. **To update settings at runtime**:
   ```graphql
   mutation { updateObjectStorageSettings(input: {...}) { ... } }
   ```

5. **To restore from remote**:
   ```graphql
   mutation { restoreDatabase(input: { storageType: ObjectStorage, identifier: "key" }) { ... } }
   ```

