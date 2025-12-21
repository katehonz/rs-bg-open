use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ExchangeRates::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExchangeRates::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ExchangeRates::FromCurrencyId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExchangeRates::ToCurrencyId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExchangeRates::Rate)
                            .decimal_len(15, 6)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExchangeRates::ReverseRate)
                            .decimal_len(15, 6)
                            .not_null(),
                    )
                    .col(ColumnDef::new(ExchangeRates::ValidDate).date().not_null())
                    .col(
                        ColumnDef::new(ExchangeRates::RateSource)
                            .string()
                            .not_null()
                            .default("BNB"), // BNB, MANUAL, ECB, etc.
                    )
                    .col(ColumnDef::new(ExchangeRates::BnbRateId).string()) // ID от БНБ API
                    .col(
                        ColumnDef::new(ExchangeRates::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(ExchangeRates::Notes).text())
                    .col(ColumnDef::new(ExchangeRates::CreatedBy).integer())
                    .col(
                        ColumnDef::new(ExchangeRates::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ExchangeRates::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_exchange_rates_from_currency")
                            .from(ExchangeRates::Table, ExchangeRates::FromCurrencyId)
                            .to(Currencies::Table, Currencies::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_exchange_rates_to_currency")
                            .from(ExchangeRates::Table, ExchangeRates::ToCurrencyId)
                            .to(Currencies::Table, Currencies::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_exchange_rates_created_by")
                            .from(ExchangeRates::Table, ExchangeRates::CreatedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index for currency pair and date
        manager
            .create_index(
                Index::create()
                    .name("idx_exchange_rates_currencies_date")
                    .table(ExchangeRates::Table)
                    .col(ExchangeRates::FromCurrencyId)
                    .col(ExchangeRates::ToCurrencyId)
                    .col(ExchangeRates::ValidDate)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create index for date queries (performance)
        manager
            .create_index(
                Index::create()
                    .name("idx_exchange_rates_valid_date")
                    .table(ExchangeRates::Table)
                    .col(ExchangeRates::ValidDate)
                    .to_owned(),
            )
            .await?;

        // Create index for BNB rate ID (for updates)
        manager
            .create_index(
                Index::create()
                    .name("idx_exchange_rates_bnb_rate_id")
                    .table(ExchangeRates::Table)
                    .col(ExchangeRates::BnbRateId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ExchangeRates::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ExchangeRates {
    Table,
    Id,
    FromCurrencyId,
    ToCurrencyId,
    Rate,
    ReverseRate,
    ValidDate,
    RateSource,
    BnbRateId,
    IsActive,
    Notes,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Currencies {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
