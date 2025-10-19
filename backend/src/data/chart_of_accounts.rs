use crate::entities::account::{AccountType, ActiveModel as AccountActiveModel, VatDirection};
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};

#[derive(Debug, Clone)]
pub struct AccountTemplate {
    pub code: &'static str,
    pub name: &'static str,
    pub account_type: &'static str,
    pub is_analytical: bool,
    pub parent_code: Option<&'static str>,
    pub is_vat_applicable: bool,
    pub vat_direction: &'static str,
}

pub const CHART_OF_ACCOUNTS: &[AccountTemplate] = &[
    // Клас 1 - Дълготрайни активи
    AccountTemplate {
        code: "1",
        name: "Дълготрайни активи",
        account_type: "ASSET",
        is_analytical: false,
        parent_code: None,
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "201",
        name: "Дълготрайни материални активи",
        account_type: "ASSET",
        is_analytical: false,
        parent_code: Some("1"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "203",
        name: "Сгради",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("201"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "204",
        name: "Машини и оборудване",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("201"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "207",
        name: "Транспортни средства",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("201"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "209",
        name: "Други ДМА",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("201"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    // Клас 2 - Материални запаси
    AccountTemplate {
        code: "2",
        name: "Материални запаси",
        account_type: "ASSET",
        is_analytical: false,
        parent_code: None,
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "302",
        name: "Материали",
        account_type: "ASSET",
        is_analytical: false,
        parent_code: Some("2"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "30201",
        name: "Основни материали",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("302"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "30202",
        name: "Спомагателни материали",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("302"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "303",
        name: "Продукция",
        account_type: "ASSET",
        is_analytical: false,
        parent_code: Some("2"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "30301",
        name: "Готова продукция",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("303"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "304",
        name: "Стоки",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("2"),
        is_vat_applicable: true,
        vat_direction: "INPUT",
    },
    // Клас 3 - Финансови средства и вземания
    AccountTemplate {
        code: "3",
        name: "Финансови средства и вземания",
        account_type: "ASSET",
        is_analytical: false,
        parent_code: None,
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "411",
        name: "Клиенти",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("3"),
        is_vat_applicable: true,
        vat_direction: "OUTPUT",
    },
    AccountTemplate {
        code: "501",
        name: "Каса",
        account_type: "ASSET",
        is_analytical: false,
        parent_code: Some("3"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "50101",
        name: "Каса в лева",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("501"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "50102",
        name: "Каса във валута",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("501"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "503",
        name: "Разплащателни сметки",
        account_type: "ASSET",
        is_analytical: false,
        parent_code: Some("3"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "50301",
        name: "Разплащателна сметка в лева",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("503"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "50302",
        name: "Разплащателна сметка във валута",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("503"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    // Клас 4 - Разчети
    AccountTemplate {
        code: "4",
        name: "Разчети",
        account_type: "LIABILITY",
        is_analytical: false,
        parent_code: None,
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "401",
        name: "Доставчици",
        account_type: "LIABILITY",
        is_analytical: true,
        parent_code: Some("4"),
        is_vat_applicable: true,
        vat_direction: "INPUT",
    },
    AccountTemplate {
        code: "402",
        name: "Доставчици по аванси",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("4"),
        is_vat_applicable: true,
        vat_direction: "INPUT",
    },
    AccountTemplate {
        code: "4532",
        name: "ДДС за начисляване",
        account_type: "LIABILITY",
        is_analytical: true,
        parent_code: Some("4"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "4531",
        name: "Начислен ДДС",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("4"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "4533",
        name: "ДДС за внасяне",
        account_type: "LIABILITY",
        is_analytical: true,
        parent_code: Some("4"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "4534",
        name: "ДДС за възстановяване",
        account_type: "ASSET",
        is_analytical: true,
        parent_code: Some("4"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    // Клас 5 - Собствен капитал
    AccountTemplate {
        code: "5",
        name: "Собствен капитал",
        account_type: "EQUITY",
        is_analytical: false,
        parent_code: None,
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "101",
        name: "Основен капитал",
        account_type: "EQUITY",
        is_analytical: true,
        parent_code: Some("5"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "117",
        name: "Резерви",
        account_type: "EQUITY",
        is_analytical: true,
        parent_code: Some("5"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "123",
        name: "Печалба и загуба от минали години",
        account_type: "EQUITY",
        is_analytical: true,
        parent_code: Some("5"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "124",
        name: "Текуща печалба/загуба",
        account_type: "EQUITY",
        is_analytical: true,
        parent_code: Some("5"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    // Клас 6 - Разходи
    AccountTemplate {
        code: "6",
        name: "Разходи",
        account_type: "EXPENSE",
        is_analytical: false,
        parent_code: None,
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "601",
        name: "Разходи за материали",
        account_type: "EXPENSE",
        is_analytical: true,
        parent_code: Some("6"),
        is_vat_applicable: true,
        vat_direction: "INPUT",
    },
    AccountTemplate {
        code: "602",
        name: "Разходи за външни услуги",
        account_type: "EXPENSE",
        is_analytical: true,
        parent_code: Some("6"),
        is_vat_applicable: true,
        vat_direction: "INPUT",
    },
    AccountTemplate {
        code: "603",
        name: "Разходи за амортизации",
        account_type: "EXPENSE",
        is_analytical: true,
        parent_code: Some("6"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "604",
        name: "Разходи за заплати",
        account_type: "EXPENSE",
        is_analytical: true,
        parent_code: Some("6"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "605",
        name: "Разходи за социални осигуровки",
        account_type: "EXPENSE",
        is_analytical: true,
        parent_code: Some("6"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "609",
        name: "Други разходи",
        account_type: "EXPENSE",
        is_analytical: true,
        parent_code: Some("6"),
        is_vat_applicable: true,
        vat_direction: "INPUT",
    },
    AccountTemplate {
        code: "611",
        name: "Разходи за лихви",
        account_type: "EXPENSE",
        is_analytical: true,
        parent_code: Some("6"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "612",
        name: "Разходи от валутни операции",
        account_type: "EXPENSE",
        is_analytical: true,
        parent_code: Some("6"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    // Клас 7 - Приходи
    AccountTemplate {
        code: "7",
        name: "Приходи",
        account_type: "REVENUE",
        is_analytical: false,
        parent_code: None,
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "701",
        name: "Приходи от продажби на продукция",
        account_type: "REVENUE",
        is_analytical: true,
        parent_code: Some("7"),
        is_vat_applicable: true,
        vat_direction: "OUTPUT",
    },
    AccountTemplate {
        code: "702",
        name: "Приходи от продажби на стоки",
        account_type: "REVENUE",
        is_analytical: true,
        parent_code: Some("7"),
        is_vat_applicable: true,
        vat_direction: "OUTPUT",
    },
    AccountTemplate {
        code: "703",
        name: "Приходи от услуги",
        account_type: "REVENUE",
        is_analytical: true,
        parent_code: Some("7"),
        is_vat_applicable: true,
        vat_direction: "OUTPUT",
    },
    AccountTemplate {
        code: "709",
        name: "Други приходи",
        account_type: "REVENUE",
        is_analytical: true,
        parent_code: Some("7"),
        is_vat_applicable: true,
        vat_direction: "OUTPUT",
    },
    AccountTemplate {
        code: "711",
        name: "Приходи от лихви",
        account_type: "REVENUE",
        is_analytical: true,
        parent_code: Some("7"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
    AccountTemplate {
        code: "712",
        name: "Приходи от валутни операции",
        account_type: "REVENUE",
        is_analytical: true,
        parent_code: Some("7"),
        is_vat_applicable: false,
        vat_direction: "NONE",
    },
];

pub async fn load_chart_of_accounts(
    db: &DatabaseConnection,
    company_id: i32,
) -> Result<(), String> {
    use crate::entities::account;
    use std::collections::HashMap;

    // Create a map to store parent codes and their IDs
    let mut parent_map: HashMap<&str, i32> = HashMap::new();

    // First, insert all root accounts (no parent)
    for template in CHART_OF_ACCOUNTS.iter().filter(|t| t.parent_code.is_none()) {
        let account_class = template
            .code
            .chars()
            .next()
            .and_then(|c| c.to_digit(10))
            .unwrap_or(1) as i32;

        let account = AccountActiveModel {
            code: Set(template.code.to_string()),
            name: Set(template.name.to_string()),
            account_type: Set(match template.account_type {
                "ASSET" => AccountType::Asset,
                "LIABILITY" => AccountType::Liability,
                "EQUITY" => AccountType::Equity,
                "REVENUE" => AccountType::Revenue,
                "EXPENSE" => AccountType::Expense,
                _ => AccountType::Asset,
            }),
            account_class: Set(account_class),
            parent_id: Set(None),
            level: Set(1),
            is_vat_applicable: Set(template.is_vat_applicable),
            vat_direction: Set(match template.vat_direction {
                "NONE" => VatDirection::None,
                "INPUT" => VatDirection::Input,
                "OUTPUT" => VatDirection::Output,
                "BOTH" => VatDirection::Both,
                _ => VatDirection::None,
            }),
            is_active: Set(true),
            is_analytical: Set(template.is_analytical),
            company_id: Set(company_id),
            ..Default::default()
        };

        let inserted = account::Entity::insert(account)
            .exec_with_returning(db)
            .await
            .map_err(|e| format!("Failed to insert account {}: {}", template.code, e))?;

        parent_map.insert(template.code, inserted.id);
    }

    // Then insert accounts with parents (in order of dependency)
    let mut remaining = CHART_OF_ACCOUNTS
        .iter()
        .filter(|t| t.parent_code.is_some())
        .collect::<Vec<_>>();

    while !remaining.is_empty() {
        let mut inserted_any = false;
        let mut next_remaining = Vec::new();

        for template in remaining {
            if let Some(parent_code) = template.parent_code {
                if let Some(&parent_id) = parent_map.get(parent_code) {
                    let account_class = template
                        .code
                        .chars()
                        .next()
                        .and_then(|c| c.to_digit(10))
                        .unwrap_or(1) as i32;
                    let level = if parent_code.len() == 1 { 2 } else { 3 };

                    let account = AccountActiveModel {
                        code: Set(template.code.to_string()),
                        name: Set(template.name.to_string()),
                        account_type: Set(match template.account_type {
                            "ASSET" => AccountType::Asset,
                            "LIABILITY" => AccountType::Liability,
                            "EQUITY" => AccountType::Equity,
                            "REVENUE" => AccountType::Revenue,
                            "EXPENSE" => AccountType::Expense,
                            _ => AccountType::Asset,
                        }),
                        account_class: Set(account_class),
                        parent_id: Set(Some(parent_id)),
                        level: Set(level),
                        is_vat_applicable: Set(template.is_vat_applicable),
                        vat_direction: Set(match template.vat_direction {
                            "NONE" => VatDirection::None,
                            "INPUT" => VatDirection::Input,
                            "OUTPUT" => VatDirection::Output,
                            "BOTH" => VatDirection::Both,
                            _ => VatDirection::None,
                        }),
                        is_active: Set(true),
                        is_analytical: Set(template.is_analytical),
                        company_id: Set(company_id),
                        ..Default::default()
                    };

                    let inserted = account::Entity::insert(account)
                        .exec_with_returning(db)
                        .await
                        .map_err(|e| {
                            format!("Failed to insert account {}: {}", template.code, e)
                        })?;

                    parent_map.insert(template.code, inserted.id);
                    inserted_any = true;
                } else {
                    next_remaining.push(template);
                }
            }
        }

        if !inserted_any && !next_remaining.is_empty() {
            return Err("Failed to resolve account parent dependencies".to_string());
        }

        remaining = next_remaining;
    }

    Ok(())
}
