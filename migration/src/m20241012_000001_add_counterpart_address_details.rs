use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Counterparts::Table.to_string();
        let street_column = Counterparts::Street.to_string();
        let postal_column = Counterparts::PostalCode.to_string();

        if !manager.has_column(&table, &street_column).await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Counterparts::Table)
                        .add_column(ColumnDef::new(Counterparts::Street).string().null())
                        .to_owned(),
                )
                .await?;
        }

        if !manager.has_column(&table, &postal_column).await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Counterparts::Table)
                        .add_column(ColumnDef::new(Counterparts::PostalCode).string().null())
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Counterparts::Table.to_string();
        let street_column = Counterparts::Street.to_string();
        let postal_column = Counterparts::PostalCode.to_string();

        if manager.has_column(&table, &postal_column).await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Counterparts::Table)
                        .drop_column(Counterparts::PostalCode)
                        .to_owned(),
                )
                .await?;
        }

        if manager.has_column(&table, &street_column).await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Counterparts::Table)
                        .drop_column(Counterparts::Street)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Counterparts {
    Table,
    Street,
    PostalCode,
}
