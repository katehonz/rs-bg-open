use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BankProfiles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BankProfiles::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BankProfiles::CompanyId).integer().not_null())
                    .col(ColumnDef::new(BankProfiles::Name).string().not_null())
                    .col(ColumnDef::new(BankProfiles::Iban).string())
                    .col(ColumnDef::new(BankProfiles::AccountId).integer().not_null())
                    .col(
                        ColumnDef::new(BankProfiles::BufferAccountId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BankProfiles::CurrencyCode)
                            .string_len(3)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BankProfiles::ImportFormat)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BankProfiles::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(BankProfiles::Settings).json())
                    .col(ColumnDef::new(BankProfiles::CreatedBy).integer().null())
                    .col(
                        ColumnDef::new(BankProfiles::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(BankProfiles::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bank_profiles_company")
                            .from(BankProfiles::Table, BankProfiles::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bank_profiles_account")
                            .from(BankProfiles::Table, BankProfiles::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bank_profiles_buffer_account")
                            .from(BankProfiles::Table, BankProfiles::BufferAccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bank_profiles_created_by")
                            .from(BankProfiles::Table, BankProfiles::CreatedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_bank_profiles_company_active")
                    .table(BankProfiles::Table)
                    .col(BankProfiles::CompanyId)
                    .col(BankProfiles::IsActive)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(BankImports::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BankImports::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(BankImports::BankProfileId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(BankImports::CompanyId).integer().not_null())
                    .col(ColumnDef::new(BankImports::FileName).string().not_null())
                    .col(
                        ColumnDef::new(BankImports::ImportFormat)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BankImports::ImportedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(BankImports::TransactionsCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(BankImports::TotalCredit)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(BankImports::TotalDebit)
                            .decimal_len(18, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(BankImports::CreatedJournalEntries)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(BankImports::JournalEntryIds).json())
                    .col(
                        ColumnDef::new(BankImports::Status)
                            .string()
                            .not_null()
                            .default("completed"),
                    )
                    .col(ColumnDef::new(BankImports::ErrorMessage).text())
                    .col(ColumnDef::new(BankImports::CreatedBy).integer().null())
                    .col(
                        ColumnDef::new(BankImports::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(BankImports::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bank_imports_profile")
                            .from(BankImports::Table, BankImports::BankProfileId)
                            .to(BankProfiles::Table, BankProfiles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bank_imports_company")
                            .from(BankImports::Table, BankImports::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bank_imports_created_by")
                            .from(BankImports::Table, BankImports::CreatedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_bank_imports_profile")
                    .table(BankImports::Table)
                    .col(BankImports::BankProfileId)
                    .col(BankImports::ImportedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_bank_imports_company")
                    .table(BankImports::Table)
                    .col(BankImports::CompanyId)
                    .col(BankImports::ImportedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BankImports::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BankProfiles::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum BankProfiles {
    Table,
    Id,
    CompanyId,
    Name,
    Iban,
    AccountId,
    BufferAccountId,
    CurrencyCode,
    ImportFormat,
    IsActive,
    Settings,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum BankImports {
    Table,
    Id,
    BankProfileId,
    CompanyId,
    FileName,
    ImportFormat,
    ImportedAt,
    TransactionsCount,
    TotalCredit,
    TotalDebit,
    CreatedJournalEntries,
    JournalEntryIds,
    Status,
    ErrorMessage,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
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
enum Users {
    Table,
    Id,
}
