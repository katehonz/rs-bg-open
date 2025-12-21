use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// SAF-T file type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafTFileType {
    Annual,
    Monthly,
    OnDemand,
}

/// Main SAF-T structure for Bulgaria v1.0.1
#[derive(Debug, Serialize, Deserialize)]
pub struct BulgarianSafT {
    pub header: SafTHeader,
    pub file_type: SafTFileType,
    // Annual variant
    pub master_files_annual: Option<MasterFilesAnnual>,
    pub source_documents_annual: Option<SourceDocumentsAnnual>,
    // Monthly variant
    pub master_files_monthly: Option<MasterFilesMonthly>,
    pub corresponding_accounts_report: Option<CorrespondingAccountsReport>,
    pub general_ledger_entries: Option<GeneralLedgerEntries>,
    pub source_documents_monthly: Option<SourceDocumentsMonthly>,
    // OnDemand variant
    pub master_files_on_demand: Option<MasterFilesOnDemand>,
    pub source_documents_on_demand: Option<SourceDocumentsOnDemand>,
}

/// SAF-T Header according to Bulgarian requirements v1.0.1
#[derive(Debug, Serialize, Deserialize)]
pub struct SafTHeader {
    pub audit_file_version: String,
    pub audit_file_country: String,        // "BG"
    pub audit_file_region: Option<String>, // BG-01 to BG-28
    pub audit_file_date_created: NaiveDate,
    pub software_company_name: String,
    pub software_id: String,
    pub software_version: String,
    pub company: CompanyInfo,
    pub ownership: Option<Ownership>,
    pub default_currency_code: String, // "BGN" or "EUR"
    pub selection_criteria: SelectionCriteria,
    pub header_comment: String, // "A" for Annual, "M" for Monthly, "O" for OnDemand
    pub segment_index: Option<i32>,
    pub total_segments_in_sequence: Option<i32>,
    pub tax_accounting_basis: String, // "A", "P", "BANK", "INSURANCE"
    pub tax_entity: Option<String>,
}

/// Company information for SAF-T header v1.0.1
#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyInfo {
    pub registration_number: String, // EIK
    pub name: String,
    pub address: Address,
    pub contact: Contact,
    pub tax_registration: Vec<TaxRegistration>,
    pub bank_account: Vec<BankAccount>,
}

/// Address structure v1.0.1
#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    pub street_name: Option<String>,
    pub number: Option<String>,
    pub additional_address_detail: Option<String>,
    pub building: Option<String>,
    pub city: String,
    pub postal_code: Option<String>,
    pub region: Option<String>,
    pub country: String,      // ISO 3166-1 alpha-2
    pub address_type: String, // "StreetAddress"
}

/// Contact structure v1.0.1
#[derive(Debug, Serialize, Deserialize)]
pub struct Contact {
    pub contact_person: Option<ContactPerson>,
    pub telephone: Option<String>,
    pub fax: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
}

/// Contact person structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ContactPerson {
    pub title: Option<String>,
    pub first_name: Option<String>,
    pub initials: Option<String>,
    pub last_name_prefix: Option<String>,
    pub last_name: Option<String>,
    pub birth_name: Option<String>,
    pub salutation: Option<String>,
    pub other_titles: Vec<String>,
}

/// Tax Registration
#[derive(Debug, Serialize, Deserialize)]
pub struct TaxRegistration {
    pub tax_registration_number: String, // EIK
    pub tax_type: String,                // Tax type code
    pub tax_number: String,              // VAT number
    pub tax_verification_date: Option<NaiveDate>,
}

/// Bank Account
#[derive(Debug, Serialize, Deserialize)]
pub struct BankAccount {
    pub iban_number: String,
    pub bank_name: Option<String>,
    pub bank_branch: Option<String>,
    pub sort_code: Option<String>,
}

