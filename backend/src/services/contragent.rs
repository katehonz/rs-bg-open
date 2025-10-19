use std::collections::HashMap;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use quick_xml::events::Event;
use quick_xml::Reader;
use regex::Regex;
use reqwest::{Client, StatusCode};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, FromQueryResult, IntoActiveModel,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use sea_orm::{ConnectionTrait, DatabaseConnection, TransactionTrait};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::entities::{
    contragent_setting, global_contragent, ContragentSetting, ContragentSettingActiveModel,
    ContragentSettingModel, GlobalContragent, GlobalContragentActiveModel, GlobalContragentFilter,
    GlobalContragentModel, GlobalContragentSummary,
};

const DEFAULT_VIES_SOAP_URL: &str =
    "https://ec.europa.eu/taxation_customs/vies/services/checkVatService";
const VIES_REST_URL_TEMPLATE: &str =
    "https://ec.europa.eu/taxation_customs/vies/rest-api/ms/{countryCode}/vat/{vatNumber}";
const DEFAULT_MISTRAL_API_URL: &str = "https://api.mistral.ai/v1/chat/completions";
const DEFAULT_MISTRAL_MODEL: &str = "mistral-tiny";
const DEFAULT_BULGARIAN_REGISTRY_URL: &str = "https://portal.registryagency.bg/api";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContragentDataSource {
    Database,
    ViesRest,
    ViesSoap,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, async_graphql::SimpleObject)]
#[graphql(name = "AddressComponents")]
pub struct AddressComponents {
    #[serde(alias = "street", alias = "streetName")]
    pub street_name: Option<String>,
    #[serde(alias = "city")]
    pub city: Option<String>,
    #[serde(alias = "postal", alias = "postalCode")]
    pub postal_code: Option<String>,
    #[serde(alias = "country")]
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOutcome {
    pub contragent: GlobalContragentModel,
    pub existed_in_database: bool,
    pub source: ContragentDataSource,
}

pub struct ContragentService {
    vies_client: ViesClient,
    registry_client: BulgarianRegistryClient,
    address_parser: AddressParser,
}

impl ContragentService {
    pub fn new() -> Self {
        Self {
            vies_client: ViesClient::new(None),
            registry_client: BulgarianRegistryClient::new(None),
            address_parser: AddressParser::new(None),
        }
    }

