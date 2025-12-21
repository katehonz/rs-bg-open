use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AiBankAccountingSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AiBankAccountingSettings::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AiBankAccountingSettings::CompanyId)
                            .integer()
                            .not_null(),
                    )
                    // Pattern for matching transaction descriptions
                    .col(
                        ColumnDef::new(AiBankAccountingSettings::PatternName)
                            .string_len(255)
                            .not_null(), // e.g., "Покупка на ПОС", "Превод към доставчик"
                    )
                    .col(
                        ColumnDef::new(AiBankAccountingSettings::DescriptionKeywords)
                            .text()
                            .null(), // Comma-separated keywords for AI matching
                    )
                    // Transaction type classification
                    .col(
                        ColumnDef::new(AiBankAccountingSettings::TransactionType)
                            .string_len(50)
                            .not_null(), // "pos_purchase", "bank_transfer", "card_payment", etc.
                    )
                    // Accounting configuration
                    .col(
                        ColumnDef::new(AiBankAccountingSettings::AccountId)
                            .integer()
                            .null(), // Main account (e.g., 501 - Материали, 401 - Доставчици)
                    )
                    .col(
                        ColumnDef::new(AiBankAccountingSettings::CounterpartAccountId)
                            .integer()
                            .null(), // Counterpart account (used when no counterpart entity)
                    )
                    .col(
                        ColumnDef::new(AiBankAccountingSettings::VatAccountId)
                            .integer()
                            .null(), // VAT account if applicable
                    )
                    // Transaction direction
                    .col(
                        ColumnDef::new(AiBankAccountingSettings::Direction)
                            .string_len(20)
                            .not_null()
                            .default("debit"), // "debit" or "credit"
                    )
                    // Description template
                    .col(
                        ColumnDef::new(AiBankAccountingSettings::DescriptionTemplate)
                            .text()
                            .null(), // Template for entry description
                    )
                    // Priority for matching (higher = higher priority)
                    .col(
                        ColumnDef::new(AiBankAccountingSettings::Priority)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    // Active flag
                    .col(
                        ColumnDef::new(AiBankAccountingSettings::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    // Timestamps
                    .col(
                        ColumnDef::new(AiBankAccountingSettings::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AiBankAccountingSettings::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ai_bank_accounting_settings_company")
                            .from(AiBankAccountingSettings::Table, AiBankAccountingSettings::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ai_bank_accounting_settings_account")
                            .from(AiBankAccountingSettings::Table, AiBankAccountingSettings::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ai_bank_accounting_settings_counterpart_account")
                            .from(AiBankAccountingSettings::Table, AiBankAccountingSettings::CounterpartAccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ai_bank_accounting_settings_vat_account")
                            .from(AiBankAccountingSettings::Table, AiBankAccountingSettings::VatAccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on company_id for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_ai_bank_accounting_settings_company")
                    .table(AiBankAccountingSettings::Table)
                    .col(AiBankAccountingSettings::CompanyId)
                    .to_owned(),
            )
            .await?;

        // Create index on transaction_type for faster matching
        manager
            .create_index(
                Index::create()
                    .name("idx_ai_bank_accounting_settings_type")
                    .table(AiBankAccountingSettings::Table)
                    .col(AiBankAccountingSettings::TransactionType)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AiBankAccountingSettings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AiBankAccountingSettings {
    #[sea_orm(iden = "ai_bank_accounting_settings")]
    Table,
    Id,
    CompanyId,
    PatternName,
    DescriptionKeywords,
    TransactionType,
    AccountId,
    CounterpartAccountId,
    VatAccountId,
    Direction,
    DescriptionTemplate,
    Priority,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Companies {
    #[sea_orm(iden = "companies")]
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Accounts {
    #[sea_orm(iden = "accounts")]
    Table,
    Id,
}
