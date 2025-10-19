use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create inventory_movements table
        // This table tracks all inventory movements (receipts and issues)
        manager
            .create_table(
                Table::create()
                    .table(InventoryMovements::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InventoryMovements::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::CompanyId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::AccountId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::EntryLineId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::JournalEntryId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::MovementDate)
                            .date()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::MovementType)
                            .string()
                            .not_null()
                            .comment("DEBIT (receipt) or CREDIT (issue)"),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::Quantity)
                            .decimal_len(19, 6)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::UnitPrice)
                            .decimal_len(19, 6)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::TotalAmount)
                            .decimal_len(19, 6)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::UnitOfMeasure)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::Description)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::BalanceAfterQuantity)
                            .decimal_len(19, 6)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::BalanceAfterAmount)
                            .decimal_len(19, 6)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::AverageCostAtTime)
                            .decimal_len(19, 6)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(InventoryMovements::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InventoryMovements::Table, InventoryMovements::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InventoryMovements::Table, InventoryMovements::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InventoryMovements::Table, InventoryMovements::EntryLineId)
                            .to(EntryLines::Table, EntryLines::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InventoryMovements::Table, InventoryMovements::JournalEntryId)
                            .to(JournalEntries::Table, JournalEntries::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for inventory_movements
        manager
            .create_index(
                Index::create()
                    .name("idx_inventory_movements_company_account")
                    .table(InventoryMovements::Table)
                    .col(InventoryMovements::CompanyId)
                    .col(InventoryMovements::AccountId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_inventory_movements_date")
                    .table(InventoryMovements::Table)
                    .col(InventoryMovements::MovementDate)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_inventory_movements_entry_line")
                    .table(InventoryMovements::Table)
                    .col(InventoryMovements::EntryLineId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create inventory_balances table
        // This table maintains current balances and average costs for each material account
        manager
            .create_table(
                Table::create()
                    .table(InventoryBalances::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InventoryBalances::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(InventoryBalances::CompanyId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryBalances::AccountId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryBalances::CurrentQuantity)
                            .decimal_len(19, 6)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(InventoryBalances::CurrentAmount)
                            .decimal_len(19, 6)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(InventoryBalances::CurrentAverageCost)
                            .decimal_len(19, 6)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(InventoryBalances::LastMovementDate)
                            .date()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(InventoryBalances::LastMovementId)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(InventoryBalances::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InventoryBalances::Table, InventoryBalances::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InventoryBalances::Table, InventoryBalances::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InventoryBalances::Table, InventoryBalances::LastMovementId)
                            .to(InventoryMovements::Table, InventoryMovements::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index for company + account
        manager
            .create_index(
                Index::create()
                    .name("idx_inventory_balances_company_account")
                    .table(InventoryBalances::Table)
                    .col(InventoryBalances::CompanyId)
                    .col(InventoryBalances::AccountId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create average_cost_corrections table
        // This table tracks corrections when entries are added out of chronological order
        manager
            .create_table(
                Table::create()
                    .table(AverageCostCorrections::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AverageCostCorrections::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AverageCostCorrections::CompanyId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AverageCostCorrections::AccountId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AverageCostCorrections::TriggeringMovementId)
                            .integer()
                            .not_null()
                            .comment("The movement that was added and triggered recalculation"),
                    )
                    .col(
                        ColumnDef::new(AverageCostCorrections::AffectedMovementId)
                            .integer()
                            .not_null()
                            .comment("The later movement that needs correction"),
                    )
                    .col(
                        ColumnDef::new(AverageCostCorrections::CorrectionJournalEntryId)
                            .integer()
                            .null()
                            .comment("The correction journal entry created"),
                    )
                    .col(
                        ColumnDef::new(AverageCostCorrections::OldAverageCost)
                            .decimal_len(19, 6)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AverageCostCorrections::NewAverageCost)
                            .decimal_len(19, 6)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AverageCostCorrections::CorrectionAmount)
                            .decimal_len(19, 6)
                            .not_null()
                            .comment("Difference to be corrected (positive or negative)"),
                    )
                    .col(
                        ColumnDef::new(AverageCostCorrections::IsApplied)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AverageCostCorrections::AppliedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AverageCostCorrections::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(AverageCostCorrections::Table, AverageCostCorrections::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(AverageCostCorrections::Table, AverageCostCorrections::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(AverageCostCorrections::Table, AverageCostCorrections::TriggeringMovementId)
                            .to(InventoryMovements::Table, InventoryMovements::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(AverageCostCorrections::Table, AverageCostCorrections::AffectedMovementId)
                            .to(InventoryMovements::Table, InventoryMovements::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(AverageCostCorrections::Table, AverageCostCorrections::CorrectionJournalEntryId)
                            .to(JournalEntries::Table, JournalEntries::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for corrections
        manager
            .create_index(
                Index::create()
                    .name("idx_corrections_company_account")
                    .table(AverageCostCorrections::Table)
                    .col(AverageCostCorrections::CompanyId)
                    .col(AverageCostCorrections::AccountId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_corrections_is_applied")
                    .table(AverageCostCorrections::Table)
                    .col(AverageCostCorrections::IsApplied)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AverageCostCorrections::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(InventoryBalances::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(InventoryMovements::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum InventoryMovements {
    Table,
    Id,
    CompanyId,
    AccountId,
    EntryLineId,
    JournalEntryId,
    MovementDate,
    MovementType,
    Quantity,
    UnitPrice,
    TotalAmount,
    UnitOfMeasure,
    Description,
    BalanceAfterQuantity,
    BalanceAfterAmount,
    AverageCostAtTime,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum InventoryBalances {
    Table,
    Id,
    CompanyId,
    AccountId,
    CurrentQuantity,
    CurrentAmount,
    CurrentAverageCost,
    LastMovementDate,
    LastMovementId,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AverageCostCorrections {
    Table,
    Id,
    CompanyId,
    AccountId,
    TriggeringMovementId,
    AffectedMovementId,
    CorrectionJournalEntryId,
    OldAverageCost,
    NewAverageCost,
    CorrectionAmount,
    IsApplied,
    AppliedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Companies {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum EntryLines {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum JournalEntries {
    Table,
    Id,
}
