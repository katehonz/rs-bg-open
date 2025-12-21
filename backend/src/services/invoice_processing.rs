use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::NaiveDate;
use reqwest::Client;
use rust_decimal::Decimal;
use sea_orm::{DatabaseConnection, EntityTrait, Set, ColumnTrait, QueryFilter};
use serde::Deserialize;

use crate::entities::{GlobalContragentModel, counterpart};
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
        let mut auto_created_counterpart_id: Option<i32> = None;

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

        // Auto-create counterpart if we have validated VIES data and company_id
        if let (Some(company_id), Some(ref validated)) = (doc.company_id, &validated_contragent) {
            tracing::info!(
                "Attempting to auto-create counterpart for company_id={}, VAT={}",
                company_id,
                &validated.vat_number
            );
            match self
                .get_or_create_counterpart_from_validated(
                    db,
                    company_id,
                    validated,
                    extracted.transaction_type.as_deref(),
                )
                .await
            {
                Ok(counterpart_id) => {
                    auto_created_counterpart_id = Some(counterpart_id);
                    tracing::info!(
                        "✓ Auto-created/found counterpart id={} for invoice (company_id={})",
                        counterpart_id,
                        company_id
                    );
                }
                Err(err) => {
                    tracing::error!("✗ Failed to auto-create counterpart: {:?}", err);
                    requires_manual_review = true;
                }
            }
        } else {
            tracing::warn!(
                "Cannot auto-create counterpart: company_id={:?}, validated_contragent={}",
                doc.company_id,
                validated_contragent.is_some()
            );
        }

        Ok(ProcessedInvoice {
            company_id: doc.company_id,
            extracted,
            validated_contragent,
            validation_source,
            existed_in_database,
            requires_manual_review,
            auto_created_counterpart_id,
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
        tracing::error!("🔍 RAW Mistral Response:\n{}", raw);
        let cleaned = clean_json_block(raw);
        tracing::error!("🧹 CLEANED JSON:\n{}", cleaned);
        let mut parse_attempt =
            serde_json::from_str::<RawInvoiceExtraction>(&cleaned).or_else(|e| {
                tracing::error!("❌ JSON Parse Error: {}", e);
                extract_json_object(&cleaned)
                    .ok_or_else(|| anyhow!("Неуспешно парсиране на JSON"))
                    .and_then(|json| {
                        tracing::error!("🔍 EXTRACTED JSON:\n{}", json);
                        serde_json::from_str::<RawInvoiceExtraction>(&json).map_err(|e| {
                            tracing::error!("❌ Second Parse Error: {}", e);
                            anyhow!(e)
                        })
                    })
            })?;

        parse_attempt.normalise();
        Ok(parse_attempt.into())
    }

    /// Auto-create Counterpart from validated GlobalContragent (VIES data)
    /// Returns existing counterpart_id if found by VAT or EIK, otherwise creates new
    pub async fn get_or_create_counterpart_from_validated(
        &self,
        db: &DatabaseConnection,
        company_id: i64,
        validated_contragent: &GlobalContragentModel,
        transaction_type: Option<&str>,
    ) -> Result<i32> {
        // First, check if counterpart already exists by VAT number or EIK
        let vat = &validated_contragent.vat_number;
        let existing = counterpart::Entity::find()
            .filter(counterpart::Column::CompanyId.eq(company_id))
            .filter(counterpart::Column::VatNumber.eq(vat))
            .one(db)
            .await?
            .or_else(|| {
                // If not found by VAT, try to find by EIK
                None
            });

        let existing = if existing.is_none() {
            if let Some(eik) = &validated_contragent.eik {
                counterpart::Entity::find()
                    .filter(counterpart::Column::CompanyId.eq(company_id))
                    .filter(counterpart::Column::Eik.eq(eik))
                    .one(db)
                    .await?
            } else {
                None
            }
        } else {
            existing
        };

        if let Some(existing) = existing {
            tracing::info!(
                "Found existing counterpart id={} for company={}, VAT={}",
                existing.id,
                company_id,
                &validated_contragent.vat_number
            );
            return Ok(existing.id);
        }

        // Determine counterpart type and flags based on transaction type
        let (is_customer, is_supplier, counterpart_type) = match transaction_type {
            Some("SALE") => (true, false, counterpart::CounterpartType::Customer),
            Some("PURCHASE") => (false, true, counterpart::CounterpartType::Supplier),
            _ => (false, true, counterpart::CounterpartType::Supplier), // Default to supplier
        };

        // Create new counterpart from VIES validated data
        let new_counterpart = counterpart::ActiveModel {
            name: Set(validated_contragent.company_name.clone()
                .or(validated_contragent.company_name_bg.clone())
                .unwrap_or_else(|| "Unknown".to_string())),
            eik: Set(validated_contragent.eik.clone()),
            vat_number: Set(Some(validated_contragent.vat_number.clone())),
            address: Set(validated_contragent.long_address.clone()
                .or(validated_contragent.address.clone())),
            street: Set(validated_contragent.street_name.clone()),
            city: Set(validated_contragent.city.clone()),
            postal_code: Set(validated_contragent.postal_code.clone()),
            country: Set(validated_contragent.country.clone()),
            is_vat_registered: Set(validated_contragent.valid),
            is_customer: Set(is_customer),
            is_supplier: Set(is_supplier),
            counterpart_type: Set(counterpart_type),
            is_active: Set(true),
            company_id: Set(company_id as i32),
            phone: Set(validated_contragent.phone.clone()),
            email: Set(validated_contragent.email.clone()),
            contact_person: Set(validated_contragent.contact_person.clone()),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        let result = counterpart::Entity::insert(new_counterpart)
            .exec_with_returning(db)
            .await
            .context("Failed to create counterpart from VIES data")?;

        tracing::info!(
            "Auto-created counterpart id={} from VIES data: {} (VAT: {}, EIK: {:?})",
            result.id,
            result.name,
            &validated_contragent.vat_number,
            result.eik
        );

        Ok(result.id)
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
    pub auto_created_counterpart_id: Option<i32>,
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
    {\n  \"documentType\": \"INVOICE\" | \"CREDIT_NOTE\" | \"DEBIT_NOTE\",\n  \"transactionType\": \"PURCHASE\" | \"SALE\",\n  \"documentNumber\": \"номер\",\n  \"documentDate\": \"YYYY-MM-DD\",\n  \"dueDate\": \"YYYY-MM-DD\",\n  \"currency\": \"валута\",\n  \"counterpart\": {\n    \"name\": \"име\",\n    \"eik\": \"ЕИК\",\n    \"vatNumber\": \"ДДС номер\",\n    \"address\": \"адрес\"\n  },\n  \"netAmount\": число,\n  \"vatAmount\": число,\n  \"totalAmount\": число,\n  \"items\": [\n    {\n      \"description\": \"описание\",\n      \"quantity\": число,\n      \"unit\": \"брой/кг/...\",\n      \"unitPrice\": число,\n      \"totalPrice\": число,\n      \"vatRate\": число\n    }\n  ]\n}\n\nКРИТИЧНО ВАЖНО за определяне на transactionType:\n- PURCHASE (Покупка): Ако на фактурата пише \"ПОЛУЧАТЕЛ\", \"КУПУВАЧ\", \"КЛИЕНТ\" в долната част - това е ПОКУПКА\n- SALE (Продажба): Ако на фактурата пише \"ДОСТАВЧИК\", \"ПРОДАВАЧ\", \"ИЗДАТЕЛ\" в долната част - това е ПРОДАЖБА\n\nКРИТИЧНО ВАЖНО за counterpart (контрагент):\n⚠️ НА ФАКТУРАТА ИМА 2 ФИРМИ - ТРЯБВА ДА ИЗВЛЕЧЕШ ПРАВИЛНАТА!\n\n1. За ПОКУПКА (PURCHASE):\n   - counterpart е ДОСТАВЧИКЪТ (фирмата ГОРЕ, която издава фактурата)\n   - НЕ извличай данните на ПОЛУЧАТЕЛЯ (долу) - това е нашата фирма!\n   - Търси секция \"ДОСТАВЧИК\", \"ИЗДАТЕЛ\", \"ПРОДАВАЧ\" - ТОВА е counterpart\n   - Секция \"ПОЛУЧАТЕЛ\", \"КУПУВАЧ\" НЕ Е counterpart!\n\n2. За ПРОДАЖБА (SALE):\n   - counterpart е КЛИЕНТЪТ (фирмата ДОЛУ, на която издаваме)\n   - НЕ извличай данните на ИЗДАТЕЛЯ (горе) - това е нашата фирма!\n   - Търси секция \"ПОЛУЧАТЕЛ\", \"КУПУВАЧ\", \"КЛИЕНТ\" - ТОВА е counterpart\n   - Секция \"ИЗДАТЕЛ\", \"ПРОДАВАЧ\" НЕ Е counterpart!\n\n3. Правила за ДДС номер:\n   - vatNumber е НАЙ-ВАЖНОТО поле\n   - Формат: BGxxxxxxxxx или BG + 9-10 цифри\n   - Не бъркай с МОЛ, касиери, продавачи\n   - ЕИК е само 9-13 цифри БЕЗ префикс BG\n\n4. Ако има САМО ЕДИН ДДС номер на фактурата:\n   - За ПОКУПКА: Това е ДДС-то на ДОСТАВЧИКА → извлечи го\n   - За ПРОДАЖБА: Това е ДДС-то на КЛИЕНТА → извлечи го\n\n5. Ако има ДВА ДДС номера (често при покупки):\n   - За ПОКУПКА: Извлечи ДДС-то от секция \"ДОСТАВЧИК\" (горе)\n   - За ПРОДАЖБА: Извлечи ДДС-то от секция \"ПОЛУЧАТЕЛ\" (долу)\n\nОтговори само с JSON без пояснения."
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
