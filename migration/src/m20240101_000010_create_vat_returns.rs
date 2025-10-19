use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(VatReturns::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(VatReturns::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(VatReturns::PeriodYear).integer().not_null())
                    .col(ColumnDef::new(VatReturns::PeriodMonth).integer().not_null())
                    .col(ColumnDef::new(VatReturns::PeriodFrom).date().not_null())
                    .col(ColumnDef::new(VatReturns::PeriodTo).date().not_null())
                    // VAT amounts in BGN
                    .col(
                        ColumnDef::new(VatReturns::OutputVatAmount)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(VatReturns::InputVatAmount)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(VatReturns::VatToPay)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(VatReturns::VatToRefund)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00),
                    )
                    // Base amounts for different VAT rates (20%, 9%, 0%)
                    .col(
                        ColumnDef::new(VatReturns::BaseAmount20)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(VatReturns::VatAmount20)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(VatReturns::BaseAmount9)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(VatReturns::VatAmount9)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(VatReturns::BaseAmount0)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00),
                    )
                    .col(
                        ColumnDef::new(VatReturns::ExemptAmount)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00),
                    )
                    // Status and submission
                    .col(
                        ColumnDef::new(VatReturns::Status)
                            .string()
                            .not_null()
                            .default("DRAFT"), // DRAFT, SUBMITTED, APPROVED
                    )
                    .col(ColumnDef::new(VatReturns::SubmittedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(VatReturns::SubmittedBy).integer())
                    .col(ColumnDef::new(VatReturns::DueDate).date().not_null()) // 14th of next month
                    .col(ColumnDef::new(VatReturns::Notes).text())
                    .col(ColumnDef::new(VatReturns::CompanyId).integer().not_null())
                    .col(ColumnDef::new(VatReturns::CreatedBy).integer().not_null())
                    .col(
                        ColumnDef::new(VatReturns::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(VatReturns::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_vat_returns_company_id")
                            .from(VatReturns::Table, VatReturns::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_vat_returns_created_by")
                            .from(VatReturns::Table, VatReturns::CreatedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index for monthly period per company
        manager
            .create_index(
                Index::create()
                    .name("idx_vat_returns_company_period")
                    .table(VatReturns::Table)
                    .col(VatReturns::CompanyId)
                    .col(VatReturns::PeriodYear)
                    .col(VatReturns::PeriodMonth)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Index for status queries
        manager
            .create_index(
                Index::create()
                    .name("idx_vat_returns_status")
                    .table(VatReturns::Table)
                    .col(VatReturns::Status)
                    .to_owned(),
            )
            .await?;

        // Index for due date queries
        manager
            .create_index(
                Index::create()
                    .name("idx_vat_returns_due_date")
                    .table(VatReturns::Table)
                    .col(VatReturns::DueDate)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(VatReturns::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum VatReturns {
    Table,
    Id,
    PeriodYear,
    PeriodMonth,
    PeriodFrom,
    PeriodTo,
    OutputVatAmount,
    InputVatAmount,
    VatToPay,
    VatToRefund,
    BaseAmount20,
    VatAmount20,
    BaseAmount9,
    VatAmount9,
    BaseAmount0,
    ExemptAmount,
    Status,
    SubmittedAt,
    SubmittedBy,
    DueDate,
    Notes,
    CompanyId,
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
enum Users {
    Table,
    Id,
}
