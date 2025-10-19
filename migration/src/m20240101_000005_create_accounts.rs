use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Accounts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Accounts::Code).string().not_null())
                    .col(ColumnDef::new(Accounts::Name).string().not_null())
                    .col(ColumnDef::new(Accounts::AccountType).string().not_null())
                    .col(ColumnDef::new(Accounts::AccountClass).integer().not_null())
                    .col(ColumnDef::new(Accounts::ParentId).integer())
                    .col(
                        ColumnDef::new(Accounts::Level)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(Accounts::IsVatApplicable)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Accounts::VatDirection)
                            .string()
                            .not_null()
                            .default("NONE"),
                    )
                    .col(
                        ColumnDef::new(Accounts::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Accounts::IsAnalytical)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Accounts::CompanyId).integer().not_null())
                    .col(
                        ColumnDef::new(Accounts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Accounts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_accounts_company_id")
                            .from(Accounts::Table, Accounts::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_accounts_parent_id")
                            .from(Accounts::Table, Accounts::ParentId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique constraint for company_id + code
        manager
            .create_index(
                Index::create()
                    .name("idx_accounts_company_code_unique")
                    .table(Accounts::Table)
                    .col(Accounts::CompanyId)
                    .col(Accounts::Code)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create index for account class lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_accounts_class")
                    .table(Accounts::Table)
                    .col(Accounts::AccountClass)
                    .to_owned(),
            )
            .await?;

        // Create index for VAT accounts
        manager
            .create_index(
                Index::create()
                    .name("idx_accounts_vat_applicable")
                    .table(Accounts::Table)
                    .col(Accounts::IsVatApplicable)
                    .to_owned(),
            )
            .await?;

        // Create index for parent-child hierarchy
        manager
            .create_index(
                Index::create()
                    .name("idx_accounts_parent_id")
                    .table(Accounts::Table)
                    .col(Accounts::ParentId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Accounts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
    Code,
    Name,
    AccountType,
    AccountClass,
    ParentId,
    Level,
    IsVatApplicable,
    VatDirection,
    IsActive,
    IsAnalytical,
    CompanyId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Companies {
    Table,
    Id,
}
