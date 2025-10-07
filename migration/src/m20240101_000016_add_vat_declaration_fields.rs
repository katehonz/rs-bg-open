use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add manager_name column to companies table
        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Companies::ManagerName)
                            .string()
                            .comment("Име на управител/представляващ за ДДС декларации"),
                    )
                    .to_owned(),
            )
            .await?;

        // Add authorized_person column to companies table
        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Companies::AuthorizedPerson)
                            .string()
                            .comment("Име на пълномощник за ДДС декларации"),
                    )
                    .to_owned(),
            )
            .await?;

        // Add manager_egn column (ЕГН на управител)
        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Companies::ManagerEgn)
                            .string()
                            .comment("ЕГН на управител"),
                    )
                    .to_owned(),
            )
            .await?;

        // Add authorized_person_egn column (ЕГН на пълномощник)
        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Companies::AuthorizedPersonEgn)
                            .string()
                            .comment("ЕГН на пълномощник"),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove columns
        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .drop_column(Companies::ManagerName)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .drop_column(Companies::AuthorizedPerson)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .drop_column(Companies::ManagerEgn)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Companies::Table)
                    .drop_column(Companies::AuthorizedPersonEgn)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Companies {
    Table,
    ManagerName,
    AuthorizedPerson,
    ManagerEgn,
    AuthorizedPersonEgn,
}
