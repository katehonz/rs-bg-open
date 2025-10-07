use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add Bulgarian VAT codes to journal_entries table
        manager
            .alter_table(
                Table::alter()
                    .table(JournalEntries::Table)
                    .add_column(
                        ColumnDef::new(JournalEntries::VatDocumentType)
                            .string()
                            .null()
                            .comment(
                                "Bulgarian VAT document type code (01-95) according to PPZDDS",
                            ),
                    )
                    .add_column(
                        ColumnDef::new(JournalEntries::VatPurchaseOperation)
                            .string()
                            .null()
                            .comment("Bulgarian VAT purchase operation code (0-6)"),
                    )
                    .add_column(
                        ColumnDef::new(JournalEntries::VatSalesOperation)
                            .string()
                            .null()
                            .comment("Bulgarian VAT sales operation code (0-10, 9001-9002)"),
                    )
                    .add_column(
                        ColumnDef::new(JournalEntries::VatAdditionalOperation)
                            .string()
                            .null()
                            .comment("Bulgarian additional VAT operation code (0-8)"),
                    )
                    .add_column(
                        ColumnDef::new(JournalEntries::VatAdditionalData)
                            .string()
                            .null()
                            .comment("Bulgarian additional VAT data (1-2) for appendix 2"),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(JournalEntries::Table)
                    .drop_column(JournalEntries::VatDocumentType)
                    .drop_column(JournalEntries::VatPurchaseOperation)
                    .drop_column(JournalEntries::VatSalesOperation)
                    .drop_column(JournalEntries::VatAdditionalOperation)
                    .drop_column(JournalEntries::VatAdditionalData)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum JournalEntries {
    Table,
    VatDocumentType,
    VatPurchaseOperation,
    VatSalesOperation,
    VatAdditionalOperation,
    VatAdditionalData,
}
