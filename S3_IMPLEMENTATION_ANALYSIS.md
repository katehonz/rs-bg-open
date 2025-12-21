# S3/Object Storage Implementation Analysis

## Overview
This is a comprehensive analysis of how S3/object storage is implemented in the rs-ac-bg (Rust Accounting Backend) application for database backup management.

---

## 1. Configuration & Credentials Setup

### Configuration File Format (`configdb.json` or `configdb.example.json`)
The object storage configuration is defined as part of the main application config:

```json
{
  "object_storage": {
    "enabled": false,
    "endpoint": "https://your-object-storage-endpoint",
    "access_key": "YOUR_ACCESS_KEY",
    "secret_key": "YOUR_SECRET_KEY",
    "region": "eu-central",
    "bucket": "your-bucket-name",
    "prefix": "rs-ac-bg/backups",
    "force_path_style": true
  }
}
```

### Configuration Structure (`ObjectStorageConfig`)
Location: `backend/src/config.rs` (lines 44-54)

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct ObjectStorageConfig {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub bucket: String,
    pub prefix: Option<String>,
    pub force_path_style: Option<bool>,
}
```

**Key Configuration Properties:**
- **enabled**: Boolean flag to enable/disable object storage
- **endpoint**: S3-compatible endpoint URL (supports custom endpoints like Hetzner, Minio, etc.)
- **access_key**: AWS-style access key ID
- **secret_key**: AWS-style secret key
- **region**: AWS region code (e.g., "eu-central" for Hetzner)
- **bucket**: Target S3 bucket name
- **prefix**: Optional path prefix for organizing backups (e.g., "rs-ac-bg/backups")
- **force_path_style**: Forces path-style S3 URLs instead of virtual-hosted-style (important for non-AWS S3 services)

### Dependencies
From `backend/Cargo.toml` (line 30):
```toml
aws-sdk-s3 = { version = "1.37.0", features = ["rustls"] }
```

The application uses the official AWS SDK for Rust with rustls for TLS connections.

---

## 2. Client Initialization & Configuration Flow

### Building S3 Client
Location: `backend/src/services/maintenance.rs` (lines 749-784)

```rust
async fn build_s3_client(&self, cfg: &ObjectStorageConfig) -> Result<Option<S3Client>> {
    if !cfg.enabled {
        return Ok(None);
    }

    if cfg.access_key.trim().is_empty() || cfg.secret_key.trim().is_empty() {
        return Ok(None);
    }

    let region = Region::new(cfg.region.clone());
    let credentials = Credentials::new(
        cfg.access_key.clone(),
        cfg.secret_key.clone(),
        None,
        None,
        "rs-ac-bg-maintenance",
    );

    let mut builder = S3ConfigBuilder::new()
        .region(region)
        .credentials_provider(credentials)
        .behavior_version(BehaviorVersion::latest());

    // Support for custom endpoints (Hetzner, Minio, etc.)
    if let Some(endpoint) = &cfg.endpoint {
        if !endpoint.trim().is_empty() {
            builder = builder.endpoint_url(endpoint);
        }
    }

    // Force path-style URLs (needed for non-AWS S3 services)
    if cfg.force_path_style.unwrap_or(true) {
        builder = builder.force_path_style(true);
    }

    let config = builder.build();
    Ok(Some(S3Client::from_conf(config)))
}
```

**Key Features:**
- Validates that storage is enabled
- Validates that credentials are not empty
- Creates custom region from configuration
- Builds credentials with provider name "rs-ac-bg-maintenance"
- Supports custom endpoints (Hetzner Object Storage, Minio, etc.)
- Forces path-style URLs for compatibility with non-AWS providers

---

## 3. Configuration Persistence & Updates

### Database Storage
Settings are persisted in the `contragent_settings` table using key-value pairs:

Settings Keys:
- `maintenance.object_storage.enabled`
- `maintenance.object_storage.endpoint`
- `maintenance.object_storage.access_key`
- `maintenance.object_storage.secret_key`
- `maintenance.object_storage.region`
- `maintenance.object_storage.bucket`
- `maintenance.object_storage.prefix`
- `maintenance.object_storage.force_path_style`

Location: `backend/src/services/maintenance.rs` (lines 499-571)

```rust
async fn load_object_storage_state(
    &self,
    db: &DatabaseConnection,
) -> Result<ObjectStorageState> {
    let mut config = self.base_object_storage_config();
    let mut has_secret_key = !config.secret_key.trim().is_empty();

    let settings = ContragentSetting::find()
        .filter(contragent_setting::Column::Key.starts_with("maintenance.object_storage."))
        .select_only()
        .column(contragent_setting::Column::Key)
        .column(contragent_setting::Column::Value)
        .into_model::<SettingRow>()
        .all(db)
        .await?;
    
    // Parse settings into config...
}
```

### Updating Settings
Location: `backend/src/services/maintenance.rs` (lines 573-677)

The `update_object_storage_settings()` method allows runtime updates:
1. Loads current settings
2. Updates with new values
3. Persists back to database
4. Uses `upsert_setting()` for safe create/update operations

**Special Handling:**
- Secret keys are marked as `encrypted = true` in database
- Empty values are deleted rather than stored
- Updates maintain separation between base config and database overrides

---

## 4. File Upload Logic (Database Backups)

### Backup Creation & Upload Flow
Location: `backend/src/services/maintenance.rs` (lines 134-224)

**Complete Flow:**

1. **Create local backup**
   - Creates `backups/` directory
   - Generates timestamp: `{database}_{YYYYMMDD_HHMMSS}.dump`
   - Executes `pg_dump` with custom format (`-F c`)
   - Captures file metadata (size, creation time)

2. **Load object storage configuration**
   - Merges base config with database overrides
   - Validates credentials are present

3. **Upload to object storage** (if configured)
   - Calls `upload_to_object_storage()`
   - If successful, deletes local backup
   - Updates backup metadata with S3 object key and storage location

### Upload Implementation
Location: `backend/src/services/maintenance.rs` (lines 786-821)

```rust
async fn upload_to_object_storage(
    &self,
    file_path: &Path,
    file_name: &str,
    cfg: &ObjectStorageConfig,
) -> Result<Option<String>> {
    let Some(client) = self.build_s3_client(cfg).await? else {
        return Ok(None);
    };

    if cfg.bucket.trim().is_empty() {
        return Ok(None);
    }

    // Build S3 object key with optional prefix
    let mut key = file_name.to_string();
    if let Some(prefix) = &cfg.prefix {
        if !prefix.trim().is_empty() {
            key = format!("{}/{}", prefix.trim_end_matches('/'), file_name);
        }
    }

    // Upload backup file
    let body = ByteStream::from_path(file_path).await.with_context(|| {
        format!("Неуспешно четене на архивния файл {}", file_path.display())
    })?;

    client
        .put_object()
        .bucket(&cfg.bucket)
        .key(&key)
        .body(body)
        .content_type("application/octet-stream")
        .send()
        .await?;

    Ok(Some(key))
}
```

**Upload Details:**
- Uses `ByteStream::from_path()` for efficient streaming upload
- Sets content type to `application/octet-stream`
- Respects prefix configuration for organizing backups
- Returns the S3 object key on success
- S3 client is built on-demand (no persistent connection pooling)

### Backup File Tracking
After successful upload, the backup record includes:
```rust
pub struct BackupFile {
    pub file_name: String,
    pub full_path: String,        // "s3://bucket-name/key" for remote backups
    pub size_bytes: u64,
    pub size_pretty: String,
    pub created_at: DateTime<Utc>,
    pub storage: BackupStorage,   // Local or ObjectStorage enum
    pub object_key: Option<String>, // S3 object key
}
```

---

## 5. Backup Retrieval & Download Logic

### Listing Remote Backups
Location: `backend/src/services/maintenance.rs` (lines 409-461)

```rust
async fn collect_remote_backups(&self, cfg: &ObjectStorageConfig) -> Result<Vec<BackupFile>> {
    let Some(client) = self.build_s3_client(cfg).await? else {
        return Ok(Vec::new());
    }

    if cfg.bucket.trim().is_empty() {
        return Ok(Vec::new());
    }

    // List objects with optional prefix filtering
    let mut request = client.list_objects_v2().bucket(&cfg.bucket);
    if let Some(prefix) = &cfg.prefix {
        if !prefix.trim().is_empty() {
            request = request.prefix(prefix);
        }
    }

    let response = request.max_keys(50).send().await?;
    let mut backups = Vec::new();

    // Filter to only .dump files and extract metadata
    for object in response.contents() {
        let key = match object.key() {
            Some(k) => k,
            None => continue,
        };

        if !key.ends_with(".dump") {
            continue;
        }

        let size = object.size().unwrap_or_default() as u64;
        let last_modified = object
            .last_modified()
            .and_then(|dt| dt.fmt(DateTimeFormat::DateTime).ok())
            .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| Utc::now());

        let file_name = key.split('/').last().unwrap_or(key).to_string();
        let full_path = format!("s3://{}/{}", cfg.bucket, key);

        backups.push(BackupFile {
            file_name,
            full_path,
            size_bytes: size,
            size_pretty: format_bytes(size),
            created_at: last_modified,
            storage: BackupStorage::ObjectStorage,
            object_key: Some(key.to_string()),
        });
    }

    Ok(backups)
}
```

**Features:**
- Limits results to 50 objects for pagination
- Filters to only `.dump` files
- Uses prefix for scoped listing
- Converts S3 timestamps to UTC DateTime
- Maintains object key for download operations

### Download for Restore
Location: `backend/src/services/maintenance.rs` (lines 823-856)

```rust
async fn download_from_object_storage(
    &self,
    key: &str,
    cfg: &ObjectStorageConfig,
) -> Result<PathBuf> {
    let Some(client) = self.build_s3_client(cfg).await? else {
        bail!("Object storage не е конфигуриран или липсват ключове");
    };

    if cfg.bucket.trim().is_empty() {
        bail!("Не е зададен bucket за object storage");
    }

    // Download from S3
    let response = client
        .get_object()
        .bucket(&cfg.bucket)
        .key(key)
        .send()
        .await?;

    // Stream to bytes
    let bytes = response.body.collect().await?.into_bytes();

    // Write to temporary file
    let temp_path = std::env::temp_dir().join(format!(
        "restore-{}-{}.dump",
        Utc::now().format("%Y%m%d%H%M%S"),
        Uuid::new_v4()
    ));

    let mut file = fs::File::create(&temp_path).await?;
    file.write_all(&bytes).await?;
    file.flush().await?;

    Ok(temp_path)
}
```

**Features:**
- Downloads to system temp directory
- Generates unique temp file names using UUID
- Fully streams file to memory before writing
- Flushes file to ensure complete write

### Restore Process
Location: `backend/src/services/maintenance.rs` (lines 284-348)

1. Determines source (local file or S3 object key)
2. Downloads from S3 if needed to temporary location
3. Executes `pg_restore` with appropriate flags
4. Cleans up temporary file after restore completes

---

## 6. Error Handling & Logging

### Error Handling Strategy
The entire S3 implementation uses the `anyhow::Result` type for error propagation:

**Validation Errors:**
- Storage disabled → Returns `Ok(None)` (graceful bypass)
- Missing credentials → Returns `Ok(None)` (graceful bypass)
- Empty bucket name → Returns `Ok(None)` (graceful bypass)
- Invalid file paths → Returns with context message

**Runtime Errors:**
- File read failures → Propagated with context
- S3 API failures → Propagated from AWS SDK
- Temporary file operations → Propagated with context

**Example Error Handling (Upload):**
```rust
let body = ByteStream::from_path(file_path).await.with_context(|| {
    format!("Неуспешно четене на архивния файл {}", file_path.display())
})?;

