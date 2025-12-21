use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Insert the new "observer" user group with only view_reports permission
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
                        "observer".into(),
                        "Наблюдател - достъп само до справки".into(),
                        false.into(),  // can_create_companies
                        false.into(),  // can_edit_companies
                        false.into(),  // can_delete_companies
                        false.into(),  // can_manage_users
                        true.into(),   // can_view_reports - ONLY this is true
                        false.into(),  // can_post_entries
                    ])
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove the observer group
        manager
            .exec_stmt(
                Query::delete()
                    .from_table(UserGroups::Table)
                    .and_where(Expr::col(UserGroups::Name).eq("observer"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum UserGroups {
    Table,
    Name,
    Description,
    CanCreateCompanies,
    CanEditCompanies,
    CanDeleteCompanies,
    CanManageUsers,
    CanViewReports,
    CanPostEntries,
}
