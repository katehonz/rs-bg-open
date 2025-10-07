use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Companies::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Companies::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Companies::Name).string().not_null())
                    .col(ColumnDef::new(Companies::VatNumber).string())
                    .col(ColumnDef::new(Companies::Address).text())
                    .col(ColumnDef::new(Companies::Phone).string())
                    .col(ColumnDef::new(Companies::Email).string())
                    .col(ColumnDef::new(Companies::ContactPerson).string())
                    .col(
                        ColumnDef::new(Companies::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Companies::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Companies::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for VAT number lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_companies_vat_number")
                    .table(Companies::Table)
                    .col(Companies::VatNumber)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Companies::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Companies {
    Table,
    Id,
    Name,
    VatNumber,
    Address,
    Phone,
    Email,
    ContactPerson,
    IsActive,
    CreatedAt,
    UpdatedAt,
}
