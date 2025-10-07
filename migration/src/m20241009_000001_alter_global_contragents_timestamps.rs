use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "ALTER TABLE global_contragents \
             ALTER COLUMN last_validated_at TYPE TIMESTAMPTZ \
             USING last_validated_at AT TIME ZONE 'UTC', \
             ALTER COLUMN created_at TYPE TIMESTAMPTZ \
             USING created_at AT TIME ZONE 'UTC', \
             ALTER COLUMN updated_at TYPE TIMESTAMPTZ \
             USING updated_at AT TIME ZONE 'UTC'"
                .to_string(),
        ))
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "ALTER TABLE global_contragents \
             ALTER COLUMN last_validated_at TYPE TIMESTAMP \
             USING last_validated_at AT TIME ZONE 'UTC', \
             ALTER COLUMN created_at TYPE TIMESTAMP \
             USING created_at AT TIME ZONE 'UTC', \
             ALTER COLUMN updated_at TYPE TIMESTAMP \
             USING updated_at AT TIME ZONE 'UTC'"
                .to_string(),
        ))
        .await?;
        Ok(())
    }
}