/// Ownership structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Ownership {
    pub is_part_of_group: String, // "0" or "1"
    pub beneficial_owner_name_cyrillic_bg: Option<String>,
    pub beneficial_owner_egn: Option<String>,
    pub beneficial_owner_name_latin_foreign: Option<String>,
    pub beneficial_country_foreign: Option<String>,
    pub beneficial_country_foreign_code: Option<String>,
    pub ultimate_owner_name_cyrillic_bg: Option<String>,
    pub ultimate_owner_uic_bg: Option<String>,
    pub ultimate_owner_name_cyrillic_foreign: Option<String>,
    pub ultimate_owner_name_latin_foreign: Option<String>,
    pub country_foreign: Option<String>,
}

/// Selection criteria for the SAF-T file v1.0.1
#[derive(Debug, Serialize, Deserialize)]
pub struct SelectionCriteria {
    pub tax_reporting_jurisdiction: String, // "NRA"
    pub company_entity: Option<String>,
    pub period_start: i32, // Month number 1-12
    pub period_start_year: i32,
    pub period_end: i32, // Month number 1-12
    pub period_end_year: i32,
    pub document_type: Option<String>,
    pub other_criteria: Option<String>,
}

/// Master files for Annual report
#[derive(Debug, Serialize, Deserialize)]
pub struct MasterFilesAnnual {
    pub owners: Option<Owners>,
    pub assets: Assets,
}

/// Master files for Monthly report
#[derive(Debug, Serialize, Deserialize)]
pub struct MasterFilesMonthly {
    pub general_ledger_accounts: GeneralLedgerAccounts,
    pub taxonomies: Option<Taxonomies>,
    pub customers: Customers,
    pub suppliers: Suppliers,
    pub tax_table: TaxTable,
    pub uom_table: UOMTable,
    pub analysis_type_table: Option<AnalysisTypeTable>,
    pub products: Products,
    pub owners: Option<Owners>,
}

/// Master files for OnDemand report
#[derive(Debug, Serialize, Deserialize)]
pub struct MasterFilesOnDemand {
    pub tax_table: Option<TaxTable>,
    pub movement_type_table: MovementTypeTable,
    pub uom_table: UOMTable,
    pub products: Products,
    pub physical_stock: PhysicalStock,
    pub owners: Option<Owners>,
}

/// General Ledger Accounts container
#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralLedgerAccounts {
    pub account: Vec<SafTAccount>,
}

/// Customers container
#[derive(Debug, Serialize, Deserialize)]
pub struct Customers {
    pub customer: Vec<SafTCustomer>,
}

/// Suppliers container
#[derive(Debug, Serialize, Deserialize)]
pub struct Suppliers {
    pub supplier: Vec<SafTSupplier>,
}

/// Tax Table container
#[derive(Debug, Serialize, Deserialize)]
pub struct TaxTable {
    pub tax_code_details: Vec<SafTTaxCodeDetails>,
}

/// UOM Table container
#[derive(Debug, Serialize, Deserialize)]
pub struct UOMTable {
    pub uom_entry: Vec<SafTUomEntry>,
}

/// Analysis Type Table container
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisTypeTable {
    pub analysis_type_entry: Vec<SafTAnalysisType>,
}

/// Movement Type Table container
#[derive(Debug, Serialize, Deserialize)]
pub struct MovementTypeTable {
    pub movement_type_entry: Vec<SafTMovementType>,
}

/// Products container
#[derive(Debug, Serialize, Deserialize)]
pub struct Products {
    pub product: Vec<SafTProduct>,
}

/// Owners placeholder
#[derive(Debug, Serialize, Deserialize)]
pub struct Owners {
    // Placeholder for owners data
}

/// Assets placeholder
#[derive(Debug, Serialize, Deserialize)]
pub struct Assets {
    // Placeholder for assets data
}

/// Taxonomies placeholder
#[derive(Debug, Serialize, Deserialize)]
pub struct Taxonomies {
    // Placeholder for taxonomies data
}

/// Physical Stock placeholder
#[derive(Debug, Serialize, Deserialize)]
pub struct PhysicalStock {
    // Placeholder for physical stock data
}

/// SAF-T Account v1.0.1
#[derive(Debug, Serialize, Deserialize)]
pub struct SafTAccount {
    pub account_id: i32, // Numeric account code
    pub account_description: String,
    pub taxpayer_account_id: String, // Taxpayer's own account code
    pub grouping_category: Option<String>,
    pub grouping_code: Option<String>,
    pub account_type: String, // "Bifunctional", "Debit", "Credit"
    pub account_creation_date: Option<NaiveDate>,
    pub opening_debit_balance: Option<Decimal>,
    pub opening_credit_balance: Option<Decimal>,
    pub closing_debit_balance: Option<Decimal>,
    pub closing_credit_balance: Option<Decimal>,
}

