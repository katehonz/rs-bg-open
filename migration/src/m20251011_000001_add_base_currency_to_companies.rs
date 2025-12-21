use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add base_currency_id column to companies table
        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .add_column(
                        ColumnDef::new(Companies::BaseCurrencyId)
                            .integer()
                            .null() // Nullable initially for existing companies
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key to currencies table
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_companies_base_currency")
                    .from(Companies::Table, Companies::BaseCurrencyId)
                    .to(Currencies::Table, Currencies::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        // Set default base currency to BGN for existing companies
        // Use raw SQL for this operation
        let sql = r#"
            UPDATE companies
            SET base_currency_id = (
                SELECT id FROM currencies WHERE code = 'BGN' LIMIT 1
            )
            WHERE base_currency_id IS NULL
        "#;

        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop foreign key first
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_companies_base_currency")
                    .table(Companies::Table)
                    .to_owned(),
            )
            .await?;

        // Drop column
        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .drop_column(Companies::BaseCurrencyId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Companies {
    Table,
    BaseCurrencyId,
}

#[derive(DeriveIden)]
enum Currencies {
    Table,
    Id,
    Code,
}
