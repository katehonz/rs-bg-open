use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create user_companies junction table for many-to-many relationship
        manager
            .create_table(
                Table::create()
                    .table(UserCompanies::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserCompanies::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserCompanies::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(UserCompanies::CompanyId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserCompanies::Role)
                            .string()
                            .not_null()
                            .default("user"), // admin, user, viewer
                    )
                    .col(
                        ColumnDef::new(UserCompanies::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(UserCompanies::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserCompanies::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_companies_user_id")
                            .from(UserCompanies::Table, UserCompanies::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_companies_company_id")
                            .from(UserCompanies::Table, UserCompanies::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index to prevent duplicate user-company assignments
        manager
            .create_index(
                Index::create()
                    .name("idx_user_companies_unique")
                    .table(UserCompanies::Table)
                    .col(UserCompanies::UserId)
                    .col(UserCompanies::CompanyId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create index for faster lookups by company
        manager
            .create_index(
                Index::create()
                    .name("idx_user_companies_company_id")
                    .table(UserCompanies::Table)
                    .col(UserCompanies::CompanyId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserCompanies::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserCompanies {
    Table,
    Id,
    UserId,
    CompanyId,
    Role,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Companies {
    Table,
    Id,
}