client
    .put_object()
    .bucket(&cfg.bucket)
    .key(&key)
    .body(body)
    .content_type("application/octet-stream")
    .send()
    .await?; // AWS SDK errors propagated here
```

### Logging Strategy
Location: `backend/src/services/maintenance.rs`

**Warning Logs:**
- Lines 206-211: Logs when local backup deletion fails after successful S3 upload (non-fatal)

**Error Context Messages (Bulgarian):**
- "Неуспешно четене на архивния файл" - Failed to read archive file
- "Неуспешно четене на метаданни за" - Failed to read metadata
- "Object storage не е конфигуриран или липсват ключове" - Object storage not configured or missing credentials
- "Не е зададен bucket за object storage" - Object storage bucket not configured

**No explicit error logging** - Errors are propagated to GraphQL resolvers which handle response formatting.

---

## 7. Compatibility & Provider Support

### S3-Compatible Services
The implementation supports any S3-compatible object storage:

**Configuration Examples:**

**Hetzner Object Storage:**
```json
{
  "endpoint": "https://eu-central.s3.hetzner.cloud",
  "region": "eu-central",
  "force_path_style": true
}
```

**MinIO:**
```json
{
  "endpoint": "http://minio-server:9000",
  "region": "us-east-1",
  "force_path_style": true
}
```

**AWS S3:**
```json
{
  "endpoint": null,  // Use default AWS endpoint
  "region": "us-east-1",
  "force_path_style": false  // Or true, both work
}
```

**Mega.nz (NOT DIRECTLY SUPPORTED):**
The current implementation uses AWS S3 SDK and doesn't have built-in Mega.nz support. Mega.nz would require:
- A separate SDK or API client
- Custom upload/download logic
- Different credential format

---

## 8. GraphQL API Integration

### Query: Get Status
Location: `backend/src/graphql/maintenance_resolver.rs` (lines 215-223)

```rust
async fn database_maintenance_status(
    &self,
    ctx: &Context<'_>,
) -> FieldResult<MaintenanceStatus> {
    let db = ctx.data::<Arc<DatabaseConnection>>()?;
    let service = ctx.data::<Arc<MaintenanceService>>()?;
    let status = service.status(db.as_ref()).await?;
    Ok(MaintenanceStatus::from(status))
}
```

Returns:
- Database name and size
- Directory path for local backups
- List of 5 most recent backups (local + remote)
- Object storage enabled status

### Query: Get Settings
```rust
async fn object_storage_settings(
    &self,
    ctx: &Context<'_>,
) -> FieldResult<ObjectStorageSettings> {
    let db = ctx.data::<Arc<DatabaseConnection>>()?;
    let service = ctx.data::<Arc<MaintenanceService>>()?;
    let state = service.get_object_storage_settings(db.as_ref()).await?;
    Ok(ObjectStorageSettings::from(state))
}
```

Returns:
- All configuration values
- Boolean flag `has_secret_key` (doesn't expose actual secret)

### Mutation: Create Backup
```rust
async fn create_database_backup(&self, ctx: &Context<'_>) -> FieldResult<BackupPayload> {
    let db = ctx.data::<Arc<DatabaseConnection>>()?;
    let service = ctx.data::<Arc<MaintenanceService>>()?;
    let summary = service.create_backup(db.as_ref()).await?;
    Ok(BackupPayload::from(summary))
}
```

Returns:
- Backup file details
- Duration in milliseconds
- Remote object key (if uploaded)

### Mutation: Update Settings
```rust
async fn update_object_storage_settings(
    &self,
    ctx: &Context<'_>,
    input: UpdateObjectStorageSettingsInput,
) -> FieldResult<ObjectStorageSettings> {
    // Validates and persists settings to database
    // Returns updated settings
}
```

### Mutation: Restore Database
```rust
async fn restore_database(
    &self,
    ctx: &Context<'_>,
    input: RestoreDatabaseInput,
) -> FieldResult<RestorePayload> {
    // Accepts storage_type (Local or ObjectStorage) and identifier
    // For remote: identifier is the S3 object key
    // For local: identifier is the file path
}
```

---

## 9. Complete Data Flow Diagram

### Backup Creation with S3 Upload:
```
GraphQL Mutation: createDatabaseBackup
    ↓
