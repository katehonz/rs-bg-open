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
                    .add_column(ColumnDef::new(Companies::ContragentApiUrl).text().null())
                    .add_column(
                        ColumnDef::new(Companies::EnableViesValidation)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(
                        ColumnDef::new(Companies::EnableAiMapping)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(
                        ColumnDef::new(Companies::AutoValidateOnImport)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .drop_column(Companies::ContragentApiUrl)
                    .drop_column(Companies::EnableViesValidation)
                    .drop_column(Companies::EnableAiMapping)
                    .drop_column(Companies::AutoValidateOnImport)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Companies {
    #[sea_orm(iden = "companies")]
    Table,
    ContragentApiUrl,
    EnableViesValidation,
    EnableAiMapping,
    AutoValidateOnImport,
}
