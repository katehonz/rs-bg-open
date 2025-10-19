use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Counterparts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Counterparts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Counterparts::Name).string().not_null())
                    .col(ColumnDef::new(Counterparts::VatNumber).string())
                    .col(ColumnDef::new(Counterparts::Address).text())
                    .col(ColumnDef::new(Counterparts::Phone).string())
                    .col(ColumnDef::new(Counterparts::Email).string())
                    .col(ColumnDef::new(Counterparts::ContactPerson).string())
                    .col(
                        ColumnDef::new(Counterparts::CounterpartType)
                            .string()
                            .not_null()
                            .default("CUSTOMER"),
                    )
                    .col(
                        ColumnDef::new(Counterparts::IsVatRegistered)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Counterparts::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(Counterparts::CompanyId).integer().not_null())
                    .col(
                        ColumnDef::new(Counterparts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Counterparts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_counterparts_company_id")
                            .from(Counterparts::Table, Counterparts::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for VAT number lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_counterparts_vat_number")
                    .table(Counterparts::Table)
                    .col(Counterparts::VatNumber)
                    .to_owned(),
            )
            .await?;

        // Create index for counterpart type
        manager
            .create_index(
                Index::create()
                    .name("idx_counterparts_type")
                    .table(Counterparts::Table)
                    .col(Counterparts::CounterpartType)
                    .to_owned(),
            )
            .await?;

        // Create index for company + name (for searches)
        manager
            .create_index(
                Index::create()
                    .name("idx_counterparts_company_name")
                    .table(Counterparts::Table)
                    .col(Counterparts::CompanyId)
                    .col(Counterparts::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Counterparts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Counterparts {
    Table,
    Id,
    Name,
    VatNumber,
    Address,
    Phone,
    Email,
    ContactPerson,
    CounterpartType,
    IsVatRegistered,
    IsActive,
    CompanyId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Companies {
    Table,
    Id,
}
