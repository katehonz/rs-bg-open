use anyhow::{anyhow, Result};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::entities::{ai_bank_accounting_setting, AiBankAccountingSetting};
use crate::services::contragent::{clean_json_block, extract_json_object, ContragentService};

/// Service за парсване на банкови транзакции и извличане на контрагенти
pub struct BankTransactionParser {
    contragent_service: Arc<ContragentService>,
}

impl BankTransactionParser {
    pub fn new(contragent_service: Arc<ContragentService>) -> Self {
        Self {
            contragent_service,
        }
    }

    /// Парсва описание на банкова транзакция и извлича информация за контрагент
    pub async fn parse_transaction_description(
        &self,
        db: &DatabaseConnection,
        description: &str,
    ) -> Result<Option<ParsedTransactionData>> {
        // Проверка дали има конфигуриран Mistral ключ
        let api_key = self
            .contragent_service
            .get_setting_value(db, "mistral.api.key")
            .await?
            .ok_or_else(|| anyhow!("Mistral API ключ не е конфигуриран"))?;

        let model = self
            .contragent_service
            .get_setting_value(db, "mistral.api.model")
            .await?
            .unwrap_or_else(|| "mistral-small-latest".to_string());

        // Извикване на Mistral API
        let raw_response = self
            .invoke_mistral_parser(&api_key, &model, description)
            .await?;

        // Парсване на JSON отговора
        self.parse_mistral_response(&raw_response)
    }

