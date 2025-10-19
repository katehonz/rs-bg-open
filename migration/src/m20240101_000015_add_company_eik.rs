use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add EIK column to companies table
        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Companies::Eik)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await?;

        // Add city column to companies table
        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .add_column_if_not_exists(ColumnDef::new(Companies::City).string())
                    .to_owned(),
            )
            .await?;

        // Add country column to companies table
        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Companies::Country)
                            .string()
                            .default("България"),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for EIK lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_companies_eik")
                    .table(Companies::Table)
                    .col(Companies::Eik)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop index
        manager
            .drop_index(Index::drop().name("idx_companies_eik").to_owned())
            .await?;

        // Remove columns
        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .drop_column(Companies::Eik)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .drop_column(Companies::City)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .drop_column(Companies::Country)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Companies {
    Table,
    Eik,
    City,
    Country,
}
