use std::sync::Arc;

use async_graphql::{Context, Enum, FieldResult, InputObject, Object, SimpleObject};
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

use crate::entities::company::{
    ActiveModel as CompanyActiveModel, Entity as CompanyEntity, Model as CompanyModel,
};
use crate::services::maintenance::{
    BackupFile as ServiceBackupFile, BackupStorage, BackupSummary as ServiceBackupSummary,
    MaintenanceService, MaintenanceStatus as ServiceMaintenanceStatus,
    ObjectStorageSettingsInput as ServiceObjectStorageSettingsInput, ObjectStorageState,
    OptimizationSummary as ServiceOptimizationSummary, RestoreRequest,
    RestoreSummary as ServiceRestoreSummary,
};

#[derive(Enum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum BackupStorageType {
    Local,
    ObjectStorage,
}

impl From<BackupStorage> for BackupStorageType {
    fn from(value: BackupStorage) -> Self {
        match value {
            BackupStorage::Local => BackupStorageType::Local,
            BackupStorage::ObjectStorage => BackupStorageType::ObjectStorage,
        }
    }
}

#[derive(SimpleObject, Clone, Debug)]
pub struct BackupFile {
    file_name: String,
    full_path: String,
    size_bytes: i64,
    size_pretty: String,
    created_at: DateTime<Utc>,
    storage_type: BackupStorageType,
    object_key: Option<String>,
}

impl From<ServiceBackupFile> for BackupFile {
    fn from(value: ServiceBackupFile) -> Self {
        Self {
            file_name: value.file_name,
            full_path: value.full_path,
            size_bytes: value.size_bytes as i64,
            size_pretty: value.size_pretty,
            created_at: value.created_at,
            storage_type: BackupStorageType::from(value.storage),
            object_key: value.object_key,
        }
    }
}

#[derive(SimpleObject, Clone, Debug)]
pub struct MaintenanceStatus {
    database_name: String,
    database_size_bytes: i64,
    database_size_pretty: String,
    backups_directory: String,
    recent_backups: Vec<BackupFile>,
    object_storage_enabled: bool,
}

impl From<ServiceMaintenanceStatus> for MaintenanceStatus {
    fn from(status: ServiceMaintenanceStatus) -> Self {
        Self {
            database_name: status.database_name,
            database_size_bytes: status.database_size_bytes,
            database_size_pretty: status.database_size_pretty,
            backups_directory: status.backups_directory,
            recent_backups: status
                .recent_backups
                .into_iter()
                .map(BackupFile::from)
                .collect(),
            object_storage_enabled: status.object_storage_enabled,
        }
    }
}

#[derive(SimpleObject, Clone, Debug)]
pub struct BackupPayload {
    backup: BackupFile,
    duration_ms: i64,
    remote_object_key: Option<String>,
}

impl From<ServiceBackupSummary> for BackupPayload {
    fn from(summary: ServiceBackupSummary) -> Self {
        Self {
            backup: BackupFile::from(summary.backup),
            duration_ms: summary.duration.as_millis() as i64,
            remote_object_key: summary.remote_object_key,
        }
    }
}

#[derive(SimpleObject, Clone, Debug)]
pub struct OptimizationPayload {
    database_name: String,
    size_before_bytes: i64,
    size_before_pretty: String,
    size_after_bytes: i64,
    size_after_pretty: String,
    vacuum_ran: bool,
    analyze_ran: bool,
    reindex_ran: bool,
    duration_ms: i64,
}

impl From<ServiceOptimizationSummary> for OptimizationPayload {
    fn from(summary: ServiceOptimizationSummary) -> Self {
        Self {
            database_name: summary.database_name,
            size_before_bytes: summary.size_before_bytes,
            size_before_pretty: summary.size_before_pretty,
            size_after_bytes: summary.size_after_bytes,
            size_after_pretty: summary.size_after_pretty,
            vacuum_ran: summary.vacuum_ran,
            analyze_ran: summary.analyze_ran,
            reindex_ran: summary.reindex_ran,
            duration_ms: summary.duration.as_millis() as i64,
        }
    }
}

#[derive(SimpleObject, Clone, Debug)]
pub struct RestorePayload {
    source: String,
    storage_type: BackupStorageType,
    duration_ms: i64,
    started_at: DateTime<Utc>,
}

