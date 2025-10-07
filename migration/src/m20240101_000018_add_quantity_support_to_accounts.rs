use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add quantity support fields to accounts
        manager
            .alter_table(
                Table::alter()
                    .table(Accounts::Table)
                    .add_column(
                        ColumnDef::new(Accounts::SupportsQuantities)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(ColumnDef::new(Accounts::DefaultUnit).string().null())
                    .to_owned(),
            )
            .await?;

        // Update accounts in groups 2 and 3 to support quantities by default
        // Group 2: Materials, Group 3: Production
        manager
            .exec_stmt(
                Query::update()
                    .table(Accounts::Table)
                    .values([
                        (Accounts::SupportsQuantities, true.into()),
                        (Accounts::DefaultUnit, "бр".into()),
                    ])
                    .and_where(Expr::col(Accounts::AccountClass).is_in([2, 3]))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove the added columns
        manager
            .alter_table(
                Table::alter()
                    .table(Accounts::Table)
                    .drop_column(Accounts::SupportsQuantities)
                    .drop_column(Accounts::DefaultUnit)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Accounts {
    Table,
    SupportsQuantities,
    DefaultUnit,
    AccountClass,
}
