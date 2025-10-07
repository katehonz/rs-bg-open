use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::NaiveDate;
use reqwest::Client;
use rust_decimal::Decimal;
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::entities::GlobalContragentModel;
use crate::services::contragent::{
    clean_json_block, extract_json_object, ContragentDataSource, ContragentService,
};

const DEFAULT_MISTRAL_URL: &str = "https://api.mistral.ai/v1/chat/completions";
const DEFAULT_MISTRAL_MODEL: &str = "mistral-tiny";

#[derive(Clone)]
pub struct InvoiceProcessingService {
    client: Client,
    contragent_service: Arc<ContragentService>,
    mistral_url: String,
    mistral_model: String,
}

impl InvoiceProcessingService {
    pub fn new(contragent_service: Arc<ContragentService>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to build reqwest client for invoice processing");

        Self {
            client,
            contragent_service,
            mistral_url: DEFAULT_MISTRAL_URL.to_string(),
            mistral_model: DEFAULT_MISTRAL_MODEL.to_string(),
        }
    }

    pub async fn process_document(
        &self,
        db: &DatabaseConnection,
        doc: InvoiceDocument,
    ) -> Result<ProcessedInvoice> {
        let ai_provider = self
            .contragent_service
            .get_setting_value(db, "ai.provider")
            .await?
            .unwrap_or_else(|| "mistral".to_string());

        if !ai_provider.eq_ignore_ascii_case("mistral") {
            return Err(anyhow!(
                "Поддържа се само Mistral като AI доставчик към момента, а е конфигуриран: {}",
                ai_provider
            ));
        }

        let api_key = self
            .contragent_service
            .get_setting_value(db, "mistral.api.key")
            .await?
            .or_else(|| std::env::var("MISTRAL_API_KEY").ok())
            .ok_or_else(|| anyhow!("Моля конфигурирайте Mistral API ключ"))?;

        let mistral_url = self
            .contragent_service
            .get_setting_value(db, "mistral.api.url")
            .await?
            .unwrap_or_else(|| self.mistral_url.clone());

        let mistral_model = self
            .contragent_service
            .get_setting_value(db, "mistral.api.model")
            .await?
            .unwrap_or_else(|| self.mistral_model.clone());

        let payload = self.prepare_document_payload(&doc)?;
        let raw = self
            .invoke_mistral(&api_key, &mistral_url, &mistral_model, payload)
            .await?;

        let extracted = Self::parse_extraction(&raw)?;

        let mut requires_manual_review = false;
        let mut validation_source = None;
        let mut existed_in_database = None;
        let mut validated_contragent: Option<GlobalContragentModel> = None;

        if let Some(counterpart) = &extracted.counterpart {
            if let Some(vat) = counterpart
                .vat_number
                .as_ref()
                .map(|v| v.trim())
                .filter(|v| !v.is_empty())
            {
                match self.contragent_service.validate_vat(db, vat).await {
                    Ok(outcome) => {
                        validation_source = Some(outcome.source);
                        existed_in_database = Some(outcome.existed_in_database);
                        validated_contragent = Some(outcome.contragent);
                    }
                    Err(err) => {
                        tracing::warn!("VAT validation failed: {}", err);
                        requires_manual_review = true;
                    }
                }
            } else if let Some(eik) = counterpart
                .eik
                .as_ref()
                .map(|v| v.trim())
                .filter(|v| !v.is_empty())
            {
                match self.contragent_service.validate_eik(db, eik).await {
                    Ok(model) => {
                        validation_source = Some(ContragentDataSource::Database);
                        existed_in_database = Some(true);
                        validated_contragent = Some(model);
                        requires_manual_review = false;
                    }
                    Err(err) => {
                        tracing::warn!("EIK validation failed: {}", err);
                        requires_manual_review = true;
                    }
                }
            } else {
                requires_manual_review = true;
            }
        } else {
            requires_manual_review = true;
        }

        if extracted.document_number.is_none()
            || extracted.document_date.is_none()
            || extracted.net_amount.is_none()
            || extracted.vat_amount.is_none()
            || extracted.total_amount.is_none()
        {
            requires_manual_review = true;
        }

        Ok(ProcessedInvoice {
            company_id: doc.company_id,
            extracted,
            validated_contragent,
            validation_source,
            existed_in_database,
            requires_manual_review,
        })
    }