impl From<ServiceRestoreSummary> for RestorePayload {
    fn from(summary: ServiceRestoreSummary) -> Self {
        Self {
            source: summary.source,
            storage_type: BackupStorageType::from(summary.storage),
            duration_ms: summary.duration.as_millis() as i64,
            started_at: summary.started_at,
        }
    }
}

#[derive(SimpleObject, Clone, Debug)]
pub struct ObjectStorageSettings {
    enabled: bool,
    endpoint: Option<String>,
    access_key: String,
    has_secret_key: bool,
    region: String,
    bucket: String,
    prefix: Option<String>,
    force_path_style: bool,
}

impl From<ObjectStorageState> for ObjectStorageSettings {
    fn from(state: ObjectStorageState) -> Self {
        Self {
            enabled: state.config.enabled,
            endpoint: state.config.endpoint.clone(),
            access_key: state.config.access_key.clone(),
            has_secret_key: state.has_secret_key,
            region: state.config.region.clone(),
            bucket: state.config.bucket.clone(),
            prefix: state.config.prefix.clone(),
            force_path_style: state.config.force_path_style.unwrap_or(true),
        }
    }
}

#[derive(InputObject)]
pub struct UpdateObjectStorageSettingsInput {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub access_key: String,
    pub secret_key: Option<String>,
    pub region: String,
    pub bucket: String,
    pub prefix: Option<String>,
    pub force_path_style: bool,
}

impl From<UpdateObjectStorageSettingsInput> for ServiceObjectStorageSettingsInput {
    fn from(input: UpdateObjectStorageSettingsInput) -> Self {
        ServiceObjectStorageSettingsInput {
            enabled: input.enabled,
            endpoint: input.endpoint,
            access_key: input.access_key,
            secret_key: input.secret_key,
            region: input.region,
            bucket: input.bucket,
            prefix: input.prefix,
            force_path_style: input.force_path_style,
        }
    }
}

#[derive(InputObject)]
pub struct RestoreDatabaseInput {
    pub storage_type: BackupStorageType,
    pub identifier: String,
}

#[derive(Default)]
pub struct MaintenanceQuery;

#[Object]
impl MaintenanceQuery {
    async fn database_maintenance_status(
        &self,
        ctx: &Context<'_>,
    ) -> FieldResult<MaintenanceStatus> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let service = ctx.data::<Arc<MaintenanceService>>()?;
        let status = service.status(db.as_ref()).await?;
        Ok(MaintenanceStatus::from(status))
    }

    async fn object_storage_settings(
        &self,
        ctx: &Context<'_>,
    ) -> FieldResult<ObjectStorageSettings> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let service = ctx.data::<Arc<MaintenanceService>>()?;
        let state = service.get_object_storage_settings(db.as_ref()).await?;
        Ok(ObjectStorageSettings::from(state))
    }
}

#[derive(Default)]
pub struct MaintenanceMutation;

#[Object]
impl MaintenanceMutation {
    async fn create_database_backup(&self, ctx: &Context<'_>) -> FieldResult<BackupPayload> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let service = ctx.data::<Arc<MaintenanceService>>()?;
        let summary = service.create_backup(db.as_ref()).await?;
        Ok(BackupPayload::from(summary))
    }

    async fn optimize_database(&self, ctx: &Context<'_>) -> FieldResult<OptimizationPayload> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let service = ctx.data::<Arc<MaintenanceService>>()?;
        let summary = service.optimize_database(db.as_ref()).await?;
        Ok(OptimizationPayload::from(summary))
    }

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

    async fn update_company_integration_settings(
        &self,
        ctx: &Context<'_>,
        input: UpdateCompanyIntegrationSettingsInput,
    ) -> FieldResult<CompanyModel> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let company = CompanyEntity::find_by_id(input.company_id)
            .one(db.as_ref())
            .await?
            .ok_or_else(|| "Company not found".to_string())?;

        let mut active: CompanyActiveModel = company.into();
        active.enable_vies_validation = Set(input.enable_vies_validation);
        active.enable_ai_mapping = Set(input.enable_ai_mapping);
        active.auto_validate_on_import = Set(input.auto_validate_on_import);

        let updated = active.update(db.as_ref()).await?;
        Ok(updated)
    }
}

#[derive(InputObject)]
pub struct UpdateCompanyIntegrationSettingsInput {
    pub company_id: i32,
    pub enable_vies_validation: bool,
    pub enable_ai_mapping: bool,
    pub auto_validate_on_import: bool,
}