MaintenanceService::create_backup()
    ├→ Create backups/ directory
    ├→ Execute pg_dump command
    │   └→ Creates: backups/{db}_{timestamp}.dump
    ├→ Load object storage config (from DB + config file)
    ├→ If object storage enabled:
    │   ├→ build_s3_client() [with custom endpoint support]
    │   ├→ upload_to_object_storage()
    │   │   ├→ Build S3 object key with prefix
    │   │   ├→ Stream file via ByteStream
    │   │   ├→ PUT request to bucket/key
    │   │   └→ Return object key
    │   └→ Delete local backup file (keep only in S3)
    ├→ Return BackupSummary
    │   ├→ BackupFile with S3 path
    │   ├→ Duration
    │   └→ Remote object key
    ↓
GraphQL Response to client
```

### Backup Listing with Remote Discovery:
```
GraphQL Query: databaseMaintenanceStatus
    ↓
MaintenanceService::status()
    ├→ collect_local_backups() → Lists backups/ directory
    ├→ If object storage enabled:
    │   ├→ Load config from DB
    │   ├→ build_s3_client()
    │   ├→ collect_remote_backups()
    │   │   ├→ LIST_OBJECTS_V2 with prefix filter
    │   │   ├→ Filter to *.dump files
    │   │   └→ Parse metadata (size, last_modified)
    │   └→ Merge with local backups
    ├→ Sort by date (newest first)
    ├→ Truncate to 5 most recent
    ↓
