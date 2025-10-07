use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Check if any company exists
        let existing_companies = db
            .query_one(Statement::from_string(
                manager.get_database_backend(),
                "SELECT COUNT(*) as count FROM companies".to_string(),
            ))
            .await?;

        let count: i64 = existing_companies
            .and_then(|row| row.try_get_by_index(0).ok())
            .unwrap_or(0);

        if count == 0 {
            // Insert demo company
            let _insert_result = db
                .execute(Statement::from_string(
                    manager.get_database_backend(),
                    r#"
                    INSERT INTO companies (name, vat_number, address, phone, email, contact_person, is_active, eik)
                    VALUES ('Демо Фирма ООД', 'BG123456789', 'гр. София, ул. Тестова 1', 
                            '+359 88 123 4567', 'demo@example.com', 'Иван Иванов', true, '123456789')
                    RETURNING id
                    "#.to_string(),
                ))
                .await?;

            // Get the inserted company ID
            let company_id_result = db
                .query_one(Statement::from_string(
                    manager.get_database_backend(),
                    "SELECT id FROM companies WHERE name = 'Демо Фирма ООД'".to_string(),
                ))
                .await?;

            if let Some(row) = company_id_result {
                let company_id: i32 = row.try_get_by_index(0)?;

                // Insert chart of accounts for demo company
                self.insert_chart_of_accounts(manager, company_id).await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Delete demo company and related accounts (cascade will handle accounts)
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "DELETE FROM companies WHERE name = 'Демо Фирма ООД'".to_string(),
        ))
        .await?;

        Ok(())
    }
}

impl Migration {
    async fn insert_chart_of_accounts(
        &self,
        manager: &SchemaManager<'_>,
        company_id: i32,
    ) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // Insert root accounts
        let accounts = vec![
            // Клас 1 - Дълготрайни активи
            (
                "1",
                "Дълготрайни активи",
                "ASSET",
                1,
                None::<i32>,
                1,
                false,
                false,
                "NONE",
            ),
            // Клас 2 - Материални запаси
            (
                "2",
                "Материални запаси",
                "ASSET",
                2,
                None::<i32>,
                1,
                false,
                false,
                "NONE",
            ),
            // Клас 3 - Финансови средства и вземания
            (
                "3",
                "Финансови средства и вземания",
                "ASSET",
                3,
                None::<i32>,
                1,
                false,
                false,
                "NONE",
            ),
            // Клас 4 - Разчети
            (
                "4",
                "Разчети",
                "LIABILITY",
                4,
                None::<i32>,
                1,
                false,
                false,
                "NONE",
            ),
            // Клас 5 - Собствен капитал
            (
                "5",
                "Собствен капитал",
                "EQUITY",
                5,
                None::<i32>,
                1,
                false,
                false,
                "NONE",
            ),
            // Клас 6 - Разходи
            (
                "6",
                "Разходи",
                "EXPENSE",
                6,
                None::<i32>,
                1,
                false,
                false,
                "NONE",
            ),
            // Клас 7 - Приходи
            (
                "7",
                "Приходи",
                "REVENUE",
                7,
                None::<i32>,
                1,
                false,
                false,
                "NONE",
            ),
        ];

