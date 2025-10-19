use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserGroups::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserGroups::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserGroups::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(UserGroups::Description).text())
                    .col(
                        ColumnDef::new(UserGroups::CanCreateCompanies)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(UserGroups::CanEditCompanies)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(UserGroups::CanDeleteCompanies)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(UserGroups::CanManageUsers)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(UserGroups::CanViewReports)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(UserGroups::CanPostEntries)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(UserGroups::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Insert default user groups
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(UserGroups::Table)
                    .columns([
                        UserGroups::Name,
                        UserGroups::Description,
                        UserGroups::CanCreateCompanies,
                        UserGroups::CanEditCompanies,
                        UserGroups::CanDeleteCompanies,
                        UserGroups::CanManageUsers,
                        UserGroups::CanViewReports,
                        UserGroups::CanPostEntries,
                    ])
                    .values_panic([
                        "superadmin".into(),
                        "Super Administrator - full access".into(),
                        true.into(),
                        true.into(),
                        true.into(),
                        true.into(),
                        true.into(),
                        true.into(),
                    ])
                    .values_panic([
                        "admin".into(),
                        "Administrator - company management".into(),
                        true.into(),
                        true.into(),
                        false.into(),
                        true.into(),
                        true.into(),
                        true.into(),
                    ])
                    .values_panic([
                        "accountant".into(),
                        "Accountant - can post entries".into(),
                        false.into(),
                        false.into(),
                        false.into(),
                        false.into(),
                        true.into(),
                        true.into(),
                    ])
                    .values_panic([
                        "user".into(),
                        "Regular User - view only".into(),
                        false.into(),
                        false.into(),
                        false.into(),
                        false.into(),
                        true.into(),
                        false.into(),
                    ])
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserGroups::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserGroups {
    Table,
    Id,
    Name,
    Description,
    CanCreateCompanies,
    CanEditCompanies,
    CanDeleteCompanies,
    CanManageUsers,
    CanViewReports,
    CanPostEntries,
    CreatedAt,
}
