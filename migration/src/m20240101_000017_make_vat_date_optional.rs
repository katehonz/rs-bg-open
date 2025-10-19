use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Make vat_date nullable for general journal entries
        // Only VAT-specific entries require this field
        manager
            .alter_table(
                Table::alter()
                    .table(JournalEntries::Table)
                    .modify_column(
                        ColumnDef::new(JournalEntries::VatDate).date().null(), // Make it nullable
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert back to not null (note: this may fail if there are NULL values)
        manager
            .alter_table(
                Table::alter()
                    .table(JournalEntries::Table)
                    .modify_column(ColumnDef::new(JournalEntries::VatDate).date().not_null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum JournalEntries {
    Table,
    VatDate,
}