    fn prepare_document_payload(&self, doc: &InvoiceDocument) -> Result<DocumentPayload> {
        let mime = doc
            .content_type
            .as_deref()
            .or_else(|| mime_from_filename(&doc.file_name));

        match mime {
            Some(mime) if mime.starts_with("image/") => {
                let data_url = format!("data:{};base64,{}", mime, BASE64.encode(&doc.file_bytes));
                Ok(DocumentPayload::Image { data_url })
            }
            Some("application/pdf") | Some("application/x-pdf") => {
                let text = extract_text_from_pdf(&doc.file_bytes)?;
                Ok(DocumentPayload::Text { text })
            }
            _ => {
                // fallback: try to treat as text (e.g. XML or already OCR-ed)
                if let Ok(text) = String::from_utf8(doc.file_bytes.clone()) {
                    Ok(DocumentPayload::Text { text })
                } else {
                    Err(anyhow!(
                        "Неподдържан формат за документ: {:?}. Допустими са изображения или PDF",
                        doc.content_type
                    ))
                }
            }
        }
    }

    async fn invoke_mistral(
        &self,
        api_key: &str,
        url: &str,
        model: &str,
        payload: DocumentPayload,
    ) -> Result<String> {
        let body = payload.into_mistral_payload(model);

        let response = self
            .client
            .post(url)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .context("Грешка при извикване на Mistral API")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Mistral API error: {} - {}", status, text));
        }

        let payload: MistralChatResponse = response.json().await?;
        let content = payload
            .choices
            .into_iter()
            .find_map(|choice| choice.message.content)
            .ok_or_else(|| anyhow!("Празен отговор от Mistral"))?;

        Ok(content)
    }

    fn parse_extraction(raw: &str) -> Result<ParsedInvoice> {
        let cleaned = clean_json_block(raw);
        let mut parse_attempt =
            serde_json::from_str::<RawInvoiceExtraction>(&cleaned).or_else(|_| {
                extract_json_object(&cleaned)
                    .ok_or_else(|| anyhow!("Неуспешно парсиране на JSON"))
                    .and_then(|json| {
                        serde_json::from_str::<RawInvoiceExtraction>(&json).map_err(|e| anyhow!(e))
                    })
            })?;

        parse_attempt.normalise();
        Ok(parse_attempt.into())
    }
}

