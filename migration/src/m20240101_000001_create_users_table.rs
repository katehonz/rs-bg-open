use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(ColumnDef::new(Users::FirstName).string().not_null())
                    .col(ColumnDef::new(Users::LastName).string().not_null())
                    .col(ColumnDef::new(Users::GroupId).integer().not_null())
                    .col(
                        ColumnDef::new(Users::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    // Personal input periods - Documents
                    .col(
                        ColumnDef::new(Users::DocumentPeriodStart)
                            .date()
                            .not_null()
                            .default("2020-01-01"),
                    )
                    .col(
                        ColumnDef::new(Users::DocumentPeriodEnd)
                            .date()
                            .not_null()
                            .default("2030-12-31"),
                    )
                    .col(
                        ColumnDef::new(Users::DocumentPeriodActive)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    // Personal input periods - Accounting
                    .col(
                        ColumnDef::new(Users::AccountingPeriodStart)
                            .date()
                            .not_null()
                            .default("2020-01-01"),
                    )
                    .col(
                        ColumnDef::new(Users::AccountingPeriodEnd)
                            .date()
                            .not_null()
                            .default("2030-12-31"),
                    )
                    .col(
                        ColumnDef::new(Users::AccountingPeriodActive)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    // Personal input periods - VAT
                    .col(
                        ColumnDef::new(Users::VatPeriodStart)
                            .date()
                            .not_null()
                            .default("2020-01-01"),
                    )
                    .col(
                        ColumnDef::new(Users::VatPeriodEnd)
                            .date()
                            .not_null()
                            .default("2030-12-31"),
                    )
                    .col(
                        ColumnDef::new(Users::VatPeriodActive)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_users_group_id")
                            .from(Users::Table, Users::GroupId)
                            .to(UserGroups::Table, UserGroups::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Username,
    Email,
    PasswordHash,
    FirstName,
    LastName,
    GroupId,
    IsActive,
    DocumentPeriodStart,
    DocumentPeriodEnd,
    DocumentPeriodActive,
    AccountingPeriodStart,
    AccountingPeriodEnd,
    AccountingPeriodActive,
    VatPeriodStart,
    VatPeriodEnd,
    VatPeriodActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum UserGroups {
    Table,
    Id,
}
