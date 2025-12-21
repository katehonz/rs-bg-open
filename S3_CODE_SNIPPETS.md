# S3 Implementation - Code Snippets Reference

## Key File Locations

```
├── backend/src/
│   ├── config.rs                                    # Config struct definitions
│   ├── services/maintenance.rs                      # Main S3 implementation
│   ├── entities/contragent_setting.rs              # Settings persistence model
│   └── graphql/maintenance_resolver.rs             # GraphQL API endpoints
├── configdb.example.json                           # Configuration template
└── backend/Cargo.toml                              # Dependencies (aws-sdk-s3)
```

---

## 1. Configuration Structures

### From `backend/src/config.rs`

**ObjectStorageConfig Definition (lines 44-54):**
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

**ConfigFile contains object_storage field (lines 57-63):**
```rust
#[derive(Debug, Deserialize, Clone)]
pub struct ConfigFile {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub initial_setup: InitialSetupConfig,
    pub object_storage: Option<ObjectStorageConfig>,  // <-- HERE
}
```

**Config::object_storage() getter (lines 221-226):**
```rust
pub fn object_storage(&self) -> Option<&ObjectStorageConfig> {
    self.config_file
        .object_storage
        .as_ref()
        .filter(|cfg| cfg.enabled)
}
```

---

## 2. Maintenance Service Structures

### From `backend/src/services/maintenance.rs`

**BackupStorage Enum (lines 24-37):**
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackupStorage {
    Local,
    ObjectStorage,
}

impl BackupStorage {
    pub fn as_str(&self) -> &'static str {
        match self {
            BackupStorage::Local => "LOCAL",
            BackupStorage::ObjectStorage => "OBJECT_STORAGE",
        }
    }
}
```

**BackupFile Struct (lines 39-48):**
```rust
#[derive(Clone, Debug)]
pub struct BackupFile {
    pub file_name: String,
    pub full_path: String,
    pub size_bytes: u64,
    pub size_pretty: String,
    pub created_at: DateTime<Utc>,
    pub storage: BackupStorage,
    pub object_key: Option<String>,
}
```

**ObjectStorageState (lines 94-98):**
```rust
#[derive(Clone, Debug)]
pub struct ObjectStorageState {
    pub config: ObjectStorageConfig,
    pub has_secret_key: bool,
}
```

**ObjectStorageSettingsInput (lines 100-110):**
```rust
#[derive(Clone, Debug)]
pub struct ObjectStorageSettingsInput {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub access_key: String,
    pub secret_key: Option<String>,
    pub region: String,
    pub bucket: String,
    pub prefix: Option<String>,
    pub force_path_style: bool,
}
```

**MaintenanceService Constructor (lines 119-128):**
```rust
impl MaintenanceService {
    pub fn new(config: Config) -> Self {
        let backups_dir = PathBuf::from("backups");
        let config_object_storage = config.config_file.object_storage.clone();
        Self {
            config,
            config_object_storage,
            backups_dir,
        }
    }
}
```

---

## 3. S3 Client Building

### From `backend/src/services/maintenance.rs` (lines 749-784)

```rust
async fn build_s3_client(&self, cfg: &ObjectStorageConfig) -> Result<Option<S3Client>> {
    // Check if enabled
    if !cfg.enabled {
        return Ok(None);
    }

    // Validate credentials exist
    if cfg.access_key.trim().is_empty() || cfg.secret_key.trim().is_empty() {
        return Ok(None);
    }

    // Create region and credentials
    let region = Region::new(cfg.region.clone());
    let credentials = Credentials::new(
        cfg.access_key.clone(),
        cfg.secret_key.clone(),
        None,
        None,
        "rs-ac-bg-maintenance",
    );

    // Build base configuration
    let mut builder = S3ConfigBuilder::new()
        .region(region)
        .credentials_provider(credentials)
        .behavior_version(BehaviorVersion::latest());

    // Add custom endpoint if provided (for Hetzner, Minio, etc)
    if let Some(endpoint) = &cfg.endpoint {
        if !endpoint.trim().is_empty() {
            builder = builder.endpoint_url(endpoint);
        }
    }

    // Force path-style URLs (for non-AWS S3 compatibility)
    if cfg.force_path_style.unwrap_or(true) {
        builder = builder.force_path_style(true);
    }

    // Build and return client
    let config = builder.build();
    Ok(Some(S3Client::from_conf(config)))
}
```

---

## 4. Upload Implementation

### From `backend/src/services/maintenance.rs` (lines 786-821)

```rust
async fn upload_to_object_storage(
    &self,
    file_path: &Path,
    file_name: &str,
    cfg: &ObjectStorageConfig,
) -> Result<Option<String>> {
    // Build client (returns None if not configured)
    let Some(client) = self.build_s3_client(cfg).await? else {
        return Ok(None);
    };

    // Validate bucket
    if cfg.bucket.trim().is_empty() {
        return Ok(None);
    }

    // Build object key with optional prefix
    let mut key = file_name.to_string();
    if let Some(prefix) = &cfg.prefix {
        if !prefix.trim().is_empty() {
            key = format!("{}/{}", prefix.trim_end_matches('/'), file_name);
        }
    }

    // Stream file from disk
    let body = ByteStream::from_path(file_path).await.with_context(|| {
        format!("Неуспешно четене на архивния файл {}", file_path.display())
    })?;

    // Upload to S3
    client
        .put_object()
        .bucket(&cfg.bucket)
        .key(&key)
        .body(body)
        .content_type("application/octet-stream")
        .send()
        .await?;

    // Return the object key
    Ok(Some(key))
}
```

---

## 5. Backup Creation (Integrated Flow)

### From `backend/src/services/maintenance.rs` (lines 134-224)

**Key sections:**

1. **Create dump file (lines 144-159):**
```rust
let mut command = Command::new("pg_dump");
command
    .env("PGPASSWORD", &db_cfg.password)
    .arg("-h")
    .arg(&db_cfg.host)
    .arg("-p")
    .arg(db_cfg.port.to_string())
    .arg("-U")
    .arg(&db_cfg.username)
    .arg("-d")
    .arg(&db_cfg.database)
    .arg("-F")
    .arg("c")  // Custom format
    .arg("-f")
    .arg(&full_path)
    .kill_on_drop(true);
