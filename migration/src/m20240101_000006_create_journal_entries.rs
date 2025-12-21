use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(JournalEntries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(JournalEntries::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(JournalEntries::EntryNumber)
                            .string()
                            .not_null(),
                    )
                    // Bulgarian triple date system
                    .col(
                        ColumnDef::new(JournalEntries::DocumentDate)
                            .date()
                            .not_null(),
                    )
                    .col(ColumnDef::new(JournalEntries::VatDate).date().not_null())
                    .col(
                        ColumnDef::new(JournalEntries::AccountingDate)
                            .date()
                            .not_null(),
                    )
                    .col(ColumnDef::new(JournalEntries::DocumentNumber).string())
                    .col(
                        ColumnDef::new(JournalEntries::Description)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(JournalEntries::TotalAmount)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(JournalEntries::TotalVatAmount)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(JournalEntries::IsPosted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(JournalEntries::PostedBy).integer())
                    .col(ColumnDef::new(JournalEntries::PostedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(JournalEntries::CreatedBy)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(JournalEntries::CompanyId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(JournalEntries::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(JournalEntries::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_journal_entries_company_id")
                            .from(JournalEntries::Table, JournalEntries::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_journal_entries_created_by")
                            .from(JournalEntries::Table, JournalEntries::CreatedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_journal_entries_posted_by")
                            .from(JournalEntries::Table, JournalEntries::PostedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique constraint for company_id + entry_number
        manager
            .create_index(
                Index::create()
                    .name("idx_journal_entries_company_number_unique")
                    .table(JournalEntries::Table)
                    .col(JournalEntries::CompanyId)
                    .col(JournalEntries::EntryNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create indexes for date-based queries
        manager
            .create_index(
                Index::create()
                    .name("idx_journal_entries_document_date")
                    .table(JournalEntries::Table)
                    .col(JournalEntries::DocumentDate)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_journal_entries_vat_date")
                    .table(JournalEntries::Table)
                    .col(JournalEntries::VatDate)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_journal_entries_accounting_date")
                    .table(JournalEntries::Table)
                    .col(JournalEntries::AccountingDate)
                    .to_owned(),
            )
            .await?;

        // Index for posted entries
        manager
            .create_index(
                Index::create()
                    .name("idx_journal_entries_is_posted")
                    .table(JournalEntries::Table)
                    .col(JournalEntries::IsPosted)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(JournalEntries::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum JournalEntries {
    Table,
    Id,
    EntryNumber,
    DocumentDate,
    VatDate,
    AccountingDate,
    DocumentNumber,
    Description,
    TotalAmount,
    TotalVatAmount,
    IsPosted,
    PostedBy,
    PostedAt,
    CreatedBy,
    CompanyId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Companies {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
