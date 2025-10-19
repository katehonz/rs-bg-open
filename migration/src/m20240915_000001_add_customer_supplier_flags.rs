use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Counterpart::Table)
                    .add_column(
                        ColumnDef::new(Counterpart::IsCustomer)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(
                        ColumnDef::new(Counterpart::IsSupplier)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        // Update existing records based on counterpart_type
        let update_customers_sql =
            "UPDATE counterparts SET is_customer = true WHERE counterpart_type = 'CUSTOMER'";
        let update_suppliers_sql =
            "UPDATE counterparts SET is_supplier = true WHERE counterpart_type = 'SUPPLIER'";

        manager
            .get_connection()
            .execute_unprepared(update_customers_sql)
            .await?;
        manager
            .get_connection()
            .execute_unprepared(update_suppliers_sql)
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Counterpart::Table)
                    .drop_column(Counterpart::IsCustomer)
                    .drop_column(Counterpart::IsSupplier)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Counterpart {
    #[sea_orm(iden = "counterparts")]
    Table,
    IsCustomer,
    IsSupplier,
}