```

2. **Upload if configured (lines 196-202):**
```rust
let remote_object_key = self
    .upload_to_object_storage(
        Path::new(&backup.full_path),
        &backup.file_name,
        &object_storage_state.config,
    )
    .await?;
```

3. **Clean up local file after successful upload (lines 204-217):**
```rust
if let Some(remote_key) = remote_object_key.as_ref() {
    if let Err(err) = fs::remove_file(&local_backup_path).await {
        tracing::warn!(
            "Неуспешно изтриване на локалния архив {}: {}",
            local_backup_path.display(),
            err
        );
    }

    backup.object_key = Some(remote_key.clone());
    backup.storage = BackupStorage::ObjectStorage;
    backup.full_path =
        format!("s3://{}/{}", object_storage_state.config.bucket, remote_key);
}
```

---

## 6. Listing Remote Backups

### From `backend/src/services/maintenance.rs` (lines 409-461)

```rust
async fn collect_remote_backups(&self, cfg: &ObjectStorageConfig) -> Result<Vec<BackupFile>> {
    // Build S3 client
    let Some(client) = self.build_s3_client(cfg).await? else {
        return Ok(Vec::new());
    }

    if cfg.bucket.trim().is_empty() {
        return Ok(Vec::new());
    }

    // Build list request with optional prefix
    let mut request = client.list_objects_v2().bucket(&cfg.bucket);
    if let Some(prefix) = &cfg.prefix {
        if !prefix.trim().is_empty() {
            request = request.prefix(prefix);
        }
    }

    // Execute request (limited to 50 objects)
    let response = request.max_keys(50).send().await?;
    let mut backups = Vec::new();

    // Process each object
    for object in response.contents() {
        let key = match object.key() {
            Some(k) => k,
            None => continue,
        };

        // Only include .dump files
        if !key.ends_with(".dump") {
            continue;
        }

        // Extract metadata
        let size = object.size().unwrap_or_default() as u64;
        let last_modified = object
            .last_modified()
            .and_then(|dt| dt.fmt(DateTimeFormat::DateTime).ok())
            .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| Utc::now());

        let file_name = key.split('/').last().unwrap_or(key).to_string();
        let full_path = format!("s3://{}/{}", cfg.bucket, key);

        // Add to results
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

---

## 7. Download for Restore

### From `backend/src/services/maintenance.rs` (lines 823-856)

```rust
async fn download_from_object_storage(
    &self,
    key: &str,
    cfg: &ObjectStorageConfig,
) -> Result<PathBuf> {
    // Build client
    let Some(client) = self.build_s3_client(cfg).await? else {
        bail!("Object storage не е конфигуриран или липсват ключове");
    };

    // Validate bucket
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

    // Stream to bytes in memory
    let bytes = response.body.collect().await?.into_bytes();

    // Generate temp file path
    let temp_path = std::env::temp_dir().join(format!(
        "restore-{}-{}.dump",
        Utc::now().format("%Y%m%d%H%M%S"),
        Uuid::new_v4()
    ));

    // Write temp file
    let mut file = fs::File::create(&temp_path).await?;
    file.write_all(&bytes).await?;
    file.flush().await?;

    Ok(temp_path)
}
```

---

## 8. Settings Persistence

### From `backend/src/services/maintenance.rs` (lines 499-571)

**Loading settings from database:**
```rust
async fn load_object_storage_state(
    &self,
    db: &DatabaseConnection,
) -> Result<ObjectStorageState> {
    let mut config = self.base_object_storage_config();
    let mut has_secret_key = !config.secret_key.trim().is_empty();

    #[derive(FromQueryResult)]
    struct SettingRow {
        key: String,
        value: Option<String>,
    }

    // Query settings from database
    let settings = ContragentSetting::find()
        .filter(contragent_setting::Column::Key.starts_with("maintenance.object_storage."))
        .select_only()
        .column(contragent_setting::Column::Key)
        .column(contragent_setting::Column::Value)
        .into_model::<SettingRow>()
        .all(db)
        .await?;

    // Parse settings and update config
    for setting in settings {
        let Some(value) = setting.value.clone() else {
            continue;
        };
        match setting.key.as_str() {
            "maintenance.object_storage.enabled" => {
                if let Ok(enabled) = value.parse::<bool>() {
                    config.enabled = enabled;
                }
            }
            "maintenance.object_storage.endpoint" => {
                config.endpoint = if value.trim().is_empty() {
                    None
                } else {
                    Some(value.clone())
                };
            }
            "maintenance.object_storage.access_key" => {
                config.access_key = value.clone();
            }
            "maintenance.object_storage.secret_key" => {
                config.secret_key = value.clone();
                has_secret_key = !value.trim().is_empty();
            }
            "maintenance.object_storage.region" => {
                config.region = value.clone();
            }
            "maintenance.object_storage.bucket" => {
                config.bucket = value.clone();
            }
            "maintenance.object_storage.prefix" => {
                config.prefix = if value.trim().is_empty() {
                    None
                } else {
                    Some(value.clone())
                };
            }
            "maintenance.object_storage.force_path_style" => {
                if let Ok(force) = value.parse::<bool>() {
                    config.force_path_style = Some(force);
                }
            }
            _ => {}
        }
    }

    Ok(ObjectStorageState {
        config,
        has_secret_key,
    })
}
```

---

## 9. Upserting Settings

### From `backend/src/services/maintenance.rs` (lines 679-747)

```rust
async fn upsert_setting(
    &self,
    db: &DatabaseConnection,
    key: &str,
    value: Option<String>,
    encrypted: bool,
    description: &str,
) -> Result<()> {
    let trimmed_value = value.map(|v| v.trim().to_string());

    // If value is empty, delete the setting
    if trimmed_value
        .as_deref()
        .map(|v| v.is_empty())
        .unwrap_or(false)
    {
        ContragentSetting::delete_many()
            .filter(contragent_setting::Column::Key.eq(key))
            .exec(db)
            .await?;
        return Ok(());
    }

    if let Some(val) = trimmed_value {
        // Check if setting exists
        #[derive(FromQueryResult)]
        struct SettingId {
            id: i64,
        }

        let existing = ContragentSetting::find()
            .filter(contragent_setting::Column::Key.eq(key))
            .select_only()
            .column(contragent_setting::Column::Id)
            .into_model::<SettingId>()
            .one(db)
            .await?;

        let now = Utc::now();
        if let Some(existing) = existing {
            // Update existing
            let active = ContragentSettingActiveModel {
                id: ActiveValue::Set(existing.id),
                key: ActiveValue::NotSet,
                value: ActiveValue::Set(Some(val)),
                description: ActiveValue::Set(Some(description.to_string())),
                encrypted: ActiveValue::Set(encrypted),
                created_at: ActiveValue::NotSet,
                updated_at: ActiveValue::Set(now),
            };
            active.update(db).await?;
        } else {
            // Insert new
            let active = ContragentSettingActiveModel {
                id: ActiveValue::NotSet,
                key: ActiveValue::Set(key.to_string()),
                value: ActiveValue::Set(Some(val)),
                description: ActiveValue::Set(Some(description.to_string())),
                encrypted: ActiveValue::Set(encrypted),
                created_at: ActiveValue::Set(now),
                updated_at: ActiveValue::Set(now),
            };
            active.insert(db).await?;
        }
    } else {
        ContragentSetting::delete_many()
            .filter(contragent_setting::Column::Key.eq(key))
            .exec(db)
            .await?;
    }

    Ok(())
}
```

---

## 10. GraphQL Resolvers

### From `backend/src/graphql/maintenance_resolver.rs`

**Create Backup (line 241-246):**
```rust
async fn create_database_backup(&self, ctx: &Context<'_>) -> FieldResult<BackupPayload> {
    let db = ctx.data::<Arc<DatabaseConnection>>()?;
    let service = ctx.data::<Arc<MaintenanceService>>()?;
    let summary = service.create_backup(db.as_ref()).await?;
    Ok(BackupPayload::from(summary))
}
```

**Get Settings (lines 225-233):**
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

**Update Settings (lines 275-286):**
```rust
async fn update_object_storage_settings(
    &self,
    ctx: &Context<'_>,
    input: UpdateObjectStorageSettingsInput,
) -> FieldResult<ObjectStorageSettings> {
    let db = ctx.data::<Arc<DatabaseConnection>>()?;
    let service = ctx.data::<Arc<MaintenanceService>>()?;
    let state = service
        .update_object_storage_settings(db.as_ref(), input.into())
        .await?;
    Ok(ObjectStorageSettings::from(state))
}
```

**Restore Database (lines 255-273):**
```rust
async fn restore_database(
    &self,
    ctx: &Context<'_>,
    input: RestoreDatabaseInput,
) -> FieldResult<RestorePayload> {
    let db = ctx.data::<Arc<DatabaseConnection>>()?;
    let service = ctx.data::<Arc<MaintenanceService>>()?;

    let request = RestoreRequest {
        storage: match input.storage_type {
            BackupStorageType::Local => BackupStorage::Local,
            BackupStorageType::ObjectStorage => BackupStorage::ObjectStorage,
        },
        identifier: input.identifier,
    };

    let summary = service.restore_database(db.as_ref(), request).await?;
    Ok(RestorePayload::from(summary))
}
```

---

## 11. Dependencies in Cargo.toml

### From `backend/Cargo.toml` (line 30)

```toml
aws-sdk-s3 = { version = "1.37.0", features = ["rustls"] }
aws-smithy-types = "1.3.2"
```

Additional required imports:
```rust
use aws_sdk_s3::config::{BehaviorVersion, Builder as S3ConfigBuilder, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;
use aws_smithy_types::date_time::Format as DateTimeFormat;
```

---

## 12. Configuration JSON Example

### From `configdb.example.json`

```json
{
  "database": {
    "host": "localhost",
    "port": 5432,
    "database": "rs_ac_bg",
    "username": "postgres",
    "password": "your_password_here",
    "max_connections": 10,
    "min_connections": 5,
    "connect_timeout": 10,
    "acquire_timeout": 10,
    "idle_timeout": 600
  },
  "server": {
    "host": "0.0.0.0",
    "port": 8080,
    "workers": 4,
    "enable_cors": true,
    "cors_origins": ["http://localhost:3000", "http://localhost:5173"]
  },
  "logging": {
    "level": "info",
    "file": "logs/backend.log",
    "stdout": true
  },
  "initial_setup": {
    "create_demo_company": true,
    "demo_company_name": "Демо Фирма ООД",
    "demo_company_eik": "123456789",
    "demo_company_vat": "BG123456789",
    "load_chart_of_accounts": true
  },
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