        for (
            code,
            name,
            account_type,
            account_class,
            parent_id,
            level,
            is_analytical,
            is_vat_applicable,
            vat_direction,
        ) in accounts
        {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!(
                    r#"
                    INSERT INTO accounts (code, name, account_type, account_class, parent_id, level, 
                                        is_analytical, is_vat_applicable, vat_direction, is_active, company_id)
                    VALUES ('{}', '{}', '{}', {}, {}, {}, {}, {}, '{}', true, {})
                    "#,
                    code, name, account_type, account_class, 
                    parent_id.map_or("NULL".to_string(), |id| id.to_string()),
                    level, is_analytical, is_vat_applicable, vat_direction, company_id
                ),
            ))
            .await?;
        }

        // Get parent IDs for second level accounts
        let parent_ids = self.get_parent_ids(manager, company_id).await?;

        // Insert second level accounts
        let second_level_accounts = vec![
            // Under Клас 1
            (
                "201",
                "Дълготрайни материални активи",
                "ASSET",
                2,
                parent_ids.get("1"),
                2,
                false,
            ),
            // Under Клас 2
            (
                "302",
                "Материали",
                "ASSET",
                3,
                parent_ids.get("2"),
                2,
                false,
            ),
            (
                "303",
                "Продукция",
                "ASSET",
                3,
                parent_ids.get("2"),
                2,
                false,
            ),
            ("304", "Стоки", "ASSET", 3, parent_ids.get("2"), 2, true),
            // Under Клас 3
            ("411", "Клиенти", "ASSET", 4, parent_ids.get("3"), 2, true),
            ("501", "Каса", "ASSET", 5, parent_ids.get("3"), 2, false),
            (
                "503",
                "Разплащателни сметки",
                "ASSET",
                5,
                parent_ids.get("3"),
                2,
                false,
            ),
            // Under Клас 4
            (
                "401",
                "Доставчици",
                "LIABILITY",
                4,
                parent_ids.get("4"),
                2,
                true,
            ),
            (
                "402",
                "Доставчици по аванси",
                "ASSET",
                4,
                parent_ids.get("4"),
                2,
                true,
            ),
            (
                "4531",
                "Начислен ДДС",
                "ASSET",
                4,
                parent_ids.get("4"),
                2,
                true,
            ),
            (
                "4532",
                "ДДС за начисляване",
                "LIABILITY",
                4,
                parent_ids.get("4"),
                2,
                true,
            ),
            (
                "4533",
                "ДДС за внасяне",
                "LIABILITY",
                4,
                parent_ids.get("4"),
                2,
                true,
            ),
            (
                "4534",
                "ДДС за възстановяване",
                "ASSET",
                4,
                parent_ids.get("4"),
                2,
                true,
            ),
            // Under Клас 5
            (
                "101",
                "Основен капитал",
                "EQUITY",
                1,
                parent_ids.get("5"),
                2,
                true,
            ),
            ("117", "Резерви", "EQUITY", 1, parent_ids.get("5"), 2, true),
            (
                "123",
                "Печалба и загуба от минали години",
                "EQUITY",
                1,
                parent_ids.get("5"),
                2,
                true,
            ),
            (
                "124",
                "Текуща печалба/загуба",
                "EQUITY",
                1,
                parent_ids.get("5"),
                2,
                true,
            ),
            // Under Клас 6
            (
                "601",
                "Разходи за материали",
                "EXPENSE",
                6,
                parent_ids.get("6"),
                2,
                true,
            ),
            (
                "602",
                "Разходи за външни услуги",
                "EXPENSE",
                6,
                parent_ids.get("6"),
                2,
                true,
            ),
            (
                "603",
                "Разходи за амортизации",
                "EXPENSE",
                6,
                parent_ids.get("6"),
                2,
                true,
            ),
            (
                "604",
                "Разходи за заплати",
                "EXPENSE",
                6,
                parent_ids.get("6"),
                2,
                true,
            ),
            (
                "605",
                "Разходи за социални осигуровки",
                "EXPENSE",
                6,
                parent_ids.get("6"),
                2,
                true,
            ),
            (
                "609",
                "Други разходи",
                "EXPENSE",
                6,
                parent_ids.get("6"),
                2,
                true,
            ),
            // Under Клас 7
            (
                "701",
                "Приходи от продажби на продукция",
                "REVENUE",
                7,
                parent_ids.get("7"),
                2,
                true,
            ),
            (
                "702",
                "Приходи от продажби на стоки",
                "REVENUE",
                7,
                parent_ids.get("7"),
                2,
                true,
            ),
            (
                "703",
                "Приходи от услуги",
                "REVENUE",
                7,
                parent_ids.get("7"),
                2,
                true,
            ),
            (
                "709",
                "Други приходи",
                "REVENUE",
                7,
                parent_ids.get("7"),
                2,
                true,
            ),
        ];

        for (code, name, account_type, account_class, parent_id, level, is_analytical) in
            second_level_accounts
        {
            if let Some(&pid) = parent_id {
                // Determine VAT settings based on account code
                let (is_vat_applicable, vat_direction) = match code {
                    "304" | "601" | "602" | "609" => (true, "INPUT"),
                    "411" | "701" | "702" | "703" | "709" => (true, "OUTPUT"),
                    "401" | "402" => (true, "INPUT"),
                    _ => (false, "NONE"),
                };

                db.execute(Statement::from_string(
                    manager.get_database_backend(),
                    format!(
                        r#"
                        INSERT INTO accounts (code, name, account_type, account_class, parent_id, level, 
                                            is_analytical, is_vat_applicable, vat_direction, is_active, company_id)
                        VALUES ('{}', '{}', '{}', {}, {}, {}, {}, {}, '{}', true, {})
                        "#,
                        code, name, account_type, account_class, pid, level, 
                        is_analytical, is_vat_applicable, vat_direction, company_id
                    ),
                ))
                .await?;
            }
        }

        // Get parent IDs for third level accounts
        let second_level_ids = self
            .get_parent_ids_for_codes(manager, company_id, &["201", "302", "303", "501", "503"])
            .await?;

        // Insert third level accounts
        let third_level_accounts = vec![
            // Under 201 - ДМА
            (
                "203",
                "Сгради",
                "ASSET",
                2,
                second_level_ids.get("201"),
                3,
                true,
            ),
            (
                "204",
                "Машини и оборудване",
                "ASSET",
                2,
                second_level_ids.get("201"),
                3,
                true,
            ),
            (
                "207",
                "Транспортни средства",
                "ASSET",
                2,
                second_level_ids.get("201"),
                3,
                true,
            ),
            (
                "209",
                "Други ДМА",
                "ASSET",
                2,
                second_level_ids.get("201"),
                3,
                true,
            ),
            // Under 302 - Материали
            (
                "30201",
                "Основни материали",
                "ASSET",
                3,
                second_level_ids.get("302"),
                3,
                true,
            ),
            (
                "30202",
                "Спомагателни материали",
                "ASSET",
                3,
                second_level_ids.get("302"),
                3,
                true,
            ),
            // Under 303 - Продукция
            (
                "30301",
                "Готова продукция",
                "ASSET",
                3,
                second_level_ids.get("303"),
                3,
                true,
            ),
            // Under 501 - Каса
            (
                "50101",
                "Каса в лева",
                "ASSET",
                5,
                second_level_ids.get("501"),
                3,
                true,
            ),
            (
                "50102",
                "Каса във валута",
                "ASSET",
                5,
                second_level_ids.get("501"),
                3,
                true,
            ),
            // Under 503 - Разплащателни сметки
            (
                "50301",
                "Разплащателна сметка в лева",
                "ASSET",
                5,
                second_level_ids.get("503"),
                3,
                true,
            ),
            (
                "50302",
                "Разплащателна сметка във валута",
                "ASSET",
                5,
                second_level_ids.get("503"),
                3,
                true,
            ),
        ];

        for (code, name, account_type, account_class, parent_id, level, is_analytical) in
            third_level_accounts
        {
            if let Some(&pid) = parent_id {
                db.execute(Statement::from_string(
                    manager.get_database_backend(),
                    format!(
                        r#"
                        INSERT INTO accounts (code, name, account_type, account_class, parent_id, level, 
                                            is_analytical, is_vat_applicable, vat_direction, is_active, company_id)
                        VALUES ('{}', '{}', '{}', {}, {}, {}, {}, false, 'NONE', true, {})
                        "#,
                        code, name, account_type, account_class, pid, level, is_analytical, company_id
                    ),
                ))
                .await?;
            }
        }

        Ok(())
    }

    async fn get_parent_ids(
        &self,
        manager: &SchemaManager<'_>,
        company_id: i32,
    ) -> Result<std::collections::HashMap<String, i32>, DbErr> {
        let db = manager.get_connection();
        use std::collections::HashMap;

        let mut map = HashMap::new();
        let codes = vec!["1", "2", "3", "4", "5", "6", "7"];

        for code in codes {
            let result = db
                .query_one(Statement::from_string(
                    manager.get_database_backend(),
                    format!(
                        "SELECT id FROM accounts WHERE code = '{}' AND company_id = {}",
                        code, company_id
                    ),
                ))
                .await?;

            if let Some(row) = result {
                let id: i32 = row.try_get_by_index(0)?;
                map.insert(ToString::to_string(&code), id);
            }
        }

        Ok(map)
    }

    async fn get_parent_ids_for_codes(
        &self,
        manager: &SchemaManager<'_>,
        company_id: i32,
        codes: &[&str],
    ) -> Result<std::collections::HashMap<String, i32>, DbErr> {
        let db = manager.get_connection();
        use std::collections::HashMap;

        let mut map = HashMap::new();

        for code in codes {
            let result = db
                .query_one(Statement::from_string(
                    manager.get_database_backend(),
                    format!(
                        "SELECT id FROM accounts WHERE code = '{}' AND company_id = {}",
                        code, company_id
                    ),
                ))
                .await?;

            if let Some(row) = result {
                let id: i32 = row.try_get_by_index(0)?;
                map.insert(ToString::to_string(&code), id);
            }
        }

        Ok(map)
    }
}
