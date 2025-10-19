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

    /// Create VAT journal entry from AI-processed invoice
    async fn create_vat_journal_from_ai(
        &self,
        ctx: &Context<'_>,
        input: CreateVatJournalFromAIInput,
    ) -> FieldResult<VatJournalCreationPayload> {
        use crate::entities::journal_entry::{CreateJournalEntryInput, CreateEntryLineInput};
        use sea_orm::{EntityTrait, Set, ColumnTrait, QueryFilter};

        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Validate required fields
        let document_number = input.document_number
            .ok_or("Document number is required")?;
        let document_date = input.document_date
            .ok_or("Document date is required")?;
        let net_amount = input.net_amount
            .ok_or("Net amount is required")?;
        let vat_amount = input.vat_amount
            .ok_or("VAT amount is required")?;
        let total_amount = input.total_amount
            .ok_or("Total amount is required")?;
        let company_id = input.company_id
            .ok_or("Company ID is required")?;
        let counterpart_id = input.counterpart_id
            .ok_or("Counterpart ID is required")?;

        // Determine VAT direction and document type from transaction_type
        let (vat_direction, vat_document_type) = match input.transaction_type.as_deref() {
            Some("PURCHASE") => ("INPUT", "03"), // Покупка
            Some("SALE") => ("OUTPUT", "01"),    // Продажба
            _ => ("OUTPUT", "01"), // Default to sale
        };

        // Use the VAT operation from input if provided, otherwise determine automatically
        let (purchase_operation, sales_operation) = if let Some(vat_op) = input.vat_operation {
            // Use the operation provided from the UI
            if vat_direction == "INPUT" {
                (Some(vat_op), None)
            } else {
                (None, Some(vat_op))
            }
        } else {
            // Fallback to automatic determination based on VAT rate
            if vat_direction == "INPUT" {
                // Purchase - default to full tax credit (пок10)
                (Some("пок10".to_string()), None)
            } else {
                // Sale - determine based on VAT rate
                let vat_rate_percent = if net_amount > Decimal::ZERO {
                    (vat_amount / net_amount * Decimal::from(100)).round()
                } else {
                    Decimal::ZERO
                };

                let sales_op = if vat_rate_percent >= Decimal::from(19) && vat_rate_percent <= Decimal::from(21) {
                    "про11" // 20% VAT
                } else if vat_rate_percent >= Decimal::from(8) && vat_rate_percent <= Decimal::from(10) {
                    "про17" // 9% VAT
                } else if vat_rate_percent == Decimal::ZERO {
                    "про19" // 0% VAT
                } else {
                    "про11" // Default to 20%
                };

                (None, Some(sales_op.to_string()))
            }
        };

        // Load AI accounting settings for the company, create defaults if not found
        use crate::entities::ai_accounting_setting;
        let settings = ai_accounting_setting::Entity::find()
            .filter(ai_accounting_setting::Column::CompanyId.eq(company_id))
            .one(db)
            .await?;

        let settings = match settings {
            Some(s) => s,
            None => {
                // Create default settings for this company
                let default_settings = ai_accounting_setting::ActiveModel {
                    company_id: Set(company_id),
                    sales_revenue_account: Set("701".to_string()),
                    sales_services_account: Set("703".to_string()),
                    sales_receivables_account: Set("411".to_string()),
                    purchase_expense_account: Set("602".to_string()),
                    purchase_payables_account: Set("401".to_string()),
                    vat_input_account: Set("4531".to_string()),
                    vat_output_account: Set("4531".to_string()),
                    non_registered_persons_account: Set(None),
                    non_registered_vat_operation: Set("пок09".to_string()),
                    account_code_length: Set(3),
                    sales_description_template: Set("{counterpart} - {document_number}".to_string()),
                    purchase_description_template: Set("{counterpart} - {document_number}".to_string()),
                    created_at: Set(chrono::Utc::now().naive_utc()),
                    updated_at: Set(chrono::Utc::now().naive_utc()),
                    ..Default::default()
                };

                ai_accounting_setting::Entity::insert(default_settings)
                    .exec_with_returning(db)
                    .await?
            }
        };

        // Determine appropriate accounts based on VAT direction using settings
        let (revenue_expense_account, receivable_payable_account, vat_account, description_template) =
            if vat_direction == "INPUT" {
                // Purchase: Dt 602 (expenses), Dt 4531 (VAT), Ct 401 (payables)
                (
                    settings.format_account_code(&settings.purchase_expense_account),
                    settings.format_account_code(&settings.purchase_payables_account),
                    settings.format_account_code(&settings.vat_input_account),
                    &settings.purchase_description_template,
                )
            } else {
                // Sale: Dt 411 (receivables), Ct 701 (revenue), Ct 4531 (VAT)
                // For sales, use revenue account (701) - can be enhanced later for services
                (
                    settings.format_account_code(&settings.sales_revenue_account),
                    settings.format_account_code(&settings.sales_receivables_account),
                    settings.format_account_code(&settings.vat_output_account),
                    &settings.sales_description_template,
                )
            };

        // Find account IDs
        use crate::entities::account;
        let accounts: Vec<account::Model> = account::Entity::find()
            .filter(account::Column::CompanyId.eq(company_id))
            .filter(account::Column::IsActive.eq(true))
            .all(db)
            .await?;

        let find_account_by_code = |code: &str| -> Result<i32, async_graphql::Error> {
            accounts
                .iter()
                .find(|a| a.code == code)
                .map(|a| a.id)
                .ok_or_else(|| async_graphql::Error::new(format!("Account {} not found", code)))
        };

        let revenue_expense_acc_id = find_account_by_code(&revenue_expense_account)?;
        let receivable_payable_acc_id = find_account_by_code(&receivable_payable_account)?;
        let vat_acc_id = find_account_by_code(&vat_account)?;

        // Get counterpart info for description formatting and VAT status check
        use crate::entities::counterpart;
        let counterpart = counterpart::Entity::find_by_id(counterpart_id)
            .one(db)
            .await?
            .ok_or("Counterpart not found")?;

        // Check if this is a purchase from unregistered VAT person (колона 09)
        let is_unregistered_vat_purchase = vat_direction == "INPUT"
            && !counterpart.is_vat_registered
            && vat_amount == Decimal::ZERO;

        // Override purchase operation if this is unregistered VAT purchase
        let (purchase_operation, sales_operation) = if is_unregistered_vat_purchase {
            // Use the special operation from settings for unregistered persons (колона 09)
            (Some(settings.non_registered_vat_operation.clone()), sales_operation)
        } else {
            (purchase_operation, sales_operation)
        };

        // Format description using settings template
        let formatted_description = settings.format_description(
            description_template,
            &counterpart.name,
            &document_number,
        );

        // Create journal entry lines based on VAT direction
        let lines = if vat_direction == "INPUT" {
            if is_unregistered_vat_purchase {
                // Purchase from unregistered person: Dt 602, Ct 401 (NO VAT line)
                vec![
                    CreateEntryLineInput {
                        account_id: revenue_expense_acc_id,
                        debit_amount: Some(total_amount), // Full amount without VAT
                        credit_amount: None,
                        counterpart_id: Some(counterpart_id),
                        description: Some(formatted_description.clone()),
                        currency_code: Some(input.currency.clone().unwrap_or("BGN".to_string())),
                        currency_amount: None,
                        exchange_rate: None,
                        vat_amount: None,
                        quantity: None,
                        unit_of_measure_code: None,
                        line_order: Some(1),
                    },
                    CreateEntryLineInput {
                        account_id: receivable_payable_acc_id,
                        debit_amount: None,
                        credit_amount: Some(total_amount),
                        counterpart_id: Some(counterpart_id),
                        description: Some(formatted_description.clone()),
                        currency_code: Some(input.currency.clone().unwrap_or("BGN".to_string())),
                        currency_amount: None,
                        exchange_rate: None,
                        vat_amount: None,
                        quantity: None,
                        unit_of_measure_code: None,
                        line_order: Some(2),
                    },
                ]
            } else {
                // Standard purchase: Dt 602 + Dt 4531, Ct 401
            vec![
                CreateEntryLineInput {
                    account_id: revenue_expense_acc_id,
                    debit_amount: Some(net_amount),
                    credit_amount: None,
                    counterpart_id: Some(counterpart_id),
                    description: Some(formatted_description.clone()),
                    currency_code: Some(input.currency.clone().unwrap_or("BGN".to_string())),
                    currency_amount: None,
                    exchange_rate: None,
                    vat_amount: None,
                    quantity: None,
                    unit_of_measure_code: None,
                    line_order: Some(1),
                },
                CreateEntryLineInput {
                    account_id: vat_acc_id,
                    debit_amount: Some(vat_amount),
                    credit_amount: None,
                    counterpart_id: Some(counterpart_id),
                    description: Some("ДДС".to_string()),
                    currency_code: Some(input.currency.clone().unwrap_or("BGN".to_string())),
                    currency_amount: None,
                    exchange_rate: None,
                    vat_amount: Some(vat_amount),
                    quantity: None,
                    unit_of_measure_code: None,
                    line_order: Some(2),
                },
                CreateEntryLineInput {
                    account_id: receivable_payable_acc_id,
                    debit_amount: None,
                    credit_amount: Some(total_amount),
                    counterpart_id: Some(counterpart_id),
                    description: Some(formatted_description.clone()),
                    currency_code: Some(input.currency.clone().unwrap_or("BGN".to_string())),
                    currency_amount: None,
                    exchange_rate: None,
                    vat_amount: None,
                    quantity: None,
                    unit_of_measure_code: None,
                    line_order: Some(3),
                },
            ]
            }
        } else {
            // Sale: Dt 411, Ct 701 + Ct 4531
            vec![
                CreateEntryLineInput {
                    account_id: receivable_payable_acc_id,
                    debit_amount: Some(total_amount),
                    credit_amount: None,
                    counterpart_id: Some(counterpart_id),
                    description: Some(formatted_description.clone()),
                    currency_code: Some(input.currency.clone().unwrap_or("BGN".to_string())),
                    currency_amount: None,
                    exchange_rate: None,
                    vat_amount: None,
                    quantity: None,
                    unit_of_measure_code: None,
                    line_order: Some(1),
                },
                CreateEntryLineInput {
                    account_id: revenue_expense_acc_id,
                    debit_amount: None,
                    credit_amount: Some(net_amount),
                    counterpart_id: Some(counterpart_id),
                    description: Some(formatted_description.clone()),
                    currency_code: Some(input.currency.clone().unwrap_or("BGN".to_string())),
                    currency_amount: None,
                    exchange_rate: None,
                    vat_amount: None,
                    quantity: None,
                    unit_of_measure_code: None,
                    line_order: Some(2),
                },
                CreateEntryLineInput {
                    account_id: vat_acc_id,
                    debit_amount: None,
                    credit_amount: Some(vat_amount),
                    counterpart_id: Some(counterpart_id),
                    description: Some("ДДС".to_string()),
                    currency_code: Some(input.currency.clone().unwrap_or("BGN".to_string())),
                    currency_amount: None,
                    exchange_rate: None,
                    vat_amount: Some(vat_amount),
                    quantity: None,
                    unit_of_measure_code: None,
                    line_order: Some(3),
                },
            ]
        };

        // Create the journal entry using existing infrastructure
        let journal_input = CreateJournalEntryInput {
            entry_number: None, // Will be auto-generated
            document_date,
            vat_date: Some(input.vat_date.unwrap_or(document_date)),
            accounting_date: input.accounting_date.unwrap_or(document_date),
            document_number: Some(document_number.clone()),
            description: input.description.unwrap_or_else(||
                format!("AI Import: {}", document_number)
            ),
            company_id,
            lines,
            vat_document_type: Some(vat_document_type.to_string()),
            vat_purchase_operation: purchase_operation,
            vat_sales_operation: sales_operation,
            vat_additional_operation: None,
            vat_additional_data: None,
        };

        // Use existing create_journal_entry logic
        use crate::entities::{journal_entry as je, entry_line};

        let mut entry_model = je::ActiveModel::from(journal_input.clone());
        entry_model.created_by = Set(1); // TODO: Get from auth context
        entry_model.total_amount = Set(total_amount);
        entry_model.total_vat_amount = Set(vat_amount);

        let entry = je::Entity::insert(entry_model)
            .exec_with_returning(db)
            .await?;

        // Create entry lines
        for (index, line_input) in journal_input.lines.into_iter().enumerate() {
            let line_input_proper = crate::entities::entry_line::CreateEntryLineInput {
                account_id: line_input.account_id,
                debit_amount: line_input.debit_amount,
                credit_amount: line_input.credit_amount,
                description: line_input.description,
                counterpart_id: line_input.counterpart_id,
                vat_rate_id: None,
                vat_amount: line_input.vat_amount,
                currency_code: line_input.currency_code,
                currency_amount: line_input.currency_amount,
                exchange_rate: line_input.exchange_rate,
                quantity: line_input.quantity,
                unit_of_measure_code: line_input.unit_of_measure_code,
                line_order: Some((index + 1) as i32),
            };
            let mut line_model = entry_line::ActiveModel::from(line_input_proper);
            line_model.journal_entry_id = Set(entry.id);
            entry_line::Entity::insert(line_model).exec(db).await?;
        }

        Ok(VatJournalCreationPayload {
            success: true,
            journal_entry_id: entry.id,
            entry_number: entry.entry_number,
            message: Some(format!(
                "ДДС операция създадена успешно: {} ({})",
                document_number,
                if vat_direction == "INPUT" { "Покупка" } else { "Продажба" }
            )),
        })
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

#[derive(InputObject)]
pub struct CreateVatJournalFromAIInput {
    pub company_id: Option<i32>,
    pub counterpart_id: Option<i32>,
    pub transaction_type: Option<String>, // "PURCHASE" or "SALE"
    pub document_number: Option<String>,
    pub document_date: Option<NaiveDate>,
    pub vat_date: Option<NaiveDate>,
    pub accounting_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub currency: Option<String>,
    pub net_amount: Option<Decimal>,
    pub vat_amount: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub vat_operation: Option<String>, // Selected VAT operation from UI (e.g., "пок10", "про11")
}

#[derive(SimpleObject)]
pub struct VatJournalCreationPayload {
    pub success: bool,
    pub journal_entry_id: i32,
    pub entry_number: String,
    pub message: Option<String>,
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
