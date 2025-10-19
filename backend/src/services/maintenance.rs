use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use aws_sdk_s3::config::{BehaviorVersion, Builder as S3ConfigBuilder, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;
use aws_smithy_types::date_time::Format as DateTimeFormat;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseBackend,
    DatabaseConnection, EntityTrait, FromQueryResult, IntoActiveModel, QueryFilter, QuerySelect,
    Statement, Value,
};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::Instant;
use uuid::Uuid;

use crate::config::{Config, ObjectStorageConfig};
use crate::entities::{contragent_setting, ContragentSetting, ContragentSettingActiveModel};

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

#[derive(Clone, Debug)]
pub struct BackupSummary {
    pub backup: BackupFile,
    pub duration: Duration,
    pub remote_object_key: Option<String>,
}

#[derive(Clone, Debug)]
pub struct MaintenanceStatus {
    pub database_name: String,
    pub database_size_bytes: i64,
    pub database_size_pretty: String,
    pub backups_directory: String,
    pub recent_backups: Vec<BackupFile>,
    pub object_storage_enabled: bool,
}

#[derive(Clone, Debug)]
pub struct OptimizationSummary {
    pub database_name: String,
    pub size_before_bytes: i64,
    pub size_before_pretty: String,
    pub size_after_bytes: i64,
    pub size_after_pretty: String,
    pub vacuum_ran: bool,
    pub analyze_ran: bool,
    pub reindex_ran: bool,
    pub duration: Duration,
}

#[derive(Clone, Debug)]
pub struct RestoreRequest {
    pub storage: BackupStorage,
    pub identifier: String,
}

#[derive(Clone, Debug)]
pub struct RestoreSummary {
    pub source: String,
    pub storage: BackupStorage,
    pub duration: Duration,
    pub started_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct ObjectStorageState {
    pub config: ObjectStorageConfig,
    pub has_secret_key: bool,
}

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

#[derive(Clone, Debug)]
pub struct MaintenanceService {
    config: Config,
    config_object_storage: Option<ObjectStorageConfig>,
    backups_dir: PathBuf,
}

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

    pub fn backups_dir(&self) -> &Path {
        &self.backups_dir
    }

    pub async fn create_backup(&self, db: &DatabaseConnection) -> Result<BackupSummary> {
        let db_cfg = &self.config.config_file.database;
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        fs::create_dir_all(&self.backups_dir)
            .await
            .context("Неуспешно създаване на директория за резервни копия")?;

        let file_name = format!("{}_{}.dump", db_cfg.database, timestamp);
        let full_path = self.backups_dir.join(&file_name);

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
            .arg("c")
            .arg("-f")
            .arg(&full_path)
            .kill_on_drop(true);

        let start = Instant::now();
        let status = command
            .status()
            .await
            .context("Неуспешно стартиране на pg_dump. Уверете се, че инструментът е инсталиран")?;

        if !status.success() {
            return Err(anyhow!(
                "pg_dump завърши с грешка (код: {}). Провери логовете за подробности",
                status.code().unwrap_or(-1)
            ));
        }

        let metadata = fs::metadata(&full_path)
            .await
            .with_context(|| format!("Неуспешно четене на метаданни за {}", full_path.display()))?;
        let created_at = metadata
            .modified()
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(|_| Utc::now());
        let size_bytes = metadata.len();

        let object_storage_state = self.load_object_storage_state(db).await?;

        let mut backup = BackupFile {
            file_name,
            full_path: full_path.to_string_lossy().to_string(),
            size_bytes,
            size_pretty: format_bytes(size_bytes),
            created_at,
            storage: BackupStorage::Local,
            object_key: None,
        };

        let local_backup_path = PathBuf::from(&backup.full_path);
        let remote_object_key = self
            .upload_to_object_storage(
                Path::new(&backup.full_path),
                &backup.file_name,
                &object_storage_state.config,
            )
            .await?;

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

        Ok(BackupSummary {
            backup,
            duration: start.elapsed(),
            remote_object_key,
        })
    }

    pub async fn status(&self, db: &DatabaseConnection) -> Result<MaintenanceStatus> {
        let database_name = self.config.config_file.database.database.clone();
        let size_before = self.fetch_database_size(db).await?;
        let object_storage_state = self.load_object_storage_state(db).await?;

        let mut backups = self.collect_local_backups().await?;

        if object_storage_state.config.enabled {
            let mut remote_backups = self
                .collect_remote_backups(&object_storage_state.config)
                .await?;
            backups.append(&mut remote_backups);
        }

        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        if backups.len() > 5 {
            backups.truncate(5);
        }

        Ok(MaintenanceStatus {
            database_name,
            database_size_bytes: size_before,
            database_size_pretty: format_bytes(size_before as u64),
            backups_directory: self.backups_dir.to_string_lossy().to_string(),
            recent_backups: backups,
            object_storage_enabled: object_storage_state.config.enabled,
        })
    }

