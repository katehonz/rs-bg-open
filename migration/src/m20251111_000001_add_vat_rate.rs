use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add vat_rate column to journal_entries table
        manager
            .alter_table(
                Table::alter()
                    .table(JournalEntries::Table)
                    .add_column(
                        ColumnDef::new(JournalEntries::VatRate)
                            .decimal_len(5, 2)
                            .null()
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop vat_rate column from journal_entries table
        manager
            .alter_table(
                Table::alter()
                    .table(JournalEntries::Table)
                    .drop_column(JournalEntries::VatRate)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum JournalEntries {
    Table,
    VatRate,
}
