use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Currencies::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Currencies::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Currencies::Code).string_len(3).not_null())
                    .col(ColumnDef::new(Currencies::Name).string().not_null())
                    .col(ColumnDef::new(Currencies::NameBg).string().not_null())
                    .col(ColumnDef::new(Currencies::Symbol).string_len(10))
                    .col(
                        ColumnDef::new(Currencies::DecimalPlaces)
                            .integer()
                            .not_null()
                            .default(2),
                    )
                    .col(
                        ColumnDef::new(Currencies::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Currencies::IsBaseCurrency)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Currencies::BnbCode).string_len(3)) // БНБ код за автоматично обновяване
                    .col(
                        ColumnDef::new(Currencies::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Currencies::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index for currency code
        manager
            .create_index(
                Index::create()
                    .name("idx_currencies_code")
                    .table(Currencies::Table)
                    .col(Currencies::Code)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Insert default currencies
        let insert = Query::insert()
            .into_table(Currencies::Table)
            .columns([
                Currencies::Code,
                Currencies::Name,
                Currencies::NameBg,
                Currencies::Symbol,
                Currencies::IsBaseCurrency,
                Currencies::BnbCode,
            ])
            .values_panic([
                "BGN".into(),
                "Bulgarian Lev".into(),
                "Български лев".into(),
                "лв.".into(),
                true.into(),
                Option::<String>::None.into(), // БНБ не дава курс за BGN (това е базовата валута)
            ])
            .values_panic([
                "EUR".into(),
                "Euro".into(),
                "Евро".into(),
                "€".into(),
                false.into(),
                "EUR".into(),
            ])
            .values_panic([
                "USD".into(),
                "US Dollar".into(),
                "Американски долар".into(),
                "$".into(),
                false.into(),
                "USD".into(),
            ])
            .values_panic([
                "GBP".into(),
                "British Pound".into(),
                "Британска лира".into(),
                "£".into(),
                false.into(),
                "GBP".into(),
            ])
            .values_panic([
                "CHF".into(),
                "Swiss Franc".into(),
                "Швейцарски франк".into(),
                "Fr.".into(),
                false.into(),
                "CHF".into(),
            ])
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Currencies::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Currencies {
    Table,
    Id,
    Code,
    Name,
    NameBg,
    Symbol,
    DecimalPlaces,
    IsActive,
    IsBaseCurrency,
    BnbCode,
    CreatedAt,
    UpdatedAt,
}