    pub async fn optimize_database(&self, db: &DatabaseConnection) -> Result<OptimizationSummary> {
        let database_name = self.config.config_file.database.database.clone();
        let size_before = self.fetch_database_size(db).await?;
        let start = Instant::now();

        db.execute_unprepared("VACUUM (ANALYZE)")
            .await
            .context("Неуспешно изпълнение на VACUUM (ANALYZE)")?;

        let analyze_ran = db.execute_unprepared("ANALYZE").await.is_ok();

        let reindex_stmt = format!("REINDEX DATABASE \"{}\"", database_name);
        let reindex_ran = db.execute_unprepared(&reindex_stmt).await.is_ok();

        let size_after = self.fetch_database_size(db).await?;

        Ok(OptimizationSummary {
            database_name,
            size_before_bytes: size_before,
            size_before_pretty: format_bytes(size_before as u64),
            size_after_bytes: size_after,
            size_after_pretty: format_bytes(size_after as u64),
            vacuum_ran: true,
            analyze_ran,
            reindex_ran,
            duration: start.elapsed(),
        })
    }

    pub async fn restore_database(
        &self,
        db: &DatabaseConnection,
        request: RestoreRequest,
    ) -> Result<RestoreSummary> {
        let object_storage_state = self.load_object_storage_state(db).await?;
        let db_cfg = &self.config.config_file.database;
        let start = Instant::now();
        let started_at = Utc::now();

        let local_path = match request.storage {
            BackupStorage::Local => PathBuf::from(&request.identifier),
            BackupStorage::ObjectStorage => self
                .download_from_object_storage(&request.identifier, &object_storage_state.config)
                .await
                .context("Неуспешно изтегляне на архив от object storage")?,
        };

        if !local_path.exists() {
            return Err(anyhow!(
                "Архивният файл не беше намерен: {}",
                local_path.display()
            ));
        }

        let mut command = Command::new("pg_restore");
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
            .arg("--clean")
            .arg("--if-exists")
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg(local_path.to_string_lossy().to_string())
            .kill_on_drop(true);

        let status = command.status().await.context(
            "Неуспешно стартиране на pg_restore. Уверете се, че инструментът е инсталиран",
        )?;

        if !status.success() {
            return Err(anyhow!(
                "pg_restore завърши с грешка (код: {}). Провери логовете за подробности",
                status.code().unwrap_or(-1)
            ));
        }

        if request.storage == BackupStorage::ObjectStorage {
            let _ = fs::remove_file(&local_path).await;
        }

        Ok(RestoreSummary {
            source: request.identifier,
            storage: request.storage,
            duration: start.elapsed(),
            started_at,
        })
    }

    pub async fn get_object_storage_settings(
        &self,
        db: &DatabaseConnection,
    ) -> Result<ObjectStorageState> {
        self.load_object_storage_state(db).await
    }

    pub async fn update_object_storage_settings(
        &self,
        db: &DatabaseConnection,
        input: ObjectStorageSettingsInput,
    ) -> Result<ObjectStorageState> {
        self.update_object_storage_state(db, input).await
    }