#[derive(Debug, Clone)]
pub struct InvoiceDocument {
    pub company_id: Option<i64>,
    pub file_name: String,
    pub content_type: Option<String>,
    pub file_bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct ProcessedInvoice {
    pub company_id: Option<i64>,
    pub extracted: ParsedInvoice,
    pub validated_contragent: Option<GlobalContragentModel>,
    pub validation_source: Option<ContragentDataSource>,
    pub existed_in_database: Option<bool>,
    pub requires_manual_review: bool,
}

#[derive(Debug, Clone)]
pub struct ParsedInvoice {
    pub document_type: Option<String>,
    pub transaction_type: Option<String>,
    pub document_number: Option<String>,
    pub document_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub currency: Option<String>,
    pub net_amount: Option<Decimal>,
    pub vat_amount: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub counterpart: Option<ParsedCounterpart>,
    pub items: Vec<ParsedInvoiceItem>,
}

#[derive(Debug, Clone)]
pub struct ParsedCounterpart {
    pub name: Option<String>,
    pub eik: Option<String>,
    pub vat_number: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedInvoiceItem {
    pub description: Option<String>,
    pub quantity: Option<Decimal>,
    pub unit: Option<String>,
    pub unit_price: Option<Decimal>,
    pub total_price: Option<Decimal>,
    pub vat_rate: Option<Decimal>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawInvoiceExtraction {
    document_type: Option<String>,
    transaction_type: Option<String>,
    document_number: Option<String>,
    #[serde(deserialize_with = "deserialize_date_opt", default)]
    document_date: Option<NaiveDate>,
    #[serde(deserialize_with = "deserialize_date_opt", default)]
    due_date: Option<NaiveDate>,
    currency: Option<String>,
    #[serde(deserialize_with = "deserialize_decimal_opt", default)]
    net_amount: Option<Decimal>,
    #[serde(deserialize_with = "deserialize_decimal_opt", default)]
    vat_amount: Option<Decimal>,
    #[serde(deserialize_with = "deserialize_decimal_opt", default)]
    total_amount: Option<Decimal>,
    counterpart: Option<RawCounterpart>,
    items: Option<Vec<RawItem>>,
}

impl RawInvoiceExtraction {
    fn normalise(&mut self) {
        self.document_type = self
            .document_type
            .take()
            .map(|v| v.trim().to_uppercase())
            .filter(|v| !v.is_empty());
        self.transaction_type = self
            .transaction_type
            .take()
            .map(|v| v.trim().to_uppercase())
            .filter(|v| !v.is_empty());
        self.document_number = self
            .document_number
            .take()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawCounterpart {
    name: Option<String>,
    eik: Option<String>,
    vat_number: Option<String>,
    address: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawItem {
    description: Option<String>,
    #[serde(deserialize_with = "deserialize_decimal_opt", default)]
    quantity: Option<Decimal>,
    unit: Option<String>,
    #[serde(deserialize_with = "deserialize_decimal_opt", default)]
    unit_price: Option<Decimal>,
    #[serde(deserialize_with = "deserialize_decimal_opt", default)]
    total_price: Option<Decimal>,
    #[serde(deserialize_with = "deserialize_decimal_opt", default)]
    vat_rate: Option<Decimal>,
}

impl From<RawInvoiceExtraction> for ParsedInvoice {
    fn from(raw: RawInvoiceExtraction) -> Self {
        let items = raw
            .items
            .unwrap_or_default()
            .into_iter()
            .map(|item| ParsedInvoiceItem {
                description: item.description.map(normalise_string),
                quantity: item.quantity,
                unit: item.unit.map(normalise_string),
                unit_price: item.unit_price,
                total_price: item.total_price,
                vat_rate: item.vat_rate,
            })
            .collect();

        Self {
            document_type: raw.document_type,
            transaction_type: raw.transaction_type,
            document_number: raw.document_number,
            document_date: raw.document_date,
            due_date: raw.due_date,
            currency: raw.currency,
            net_amount: raw.net_amount,
            vat_amount: raw.vat_amount,
            total_amount: raw.total_amount,
            counterpart: raw.counterpart.map(|c| ParsedCounterpart {
                name: c.name.map(normalise_string),
                eik: c.eik.map(normalise_string),
                vat_number: c.vat_number.map(normalise_string),
                address: c.address.map(normalise_string),
            }),
            items,
        }
    }
}

fn normalise_string(value: String) -> String {
    value.trim().trim_matches('"').to_string()
}

fn mime_from_filename(name: &str) -> Option<&'static str> {
    let lowered = name.to_lowercase();
    if lowered.ends_with(".png") {
        Some("image/png")
    } else if lowered.ends_with(".jpg") || lowered.ends_with(".jpeg") {
        Some("image/jpeg")
    } else if lowered.ends_with(".pdf") {
        Some("application/pdf")
    } else {
        None
    }
}

fn extract_text_from_pdf(bytes: &[u8]) -> Result<String> {
    pdf_extract::extract_text_from_mem(bytes)
        .map_err(|err| anyhow!("Неуспешно извличане на текст от PDF: {}", err))
}

fn deserialize_decimal_opt<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let opt = Option::<serde_json::Value>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(serde_json::Value::Null) => Ok(None),
        Some(value) => {
            let number_string = match value {
                serde_json::Value::String(s) => s,
                serde_json::Value::Number(num) => num.to_string(),
                serde_json::Value::Bool(b) => (b as i32).to_string(),
                other => {
                    return Err(D::Error::custom(format!(
                        "Неочакван тип за числова стойност: {}",
                        other
                    )))
                }
            };

            let trimmed = number_string.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }

            Decimal::from_str(trimmed)
                .map(Some)
                .map_err(|_| D::Error::custom("Невалидно десетично число"))
        }
    }
}

fn deserialize_date_opt<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(value) => {
            let value = value.trim();
            if value.is_empty() || value.eq_ignore_ascii_case("null") {
                return Ok(None);
            }

            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .or_else(|_| NaiveDate::parse_from_str(value, "%d.%m.%Y"))
                .or_else(|_| NaiveDate::parse_from_str(value, "%d/%m/%Y"))
                .map(Some)
                .map_err(|err| D::Error::custom(format!("Невалидна дата: {} ({})", value, err)))
        }
    }
}

