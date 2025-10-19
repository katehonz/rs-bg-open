use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .add_column(ColumnDef::new(Companies::ContragentApiKey).text().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .drop_column(Companies::ContragentApiKey)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Companies {
    #[sea_orm(iden = "companies")]
    Table,
    ContragentApiKey,
}
