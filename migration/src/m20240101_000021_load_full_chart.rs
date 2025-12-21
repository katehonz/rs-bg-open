use sea_orm::Statement;
use sea_orm_migration::prelude::*;
use std::fs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Get demo company ID
        let company_result = db
            .query_one(Statement::from_string(
                manager.get_database_backend(),
                "SELECT id FROM companies WHERE name = 'Демо Фирма ООД'".to_string(),
            ))
            .await?;

        if let Some(row) = company_result {
            let company_id: i32 = row.try_get_by_index(0)?;

            // Delete existing accounts for this company
            db.execute(Statement::from_string(
                manager.get_database_backend(),
                format!("DELETE FROM accounts WHERE company_id = {}", company_id),
            ))
            .await?;

            // Load and insert accounts from CSV
            self.load_chart_from_csv(manager, company_id).await?;
        }

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // This migration doesn't need a down since it just replaces data
        Ok(())
    }
}

impl Migration {
    async fn load_chart_from_csv(
        &self,
        manager: &SchemaManager<'_>,
        company_id: i32,
    ) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Read CSV file
        let csv_content = fs::read_to_string("file/ac_chart.csv")
            .map_err(|e| DbErr::Custom(format!("Failed to read CSV file: {}", e)))?;

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_content.as_bytes());

        // First pass: insert synthetic accounts
        for result in reader.records() {
            let record = result.map_err(|e| DbErr::Custom(format!("CSV parse error: {}", e)))?;

            let code = &record[0];
            let name = &record[1];
            let account_category = &record[2]; // СИНТЕТИЧНА or АНАЛИТИЧНА
            let _parent_code = &record[3]; // Empty for synthetic
            let account_type = &record[4];
            let vat_applicable = &record[5];
            let vat_direction = &record[6];
            let _description = &record[7];

            if account_category == "СИНТЕТИЧНА" {
                // Determine account class from code
                let account_class = match code.chars().next() {
                    Some('1') => 1,
                    Some('2') => 2,
                    Some('3') => 3,
                    Some('4') => 4,
                    Some('5') => 5,
                    Some('6') => 6,
                    Some('7') => 7,
                    _ => 0,
                };

                let is_vat_applicable = vat_applicable == "ДА";

                db.execute(Statement::from_string(
                    db.get_database_backend(),
                    format!(
                        r#"
                        INSERT INTO accounts (code, name, account_type, account_class, parent_id, level, 
                                            is_analytical, is_vat_applicable, vat_direction, is_active, company_id)
                        VALUES ('{}', '{}', '{}', {}, NULL, 1, false, {}, '{}', true, {})
                        "#,
                        code, name, account_type, account_class,
                        is_vat_applicable, vat_direction, company_id
                    ),
                ))
                .await?;
            }
        }

        // Second pass: insert analytical accounts
        let csv_content = fs::read_to_string("file/ac_chart.csv")
            .map_err(|e| DbErr::Custom(format!("Failed to read CSV file: {}", e)))?;

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_content.as_bytes());

        for result in reader.records() {
            let record = result.map_err(|e| DbErr::Custom(format!("CSV parse error: {}", e)))?;

            let code = &record[0];
            let name = &record[1];
            let account_category = &record[2];
            let parent_code = &record[3];
            let account_type = &record[4];
            let vat_applicable = &record[5];
            let vat_direction = &record[6];
            let _description = &record[7];

            if account_category == "АНАЛИТИЧНА" && !parent_code.is_empty() {
                // Get parent account ID
                let parent_result = db
                    .query_one(Statement::from_string(
                        manager.get_database_backend(),
                        format!(
                            "SELECT id FROM accounts WHERE code = '{}' AND company_id = {}",
                            parent_code, company_id
                        ),
                    ))
                    .await?;

                if let Some(parent_row) = parent_result {
                    let parent_id: i32 = parent_row.try_get_by_index(0)?;

                    // Determine account class from code
                    let account_class = match code.chars().next() {
                        Some('1') => 1,
                        Some('2') => 2,
                        Some('3') => 3,
                        Some('4') => 4,
                        Some('5') => 5,
                        Some('6') => 6,
                        Some('7') => 7,
                        _ => 0,
                    };

                    let is_vat_applicable = vat_applicable == "ДА";

                    db.execute(Statement::from_string(
                        db.get_database_backend(),
                        format!(
                            r#"
                            INSERT INTO accounts (code, name, account_type, account_class, parent_id, level, 
                                                is_analytical, is_vat_applicable, vat_direction, is_active, company_id)
                            VALUES ('{}', '{}', '{}', {}, {}, 2, true, {}, '{}', true, {})
                            "#,
                            code, name, account_type, account_class, parent_id,
                            is_vat_applicable, vat_direction, company_id
                        ),
                    ))
                    .await?;
                }
            }
        }

        Ok(())
    }
}
