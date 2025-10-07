use std::sync::Arc;

use async_graphql::{Context, FieldResult, InputObject, Object, SimpleObject};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::DatabaseConnection;

use crate::entities::GlobalContragentModel;
use crate::services::invoice_processing::{
    InvoiceDocument, InvoiceProcessingService, ParsedCounterpart, ParsedInvoice, ParsedInvoiceItem,
    ProcessedInvoice,
};

use super::contragent_resolvers::ContragentSource;

#[derive(Default)]
pub struct InvoiceMutation;

#[Object]
impl InvoiceMutation {
    async fn process_invoice(
        &self,
        ctx: &Context<'_>,
        input: ProcessInvoiceInput,
    ) -> FieldResult<InvoiceProcessingPayload> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let service = ctx.data::<Arc<InvoiceProcessingService>>()?;

        let file_bytes = decode_document(&input.file_base64)?;

        let document = InvoiceDocument {
            company_id: input.company_id,
            file_name: input.file_name.clone(),
            content_type: input.content_type.clone(),
            file_bytes,
        };

        let result = service
            .process_document(db.as_ref(), document)
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(InvoiceProcessingPayload::from(result))
    }
}

#[derive(InputObject)]
pub struct ProcessInvoiceInput {
    pub company_id: Option<i64>,
    pub file_name: String,
    pub content_type: Option<String>,
    /// Base64 съдържание на файла. Поддържа се и data URI (`data:<mime>;base64,....`).
    pub file_base64: String,
}

#[derive(SimpleObject)]
pub struct InvoiceProcessingPayload {
    pub company_id: Option<i64>,
    pub requires_manual_review: bool,
    pub document: InvoiceDocumentOutput,
    pub contragent: Option<GlobalContragentModel>,
    pub validation_source: Option<ContragentSource>,
    pub existed_in_database: Option<bool>,
}

impl From<ProcessedInvoice> for InvoiceProcessingPayload {
    fn from(value: ProcessedInvoice) -> Self {
        Self {
            company_id: value.company_id,
            requires_manual_review: value.requires_manual_review,
            document: InvoiceDocumentOutput::from(&value.extracted),
            contragent: value.validated_contragent,
            validation_source: value.validation_source.map(ContragentSource::from),
            existed_in_database: value.existed_in_database,
        }
    }
}

#[derive(SimpleObject)]
pub struct InvoiceDocumentOutput {
    pub document_type: Option<String>,
    pub transaction_type: Option<String>,
    pub document_number: Option<String>,
    pub document_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub currency: Option<String>,
    pub net_amount: Option<Decimal>,
    pub vat_amount: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub counterpart: Option<InvoiceCounterpartOutput>,
    pub items: Vec<InvoiceItemOutput>,
}

impl From<&ParsedInvoice> for InvoiceDocumentOutput {
    fn from(value: &ParsedInvoice) -> Self {
        Self {
            document_type: value.document_type.clone(),
            transaction_type: value.transaction_type.clone(),
            document_number: value.document_number.clone(),
            document_date: value.document_date,
            due_date: value.due_date,
            currency: value.currency.clone(),
            net_amount: value.net_amount,
            vat_amount: value.vat_amount,
            total_amount: value.total_amount,
            counterpart: value
                .counterpart
                .as_ref()
                .map(InvoiceCounterpartOutput::from),
            items: value.items.iter().map(InvoiceItemOutput::from).collect(),
        }
    }
}

#[derive(SimpleObject)]
pub struct InvoiceCounterpartOutput {
    pub name: Option<String>,
    pub eik: Option<String>,
    pub vat_number: Option<String>,
    pub address: Option<String>,
}

impl From<&ParsedCounterpart> for InvoiceCounterpartOutput {
    fn from(value: &ParsedCounterpart) -> Self {
        Self {
            name: value.name.clone(),
            eik: value.eik.clone(),
            vat_number: value.vat_number.clone(),
            address: value.address.clone(),
        }
    }
}

#[derive(SimpleObject)]
pub struct InvoiceItemOutput {
    pub description: Option<String>,
    pub quantity: Option<Decimal>,
    pub unit: Option<String>,
    pub unit_price: Option<Decimal>,
    pub total_price: Option<Decimal>,
    pub vat_rate: Option<Decimal>,
}

impl From<&ParsedInvoiceItem> for InvoiceItemOutput {
    fn from(value: &ParsedInvoiceItem) -> Self {
        Self {
            description: value.description.clone(),
            quantity: value.quantity,
            unit: value.unit.clone(),
            unit_price: value.unit_price,
            total_price: value.total_price,
            vat_rate: value.vat_rate,
        }
    }
}

fn decode_document(encoded: &str) -> FieldResult<Vec<u8>> {
    let trimmed = encoded.trim();
    let data = if let Some((_, data)) = trimmed.split_once(",") {
        data
    } else {
        trimmed
    };

    BASE64
        .decode(data.trim())
        .map_err(|err| async_graphql::Error::new(format!("Невалидно base64 съдържание: {}", err)))
}