/// SAF-T Customer v1.0.1
#[derive(Debug, Serialize, Deserialize)]
pub struct SafTCustomer {
    pub customer_id: String,
    pub customer_name: String,
    pub taxpayer_customer_id: Option<String>,
    pub addresses: Vec<CustomerAddress>,
    pub contact: Option<Contact>,
    pub tax_registration: Option<TaxRegistration>,
    pub bank_account: Vec<BankAccount>,
    pub self_billing_indicator: String, // "0" or "1"
    pub account_id: Option<i32>,
    pub opening_balance: Option<OpeningCloseBalance>,
    pub closing_balance: Option<OpeningCloseBalance>,
    pub party_info: Option<PartyInfo>,
}

/// Customer Address
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerAddress {
    pub address: Address,
}

/// SAF-T Supplier v1.0.1
#[derive(Debug, Serialize, Deserialize)]
pub struct SafTSupplier {
    pub supplier_id: String,
    pub supplier_name: String,
    pub taxpayer_supplier_id: Option<String>,
    pub addresses: Vec<SupplierAddress>,
    pub contact: Option<Contact>,
    pub tax_registration: Option<TaxRegistration>,
    pub bank_account: Vec<BankAccount>,
    pub self_billing_indicator: String, // "0" or "1"
    pub account_id: Option<i32>,
    pub opening_balance: Option<OpeningCloseBalance>,
    pub closing_balance: Option<OpeningCloseBalance>,
    pub party_info: Option<PartyInfo>,
}

/// Supplier Address
#[derive(Debug, Serialize, Deserialize)]
pub struct SupplierAddress {
    pub address: Address,
}

/// Opening/Closing Balance
#[derive(Debug, Serialize, Deserialize)]
pub struct OpeningCloseBalance {
    pub debit: Option<Decimal>,
    pub credit: Option<Decimal>,
}

/// Party Info
#[derive(Debug, Serialize, Deserialize)]
pub struct PartyInfo {
    pub payment_terms: Option<String>,
    pub nace_code: Option<String>,
}

/// SAF-T Tax Code Details v1.0.1
#[derive(Debug, Serialize, Deserialize)]
pub struct SafTTaxCodeDetails {
    pub tax_code: String,
    pub effective_date: NaiveDate,
    pub expiration_date: Option<NaiveDate>,
    pub description: String,
    pub tax_percentage: Option<Decimal>,
    pub flat_tax_rate: Option<Decimal>,
    pub country: String, // "BG"
    pub region: Option<String>,
    pub standard_tax_code: Option<String>,
    pub compensation: Option<String>,
}

/// SAF-T Unit of Measure
#[derive(Debug, Serialize, Deserialize)]
pub struct SafTUomEntry {
    pub unit_of_measure: String,
    pub uom_description: String,
    pub uom_to_uom_base: Decimal,
}

/// SAF-T Analysis Type (for analytical accounts)
#[derive(Debug, Serialize, Deserialize)]
pub struct SafTAnalysisType {
    pub analysis_type: String,
    pub analysis_type_description: String,
    pub analysis_id: String,
    pub analysis_id_description: String,
}

/// SAF-T Movement Type
#[derive(Debug, Serialize, Deserialize)]
pub struct SafTMovementType {
    pub movement_type: String,
    pub description: String,
}

/// SAF-T Product (for inventory tracking)
#[derive(Debug, Serialize, Deserialize)]
pub struct SafTProduct {
    pub product_type: String, // "P" for Product, "S" for Service, "O" for Other
    pub product_code: String,
    pub product_group: Option<String>,
    pub description: String,
    pub product_number_code: Option<String>,
    pub products_category: Option<String>,
    pub bar_code: Option<String>,
    pub products_other_category: Option<String>,
}