Return MaintenanceStatus with backup list
```

### Restore from Object Storage:
```
GraphQL Mutation: restoreDatabase(storage_type: ObjectStorage, identifier: key)
    ↓
MaintenanceService::restore_database()
    ├→ Load config from DB
    ├→ download_from_object_storage(key)
    │   ├→ build_s3_client()
    │   ├→ GET_OBJECT from bucket/key
    │   ├→ Stream body to bytes
    │   ├→ Write to temp file in /tmp/
    │   └→ Return temp file path
    ├→ Execute pg_restore on temp file
    ├→ Delete temp file
    ↓
Return RestoreSummary with duration and timestamp
```

---

## 10. Security Considerations

### Secrets Management
- **Secret Key Storage**: Stored in `contragent_settings` table with `encrypted = true` flag
- **Credential Validation**: Required fields are validated before use
- **Temporary Files**: Backup downloads use system temp directory with UUID-based names
- **No Credentials in Logs**: Error messages use context, not credential details

### Permissions & Access
- Settings update requires database access (GraphQL context validation)
- Object storage settings are shared across all companies (not per-company)
- No fine-grained access control per-backup

### Best Practices Implemented
- Path-style URL support for non-AWS providers
- Custom endpoint configuration
- Graceful degradation (missing credentials → no upload, not error)
- Transient S3 clients (no persistent connections to manage)

---

## 11. Summary Table

| Aspect | Implementation Details |
|--------|------------------------|
| **Library** | `aws-sdk-s3` v1.37.0 with rustls |
| **Configuration Storage** | `configdb.json` (startup) + `contragent_settings` table (runtime) |
| **Supported Providers** | AWS S3, Hetzner Object Storage, MinIO, any S3-compatible service |
| **Upload Method** | Direct streaming via `ByteStream::from_path()` |
| **Download Method** | Full download to system temp, then restore |
| **Backup Format** | PostgreSQL custom format (`.dump`) |
| **Metadata Tracked** | File name, size, creation date, S3 object key |
| **Error Handling** | `anyhow::Result` with context messages in Bulgarian |
| **Logging** | Minimal; warnings for non-fatal issues |
| **API Interface** | GraphQL mutations/queries |
| **Local Backup Path** | `backups/{database}_{timestamp}.dump` |
| **Remote Path Template** | `{bucket}/{prefix}/{file_name}` |
| **Prefix Support** | Yes (for organizing backups) |
| **Path Style** | Configurable force path-style for non-AWS |
| **Concurrent Operations** | No connection pooling; clients built per-operation |

---

## 12. Potential Enhancements

Based on current implementation:

1. **Encryption**: Add at-rest encryption for backups
2. **Compression**: Compress backups before upload
3. **Versioning**: Enable S3 versioning for additional protection
4. **Lifecycle Policies**: Auto-expire old backups in S3
5. **Parallel Upload**: Multipart upload for large backups
6. **Connection Pooling**: Persistent S3 client for improved performance
7. **Monitoring**: CloudWatch/metrics integration
8. **Mega.nz Support**: Would require separate SDK
9. **Per-Company Settings**: Currently global; could be per-company
10. **Audit Logging**: Track who created/restored backups and when