enum DocumentPayload {
    Image { data_url: String },
    Text { text: String },
}

impl DocumentPayload {
    fn into_mistral_payload(self, model: &str) -> serde_json::Value {
        match self {
            DocumentPayload::Image { data_url } => {
                let image_entry = serde_json::json!({
                    "type": "image_url",
                    "image_url": { "url": data_url }
                });

                let instructions = serde_json::json!({
                    "type": "text",
                    "text": extraction_prompt()
                });

                serde_json::json!({
                    "model": model,
                    "messages": [
                        {
                            "role": "user",
                            "content": [image_entry, instructions]
                        }
                    ],
                    "temperature": 0.1,
                    "max_tokens": 2000
                })
            }
            DocumentPayload::Text { text } => {
                let user_message = format!(
                    "{}\n\nOCR текст на документа:\n{}",
                    extraction_prompt(),
                    text
                );

                serde_json::json!({
                    "model": model,
                    "messages": [
                        {"role": "system", "content": "Ти си експерт по български счетоводни документи."},
                        {"role": "user", "content": user_message}
                    ],
                    "temperature": 0.1,
                    "max_tokens": 2000
                })
            }
        }
    }
}

fn extraction_prompt() -> &'static str {
    "Анализирай тази българска фактура и върни САМО валиден JSON със структура:\n\
    {\n  \"documentType\": \"INVOICE\" | \"CREDIT_NOTE\" | \"DEBIT_NOTE\",\n  \"transactionType\": \"PURCHASE\" | \"SALE\",\n  \"documentNumber\": \"номер\",\n  \"documentDate\": \"YYYY-MM-DD\",\n  \"dueDate\": \"YYYY-MM-DD\",\n  \"currency\": \"валута\",\n  \"counterpart\": {\n    \"name\": \"име\",\n    \"eik\": \"ЕИК\",\n    \"vatNumber\": \"ДДС номер\",\n    \"address\": \"адрес\"\n  },\n  \"netAmount\": число,\n  \"vatAmount\": число,\n  \"totalAmount\": число,\n  \"items\": [\n    {\n      \"description\": \"описание\",\n      \"quantity\": число,\n      \"unit\": \"брой/кг/...\",\n      \"unitPrice\": число,\n      \"totalPrice\": число,\n      \"vatRate\": число\n    }\n  ]\n}\nОтговори само с JSON без пояснения."
}

#[derive(Deserialize)]
struct MistralChatResponse {
    choices: Vec<MistralChoice>,
}

#[derive(Deserialize)]
struct MistralChoice {
    message: MistralMessage,
}

#[derive(Deserialize)]
struct MistralMessage {
    content: Option<String>,
}

// Re-export selected structs for GraphQL layer