    pub async fn validate_vat(
        &self,
        db: &DatabaseConnection,
        vat_number_raw: &str,
    ) -> Result<ValidationOutcome> {
        let vat_number = normalize_vat_number(vat_number_raw)?;

        if let Some(existing) = GlobalContragent::find()
            .filter(global_contragent::Column::VatNumber.eq(vat_number.clone()))
            .one(db)
            .await?
        {
            let mut active: GlobalContragentActiveModel = existing.clone().into_active_model();
            active.last_validated_at = Set(Some(Utc::now()));
            active.updated_at = Set(Utc::now());
            let updated = active.update(db).await?;
            return Ok(ValidationOutcome {
                contragent: updated,
                existed_in_database: true,
                source: ContragentDataSource::Database,
            });
        }

        let vies_response = self
            .vies_client
            .check_vat(&vat_number)
            .await
            .context("VIES validation failed")?;

        if !vies_response.valid {
            let model = GlobalContragentActiveModel {
                vat_number: Set(vat_number.clone()),
                valid: Set(false),
                vat_valid: Set(false),
                eik_valid: Set(false),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
                ..Default::default()
            }
            .insert(db)
            .await?;

            return Ok(ValidationOutcome {
                contragent: model,
                existed_in_database: false,
                source: vies_response.source,
            });
        }

        let eik = extract_bulgarian_eik(&vat_number);

        let ai_provider = self
            .get_setting_value(db, "ai.provider")
            .await?
            .unwrap_or_else(|| "mistral".to_string());
        let api_key = self.get_setting_value(db, "mistral.api.key").await?;

        let address_components = if vies_response
            .address
            .as_ref()
            .map(|addr| !addr.trim().is_empty())
            .unwrap_or(false)
        {
            self.address_parser
                .parse(
                    vies_response.address.as_deref().unwrap(),
                    api_key.as_deref(),
                    ai_provider.as_str(),
                )
                .await
                .unwrap_or(None)
        } else {
            None
        };

        let mut street_name = address_components
            .as_ref()
            .and_then(|c| c.street_name.clone());
        let mut city = address_components.as_ref().and_then(|c| c.city.clone());
        let mut postal_code = address_components
            .as_ref()
            .and_then(|c| c.postal_code.clone());
        let mut country = address_components
            .as_ref()
            .and_then(|c| c.country.clone())
            .or_else(|| Some(vies_response.country_code.clone()));

        let mut company_name = vies_response.name.clone();
        let mut company_name_bg = vies_response.name.clone();
        let mut legal_form: Option<String> = None;
        let mut status: Option<String> = None;
        let mut address = vies_response.address.clone();
        let mut long_address = vies_response.address.clone();
        let mut eik_valid = false;

        if let Some(eik_value) = eik.clone() {
            if let Ok(registry_info) = self.registry_client.check_eik(&eik_value).await {
                eik_valid = registry_info.valid.unwrap_or(true);

                if let Some(value) = registry_info.company_name.clone() {
                    if company_name.is_none() {
                        company_name = Some(value);
                    } else if let Some(existing) = &company_name {
                        if existing.trim().is_empty() {
                            company_name = Some(value);
                        }
                    }
                }

                if let Some(value) = registry_info.company_name_bg.clone() {
                    company_name_bg = Some(value);
                }

                if let Some(value) = registry_info.legal_form.clone() {
                    legal_form = Some(value);
                }

                if let Some(value) = registry_info.status.clone() {
                    status = Some(value);
                }

                if let Some(value) = registry_info.address.clone() {
                    address = Some(value);
                }

                if let Some(value) = registry_info.long_address.clone() {
                    long_address = Some(value);
                }

                if let Some(value) = registry_info.street_name.clone() {
                    street_name = Some(value);
                }

                if let Some(value) = registry_info.city.clone() {
                    city = Some(value);
                }

                if let Some(value) = registry_info.postal_code.clone() {
                    postal_code = Some(value);
                }

                if let Some(value) = registry_info.country.clone() {
                    country = Some(value);
                }
            }
        }

        if company_name_bg.is_none() {
            company_name_bg = company_name.clone();
        }

        if address.is_none() {
            address = long_address.clone();
        }

        let now = Utc::now();
        let active_model = GlobalContragentActiveModel {
            vat_number: Set(vat_number.clone()),
            eik: Set(eik.clone()),
            company_name: Set(company_name),
            company_name_bg: Set(company_name_bg),
            legal_form: Set(legal_form),
            status: Set(status),
            address: Set(address),
            long_address: Set(long_address),
            street_name: Set(street_name),
            city: Set(city),
            postal_code: Set(postal_code),
            country: Set(country),
            vat_valid: Set(true),
            valid: Set(true),
            eik_valid: Set(eik_valid),
            last_validated_at: Set(Some(now)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match active_model.insert(db).await {
            Ok(model) => Ok(ValidationOutcome {
                contragent: model,
                existed_in_database: false,
                source: vies_response.source,
            }),
            Err(err) => {
                if err.to_string().contains("unique") {
                    if let Some(existing) = GlobalContragent::find()
                        .filter(global_contragent::Column::VatNumber.eq(vat_number.clone()))
                        .one(db)
                        .await?
                    {
                        return Ok(ValidationOutcome {
                            contragent: existing,
                            existed_in_database: true,
                            source: ContragentDataSource::Database,
                        });
                    }
                }
                Err(err.into())
            }
        }
    }

    pub async fn refresh_vat(
        &self,
        db: &DatabaseConnection,
        vat_number: &str,
    ) -> Result<ValidationOutcome> {
        self.validate_vat(db, vat_number).await
    }

    pub async fn validate_eik(
        &self,
        db: &DatabaseConnection,
        eik: &str,
    ) -> Result<GlobalContragentModel> {
        let registry_data = self.registry_client.check_eik(eik).await?;

        if let Some(existing) = GlobalContragent::find()
            .filter(global_contragent::Column::Eik.eq(eik.to_string()))
            .one(db)
            .await?
        {
            let mut active = existing.into_active_model();
            active.eik_valid = Set(registry_data.valid.unwrap_or(true));
            active.company_name = Set(registry_data.company_name.clone());
            active.company_name_bg = Set(registry_data.company_name_bg.clone());
            active.legal_form = Set(registry_data.legal_form.clone());
            active.status = Set(registry_data.status.clone());
            active.address = Set(registry_data.address.clone());
            active.long_address = Set(registry_data.long_address.clone());
            active.street_name = Set(registry_data.street_name.clone());
            active.city = Set(registry_data.city.clone());
            active.postal_code = Set(registry_data.postal_code.clone());
            active.country = Set(registry_data.country.clone());
            active.updated_at = Set(Utc::now());
            return Ok(active.update(db).await?);
        }

        let now = Utc::now();
        let model = GlobalContragentActiveModel {
            eik: Set(Some(eik.to_string())),
            eik_valid: Set(registry_data.valid.unwrap_or(true)),
            company_name: Set(registry_data.company_name.clone()),
            company_name_bg: Set(registry_data.company_name_bg.clone()),
            legal_form: Set(registry_data.legal_form.clone()),
            status: Set(registry_data.status.clone()),
            address: Set(registry_data.address.clone()),
            long_address: Set(registry_data.long_address.clone()),
            street_name: Set(registry_data.street_name.clone()),
            city: Set(registry_data.city.clone()),
            postal_code: Set(registry_data.postal_code.clone()),
            country: Set(registry_data.country.clone()),
            vat_valid: Set(false),
            valid: Set(registry_data.valid.unwrap_or(true)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(model)
    }

    pub async fn process_existing_addresses(&self, db: &DatabaseConnection) -> Result<(i64, i64)> {
        let records = GlobalContragent::find()
            .filter(global_contragent::Column::LongAddress.is_not_null())
            .all(db)
            .await?;

        let ai_provider = self
            .get_setting_value(db, "ai.provider")
            .await?
            .unwrap_or_else(|| "mistral".to_string());
        let api_key = self.get_setting_value(db, "mistral.api.key").await?;

        let mut processed = 0;
        let mut failed = 0;

        for record in records {
            if has_complete_address(&record) {
                continue;
            }

            if let Some(long_address) = record.long_address.clone() {
                match self
                    .address_parser
                    .parse(&long_address, api_key.as_deref(), ai_provider.as_str())
                    .await
                {
                    Ok(Some(components)) => {
                        let needs_street = record
                            .street_name
                            .as_ref()
                            .map(|s| s.trim().is_empty())
                            .unwrap_or(true);
                        let needs_city = record
                            .city
                            .as_ref()
                            .map(|s| s.trim().is_empty())
                            .unwrap_or(true);
                        let needs_postal = record
                            .postal_code
                            .as_ref()
                            .map(|s| s.trim().is_empty())
                            .unwrap_or(true);
                        let needs_country = record
                            .country
                            .as_ref()
                            .map(|s| s.trim().is_empty())
                            .unwrap_or(true);

                        let mut active = record.into_active_model();

                        if needs_street {
                            if let Some(street) = components.street_name.clone() {
                                active.street_name = Set(Some(street));
                            }
                        }

                        if needs_city {
                            if let Some(city) = components.city.clone() {
                                active.city = Set(Some(city));
                            }
                        }

                        if needs_postal {
                            if let Some(postal) = components.postal_code.clone() {
                                active.postal_code = Set(Some(postal));
                            }
                        }

                        if needs_country {
                            if let Some(country) = components.country.clone() {
                                active.country = Set(Some(country));
                            }
                        }

                        active.updated_at = Set(Utc::now());
                        active.update(db).await?;
                        processed += 1;
                    }
                    Ok(None) => {
                        failed += 1;
                    }
                    Err(_) => {
                        failed += 1;
                    }
                }
            }
        }

        Ok((processed, failed))
    }

    pub async fn get_summary(&self, db: &DatabaseConnection) -> Result<GlobalContragentSummary> {
        let total = GlobalContragent::find().count(db).await? as i64;
        let valid_count = GlobalContragent::find()
            .filter(global_contragent::Column::Valid.eq(true))
            .count(db)
            .await? as i64;
        let invalid_count = total - valid_count;
        let last_synced = GlobalContragent::find()
            .order_by_desc(global_contragent::Column::LastValidatedAt)
            .select_only()
            .column(global_contragent::Column::LastValidatedAt)
            .into_tuple::<Option<DateTime<Utc>>>()
            .one(db)
            .await?
            .flatten();

        Ok(GlobalContragentSummary {
            total,
            valid_count,
            invalid_count,
            last_synced_at: last_synced,
        })
    }

    pub async fn get_setting_value(
        &self,
        db: &DatabaseConnection,
        key: &str,
    ) -> Result<Option<String>> {
        #[derive(FromQueryResult)]
        struct SettingValue {
            value: Option<String>,
        }

        let setting = ContragentSetting::find()
            .filter(contragent_setting::Column::Key.eq(key))
            .select_only()
            .column(contragent_setting::Column::Value)
            .into_model::<SettingValue>()
            .one(db)
            .await?;

        Ok(setting.and_then(|s| s.value))
    }

    pub async fn upsert_setting(
        &self,
        db: &DatabaseConnection,
        input: crate::entities::UpsertContragentSettingInput,
    ) -> Result<ContragentSettingModel> {
        let txn = db.begin().await?;
        #[derive(FromQueryResult)]
        struct SettingId {
            id: i64,
        }

        let existing = ContragentSetting::find()
            .filter(contragent_setting::Column::Key.eq(input.key.clone()))
            .select_only()
            .column(contragent_setting::Column::Id)
            .into_model::<SettingId>()
            .one(&txn)
            .await?;

        let now = Utc::now();
        let mut active = ContragentSettingActiveModel {
            id: ActiveValue::NotSet,
            key: ActiveValue::Set(input.key.clone()),
            value: ActiveValue::Set(input.value.clone()),
            description: ActiveValue::Set(input.description.clone()),
            encrypted: ActiveValue::Set(input.encrypted.unwrap_or(false)),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };

        if let Some(existing) = existing {
            active.id = ActiveValue::Set(existing.id);
            active.key = ActiveValue::NotSet;
            active.created_at = ActiveValue::NotSet;
        }

        let saved = if matches!(active.id, ActiveValue::NotSet) {
            active.insert(&txn).await?
        } else {
            active.update(&txn).await?
        };

        txn.commit().await?;
        Ok(saved)
    }

    pub async fn parse_address(
        &self,
        db: &DatabaseConnection,
        address: &str,
    ) -> Result<Option<AddressComponents>> {
        let ai_provider = self
            .get_setting_value(db, "ai.provider")
            .await?
            .unwrap_or_else(|| "mistral".to_string());
        let api_key = self.get_setting_value(db, "mistral.api.key").await?;

        self.address_parser
            .parse(address, api_key.as_deref(), ai_provider.as_str())
            .await
    }
}

#[derive(Debug, Clone)]
struct ViesResponse {
    valid: bool,
    name: Option<String>,
    address: Option<String>,
    country_code: String,
    source: ContragentDataSource,
}

#[derive(Clone)]
pub struct ViesClient {
    client: Client,
    soap_url: String,
}

impl ViesClient {
    pub fn new(soap_url: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build reqwest client for VIES");
        Self {
            client,
            soap_url: soap_url.unwrap_or_else(|| DEFAULT_VIES_SOAP_URL.to_string()),
        }
    }

    pub async fn check_vat(&self, vat_number: &str) -> Result<ViesResponse> {
        let (country_code, numeric) = split_vat_number(vat_number)?;

        if let Ok(rest_result) = self.check_rest(&country_code, numeric).await {
            return Ok(rest_result);
        }

        self.check_soap(&country_code, numeric).await
    }

    async fn check_rest(&self, country_code: &str, vat_number: &str) -> Result<ViesResponse> {
        let url = VIES_REST_URL_TEMPLATE
            .replace("{countryCode}", country_code)
            .replace("{vatNumber}", vat_number);
        let response = self.client.get(url).send().await?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(ViesResponse {
                valid: false,
                name: None,
                address: None,
                country_code: country_code.to_string(),
                source: ContragentDataSource::ViesRest,
            });
        }

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!("VIES REST returned status {}", status));
        }

        let rest: ViesRestResponse = serde_json::from_str(&body)?;
        Ok(ViesResponse {
            valid: rest.is_valid,
            name: rest.name,
            address: rest.address,
            country_code: rest
                .country_code
                .unwrap_or_else(|| country_code.to_string()),
            source: ContragentDataSource::ViesRest,
        })
    }

    async fn check_soap(&self, country_code: &str, vat_number: &str) -> Result<ViesResponse> {
        let request_body = build_soap_request(country_code, vat_number);
        let response = self
            .client
            .post(&self.soap_url)
            .header("Content-Type", "text/xml; charset=utf-8")
            .body(request_body)
            .send()
            .await?;

        let body = response.text().await?;
        parse_soap_response(country_code, &body)
    }
}

#[derive(Debug, Deserialize)]
struct ViesRestResponse {
    #[serde(rename = "isValid")]
    is_valid: bool,
    name: Option<String>,
    address: Option<String>,
    #[serde(rename = "countryCode")]
    country_code: Option<String>,
}

#[derive(Clone)]
pub struct AddressParser {
    client: Client,
    mistral_url: String,
    mistral_model: String,
}

impl AddressParser {
    pub fn new(base_url: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build reqwest client for address parser");
        Self {
            client,
            mistral_url: base_url.unwrap_or_else(|| DEFAULT_MISTRAL_API_URL.to_string()),
            mistral_model: DEFAULT_MISTRAL_MODEL.to_string(),
        }
    }

