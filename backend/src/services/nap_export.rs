use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use encoding_rs::WINDOWS_1251;
use rust_decimal::Decimal;
use sea_orm::*;
use std::fmt::Write;

pub struct NapExportService;

impl NapExportService {
    /// Format text field with fixed width, padded with spaces
    fn format_text_field(text: &str, width: usize) -> String {
        let mut result = text.chars().take(width).collect::<String>();
        while result.len() < width {
            result.push(' ');
        }
        result
    }

    /// Format numeric field with fixed width, right-aligned, padded with spaces
    fn format_numeric_field(num: Decimal, width: usize, decimals: usize) -> String {
        let formatted = format!("{:.prec$}", num, prec = decimals);
        format!("{:>width$}", formatted, width = width)
    }

    /// Format integer field with fixed width, right-aligned, zero-padded
    fn format_integer_field(num: i32, width: usize) -> String {
        format!("{:0width$}", num, width = width)
    }

    /// Convert string to Windows-1251 encoding
    fn to_windows_1251(text: &str) -> Vec<u8> {
        let (cow, _, _) = WINDOWS_1251.encode(text);
        cow.into_owned()
    }
    /// Generate VIES format files for NAP (Bulgarian Tax Administration)
    pub async fn generate_vies_files(
        db: &DatabaseConnection,
        company_id: i32,
        year: i32,
        month: i32,
    ) -> Result<ViesFiles> {
        // Get company info
        let company = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT name, eik, vat_number, address FROM companies WHERE id = $1",
                vec![company_id.into()],
            ))
            .await?
            .ok_or(anyhow::anyhow!("Company not found"))?;

        let company_name: String = company.try_get("", "name")?;
        let company_eik: String = company.try_get("", "eik")?;
        let company_vat: String = company
            .try_get("", "vat_number")
            .unwrap_or(format!("BG{}", company_eik));

        // Get VAT data for the period
        let start_date = NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap();
        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1).unwrap()
                - chrono::Duration::days(1)
        };

        // Generate DEKLAR.TXT
        let deklar = Self::generate_deklar(
            db,
            company_id,
            &company_name,
            &company_eik,
            &company_vat,
            start_date,
            end_date,
        )
        .await?;

        // Generate POKUPKI.TXT (Purchases)
        let pokupki =
            Self::generate_pokupki(db, company_id, &company_vat, start_date, end_date).await?;

        // Generate PRODAGBI.TXT (Sales)
        let prodagbi =
            Self::generate_prodagbi(db, company_id, &company_vat, start_date, end_date).await?;

        Ok(ViesFiles {
            deklar: Self::to_windows_1251(&deklar),
            pokupki: Self::to_windows_1251(&pokupki),
            prodagbi: Self::to_windows_1251(&prodagbi),
        })
    }

    async fn generate_deklar(
        db: &DatabaseConnection,
        company_id: i32,
        company_name: &str,
        _company_eik: &str,
        company_vat: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<String> {
        let mut deklar = String::new();

        // Debug: Get count of entries first
        let debug_count = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
            SELECT 
                COUNT(*) as total_entries,
                COUNT(CASE WHEN je.vat_document_type = '01' THEN 1 END) as sales_count,
                COUNT(CASE WHEN je.vat_document_type = '03' THEN 1 END) as purchases_count,
                COUNT(CASE WHEN je.is_posted = true THEN 1 END) as posted_count,
                COUNT(CASE WHEN je.vat_document_type IS NOT NULL THEN 1 END) as vat_entries_count
            FROM journal_entries je
            WHERE je.company_id = $1
            AND je.document_date >= $2
            AND je.document_date <= $3
            "#,
                vec![company_id.into(), start_date.into(), end_date.into()],
            ))
            .await?
            .ok_or(anyhow::anyhow!("No debug results"))?;

        println!(
            "DEBUG NAP Export for company {}, period {} to {}",
            company_id, start_date, end_date
        );
        println!(
            "Total entries: {:?}",
            debug_count.try_get::<i64>("", "total_entries")
        );
        println!(
            "Sales entries: {:?}",
            debug_count.try_get::<i64>("", "sales_count")
        );
        println!(
            "Purchase entries: {:?}",
            debug_count.try_get::<i64>("", "purchases_count")
        );
        println!(
            "Posted entries: {:?}",
            debug_count.try_get::<i64>("", "posted_count")
        );
        println!(
            "VAT entries: {:?}",
            debug_count.try_get::<i64>("", "vat_entries_count")
        );

        // Get summary data from journal entries
        let summary = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
            SELECT 
                -- Purchases (vat_document_type = '03')
                COALESCE(SUM(CASE WHEN je.vat_document_type = '03' 
                    THEN je.total_amount - je.total_vat_amount 
                    ELSE 0 END), 0) as purchases_base,
                COALESCE(SUM(CASE WHEN je.vat_document_type = '03' 
                    THEN je.total_vat_amount 
                    ELSE 0 END), 0) as purchases_vat,
                -- Sales (vat_document_type = '01')
                COALESCE(SUM(CASE WHEN je.vat_document_type = '01' 
                    THEN je.total_amount - je.total_vat_amount 
                    ELSE 0 END), 0) as sales_base,
                COALESCE(SUM(CASE WHEN je.vat_document_type = '01' 
                    THEN je.total_vat_amount 
                    ELSE 0 END), 0) as sales_vat
            FROM journal_entries je
            WHERE je.company_id = $1
            AND je.document_date >= $2
            AND je.document_date <= $3
            AND je.is_posted = true
            AND je.vat_document_type IS NOT NULL
            "#,
                vec![company_id.into(), start_date.into(), end_date.into()],
            ))
            .await?;

        let purchases_base: Decimal = summary
            .as_ref()
            .and_then(|s| s.try_get("", "purchases_base").ok())
            .unwrap_or(Decimal::ZERO);
        let purchases_vat: Decimal = summary
            .as_ref()
            .and_then(|s| s.try_get("", "purchases_vat").ok())
            .unwrap_or(Decimal::ZERO);
        let sales_base: Decimal = summary
            .as_ref()
            .and_then(|s| s.try_get("", "sales_base").ok())
            .unwrap_or(Decimal::ZERO);
        let sales_vat: Decimal = summary
            .as_ref()
            .and_then(|s| s.try_get("", "sales_vat").ok())
            .unwrap_or(Decimal::ZERO);

        // Според НАП спецификацията DEKLAR.TXT има един запис с фиксирана дължина на полетата
        // Format: VAT number (15), Company name (80), Period (6), data fields...
        deklar.push_str(&Self::format_text_field(company_vat, 15));
        deklar.push_str(&Self::format_text_field(company_name, 80));
        deklar.push_str(&format!("{}{:02}", start_date.year(), start_date.month()));

        // Номерация на дневниците (14 знака за всяко)
        deklar.push_str(&Self::format_integer_field(14, 14)); // Брой записи продажби
        deklar.push_str(&Self::format_integer_field(28, 14)); // Брой записи покупки

        // Стойности от дневник продажби (13 знака за всяка стойност, 2 десетични места)
        deklar.push_str(&Self::format_numeric_field(sales_base, 13, 2));
        deklar.push_str(&Self::format_numeric_field(sales_vat, 13, 2));
        deklar.push_str(&Self::format_numeric_field(sales_base, 13, 2)); // Дублиране за различни режими
        deklar.push_str(&Self::format_numeric_field(sales_vat, 13, 2));

        // Нулеви стойности за останалите полета (12 полета по 13 знака)
        for _ in 0..12 {
            deklar.push_str(&Self::format_numeric_field(Decimal::ZERO, 13, 2));
        }

        // Стойности от дневник покупки
        deklar.push_str(&Self::format_numeric_field(purchases_base, 13, 2));
        deklar.push_str(&Self::format_numeric_field(purchases_vat, 13, 2));

        // Нулеви стойности за останалите полета на покупките (6 полета по 13 знака)
        for _ in 0..6 {
            deklar.push_str(&Self::format_numeric_field(Decimal::ZERO, 13, 2));
        }

        deklar.push_str("\r\n");

        Ok(deklar)
    }

    async fn generate_pokupki(
        db: &DatabaseConnection,
        company_id: i32,
        company_vat: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<String> {
        let mut pokupki = String::new();

        // Get purchase documents from journal entries with VAT
        let purchases = db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
            SELECT 
                je.document_number as doc_number,
                TO_CHAR(je.document_date, 'YYYY-MM-DD') as doc_date,
                COALESCE(cp.name, je.description, '') as contractor_name,
                COALESCE(cp.vat_number, '') as contractor_vat,
                je.total_amount as total_amount,
                je.total_vat_amount as vat_amount,
                (je.total_amount - je.total_vat_amount) as net_amount,
                COALESCE(je.vat_document_type, '01') as document_type,
                COALESCE(je.description, '') as operation_description
            FROM journal_entries je
            LEFT JOIN (
                SELECT DISTINCT el.journal_entry_id, cp.name, cp.vat_number
                FROM entry_lines el
                JOIN counterparts cp ON el.counterpart_id = cp.id
            ) cp ON je.id = cp.journal_entry_id
            WHERE je.company_id = $1
            AND je.document_date BETWEEN $2 AND $3
            AND je.is_posted = true
            AND je.vat_document_type = '03'  -- Purchase invoices
            ORDER BY je.document_date, je.document_number
            "#,
                vec![company_id.into(), start_date.into(), end_date.into()],
            ))
            .await?;

        let mut line_no = 1;
        for purchase in purchases {
            let doc_number: String = purchase.try_get("", "doc_number")?;
            let doc_date: String = purchase.try_get("", "doc_date")?;
            let contractor_name: String =
                purchase.try_get("", "contractor_name").unwrap_or_default();
            let contractor_vat: String = purchase.try_get("", "contractor_vat").unwrap_or_default();
            let net_amount: Decimal = purchase.try_get("", "net_amount")?;
            let vat_amount: Decimal = purchase.try_get("", "vat_amount")?;
            let _total_amount: Decimal = purchase.try_get("", "total_amount")?;
            let document_type: String = purchase
                .try_get("", "document_type")
                .unwrap_or("01".to_string());
            let operation_description: String = purchase
                .try_get("", "operation_description")
                .unwrap_or_default();

            // Формат според НАП спецификацията за POKUPKI.TXT с фиксирана дължина
            // VAT number (15), Period (6), Branch code field (18 chars with branch code inside),
            // След това: Line no + Doc type + Doc no
            // Doc date (10), Contractor VAT (15), Contractor name (50), Operation description (50)
            // Various amounts (11 fields x 13 chars each)

            // Първата колона трябва да бъде ДДС номера на моята компания, не на контрагента
            pokupki.push_str(&Self::format_text_field(company_vat, 15));
            pokupki.push_str(&format!("{}{:02}", start_date.year(), start_date.month()));
            pokupki.push_str(&format!("   {:<15}", "0")); // клон/обособено звено - 3 spaces + "0" + padding

            // Номер на ред + тип документ + номер на документ (заедно в поле с дължина 18)
            // Според спецификацията: line_no + doc_type(2) + doc_number
            // Типът документ трябва да е двуцифрен с водеща нула (01, 02, 03)
            let doc_type_formatted = format!("{:0>2}", document_type);
            let line_doc_combined = format!("{}{}{}", line_no, &doc_type_formatted, &doc_number);
            pokupki.push_str(&Self::format_text_field(&line_doc_combined, 18));

            // Дата на документ (10 символа)
            let formatted_date = doc_date.replace("-", "/");
            pokupki.push_str(&Self::format_text_field(&formatted_date, 10));
            pokupki.push_str(&Self::format_text_field(&contractor_vat, 15));
            pokupki.push_str(&Self::format_text_field(&contractor_name, 50));
            pokupki.push_str(&Self::format_text_field(&operation_description, 50)); // Описание на операцията

            // Числови полета - най-важните са net amount и vat amount
            pokupki.push_str(&Self::format_numeric_field(Decimal::ZERO, 13, 2)); // Поле 1
            pokupki.push_str(&Self::format_numeric_field(net_amount, 13, 2)); // Поле 2 - основа
            pokupki.push_str(&Self::format_numeric_field(vat_amount, 13, 2)); // Поле 3 - ДДС

            // Останалите 8 полета като нули
            for _ in 0..8 {
                pokupki.push_str(&Self::format_numeric_field(Decimal::ZERO, 13, 2));
            }

            pokupki.push_str("\r\n");
            line_no += 1;
        }

        Ok(pokupki)
    }

    async fn generate_prodagbi(
        db: &DatabaseConnection,
        company_id: i32,
        company_vat: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<String> {
        let mut prodagbi = String::new();

        // Get sales documents from journal entries with VAT
        let sales = db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
            SELECT 
                je.document_number as doc_number,
                TO_CHAR(je.document_date, 'YYYY-MM-DD') as doc_date,
                COALESCE(cp.name, je.description, '') as contractor_name,
                COALESCE(cp.vat_number, '') as contractor_vat,
                je.total_amount as total_amount,
                je.total_vat_amount as vat_amount,
                (je.total_amount - je.total_vat_amount) as net_amount,
                COALESCE(je.vat_document_type, '01') as document_type,
                COALESCE(je.description, '') as operation_description
            FROM journal_entries je
            LEFT JOIN (
                SELECT DISTINCT el.journal_entry_id, cp.name, cp.vat_number
                FROM entry_lines el
                JOIN counterparts cp ON el.counterpart_id = cp.id
            ) cp ON je.id = cp.journal_entry_id
            WHERE je.company_id = $1
            AND je.document_date BETWEEN $2 AND $3
            AND je.is_posted = true
            AND je.vat_document_type = '01'  -- Sales invoices
            ORDER BY je.document_date, je.document_number
            "#,
                vec![company_id.into(), start_date.into(), end_date.into()],
            ))
            .await?;

        let mut line_no = 1;
        for sale in sales {
            let doc_number: String = sale.try_get("", "doc_number")?;
            let doc_date: String = sale.try_get("", "doc_date")?;
            let contractor_name: String = sale.try_get("", "contractor_name").unwrap_or_default();
            let contractor_vat: String = sale.try_get("", "contractor_vat").unwrap_or_default();
            let net_amount: Decimal = sale.try_get("", "net_amount")?;
            let vat_amount: Decimal = sale.try_get("", "vat_amount")?;
            let _total_amount: Decimal = sale.try_get("", "total_amount")?;
            let document_type: String = sale
                .try_get("", "document_type")
                .unwrap_or("01".to_string());
            let operation_description: String = sale
                .try_get("", "operation_description")
                .unwrap_or_default();

            // Формат според НАП спецификацията за PRODAGBI.TXT с фиксирана дължина
            // Същата структура като POKUPKI.TXT
            // VAT number (15), Period (6), Branch code field (18 chars with branch code inside),
            // След това: Line no + Doc type + Doc no
            // Doc date (10), Contractor VAT (15), Contractor name (50), Operation description (50)

            // Първата колона трябва да бъде ДДС номера на моята компания, не на контрагента
            prodagbi.push_str(&Self::format_text_field(company_vat, 15));
            prodagbi.push_str(&format!("{}{:02}", start_date.year(), start_date.month()));
            prodagbi.push_str(&format!("   {:<15}", "0")); // клон/обособено звено - 3 spaces + "0" + padding

            // Номер на ред + тип документ + номер на документ (заедно в поле с дължина 18)
            // Според спецификацията: line_no + doc_type(2) + doc_number
            // Типът документ трябва да е двуцифрен с водеща нула (01, 02, 03)
            let doc_type_formatted = format!("{:0>2}", document_type);
            let line_doc_combined = format!("{}{}{}", line_no, &doc_type_formatted, &doc_number);
            prodagbi.push_str(&Self::format_text_field(&line_doc_combined, 18));

            // Дата на документ (10 символа)
            let formatted_date = doc_date.replace("-", "/");
            prodagbi.push_str(&Self::format_text_field(&formatted_date, 10));
            prodagbi.push_str(&Self::format_text_field(&contractor_vat, 15));
            prodagbi.push_str(&Self::format_text_field(&contractor_name, 50));
            prodagbi.push_str(&Self::format_text_field(&operation_description, 50)); // Описание на операцията

            // Числови полета - за продажби обикновено са net amount, vat amount
            prodagbi.push_str(&Self::format_numeric_field(net_amount, 13, 2)); // Поле 1 - основа
            prodagbi.push_str(&Self::format_numeric_field(vat_amount, 13, 2)); // Поле 2 - ДДС
            prodagbi.push_str(&Self::format_numeric_field(net_amount, 13, 2)); // Поле 3 - основа (дублиране)
            prodagbi.push_str(&Self::format_numeric_field(vat_amount, 13, 2)); // Поле 4 - ДДС (дублиране)

            // Останалите 13 полета като нули (17 общо полета)
            for _ in 0..13 {
                prodagbi.push_str(&Self::format_numeric_field(Decimal::ZERO, 13, 2));
            }

            prodagbi.push_str("\r\n");
            line_no += 1;
        }

        Ok(prodagbi)
    }
}

#[derive(Debug)]
pub struct ViesFiles {
    pub deklar: Vec<u8>,
    pub pokupki: Vec<u8>,
    pub prodagbi: Vec<u8>,
}
