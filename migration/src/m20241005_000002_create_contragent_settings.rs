use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ContragentSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ContragentSettings::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ContragentSettings::Key)
                            .string_len(191)
                            .not_null(),
                    )
                    .col(ColumnDef::new(ContragentSettings::Value).text())
                    .col(ColumnDef::new(ContragentSettings::Description).text())
                    .col(
                        ColumnDef::new(ContragentSettings::Encrypted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(ContragentSettings::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ContragentSettings::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_contragent_settings_key")
                    .table(ContragentSettings::Table)
                    .col(ContragentSettings::Key)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ContragentSettings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ContragentSettings {
    #[sea_orm(iden = "contragent_settings")]
    Table,
    Id,
    Key,
    Value,
    Description,
    Encrypted,
    CreatedAt,
    UpdatedAt,
}