    async fn invoke_mistral_parser(
        &self,
        api_key: &str,
        model: &str,
        description: &str,
    ) -> Result<String> {
        let client = reqwest::Client::new();

        let prompt = format!(
            "Анализирай това описание на банкова транзакция и извлечи информация за контрагента.\n\
            Описание: \"{}\"\n\n\
            Извлечи следната информация (ако е налична):\n\
            - Име на контрагент (на латиница или кирилица)\n\
            - ЕИК (Единен идентификационен код - 9 или 13 цифри)\n\
            - Булстат (същото като ЕИК)\n\
            - IBAN (банкова сметка)\n\
            - BIC/SWIFT код\n\
            - Град/населено място\n\
            - Тип транзакция (покупка, плащане, превод и т.н.)\n\n\
            Върни САМО валиден JSON със структура:\n\
            {{\n\
              \"counterpartName\": \"име или null\",\n\
              \"eik\": \"ЕИК или null\",\n\
              \"iban\": \"IBAN или null\",\n\
              \"bic\": \"BIC или null\",\n\
              \"city\": \"град или null\",\n\
              \"transactionType\": \"тип или null\",\n\
              \"confidence\": 0.0-1.0\n\
            }}\n\n\
            Важно:\n\
            - Ако контрагентът е на кирилица (например 'МИРОВЯНЕ'), транслитерирай го на латиница\n\
            - ЕИК/Булстат са само цифри (9 или 13 цифри)\n\
            - IBAN започва с кода на държавата (напр. BG)\n\
            - confidence показва колко уверен си в резултата (0.0 = несигурен, 1.0 = много сигурен)\n\
            - Ако няма данни за дадено поле, върни null\n\n\
            Отговори САМО с JSON, без допълнителни пояснения.",
            description
        );

        let body = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "Ти си експерт по български банкови транзакции и извличане на данни за контрагенти. \
                               Можеш да разпознаваш имена на фирми както на латиница, така и на кирилица. \
                               Специализиран си в извличане на ЕИК, IBAN, BIC и друга финансова информация."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.1,
            "max_tokens": 500
        });

        let response = client
            .post("https://api.mistral.ai/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Mistral API error: {} - {}", status, text));
        }

        let payload: MistralResponse = response.json().await?;
        let content = payload
            .choices
            .into_iter()
            .find_map(|c| c.message.content)
            .ok_or_else(|| anyhow!("Празен отговор от Mistral"))?;

        Ok(content)
    }

    fn parse_mistral_response(&self, raw: &str) -> Result<Option<ParsedTransactionData>> {
        let cleaned = clean_json_block(raw);

        let parsed: RawParsedData = serde_json::from_str(&cleaned).or_else(|_| {
            extract_json_object(&cleaned)
                .ok_or_else(|| anyhow!("Не може да се извлече JSON от отговора"))
                .and_then(|json| {
                    serde_json::from_str::<RawParsedData>(&json).map_err(|e| anyhow!(e))
                })
        })?;

        // Ако няма никаква информация или confidence е много нисък, върни None
        if parsed.confidence.unwrap_or(0.0) < 0.3 {
            return Ok(None);
        }

        if parsed.counterpart_name.is_none()
            && parsed.eik.is_none()
            && parsed.iban.is_none()
            && parsed.bic.is_none()
        {
            return Ok(None);
        }

        Ok(Some(ParsedTransactionData {
            counterpart_name: parsed.counterpart_name.map(|s| s.trim().to_string()),
            eik: parsed.eik.map(|s| s.trim().to_string()),
            iban: parsed.iban.map(|s| s.trim().to_string()),
            bic: parsed.bic.map(|s| s.trim().to_string()),
            city: parsed.city.map(|s| s.trim().to_string()),
            transaction_type: parsed.transaction_type.map(|s| s.trim().to_string()),
            confidence: parsed.confidence.unwrap_or(0.5),
        }))
    }

    /// Намира подходяща AI настройка за автоматично попълване на сметки въз основа на описание
    pub async fn find_matching_ai_setting(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        description: &str,
    ) -> Result<Option<ai_bank_accounting_setting::Model>> {
        // Зареждане на всички активни AI настройки за компанията, сортирани по приоритет
        let settings = AiBankAccountingSetting::find()
            .filter(ai_bank_accounting_setting::Column::CompanyId.eq(company_id))
            .filter(ai_bank_accounting_setting::Column::IsActive.eq(true))
            .order_by_desc(ai_bank_accounting_setting::Column::Priority)
            .all(db)
            .await?;

        // Намиране на първата съвпадаща настройка (с най-висок приоритет)
        for setting in settings {
            if setting.matches_description(description) {
                return Ok(Some(setting));
            }
        }

        Ok(None)
    }

    /// Автоматично попълване на банкова транзакция с подходящи сметки
    pub async fn auto_complete_transaction(
        &self,
        db: &DatabaseConnection,
        company_id: i32,
        description: &str,
    ) -> Result<Option<AutoCompletedTransaction>> {
        // Намиране на съвпадаща AI настройка
        let setting = match self.find_matching_ai_setting(db, company_id, description).await? {
            Some(s) => s,
            None => return Ok(None),
        };

        // Парсване на транзакцията за извличане на допълнителна информация (ако е необходимо)
        let parsed_data = self.parse_transaction_description(db, description).await.ok().flatten();

        // Извличане на името на контрагента, ако е налично
        let counterpart_name = parsed_data.as_ref()
            .and_then(|d| d.counterpart_name.as_ref())
            .map(|s| s.as_str());

        Ok(Some(AutoCompletedTransaction {
            pattern_name: setting.pattern_name.clone(),
            transaction_type: setting.transaction_type.clone(),
            account_id: setting.account_id,
            counterpart_account_id: setting.counterpart_account_id,
            vat_account_id: setting.vat_account_id,
            direction: setting.direction.clone(),
            description: setting.format_description(counterpart_name, description),
            parsed_data,
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawParsedData {
    counterpart_name: Option<String>,
    eik: Option<String>,
    iban: Option<String>,
    bic: Option<String>,
    city: Option<String>,
    transaction_type: Option<String>,
    confidence: Option<f64>,
}

/// Извлечени данни от банкова транзакция
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTransactionData {
    pub counterpart_name: Option<String>,
    pub eik: Option<String>,
    pub iban: Option<String>,
    pub bic: Option<String>,
    pub city: Option<String>,
    pub transaction_type: Option<String>,
    pub confidence: f64,
}

/// Автоматично попълнена транзакция с подходящи сметки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCompletedTransaction {
    pub pattern_name: String,
    pub transaction_type: String,
    pub account_id: Option<i32>,
    pub counterpart_account_id: Option<i32>,
    pub vat_account_id: Option<i32>,
    pub direction: String,
    pub description: String,
    pub parsed_data: Option<ParsedTransactionData>,
}

#[derive(Debug, Deserialize)]
struct MistralResponse {
    choices: Vec<MistralChoice>,
}

#[derive(Debug, Deserialize)]
struct MistralChoice {
    message: MistralMessage,
}

#[derive(Debug, Deserialize)]
struct MistralMessage {
    content: Option<String>,
}