/// General Ledger Entries v1.0.1
#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralLedgerEntries {
    pub number_of_entries: i32,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub journal: Vec<SafTJournal>,
}

/// Corresponding Accounts Report
#[derive(Debug, Serialize, Deserialize)]
pub struct CorrespondingAccountsReport {
    pub corresponding_account_report: Vec<CorrespondingAccountReportEntry>,
}

/// Corresponding Account Report Entry
#[derive(Debug, Serialize, Deserialize)]
pub struct CorrespondingAccountReportEntry {
    pub line_no: i32,
    pub period: i32,
    pub period_year: i32,
    pub account_id: i32,
    pub debit_opening_balance: Decimal,
    pub credit_opening_balance: Decimal,
    pub corresponding_accounts: Vec<CorrespondingAccount>,
    pub total_debit_closing_balance: Decimal,
    pub total_credit_closing_balance: Decimal,
}

/// Corresponding Account
#[derive(Debug, Serialize, Deserialize)]
pub struct CorrespondingAccount {
    pub corresponding_account_id: i32,
    pub debit_turnover: Decimal,
    pub credit_turnover: Decimal,
}

/// SAF-T Journal
#[derive(Debug, Serialize, Deserialize)]
pub struct SafTJournal {
    pub journal_id: String,
    pub description: String,
    pub journal_type: String, // Type of journal
    pub transaction: Vec<SafTTransaction>,
}

/// SAF-T Transaction v1.0.1
#[derive(Debug, Serialize, Deserialize)]
pub struct SafTTransaction {
    pub transaction_id: String,
    pub period: i32,
    pub period_year: i32,
    pub transaction_date: NaiveDate,
    pub source_id: Option<String>,
    pub description: String,
    pub doc_archival_number: Option<String>,
    pub transaction_type: Option<String>,
    pub system_entry_date: NaiveDate,
    pub gl_posting_date: NaiveDate,
    pub customer_id: Option<String>,
    pub supplier_id: Option<String>,
    pub system_id: Option<String>,
    pub lines: Lines,
}

/// Transaction Lines container
#[derive(Debug, Serialize, Deserialize)]
pub struct Lines {
    pub debit_line: Vec<SafTTransactionLine>,
    pub credit_line: Vec<SafTTransactionLine>,
}

/// SAF-T Transaction Line v1.0.1
#[derive(Debug, Serialize, Deserialize)]
pub struct SafTTransactionLine {
    pub record_id: String,
    pub account_id: i32,
    pub taxpayer_account_id: Option<String>,
    pub analysis: Option<Vec<SafTAnalysis>>,
    pub value_date: Option<NaiveDate>,
    pub source_document_id: Option<String>,
    pub customer_id: Option<String>,
    pub supplier_id: Option<String>,
    pub description: String,
    pub debit_amount: Option<AmountStructure>,
    pub credit_amount: Option<AmountStructure>,
    pub tax_information: Option<Vec<SafTTaxInformation>>,
    pub reference_number: Option<String>,
    pub cid: Option<String>,
    pub quantity: Option<Decimal>,
    pub cross_reference: Option<String>,
    pub system_entry_time: Option<String>,
}

/// Amount Structure
#[derive(Debug, Serialize, Deserialize)]
pub struct AmountStructure {
    pub amount: Decimal,
    pub currency_code: Option<String>,
    pub currency_amount: Option<Decimal>,
    pub exchange_rate: Option<Decimal>,
}

/// SAF-T Analysis (for analytical accounts)
#[derive(Debug, Serialize, Deserialize)]
pub struct SafTAnalysis {
    pub analysis_type: String,
    pub analysis_id: String,
    pub analysis_amount: Option<Decimal>,
}

/// SAF-T Tax Information
#[derive(Debug, Serialize, Deserialize)]
pub struct SafTTaxInformation {
    pub tax_type: String,
    pub tax_code: String,
    pub tax_percentage: Decimal,
    pub tax_base: Decimal,
    pub tax_amount: Decimal,
    pub tax_exemption_reason: Option<String>,
    pub tax_country_region: String, // "BG"
}

/// Source Documents Annual placeholder
#[derive(Debug, Serialize, Deserialize)]
pub struct SourceDocumentsAnnual {
    // Placeholder - specific implementation depends on requirements
}

