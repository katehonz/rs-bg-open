use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AiAccountingSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AiAccountingSettings::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AiAccountingSettings::CompanyId)
                            .integer()
                            .not_null(),
                    )
                    // Sales accounts (Продажби)
                    .col(
                        ColumnDef::new(AiAccountingSettings::SalesRevenueAccount)
                            .string_len(20)
                            .not_null()
                            .default("701"), // Приходи от продажби на стоки
                    )
                    .col(
                        ColumnDef::new(AiAccountingSettings::SalesServicesAccount)
                            .string_len(20)
                            .not_null()
                            .default("703"), // Приходи от услуги
                    )
                    .col(
                        ColumnDef::new(AiAccountingSettings::SalesReceivablesAccount)
                            .string_len(20)
                            .not_null()
                            .default("411"), // Клиенти
                    )
                    // Purchase accounts (Покупки)
                    .col(
                        ColumnDef::new(AiAccountingSettings::PurchaseExpenseAccount)
                            .string_len(20)
                            .not_null()
                            .default("602"), // Разходи за материали
                    )
                    .col(
                        ColumnDef::new(AiAccountingSettings::PurchasePayablesAccount)
                            .string_len(20)
                            .not_null()
                            .default("401"), // Доставчици
                    )
                    // VAT accounts
                    .col(
                        ColumnDef::new(AiAccountingSettings::VatInputAccount)
                            .string_len(20)
                            .not_null()
                            .default("4531"), // ДДС за възстановяване
                    )
                    .col(
                        ColumnDef::new(AiAccountingSettings::VatOutputAccount)
                            .string_len(20)
                            .not_null()
                            .default("4531"), // ДДС за внасяне
                    )
                    // Special cases
                    .col(
                        ColumnDef::new(AiAccountingSettings::NonRegisteredPersonsAccount)
                            .string_len(20)
                            .null(), // За фактури от нерегистрирани лица (0% ДДС, кол09)
                    )
                    .col(
                        ColumnDef::new(AiAccountingSettings::NonRegisteredVatOperation)
                            .string_len(20)
                            .default("про09"), // Колона 09 в ДДС дневник
                    )
                    // Account code length (3 or 4 digits)
                    .col(
                        ColumnDef::new(AiAccountingSettings::AccountCodeLength)
                            .integer()
                            .not_null()
                            .default(3), // 3 digits by default (401, 411, 701), or 4 (4011, 4111, 7011)
                    )
                    // Default transaction descriptions
                    .col(
                        ColumnDef::new(AiAccountingSettings::SalesDescriptionTemplate)
                            .text()
                            .default("{counterpart} - {document_number}"),
                    )
                    .col(
                        ColumnDef::new(AiAccountingSettings::PurchaseDescriptionTemplate)
                            .text()
                            .default("{counterpart} - {document_number}"),
                    )
                    // Timestamps
                    .col(
                        ColumnDef::new(AiAccountingSettings::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AiAccountingSettings::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ai_accounting_settings_company")
                            .from(AiAccountingSettings::Table, AiAccountingSettings::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index on company_id
        manager
            .create_index(
                Index::create()
                    .name("idx_ai_accounting_settings_company")
                    .table(AiAccountingSettings::Table)
                    .col(AiAccountingSettings::CompanyId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AiAccountingSettings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AiAccountingSettings {
    #[sea_orm(iden = "ai_accounting_settings")]
    Table,
    Id,
    CompanyId,
    SalesRevenueAccount,
    SalesServicesAccount,
    SalesReceivablesAccount,
    PurchaseExpenseAccount,
    PurchasePayablesAccount,
    VatInputAccount,
    VatOutputAccount,
    NonRegisteredPersonsAccount,
    NonRegisteredVatOperation,
    AccountCodeLength,
    SalesDescriptionTemplate,
    PurchaseDescriptionTemplate,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Companies {
    #[sea_orm(iden = "companies")]
    Table,
    Id,
}
