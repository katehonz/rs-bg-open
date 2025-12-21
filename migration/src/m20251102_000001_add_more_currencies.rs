use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Insert additional currencies supported by BNB
        // Based on BNB official exchange rates listing
        let insert = Query::insert()
            .into_table(Currencies::Table)
            .columns([
                Currencies::Code,
                Currencies::Name,
                Currencies::NameBg,
                Currencies::Symbol,
                Currencies::IsBaseCurrency,
                Currencies::IsActive,
                Currencies::BnbCode,
            ])
            // Major European currencies
            .values_panic([
                "SEK".into(),
                "Swedish Krona".into(),
                "Шведска крона".into(),
                "kr".into(),
                false.into(),
                false.into(), // Inactive by default
                "SEK".into(),
            ])
            .values_panic([
                "NOK".into(),
                "Norwegian Krone".into(),
                "Норвежка крона".into(),
                "kr".into(),
                false.into(),
                false.into(),
                "NOK".into(),
            ])
            .values_panic([
                "DKK".into(),
                "Danish Krone".into(),
                "Датска крона".into(),
                "kr".into(),
                false.into(),
                false.into(),
                "DKK".into(),
            ])
            .values_panic([
                "CZK".into(),
                "Czech Koruna".into(),
                "Чешка крона".into(),
                "Kč".into(),
                false.into(),
                false.into(),
                "CZK".into(),
            ])
            .values_panic([
                "PLN".into(),
                "Polish Zloty".into(),
                "Полска злота".into(),
                "zł".into(),
                false.into(),
                false.into(),
                "PLN".into(),
            ])
            .values_panic([
                "HUF".into(),
                "Hungarian Forint".into(),
                "Унгарски форинт".into(),
                "Ft".into(),
                false.into(),
                false.into(),
                "HUF".into(),
            ])
            .values_panic([
                "RON".into(),
                "Romanian Leu".into(),
                "Румънска лея".into(),
                "lei".into(),
                false.into(),
                false.into(),
                "RON".into(),
            ])
            .values_panic([
                "HRK".into(),
                "Croatian Kuna".into(),
                "Хърватска куна".into(),
                "kn".into(),
                false.into(),
                false.into(),
                "HRK".into(),
            ])
            // Asian currencies
            .values_panic([
                "JPY".into(),
                "Japanese Yen".into(),
                "Японска йена".into(),
                "¥".into(),
                false.into(),
                false.into(),
                "JPY".into(),
            ])
            .values_panic([
                "CNY".into(),
                "Chinese Yuan".into(),
                "Китайски юан".into(),
                "¥".into(),
                false.into(),
                false.into(),
                "CNY".into(),
            ])
            .values_panic([
                "KRW".into(),
                "South Korean Won".into(),
                "Южнокорейски вон".into(),
                "₩".into(),
                false.into(),
                false.into(),
                "KRW".into(),
            ])
            .values_panic([
                "INR".into(),
                "Indian Rupee".into(),
                "Индийска рупия".into(),
                "₹".into(),
                false.into(),
                false.into(),
                "INR".into(),
            ])
            .values_panic([
                "THB".into(),
                "Thai Baht".into(),
                "Тайландски бат".into(),
                "฿".into(),
                false.into(),
                false.into(),
                "THB".into(),
            ])
            .values_panic([
                "IDR".into(),
                "Indonesian Rupiah".into(),
                "Индонезийска рупия".into(),
                "Rp".into(),
                false.into(),
                false.into(),
                "IDR".into(),
            ])
            .values_panic([
                "MYR".into(),
                "Malaysian Ringgit".into(),
                "Малайзийски рингит".into(),
                "RM".into(),
                false.into(),
                false.into(),
                "MYR".into(),
            ])
            .values_panic([
                "PHP".into(),
                "Philippine Peso".into(),
                "Филипинско песо".into(),
                "₱".into(),
                false.into(),
                false.into(),
                "PHP".into(),
            ])
            .values_panic([
                "SGD".into(),
                "Singapore Dollar".into(),
                "Сингапурски долар".into(),
                "S$".into(),
                false.into(),
                false.into(),
                "SGD".into(),
            ])
            // Other major currencies
            .values_panic([
                "AUD".into(),
                "Australian Dollar".into(),
                "Австралийски долар".into(),
                "A$".into(),
                false.into(),
                false.into(),
                "AUD".into(),
            ])
            .values_panic([
                "CAD".into(),
                "Canadian Dollar".into(),
                "Канадски долар".into(),
                "C$".into(),
                false.into(),
                false.into(),
                "CAD".into(),
            ])
            .values_panic([
                "NZD".into(),
                "New Zealand Dollar".into(),
                "Новозеландски долар".into(),
                "NZ$".into(),
                false.into(),
                false.into(),
                "NZD".into(),
            ])
            .values_panic([
                "BRL".into(),
                "Brazilian Real".into(),
                "Бразилски реал".into(),
                "R$".into(),
                false.into(),
                false.into(),
                "BRL".into(),
            ])
            .values_panic([
                "ZAR".into(),
                "South African Rand".into(),
                "Южноафрикански ранд".into(),
                "R".into(),
                false.into(),
                false.into(),
                "ZAR".into(),
            ])
            // Regional currencies
            .values_panic([
                "TRY".into(),
                "Turkish Lira".into(),
                "Турска лира".into(),
                "₺".into(),
                false.into(),
                false.into(),
                "TRY".into(),
            ])
            .values_panic([
                "RUB".into(),
                "Russian Ruble".into(),
                "Руска рубла".into(),
                "₽".into(),
                false.into(),
                false.into(),
                "RUB".into(),
            ])
            .values_panic([
                "UAH".into(),
                "Ukrainian Hryvnia".into(),
                "Украинска гривна".into(),
                "₴".into(),
                false.into(),
                false.into(),
                "UAH".into(),
            ])
            .values_panic([
                "MXN".into(),
                "Mexican Peso".into(),
                "Мексиканско песо".into(),
                "Mex$".into(),
                false.into(),
                false.into(),
                "MXN".into(),
            ])
            .values_panic([
                "ISK".into(),
                "Icelandic Króna".into(),
                "Исландска крона".into(),
                "kr".into(),
                false.into(),
                false.into(),
                "ISK".into(),
            ])
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Delete the added currencies
        let delete = Query::delete()
            .from_table(Currencies::Table)
            .and_where(Expr::col(Currencies::Code).is_in([
                "SEK", "NOK", "DKK", "CZK", "PLN", "HUF", "RON", "HRK",
                "JPY", "CNY", "KRW", "INR", "THB", "IDR", "MYR", "PHP", "SGD",
                "AUD", "CAD", "NZD", "BRL", "ZAR",
                "TRY", "RUB", "UAH", "MXN", "ISK",
            ]))
            .to_owned();

        manager.exec_stmt(delete).await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Currencies {
    Table,
    Code,
    Name,
    NameBg,
    Symbol,
    IsBaseCurrency,
    IsActive,
    BnbCode,
}