/// Source Documents Monthly
#[derive(Debug, Serialize, Deserialize)]
pub struct SourceDocumentsMonthly {
    pub sales_invoices: Option<SalesInvoices>,
    pub purchase_invoices: Option<PurchaseInvoices>,
    pub payments: Option<Payments>,
    pub movement_of_goods: Option<MovementOfGoods>,
    pub asset_transactions: Option<AssetTransactions>,
}

/// Source Documents OnDemand
#[derive(Debug, Serialize, Deserialize)]
pub struct SourceDocumentsOnDemand {
    pub sales_invoices: Option<SalesInvoices>,
    pub purchase_invoices: Option<PurchaseInvoices>,
    pub movement_of_goods: Option<MovementOfGoods>,
}

/// Sales Invoices
#[derive(Debug, Serialize, Deserialize)]
pub struct SalesInvoices {
    pub number_of_entries: i32,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub invoice: Vec<SalesInvoice>,
}

/// Sales Invoice
#[derive(Debug, Serialize, Deserialize)]
pub struct SalesInvoice {
    pub invoice_no: String,
    pub customer_info: CustomerInfo,
    pub invoice_date: NaiveDate,
    pub invoice_type: String,
    pub self_billing_indicator: String,
    pub currency_code: Option<String>,
    pub exchange_rate: Option<Decimal>,
    pub lines: Vec<InvoiceLine>,
    pub document_totals: DocumentTotals,
}

/// Customer Info
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerInfo {
    pub customer_id: String,
    pub billing_address: Option<Address>,
}

/// Invoice Line
#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceLine {
    pub line_number: String,
    pub product_code: Option<String>,
    pub product_description: Option<String>,
    pub quantity: Option<Decimal>,
    pub unit_of_measure: Option<String>,
    pub unit_price: Option<Decimal>,
    pub tax_base: Option<Decimal>,
    pub tax_point_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub debit_credit_indicator: String, // "D" or "C"
    pub line_amount: AmountStructure,
    pub tax: Option<Tax>,
}

/// Document Totals
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentTotals {
    pub tax_payable: Decimal,
    pub net_total: Decimal,
    pub gross_total: Decimal,
}

/// Tax
#[derive(Debug, Serialize, Deserialize)]
pub struct Tax {
    pub tax_type: String,
    pub tax_country_region: String,
    pub tax_code: String,
    pub tax_percentage: Option<Decimal>,
    pub tax_amount: Option<AmountStructure>,
}

/// Purchase Invoices (similar to Sales Invoices)
#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseInvoices {
    pub number_of_entries: i32,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub invoice: Vec<PurchaseInvoice>,
}

/// Purchase Invoice
#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseInvoice {
    pub invoice_no: String,
    pub supplier_info: SupplierInfo,
    pub invoice_date: NaiveDate,
    pub invoice_type: String,
    pub self_billing_indicator: String,
    pub currency_code: Option<String>,
    pub exchange_rate: Option<Decimal>,
    pub lines: Vec<InvoiceLine>,
    pub document_totals: DocumentTotals,
}

/// Supplier Info
#[derive(Debug, Serialize, Deserialize)]
pub struct SupplierInfo {
    pub supplier_id: String,
    pub billing_address: Option<Address>,
}

/// Payments placeholder
#[derive(Debug, Serialize, Deserialize)]
pub struct Payments {
    // Placeholder
}

/// Movement of Goods placeholder
#[derive(Debug, Serialize, Deserialize)]
pub struct MovementOfGoods {
    // Placeholder
}

/// Asset Transactions placeholder
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetTransactions {
    // Placeholder
}

/// Input structures for SAF-T generation
#[derive(Debug, Deserialize)]
pub struct SafTExportRequest {
    pub company_id: i32,
    pub period_start: i32, // Month 1-12
    pub period_start_year: i32,
    pub period_end: i32, // Month 1-12
    pub period_end_year: i32,
    pub file_type: SafTFileType,      // Annual, Monthly, OnDemand
    pub tax_accounting_basis: String, // "A", "P", "BANK", "INSURANCE"
}