    pub async fn parse(
        &self,
        address: &str,
        api_key: Option<&str>,
        provider: &str,
    ) -> Result<Option<AddressComponents>> {
        if provider.eq_ignore_ascii_case("mistral") {
            if let Some(key) = api_key {
                match self.parse_with_mistral(address, key).await {
                    Ok(Some(ai)) => return Ok(Some(ai)),
                    Ok(None) => {}
                    Err(err) => {
                        tracing::warn!(
                            error = %err,
                            "Mistral address parsing failed, falling back to regex"
                        );
                    }
                }
            }
        }

        Ok(Some(parse_with_regex(address)))
    }

    async fn parse_with_mistral(
        &self,
        address: &str,
        api_key: &str,
    ) -> Result<Option<AddressComponents>> {
        let prompt = format!(
            "You are an expert at parsing Bulgarian company addresses. Extract streetName, city, postalCode, country from: {}. For streetName, include the full location including 'жк' (residential complex), 'ул.' (street), 'бул.' (boulevard), block numbers, entrance, floor, and apartment. Return only valid JSON.",
            address
        );
        let body = json!({
            "model": self.mistral_model,
            "messages": [
                {"role": "system", "content": "Extract Bulgarian address components as JSON with fields streetName, city, postalCode, country. For streetName, capture complete location data including residential complex (жк), block (бл.), entrance (вх.), floor (ет.), and apartment (ап.)."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.1,
            "max_tokens": 200
        });

        let response = self
            .client
            .post(&self.mistral_url)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Mistral API error: {}", response.status()));
        }

        let payload: MistralResponse = response.json().await?;
        let content = payload
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .ok_or_else(|| anyhow!("Empty response from Mistral"))?;

        let cleaned = clean_json_block(&content);
        match serde_json::from_str::<AddressComponents>(&cleaned) {
            Ok(parsed) => Ok(Some(parsed)),
            Err(primary_err) => {
                if let Some(extracted) = extract_json_object(&cleaned) {
                    match serde_json::from_str::<AddressComponents>(&extracted) {
                        Ok(parsed) => {
                            tracing::debug!(
                                "Parsed address after cleaning Mistral response extras"
                            );
                            return Ok(Some(parsed));
                        }
                        Err(secondary_err) => {
                            tracing::warn!(
                                error = %secondary_err,
                                "Failed to parse JSON extracted from Mistral response"
                            );
                        }
                    }
                }

                Err(anyhow!(
                    "Failed to parse Mistral response JSON: {primary_err}"
                ))
            }
        }
    }
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

#[derive(Clone)]
pub struct BulgarianRegistryClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryContragent {
    pub eik: String,
    pub valid: Option<bool>,
    pub company_name: Option<String>,
    pub company_name_bg: Option<String>,
    pub legal_form: Option<String>,
    pub status: Option<String>,
    pub address: Option<String>,
    pub long_address: Option<String>,
    pub street_name: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

impl BulgarianRegistryClient {
    pub fn new(base_url: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build reqwest client for Bulgarian registry");
        Self {
            client,
            base_url: base_url.unwrap_or_else(|| DEFAULT_BULGARIAN_REGISTRY_URL.to_string()),
        }
    }

    pub async fn check_eik(&self, eik: &str) -> Result<RegistryContragent> {
        if !is_valid_eik(eik) {
            return Err(anyhow!("Invalid EIK format"));
        }

        let url = format!("{}/company/{}", self.base_url.trim_end_matches('/'), eik);
        let response = self.client.get(&url).send().await;

        if let Ok(resp) = response {
            if resp.status().is_success() {
                let map: HashMap<String, serde_json::Value> = resp.json().await?;
                return Ok(map_to_registry_contragent(eik, map));
            }
        }

        Ok(mock_registry_response(eik))
    }
}

fn map_to_registry_contragent(
    eik: &str,
    mut map: HashMap<String, serde_json::Value>,
) -> RegistryContragent {
    RegistryContragent {
        eik: eik.to_string(),
        valid: Some(true),
        company_name_bg: map
            .remove("name")
            .and_then(|v| v.as_str().map(|s| s.to_string())),
        company_name: map
            .remove("nameEn")
            .and_then(|v| v.as_str().map(|s| s.to_string())),
        legal_form: map
            .remove("legalForm")
            .and_then(|v| v.as_str().map(|s| s.to_string())),
        status: map
            .remove("status")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .or(Some("Активен".to_string())),
        address: map
            .remove("address")
            .and_then(|v| v.as_str().map(|s| s.to_string())),
        long_address: map
            .remove("fullAddress")
            .and_then(|v| v.as_str().map(|s| s.to_string())),
        street_name: None,
        city: None,
        postal_code: None,
        country: Some("България".to_string()),
    }
}

fn mock_registry_response(eik: &str) -> RegistryContragent {
    let mut fixtures = HashMap::new();
    fixtures.insert(
        "831641256",
        RegistryContragent {
            eik: "831641256".to_string(),
            valid: Some(true),
            company_name_bg: Some("Примерна Компания ЕООД".to_string()),
            company_name: Some("Example Company Ltd".to_string()),
            legal_form: Some("ЕООД".to_string()),
            status: Some("Активен".to_string()),
            address: Some("ул. Витоша 10, София".to_string()),
            long_address: Some("ул. Витоша 10, София 1000, България".to_string()),
            street_name: Some("ул. Витоша 10".to_string()),
            city: Some("София".to_string()),
            postal_code: Some("1000".to_string()),
            country: Some("България".to_string()),
        },
    );
    fixtures.insert(
        "200950556",
        RegistryContragent {
            eik: "200950556".to_string(),
            valid: Some(true),
            company_name_bg: Some("България Сервиз ООД".to_string()),
            company_name: Some("Bulgaria Service OOD".to_string()),
            legal_form: Some("ООД".to_string()),
            status: Some("Активен".to_string()),
            address: Some("ул. Иван Вазов 15, Варна".to_string()),
            long_address: Some("ул. Иван Вазов 15, Варна 9000, България".to_string()),
            street_name: Some("ул. Иван Вазов 15".to_string()),
            city: Some("Варна".to_string()),
            postal_code: Some("9000".to_string()),
            country: Some("България".to_string()),
        },
    );

    fixtures
        .get(eik)
        .cloned()
        .unwrap_or_else(|| RegistryContragent {
            eik: eik.to_string(),
            valid: Some(is_valid_eik(eik)),
            company_name_bg: Some(format!("Фирма с ЕИК {}", eik)),
            company_name: None,
            legal_form: Some("ООД".to_string()),
            status: Some("Активен".to_string()),
            address: None,
            long_address: None,
            street_name: None,
            city: None,
            postal_code: None,
            country: Some("България".to_string()),
        })
}

pub(crate) fn clean_json_block(input: &str) -> String {
    let trimmed = input.trim();
    let without_code_fence = if trimmed.starts_with("```json") {
        trimmed
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim()
    } else if trimmed.starts_with("```") {
        trimmed
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
    } else {
        trimmed
    };

    if let Some(json_block) = extract_json_object(without_code_fence) {
        json_block
    } else {
        without_code_fence.to_string()
    }
}

pub(crate) fn extract_json_object(input: &str) -> Option<String> {
    let mut brace_depth = 0usize;
    let mut start_index: Option<usize> = None;
    let mut in_string = false;
    let mut is_escaped = false;

    for (idx, ch) in input.char_indices() {
        match ch {
            '\\' if in_string => {
                is_escaped = !is_escaped;
                continue;
            }
            '"' if !is_escaped => {
                in_string = !in_string;
            }
            _ => {
                is_escaped = false;
            }
        }

        if in_string {
            continue;
        }

        match ch {
            '{' => {
                if brace_depth == 0 {
                    start_index = Some(idx);
                }
                brace_depth += 1;
            }
            '}' => {
                if brace_depth > 0 {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        if let Some(start) = start_index {
                            return Some(input[start..=idx].to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    None
}

fn parse_with_regex(address: &str) -> AddressComponents {
    let mut components = AddressComponents {
        street_name: None,
        city: None,
        postal_code: None,
        country: None,
    };

    let postal_regex = Regex::new(r"\\b(\\d{4,6})\\b").unwrap();
    if let Some(captures) = postal_regex.captures(address) {
        components.postal_code = captures.get(1).map(|m| m.as_str().to_string());
    }

    // Enhanced regex to handle merged addresses like "жк Дружба-2Обиколна"
    // Match: prefix + space + name (which may include numbers, dashes, letters until space or special chars)
    let street_regex =
        Regex::new(r"(?i)(ул\\.?|улица|бул\\.?|булевард|пл\\.?|площад|жк\\.?|ж\\.к\\.?|жилищен комплекс)\\s*([\\wА-Яа-я0-9.-]+(?:\\s+[\\wА-Яа-я0-9.-]+)*)").unwrap();
    if let Some(captures) = street_regex.captures(address) {
        let prefix = captures.get(1).map(|m| m.as_str()).unwrap_or("");
        let rest = captures.get(2).map(|m| m.as_str()).unwrap_or("").trim();

        // For residential complexes, try to extract full location info
        // Extract everything after the prefix until we hit "обл." or "гр." or comma
        let full_location = if prefix.to_lowercase().contains("жк") || prefix.to_lowercase().contains("жилищен") {
            // Try to capture more context for residential complexes
            let extended_regex = Regex::new(r"(?i)(жк\\.?|ж\\.к\\.?|жилищен комплекс)\\s*([^,]+?)(?=\\s+обл\\.|\\s+гр\\.|,|$)").unwrap();
            if let Some(ext_captures) = extended_regex.captures(address) {
                ext_captures.get(2).map(|m| m.as_str().trim()).unwrap_or(rest)
            } else {
                rest
            }
        } else {
            rest
        };

        components.street_name = Some(format!("{} {}", prefix.trim(), full_location));
    }

    let parts: Vec<&str> = address.split(',').map(|p| p.trim()).collect();
    for part in parts {
        if components.country.is_none() && looks_like_country(part) {
            components.country = Some(part.to_string());
        } else if components.city.is_none() && looks_like_city(part) {
            components.city = Some(part.to_string());
        }
    }

    if components.country.is_none() {
        components.country = Some("България".to_string());
    }

    components
}

fn looks_like_country(part: &str) -> bool {
    let upper = part.to_uppercase();
    upper.contains("БЪЛГ") || upper.contains("BULGARIA") || upper.contains("BG")
}

fn looks_like_city(part: &str) -> bool {
    part.contains("гр.") || part.contains("град") || part.chars().all(|c| !c.is_lowercase())
}

fn split_vat_number(vat_number: &str) -> Result<(String, &str)> {
    if vat_number.len() < 3 {
        return Err(anyhow!("VAT number too short"));
    }
    let country = vat_number[..2].to_uppercase();
    let numeric = &vat_number[2..];
    Ok((country, numeric))
}

fn extract_bulgarian_eik(vat_number: &str) -> Option<String> {
    if vat_number.starts_with("BG") && vat_number.len() > 2 {
        Some(vat_number[2..].to_string())
    } else {
        None
    }
}

fn normalize_vat_number(vat_number: &str) -> Result<String> {
    let trimmed = vat_number.trim().to_uppercase();
    if trimmed.len() < 3 {
        return Err(anyhow!("VAT number must include country code"));
    }
    Ok(trimmed)
}

fn parse_soap_response(country_code: &str, xml: &str) -> Result<ViesResponse> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let mut valid = false;
    let mut name: Option<String> = None;
    let mut address: Option<String> = None;
    let mut current_tag: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(tag)) => {
                current_tag = Some(String::from_utf8_lossy(tag.name().as_ref()).to_string());
            }
            Ok(Event::Text(text)) => {
                if let Some(tag) = &current_tag {
                    match tag.as_str() {
                        "valid" => {
                            valid = text.unescape()?.to_string().parse().unwrap_or(false);
                        }
                        "name" => {
                            name = Some(text.unescape()?.to_string());
                        }
                        "address" => {
                            address = Some(text.unescape()?.to_string());
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::End(_)) => {
                current_tag = None;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("Failed to parse SOAP response: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(ViesResponse {
        valid,
        name,
        address,
        country_code: country_code.to_string(),
        source: ContragentDataSource::ViesSoap,
    })
}

fn has_complete_address(model: &GlobalContragentModel) -> bool {
    model
        .street_name
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
        && model
            .city
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
        && model
            .postal_code
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
        && model
            .country
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
}

fn is_valid_eik(eik: &str) -> bool {
    let clean: String = eik.chars().filter(|c| c.is_ascii_digit()).collect();
    clean.len() == 9 || clean.len() == 13
}

fn build_soap_request(country_code: &str, vat_number: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:urn=\"urn:ec.europa.eu:taxud:vies:services:checkVat\">\
             <soap:Header/>\
             <soap:Body>\
                 <urn:checkVat>\
                     <urn:countryCode>{}</urn:countryCode>\
                     <urn:vatNumber>{}</urn:vatNumber>\
                 </urn:checkVat>\
             </soap:Body>\
         </soap:Envelope>",
        country_code,
        vat_number
    )
}