    async fn collect_local_backups(&self) -> Result<Vec<BackupFile>> {
        let backups_dir = self.backups_dir.clone();

        tokio::task::spawn_blocking(move || -> Result<Vec<BackupFile>> {
            if !backups_dir.exists() {
                return Ok(Vec::new());
            }

            let mut backups = Vec::new();
            for entry in std::fs::read_dir(&backups_dir)? {
                let entry = entry?;
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }

                let metadata = entry.metadata()?;
                let modified = metadata
                    .modified()
                    .map(DateTime::<Utc>::from)
                    .unwrap_or_else(|_| Utc::now());
                let size_bytes = metadata.len();
                let file_name = entry
                    .file_name()
                    .into_string()
                    .unwrap_or_else(|_| "backup.dump".to_string());

                backups.push(BackupFile {
                    file_name,
                    full_path: path.to_string_lossy().to_string(),
                    size_bytes,
                    size_pretty: format_bytes(size_bytes),
                    created_at: modified,
                    storage: BackupStorage::Local,
                    object_key: None,
                });
            }

            Ok(backups)
        })
        .await
        .context("Неуспешно прочитане на директорията с резервни копия")?
    }

    async fn collect_remote_backups(&self, cfg: &ObjectStorageConfig) -> Result<Vec<BackupFile>> {
        let Some(client) = self.build_s3_client(cfg).await? else {
            return Ok(Vec::new());
        };

        if cfg.bucket.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut request = client.list_objects_v2().bucket(&cfg.bucket);
        if let Some(prefix) = &cfg.prefix {
            if !prefix.trim().is_empty() {
                request = request.prefix(prefix);
            }
        }

        let response = request.max_keys(50).send().await?;
        let mut backups = Vec::new();

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

    async fn fetch_database_size(&self, db: &DatabaseConnection) -> Result<i64> {
        let db_name = self.config.config_file.database.database.clone();

        // Use raw SQL query with current_database() to get the size
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT pg_database_size(current_database()) as size",
            vec![],
        );

        let row = db
            .query_one(stmt)
            .await
            .context("Неуспешно извличане на размер на базата данни")?
            .ok_or_else(|| anyhow!("Базата данни не върна резултат за pg_database_size"))?;

        let size: i64 = row.try_get("", "size")
            .context("Неуспешно извличане на колона 'size' от резултата")?;
        Ok(size)
    }

    fn base_object_storage_config(&self) -> ObjectStorageConfig {
        self.config_object_storage
            .clone()
            .unwrap_or(ObjectStorageConfig {
                enabled: false,
                endpoint: None,
                access_key: String::new(),
                secret_key: String::new(),
                region: "eu-central".to_string(),
                bucket: String::new(),
                prefix: None,
                force_path_style: Some(true),
            })
    }

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

        let settings = ContragentSetting::find()
            .filter(contragent_setting::Column::Key.starts_with("maintenance.object_storage."))
            .select_only()
            .column(contragent_setting::Column::Key)
            .column(contragent_setting::Column::Value)
            .into_model::<SettingRow>()
            .all(db)
            .await?;

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

    async fn update_object_storage_state(
        &self,
        db: &DatabaseConnection,
        input: ObjectStorageSettingsInput,
    ) -> Result<ObjectStorageState> {
        let mut current = self.load_object_storage_state(db).await?.config;

        current.enabled = input.enabled;
        current.endpoint = input.endpoint.clone().filter(|s| !s.trim().is_empty());
        current.access_key = input.access_key.clone();
        current.region = input.region.clone();
        current.bucket = input.bucket.clone();
        current.prefix = input.prefix.clone().filter(|s| !s.trim().is_empty());
        current.force_path_style = Some(input.force_path_style);

        if let Some(secret) = input.secret_key.clone() {
            current.secret_key = secret.trim().to_string();
        }

        self.persist_object_storage_settings(db, &current).await?;
        self.load_object_storage_state(db).await
    }

    async fn persist_object_storage_settings(
        &self,
        db: &DatabaseConnection,
        config: &ObjectStorageConfig,
    ) -> Result<()> {
        self.upsert_setting(
            db,
            "maintenance.object_storage.enabled",
            Some(config.enabled.to_string()),
            false,
            "S3 object storage enabled",
        )
        .await?;

        self.upsert_setting(
            db,
            "maintenance.object_storage.endpoint",
            config.endpoint.clone(),
            false,
            "S3 object storage endpoint",
        )
        .await?;

        self.upsert_setting(
            db,
            "maintenance.object_storage.access_key",
            Some(config.access_key.clone()),
            false,
            "S3 object storage access key",
        )
        .await?;

        let secret_val = if config.secret_key.trim().is_empty() {
            None
        } else {
            Some(config.secret_key.clone())
        };
        self.upsert_setting(
            db,
            "maintenance.object_storage.secret_key",
            secret_val,
            true,
            "S3 object storage secret key",
        )
        .await?;

        self.upsert_setting(
            db,
            "maintenance.object_storage.region",
            Some(config.region.clone()),
            false,
            "S3 object storage region",
        )
        .await?;

        self.upsert_setting(
            db,
            "maintenance.object_storage.bucket",
            Some(config.bucket.clone()),
            false,
            "S3 object storage bucket",
        )
        .await?;

        self.upsert_setting(
            db,
            "maintenance.object_storage.prefix",
            config.prefix.clone(),
            false,
            "S3 object storage prefix",
        )
        .await?;

        self.upsert_setting(
            db,
            "maintenance.object_storage.force_path_style",
            Some(config.force_path_style.unwrap_or(true).to_string()),
            false,
            "S3 object storage force path style",
        )
        .await
    }

    async fn upsert_setting(
        &self,
        db: &DatabaseConnection,
        key: &str,
        value: Option<String>,
        encrypted: bool,
        description: &str,
    ) -> Result<()> {
        let trimmed_value = value.map(|v| v.trim().to_string());

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

        if let Some(endpoint) = &cfg.endpoint {
            if !endpoint.trim().is_empty() {
                builder = builder.endpoint_url(endpoint);
            }
        }

        if cfg.force_path_style.unwrap_or(true) {
            builder = builder.force_path_style(true);
        }

        let config = builder.build();
        Ok(Some(S3Client::from_conf(config)))
    }

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

        let mut key = file_name.to_string();
        if let Some(prefix) = &cfg.prefix {
            if !prefix.trim().is_empty() {
                key = format!("{}/{}", prefix.trim_end_matches('/'), file_name);
            }
        }

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

        let response = client
            .get_object()
            .bucket(&cfg.bucket)
            .key(key)
            .send()
            .await?;

        let bytes = response.body.collect().await?.into_bytes();

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
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    if bytes == 0 {
        return "0 B".to_string();
    }

    let base = 1024_f64;
    let bytes_f64 = bytes as f64;
    let exponent = (bytes_f64.log(base)).floor().min((UNITS.len() - 1) as f64);
    let index = exponent as usize;
    let pretty = bytes_f64 / base.powf(exponent);

    if pretty >= 10.0 {
        format!("{:.1} {}", pretty, UNITS[index])
    } else {
        format!("{:.2} {}", pretty, UNITS[index])
    }
}
