use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add EIK field (БУЛСТАТ/ЕИК)
        manager
            .alter_table(
                Table::alter()
                    .table(Counterparts::Table)
                    .add_column(ColumnDef::new(Counterparts::Eik).string().null())
                    .to_owned(),
            )
            .await?;

        // Add city field
        manager
            .alter_table(
                Table::alter()
                    .table(Counterparts::Table)
                    .add_column(ColumnDef::new(Counterparts::City).string().null())
                    .to_owned(),
            )
            .await?;

        // Add country field
        manager
            .alter_table(
                Table::alter()
                    .table(Counterparts::Table)
                    .add_column(
                        ColumnDef::new(Counterparts::Country)
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
                    .name("idx_counterparts_eik")
                    .table(Counterparts::Table)
                    .col(Counterparts::Eik)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop index first
        manager
            .drop_index(Index::drop().name("idx_counterparts_eik").to_owned())
            .await?;

        // Drop columns
        manager
            .alter_table(
                Table::alter()
                    .table(Counterparts::Table)
                    .drop_column(Counterparts::Country)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Counterparts::Table)
                    .drop_column(Counterparts::City)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Counterparts::Table)
                    .drop_column(Counterparts::Eik)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Counterparts {
    Table,
    Eik,
    City,
    Country,
}
