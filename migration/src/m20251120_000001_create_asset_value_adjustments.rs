use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create asset_value_adjustments table for tracking changes to fixed asset values
        manager
            .create_table(
                Table::create()
                    .table(AssetValueAdjustments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AssetValueAdjustments::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::FixedAssetId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::CompanyId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::AdjustmentDate)
                            .date()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::AdjustmentType)
                            .string()
                            .not_null(),
                    )
                    // Type can be: acquisition, improvement, impairment, revaluation,
                    // disposal, internal_transfer, scrap
                    .col(
                        ColumnDef::new(AssetValueAdjustments::AdjustmentAmount)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    // Book value changes
                    .col(
                        ColumnDef::new(AssetValueAdjustments::AccountingValueBefore)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::AccountingValueAfter)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::TaxValueBefore)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::TaxValueAfter)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    // Depreciation rates after adjustment (for SAF-T reporting)
                    .col(
                        ColumnDef::new(AssetValueAdjustments::NewAccountingDepreciationRate)
                            .decimal_len(5, 2),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::NewTaxDepreciationRate)
                            .decimal_len(5, 2),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::NewAccountingUsefulLife)
                            .integer(),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::NewTaxUsefulLife)
                            .integer(),
                    )
                    // Description and reasoning
                    .col(
                        ColumnDef::new(AssetValueAdjustments::Reason)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::DocumentNumber)
                            .string(),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::Notes)
                            .text(),
                    )
                    // Journal entry reference
                    .col(
                        ColumnDef::new(AssetValueAdjustments::JournalEntryId)
                            .integer(),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::IsPosted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::PostedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::PostedBy)
                            .integer(),
                    )
                    // Audit fields
                    .col(
                        ColumnDef::new(AssetValueAdjustments::CreatedBy)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AssetValueAdjustments::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    // Foreign keys
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_asset_adjustments_asset")
                            .from(
                                AssetValueAdjustments::Table,
                                AssetValueAdjustments::FixedAssetId,
                            )
                            .to(Alias::new("fixed_assets"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_asset_adjustments_company")
                            .from(
                                AssetValueAdjustments::Table,
                                AssetValueAdjustments::CompanyId,
                            )
                            .to(Alias::new("companies"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_asset_adjustments_journal_entry")
                            .from(
                                AssetValueAdjustments::Table,
                                AssetValueAdjustments::JournalEntryId,
                            )
                            .to(Alias::new("journal_entries"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_asset_adjustments_created_by")
                            .from(
                                AssetValueAdjustments::Table,
                                AssetValueAdjustments::CreatedBy,
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // Add indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_asset_adjustments_asset_date")
                    .table(AssetValueAdjustments::Table)
                    .col(AssetValueAdjustments::FixedAssetId)
                    .col(AssetValueAdjustments::AdjustmentDate)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_asset_adjustments_company_date")
                    .table(AssetValueAdjustments::Table)
                    .col(AssetValueAdjustments::CompanyId)
                    .col(AssetValueAdjustments::AdjustmentDate)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_asset_adjustments_type")
                    .table(AssetValueAdjustments::Table)
                    .col(AssetValueAdjustments::AdjustmentType)
                    .to_owned(),
            )
            .await?;

        // Add adjustment account codes to fixed_asset_categories
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("fixed_asset_categories"))
                    .add_column(
                        ColumnDef::new(Alias::new("improvement_debit_account_code"))
                            .string(),
                    )
                    .add_column(
                        ColumnDef::new(Alias::new("improvement_credit_account_code"))
                            .string(),
                    )
                    .add_column(
                        ColumnDef::new(Alias::new("impairment_debit_account_code"))
                            .string(),
                    )
                    .add_column(
                        ColumnDef::new(Alias::new("impairment_credit_account_code"))
                            .string(),
                    )
                    .add_column(
                        ColumnDef::new(Alias::new("disposal_debit_account_code"))
                            .string(),
                    )
                    .add_column(
                        ColumnDef::new(Alias::new("disposal_credit_account_code"))
                            .string(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes
        manager
            .drop_index(
                Index::drop()
                    .name("idx_asset_adjustments_type")
                    .table(AssetValueAdjustments::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_asset_adjustments_company_date")
                    .table(AssetValueAdjustments::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_asset_adjustments_asset_date")
                    .table(AssetValueAdjustments::Table)
                    .to_owned(),
            )
            .await?;

        // Drop table
        manager
            .drop_table(
                Table::drop()
                    .table(AssetValueAdjustments::Table)
                    .to_owned(),
            )
            .await?;

        // Remove columns from fixed_asset_categories
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("fixed_asset_categories"))
                    .drop_column(Alias::new("disposal_credit_account_code"))
                    .drop_column(Alias::new("disposal_debit_account_code"))
                    .drop_column(Alias::new("impairment_credit_account_code"))
                    .drop_column(Alias::new("impairment_debit_account_code"))
                    .drop_column(Alias::new("improvement_credit_account_code"))
                    .drop_column(Alias::new("improvement_debit_account_code"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AssetValueAdjustments {
    Table,
    Id,
    FixedAssetId,
    CompanyId,
    AdjustmentDate,
    AdjustmentType,
    AdjustmentAmount,
    AccountingValueBefore,
    AccountingValueAfter,
    TaxValueBefore,
    TaxValueAfter,
    NewAccountingDepreciationRate,
    NewTaxDepreciationRate,
    NewAccountingUsefulLife,
    NewTaxUsefulLife,
    Reason,
    DocumentNumber,
    Notes,
    JournalEntryId,
    IsPosted,
    PostedAt,
    PostedBy,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}
