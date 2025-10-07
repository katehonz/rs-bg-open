use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(VatRates::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(VatRates::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(VatRates::Code).string().not_null())
                    .col(ColumnDef::new(VatRates::Name).string().not_null())
                    .col(ColumnDef::new(VatRates::Rate).decimal_len(5, 2).not_null())
                    .col(
                        ColumnDef::new(VatRates::VatDirection)
                            .string()
                            .not_null()
                            .default("NONE"),
                    )
                    .col(
                        ColumnDef::new(VatRates::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(VatRates::ValidFrom)
                            .date()
                            .not_null()
                            .default(Expr::val("2007-01-01").cast_as(Alias::new("date"))), // EU membership
                    )
                    .col(ColumnDef::new(VatRates::ValidTo).date())
                    .col(ColumnDef::new(VatRates::CompanyId).integer().not_null())
                    .col(
                        ColumnDef::new(VatRates::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(VatRates::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_vat_rates_company_id")
                            .from(VatRates::Table, VatRates::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index for code per company
        manager
            .create_index(
                Index::create()
                    .name("idx_vat_rates_company_code")
                    .table(VatRates::Table)
                    .col(VatRates::CompanyId)
                    .col(VatRates::Code)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Note: Default Bulgarian VAT rates will be inserted via application logic
        // when companies are created

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(VatRates::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum VatRates {
    Table,
    Id,
    Code,
    Name,
    Rate,
    VatDirection,
    IsActive,
    ValidFrom,
    ValidTo,
    CompanyId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Companies {
    Table,
    Id,
}
