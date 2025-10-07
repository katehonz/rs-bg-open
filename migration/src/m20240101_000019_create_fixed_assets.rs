use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create fixed asset categories table
        manager
            .create_table(
                Table::create()
                    .table(FixedAssetCategories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FixedAssetCategories::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(FixedAssetCategories::Code)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(FixedAssetCategories::Name)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(FixedAssetCategories::Description).text())
                    .col(
                        ColumnDef::new(FixedAssetCategories::TaxCategory)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FixedAssetCategories::MaxTaxDepreciationRate)
                            .decimal_len(5, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FixedAssetCategories::DefaultAccountingDepreciationRate)
                            .decimal_len(5, 2),
                    )
                    .col(ColumnDef::new(FixedAssetCategories::MinUsefulLife).integer())
                    .col(ColumnDef::new(FixedAssetCategories::MaxUsefulLife).integer())
                    .col(
                        ColumnDef::new(FixedAssetCategories::AssetAccountCode)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FixedAssetCategories::DepreciationAccountCode)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FixedAssetCategories::ExpenseAccountCode)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FixedAssetCategories::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(FixedAssetCategories::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(FixedAssetCategories::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create fixed assets table
        manager
            .create_table(
                Table::create()
                    .table(FixedAssets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FixedAssets::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(FixedAssets::InventoryNumber)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(FixedAssets::Name).string().not_null())
                    .col(ColumnDef::new(FixedAssets::Description).text())
                    .col(ColumnDef::new(FixedAssets::CategoryId).integer().not_null())
                    .col(ColumnDef::new(FixedAssets::CompanyId).integer().not_null())
                    // Financial data
                    .col(
                        ColumnDef::new(FixedAssets::AcquisitionCost)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FixedAssets::AcquisitionDate)
                            .date()
                            .not_null(),
                    )
                    .col(ColumnDef::new(FixedAssets::PutIntoServiceDate).date())
                    // Accounting depreciation
                    .col(
                        ColumnDef::new(FixedAssets::AccountingUsefulLife)
                            .integer()
                            .not_null(),
                    ) // in months
                    .col(
                        ColumnDef::new(FixedAssets::AccountingDepreciationRate)
                            .decimal_len(5, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FixedAssets::AccountingDepreciationMethod)
                            .string()
                            .not_null()
                            .default("straight_line"),
                    )
                    .col(
                        ColumnDef::new(FixedAssets::AccountingSalvageValue)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(FixedAssets::AccountingAccumulatedDepreciation)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0),
                    )
                    // Tax depreciation (ZКПО)
                    .col(ColumnDef::new(FixedAssets::TaxUsefulLife).integer()) // in months, can be different from accounting
                    .col(
                        ColumnDef::new(FixedAssets::TaxDepreciationRate)
                            .decimal_len(5, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FixedAssets::TaxAccumulatedDepreciation)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(FixedAssets::IsNewFirstTimeInvestment)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    // Book values
                    .col(
                        ColumnDef::new(FixedAssets::AccountingBookValue)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FixedAssets::TaxBookValue)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    // Status
                    .col(
                        ColumnDef::new(FixedAssets::Status)
                            .string()
                            .not_null()
                            .default("active"),
                    ) // active, disposed, sold
                    .col(ColumnDef::new(FixedAssets::DisposalDate).date())
                    .col(ColumnDef::new(FixedAssets::DisposalAmount).decimal_len(15, 2))
                    // Metadata
                    .col(ColumnDef::new(FixedAssets::Location).string())
                    .col(ColumnDef::new(FixedAssets::ResponsiblePerson).string())
                    .col(ColumnDef::new(FixedAssets::SerialNumber).string())
                    .col(ColumnDef::new(FixedAssets::Manufacturer).string())
                    .col(ColumnDef::new(FixedAssets::Model).string())
                    .col(ColumnDef::new(FixedAssets::Notes).text())
                    .col(
                        ColumnDef::new(FixedAssets::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(FixedAssets::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    // Foreign keys
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fixed_assets_category")
                            .from(FixedAssets::Table, FixedAssets::CategoryId)
                            .to(FixedAssetCategories::Table, FixedAssetCategories::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fixed_assets_company")
                            .from(FixedAssets::Table, FixedAssets::CompanyId)
                            .to(Alias::new("companies"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // Create depreciation journal table (for tracking monthly depreciation entries)
        manager
            .create_table(
                Table::create()
                    .table(DepreciationJournal::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DepreciationJournal::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DepreciationJournal::FixedAssetId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DepreciationJournal::Period)
                            .date()
                            .not_null(),
                    ) // YYYY-MM-01 format
                    .col(
                        ColumnDef::new(DepreciationJournal::CompanyId)
                            .integer()
                            .not_null(),
                    )
                    // Accounting depreciation
                    .col(
                        ColumnDef::new(DepreciationJournal::AccountingDepreciationAmount)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DepreciationJournal::AccountingBookValueBefore)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DepreciationJournal::AccountingBookValueAfter)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    // Tax depreciation
                    .col(
                        ColumnDef::new(DepreciationJournal::TaxDepreciationAmount)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DepreciationJournal::TaxBookValueBefore)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DepreciationJournal::TaxBookValueAfter)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    // Journal entry reference
                    .col(ColumnDef::new(DepreciationJournal::JournalEntryId).integer())
                    .col(
                        ColumnDef::new(DepreciationJournal::IsPosted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(DepreciationJournal::PostedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(DepreciationJournal::PostedBy).integer())
                    .col(
                        ColumnDef::new(DepreciationJournal::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(DepreciationJournal::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    // Unique constraint on asset + period
                    .index(
                        Index::create()
                            .name("idx_depreciation_asset_period")
                            .table(DepreciationJournal::Table)
                            .col(DepreciationJournal::FixedAssetId)
                            .col(DepreciationJournal::Period)
                            .unique(),
                    )
                    // Foreign keys
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_depreciation_journal_asset")
                            .from(
                                DepreciationJournal::Table,
                                DepreciationJournal::FixedAssetId,
                            )
                            .to(FixedAssets::Table, FixedAssets::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_depreciation_journal_company")
                            .from(DepreciationJournal::Table, DepreciationJournal::CompanyId)
                            .to(Alias::new("companies"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // Insert default Bulgarian fixed asset categories according to ЗКПО
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(FixedAssetCategories::Table)
                    .columns([
                        FixedAssetCategories::Code,
                        FixedAssetCategories::Name,
                        FixedAssetCategories::Description,
                        FixedAssetCategories::TaxCategory,
                        FixedAssetCategories::MaxTaxDepreciationRate,
                        FixedAssetCategories::DefaultAccountingDepreciationRate,
                        FixedAssetCategories::MinUsefulLife,
                        FixedAssetCategories::MaxUsefulLife,
                        FixedAssetCategories::AssetAccountCode,
                        FixedAssetCategories::DepreciationAccountCode,
                        FixedAssetCategories::ExpenseAccountCode,
                    ])
                    .values_panic([
                        "BUILDINGS".into(),
                        "Сгради и съоръжения".into(),
                        "Категория I по ЗКПО - сгради, инфраструктура".into(),
                        1.into(),
                        4.00.into(), // 4% max tax rate
                        4.00.into(),
                        300.into(),   // 25 years min
                        600.into(),   // 50 years max
                        "201".into(), // Asset account
                        "241".into(), // Accumulated depreciation account
                        "603".into(), // Depreciation expense account
                    ])
                    .values_panic([
                        "MACHINERY".into(),
                        "Машини и оборудване".into(),
                        "Категория II по ЗКПО - машини, оборудване (до 30% или 50% за нови)".into(),
                        2.into(),
                        30.00.into(), // 30% max tax rate (50% for new investments)
                        10.00.into(),
                        36.into(),  // 3 years min
                        120.into(), // 10 years max
                        "204".into(),
                        "241".into(),
                        "603".into(),
                    ])
                    .values_panic([
                        "TRANSPORT".into(),
                        "Транспортни средства".into(),
                        "Категория III по ЗКПО - транспорт без автомобили".into(),
                        3.into(),
                        10.00.into(), // 10% max tax rate
                        12.00.into(),
                        60.into(),  // 5 years min
                        180.into(), // 15 years max
                        "205".into(),
                        "241".into(),
                        "603".into(),
                    ])
                    .values_panic([
                        "COMPUTERS".into(),
                        "Компютри и софтуер".into(),
                        "Категория IV по ЗКПО - компютри, софтуер".into(),
                        4.into(),
                        50.00.into(), // 50% max tax rate
                        33.33.into(),
                        24.into(), // 2 years min
                        60.into(), // 5 years max
                        "207".into(),
                        "241".into(),
                        "603".into(),
                    ])
                    .values_panic([
                        "VEHICLES".into(),
                        "Автомобили".into(),
                        "Категория V по ЗКПО - автомобили".into(),
                        5.into(),
                        25.00.into(), // 25% max tax rate
                        20.00.into(),
                        48.into(), // 4 years min
                        96.into(), // 8 years max
                        "206".into(),
                        "241".into(),
                        "603".into(),
                    ])
                    .values_panic([
                        "OTHER".into(),
                        "Други ДМА".into(),
                        "Категория VII по ЗКПО - други дълготрайни материални активи".into(),
                        7.into(),
                        15.00.into(), // 15% max tax rate
                        10.00.into(),
                        60.into(),  // 5 years min
                        180.into(), // 15 years max
                        "208".into(),
                        "241".into(),
                        "603".into(),
                    ])
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DepreciationJournal::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(FixedAssets::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(FixedAssetCategories::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum FixedAssetCategories {
    Table,
    Id,
    Code,
    Name,
    Description,
    TaxCategory, // 1-7 according to ЗКПО
    MaxTaxDepreciationRate,
    DefaultAccountingDepreciationRate,
    MinUsefulLife,
    MaxUsefulLife,
    AssetAccountCode,
    DepreciationAccountCode,
    ExpenseAccountCode,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum FixedAssets {
    Table,
    Id,
    InventoryNumber,
    Name,
    Description,
    CategoryId,
    CompanyId,
    AcquisitionCost,
    AcquisitionDate,
    PutIntoServiceDate,
    AccountingUsefulLife,
    AccountingDepreciationRate,
    AccountingDepreciationMethod,
    AccountingSalvageValue,
    AccountingAccumulatedDepreciation,
    TaxUsefulLife,
    TaxDepreciationRate,
    TaxAccumulatedDepreciation,
    IsNewFirstTimeInvestment,
    AccountingBookValue,
    TaxBookValue,
    Status,
    DisposalDate,
    DisposalAmount,
    Location,
    ResponsiblePerson,
    SerialNumber,
    Manufacturer,
    Model,
    Notes,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum DepreciationJournal {
    Table,
    Id,
    FixedAssetId,
    Period,
    CompanyId,
    AccountingDepreciationAmount,
    AccountingBookValueBefore,
    AccountingBookValueAfter,
    TaxDepreciationAmount,
    TaxBookValueBefore,
    TaxBookValueAfter,
    JournalEntryId,
    IsPosted,
    PostedAt,
    PostedBy,
    CreatedAt,
    UpdatedAt,
}
