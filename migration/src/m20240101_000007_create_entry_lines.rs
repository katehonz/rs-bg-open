use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(EntryLines::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EntryLines::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(EntryLines::JournalEntryId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(EntryLines::AccountId).integer().not_null())
                    .col(
                        ColumnDef::new(EntryLines::DebitAmount)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(EntryLines::CreditAmount)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(ColumnDef::new(EntryLines::CounterpartId).integer())
                    .col(
                        ColumnDef::new(EntryLines::CurrencyCode)
                            .string()
                            .default("BGN"),
                    )
                    .col(ColumnDef::new(EntryLines::CurrencyAmount).decimal_len(15, 2))
                    .col(
                        ColumnDef::new(EntryLines::ExchangeRate)
                            .decimal_len(15, 6)
                            .default(1.000000),
                    )
                    .col(
                        ColumnDef::new(EntryLines::BaseAmount)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(EntryLines::VatAmount)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(ColumnDef::new(EntryLines::VatRateId).integer())
                    .col(ColumnDef::new(EntryLines::Quantity).decimal_len(15, 4))
                    .col(ColumnDef::new(EntryLines::UnitOfMeasureCode).string())
                    .col(ColumnDef::new(EntryLines::Description).text())
                    .col(
                        ColumnDef::new(EntryLines::LineOrder)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(EntryLines::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_entry_lines_journal_entry_id")
                            .from(EntryLines::Table, EntryLines::JournalEntryId)
                            .to(JournalEntries::Table, JournalEntries::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_entry_lines_account_id")
                            .from(EntryLines::Table, EntryLines::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for performance
        manager
            .create_index(
                Index::create()
                    .name("idx_entry_lines_journal_entry_id")
                    .table(EntryLines::Table)
                    .col(EntryLines::JournalEntryId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_entry_lines_account_id")
                    .table(EntryLines::Table)
                    .col(EntryLines::AccountId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_entry_lines_counterpart_id")
                    .table(EntryLines::Table)
                    .col(EntryLines::CounterpartId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(EntryLines::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum EntryLines {
    Table,
    Id,
    JournalEntryId,
    AccountId,
    DebitAmount,
    CreditAmount,
    CounterpartId,
    CurrencyCode,
    CurrencyAmount,
    ExchangeRate,
    BaseAmount,
    VatAmount,
    VatRateId,
    Quantity,
    UnitOfMeasureCode,
    Description,
    LineOrder,
    CreatedAt,
}

#[derive(DeriveIden)]
enum JournalEntries {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
}
