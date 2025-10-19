use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Migration to add all NAP-specific fields to vat_returns table
/// Based on PPDDS_2025 specification for DEKLAR.TXT
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add sales (продажби) breakdown fields - corresponds to DEKLAR.TXT fields 01-11 to 01-19
        manager
            .alter_table(
                Table::alter()
                    .table(VatReturns::Table)
                    // Field 01-11: Данъчна основа 20%
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::SalesBase20)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-11: Данъчна основа 20% (про11)"),
                    )
                    // Field 01-21: ДДС 20%
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::SalesVat20)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-21: ДДС 20%"),
                    )
                    // Field 01-12: ВОП данъчна основа (про12)
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::SalesBaseVop)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-12: ВОП данъчна основа (про12)"),
                    )
                    // Field 01-22: ВОП ДДС
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::SalesVatVop)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-22: ВОП ДДС"),
                    )
                    // Field 01-23: ДДС за лично ползване
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::SalesVatPersonalUse)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-23: ДДС за лично ползване"),
                    )
                    // Field 01-13: Данъчна основа 9% (про17)
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::SalesBase9)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-13: Данъчна основа 9% (про17)"),
                    )
                    // Field 01-24: ДДС 9%
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::SalesVat9)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-24: ДДС 9%"),
                    )
                    // Field 01-14: Данъчна основа 0% чл.3
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::SalesBase0Art3)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-14: Данъчна основа 0% чл.3 (про14)"),
                    )
                    // Field 01-15: Данъчна основа 0% ВОД (про20)
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::SalesBase0Vod)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-15: Данъчна основа 0% ВОД (про20)"),
                    )
                    // Field 01-16: Данъчна основа 0% чл.140/146/173 (про19)
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::SalesBase0Export)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-16: Данъчна основа 0% чл.140/146/173 (про19)"),
                    )
                    // Field 01-17: Данъчна основа услуги чл.21
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::SalesBaseArt21)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-17: Данъчна основа услуги чл.21"),
                    )
                    // Field 01-18: Данъчна основа чл.69
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::SalesBaseArt69)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-18: Данъчна основа чл.69"),
                    )
                    // Field 01-19: Освободени доставки (про23)
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::SalesBaseExempt)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-19: Освободени доставки (про23)"),
                    )
                    .to_owned(),
            )
            .await?;

        // Add purchases (покупки) breakdown fields - corresponds to DEKLAR.TXT fields 01-30 to 01-43
        manager
            .alter_table(
                Table::alter()
                    .table(VatReturns::Table)
                    // Field 01-30: Данъчна основа без кредит
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::PurchaseBaseNoCredit)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-30: Данъчна основа без кредит (пок30)"),
                    )
                    // Field 01-31: Данъчна основа пълен кредит
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::PurchaseBaseFullCredit)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-31: Данъчна основа пълен кредит (пок09)"),
                    )
                    // Field 01-41: ДДС пълен кредит
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::PurchaseVatFullCredit)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-41: ДДС пълен кредит"),
                    )
                    // Field 01-32: Данъчна основа частичен кредит
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::PurchaseBasePartialCredit)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-32: Данъчна основа частичен кредит (пок10)"),
                    )
                    // Field 01-42: ДДС частичен кредит
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::PurchaseVatPartialCredit)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-42: ДДС частичен кредит"),
                    )
                    // Field 01-43: Годишно коригиране
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::PurchaseVatAnnualAdjustment)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-43: Годишно коригиране"),
                    )
                    .to_owned(),
            )
            .await?;

        // Add result fields - corresponds to DEKLAR.TXT fields 01-33, 01-40, 01-50, 01-60
        manager
            .alter_table(
                Table::alter()
                    .table(VatReturns::Table)
                    // Field 01-33: Коефициент
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::CreditCoefficient)
                            .decimal_len(4, 2)
                            .not_null()
                            .default(1.00)
                            .comment("Поле 01-33: Коефициент за частичен кредит"),
                    )
                    // Field 01-40: Общ данъчен кредит
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::TotalDeductibleVat)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-40: Общ данъчен кредит"),
                    )
                    .to_owned(),
            )
            .await?;

        // Rename existing fields to match documentation
        manager
            .alter_table(
                Table::alter()
                    .table(VatReturns::Table)
                    // Field 01-01: Обща данъчна основа (was base_amount_20 + base_amount_9 + base_amount_0)
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::TotalSalesTaxable)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-01: Обща данъчна основа продажби"),
                    )
                    // Field 01-20: Общо начислен ДДС (was output_vat_amount)
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::TotalSalesVat)
                            .decimal_len(15, 2)
                            .not_null()
                            .default(0.00)
                            .comment("Поле 01-20: Общо начислен ДДС"),
                    )
                    .to_owned(),
            )
            .await?;

        // Add document counts
        manager
            .alter_table(
                Table::alter()
                    .table(VatReturns::Table)
                    // Field 00-05: Брой документи продажби
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::SalesDocumentCount)
                            .integer()
                            .not_null()
                            .default(0)
                            .comment("Поле 00-05: Брой документи продажби"),
                    )
                    // Field 00-06: Брой документи покупки
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::PurchaseDocumentCount)
                            .integer()
                            .not_null()
                            .default(0)
                            .comment("Поле 00-06: Брой документи покупки"),
                    )
                    // Field 00-04: Подаващо лице (ЕГН/Име)
                    .add_column_if_not_exists(
                        ColumnDef::new(VatReturns::SubmittedByPerson)
                            .string()
                            .comment("Поле 00-04: Подаващо лице (ЕГН/Име)"),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop all added columns
        manager
            .alter_table(
                Table::alter()
                    .table(VatReturns::Table)
                    // Sales fields
                    .drop_column(VatReturns::SalesBase20)
                    .drop_column(VatReturns::SalesVat20)
                    .drop_column(VatReturns::SalesBaseVop)
                    .drop_column(VatReturns::SalesVatVop)
                    .drop_column(VatReturns::SalesVatPersonalUse)
                    .drop_column(VatReturns::SalesBase9)
                    .drop_column(VatReturns::SalesVat9)
                    .drop_column(VatReturns::SalesBase0Art3)
                    .drop_column(VatReturns::SalesBase0Vod)
                    .drop_column(VatReturns::SalesBase0Export)
                    .drop_column(VatReturns::SalesBaseArt21)
                    .drop_column(VatReturns::SalesBaseArt69)
                    .drop_column(VatReturns::SalesBaseExempt)
                    // Purchase fields
                    .drop_column(VatReturns::PurchaseBaseNoCredit)
                    .drop_column(VatReturns::PurchaseBaseFullCredit)
                    .drop_column(VatReturns::PurchaseVatFullCredit)
                    .drop_column(VatReturns::PurchaseBasePartialCredit)
                    .drop_column(VatReturns::PurchaseVatPartialCredit)
                    .drop_column(VatReturns::PurchaseVatAnnualAdjustment)
                    // Result fields
                    .drop_column(VatReturns::CreditCoefficient)
                    .drop_column(VatReturns::TotalDeductibleVat)
                    .drop_column(VatReturns::TotalSalesTaxable)
                    .drop_column(VatReturns::TotalSalesVat)
                    // Document counts
                    .drop_column(VatReturns::SalesDocumentCount)
                    .drop_column(VatReturns::PurchaseDocumentCount)
                    .drop_column(VatReturns::SubmittedByPerson)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum VatReturns {
    Table,
    // Sales (продажби) fields - про11, про12, про17, про19, про20, про23
    SalesBase20,              // 01-11
    SalesVat20,               // 01-21
    SalesBaseVop,             // 01-12
    SalesVatVop,              // 01-22
    SalesVatPersonalUse,      // 01-23
    SalesBase9,               // 01-13
    SalesVat9,                // 01-24
    SalesBase0Art3,           // 01-14
    SalesBase0Vod,            // 01-15 (ВОД - вътреобщностна доставка)
    SalesBase0Export,         // 01-16
    SalesBaseArt21,           // 01-17
    SalesBaseArt69,           // 01-18
    SalesBaseExempt,          // 01-19
    // Purchases (покупки) fields - пок09, пок10, пок30
    PurchaseBaseNoCredit,     // 01-30
    PurchaseBaseFullCredit,   // 01-31
    PurchaseVatFullCredit,    // 01-41
    PurchaseBasePartialCredit,// 01-32
    PurchaseVatPartialCredit, // 01-42
    PurchaseVatAnnualAdjustment, // 01-43
    // Result fields
    CreditCoefficient,        // 01-33
    TotalDeductibleVat,       // 01-40
    TotalSalesTaxable,        // 01-01
    TotalSalesVat,            // 01-20
    // Document counts and submitter
    SalesDocumentCount,       // 00-05
    PurchaseDocumentCount,    // 00-06
    SubmittedByPerson,        // 00-04
}
