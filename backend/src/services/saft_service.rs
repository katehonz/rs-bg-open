use chrono::{DateTime, Datelike, NaiveDate, Utc};
use quick_xml::{events::Event, Writer};
use rust_decimal::Decimal;
use sea_orm::*;
use std::collections::HashMap;
use std::io::Cursor;

use crate::entities::{
    account::Entity as AccountEntity, company::Entity as CompanyEntity,
    counterpart::Entity as CounterpartEntity, entry_line::Entity as EntryLineEntity,
    journal_entry::Entity as JournalEntryEntity, saft::*,
};

#[allow(dead_code)]
pub struct SafTService {
    db: DatabaseConnection,
}

impl SafTService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Generate SAF-T file for a given company and period (DEPRECATED - use SafTServiceV2)
    #[deprecated(note = "Use SafTServiceV2 for new SAF-T v1.0.1 format")]
    pub async fn generate_saft(
        &self,
        _request: SafTExportRequest,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // This method is deprecated. Use SafTServiceV2 instead.
        Err("This SAF-T service version is deprecated. Use SafTServiceV2 for v1.0.1 format".into())
    }

    // Commented out old implementation to avoid compilation errors
    /*
    // OLD IMPLEMENTATION COMMENTED OUT - USE SafTServiceV2 INSTEAD
    pub async fn generate_saft_old(&self, request: SafTExportRequest) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // 1. Get company information
        let company = CompanyEntity::find_by_id(request.company_id)
            .one(&self.db)
            .await?
            .ok_or("Company not found")?;

        // 2. Collect all necessary data
        let header = self.build_header(&company, &request).await?;
        let master_files = self.build_master_files(&request).await?;
        let general_ledger = self.build_general_ledger(&request).await?;

        // Optional sections based on request
        let accounts_payable = if request.include_accounts_payable {
            Some(self.build_accounts_payable(&request).await?)
        } else {
            None
        };

        let accounts_receivable = if request.include_accounts_receivable {
            Some(self.build_accounts_receivable(&request).await?)
        } else {
            None
        };

        let fixed_assets = if request.include_fixed_assets {
            Some(self.build_fixed_assets(&request).await?)
        } else {
            None
        };

        // 3. Build complete SAF-T structure
        let saft = BulgarianSafT {
            header,
            master_files,
            general_ledger,
            accounts_payable,
            accounts_receivable,
            fixed_assets,
        };

        // 4. Generate XML
        self.generate_xml(&saft)
    }

    async fn build_header(&self, company: &crate::entities::company::Model, request: &SafTExportRequest) -> Result<SafTHeader, Box<dyn std::error::Error + Send + Sync>> {
        Ok(SafTHeader {
            audit_file_version: "2.0".to_string(),
            audit_file_country: "BG".to_string(),
            audit_file_date_created: Utc::now(),
            software_company_name: "RS Accounting BG".to_string(),
            software_id: "RS-AC-BG".to_string(),
            software_version: "1.0.0".to_string(),
            company: CompanyInfo {
                registration_number: company.eik.clone(),
                name: company.name.clone(),
                address: CompanyAddress {
                    building_number: None,
                    street_name: company.address.clone(),
                    address_detail: None,
                    city: company.city.clone().unwrap_or("Unknown".to_string()),
                    postal_code: None,
                    region: None,
                    country: "BG".to_string(),
                },
                contact: CompanyContact {
                    contact_person: company.contact_person.clone(),
                    telephone: company.phone.clone(),
                    fax: None,
                    email: company.email.clone(),
                    website: None,
                },
                tax_registration_number: company.vat_number.clone().unwrap_or_default(),
                tax_accounting_basis: "A".to_string(), // Accrual basis
                company_id: company.id.to_string(),
            },
            default_currency_code: "BGN".to_string(),
            selection_criteria: SelectionCriteria {
                selection_start_date: request.start_date,
                selection_end_date: request.end_date,
                period_start: request.start_date,
                period_end: request.end_date,
                period_type: request.period_type.clone(),
            },
            tax_entity: company.eik.clone(),
            file_content_type: request.file_content_type.clone(),
            number_of_parts: 1,
            part_number: 1,
        })
    }

    async fn build_master_files(&self, request: &SafTExportRequest) -> Result<MasterFiles, Box<dyn std::error::Error + Send + Sync>> {
        // Get all accounts for the company
        let accounts = AccountEntity::find()
            .filter(crate::entities::account::Column::CompanyId.eq(request.company_id))
            .filter(crate::entities::account::Column::IsActive.eq(true))
            .all(&self.db)
            .await?;

        let mut saft_accounts = Vec::new();
        for account in accounts {
            saft_accounts.push(SafTAccount {
                account_id: account.code.clone(),
                account_description: account.name.clone(),
                standard_account_id: account.code.clone(),
                group_account_id: account.account_class.to_string(),
                account_type: "GL".to_string(),
                account_creation_date: account.created_at.date_naive(),
                account_class: account.account_class.to_string(),
                account_nature: match account.account_type {
                    crate::entities::account::AccountType::Asset | crate::entities::account::AccountType::Expense => "D".to_string(),
                    _ => "C".to_string(),
                },
                opening_debit_balance: Decimal::ZERO, // TODO: Calculate from previous period
                opening_credit_balance: Decimal::ZERO,
                closing_debit_balance: Decimal::ZERO, // TODO: Calculate for current period
                closing_credit_balance: Decimal::ZERO,
            });
        }

        // Get counterparts (customers and suppliers)
        let counterparts = CounterpartEntity::find()
            .filter(crate::entities::counterpart::Column::CompanyId.eq(request.company_id))
            .all(&self.db)
            .await?;

        let mut customers = Vec::new();
        let mut suppliers = Vec::new();

        for counterpart in counterparts {
            match counterpart.counterpart_type {
                crate::entities::counterpart::CounterpartType::Customer => {
                    customers.push(SafTCustomer {
                        customer_id: counterpart.id.to_string(),
                        account_id: counterpart.eik.clone().unwrap_or_default(),
                        customer_tax_id: counterpart.vat_number.clone().unwrap_or_default(),
                        company_name: counterpart.name.clone(),
                        contact: counterpart.contact_person.unwrap_or_default(),
                        billing_address: SafTAddress {
                            building_number: None,
                            street_name: counterpart.address.clone(),
                            address_detail: None,
                            city: counterpart.city.clone().unwrap_or("Unknown".to_string()),
                            postal_code: None,
                            region: None,
                            country: counterpart.country.clone().unwrap_or("BG".to_string()),
                        },
                        ship_to_address: None,
                        telephone: counterpart.phone.clone(),
                        fax: None,
                        email: counterpart.email.clone(),
                        website: None,
                        bank_account: None,
                        customer_group: None,
                        creation_date: counterpart.created_at.date_naive(),
                        self_billing_indicator: "0".to_string(),
                        opening_debit_balance: Decimal::ZERO,
                        opening_credit_balance: Decimal::ZERO,
                        closing_debit_balance: Decimal::ZERO,
                        closing_credit_balance: Decimal::ZERO,
                    });
                },
                crate::entities::counterpart::CounterpartType::Supplier => {
                    suppliers.push(SafTSupplier {
                        supplier_id: counterpart.id.to_string(),
                        account_id: counterpart.eik.clone().unwrap_or_default(),
                        supplier_tax_id: counterpart.vat_number.clone().unwrap_or_default(),
                        company_name: counterpart.name.clone(),
                        contact: counterpart.contact_person.unwrap_or_default(),
                        billing_address: SafTAddress {
                            building_number: None,
                            street_name: counterpart.address.clone(),
                            address_detail: None,
                            city: counterpart.city.clone().unwrap_or("Unknown".to_string()),
                            postal_code: None,
                            region: None,
                            country: counterpart.country.clone().unwrap_or("BG".to_string()),
                        },
                        ship_from_address: None,
                        telephone: counterpart.phone.clone(),
                        fax: None,
                        email: counterpart.email.clone(),
                        website: None,
                        bank_account: None,
                        supplier_group: None,
                        creation_date: counterpart.created_at.date_naive(),
                        self_billing_indicator: "0".to_string(),
                        opening_debit_balance: Decimal::ZERO,
                        opening_credit_balance: Decimal::ZERO,
                        closing_debit_balance: Decimal::ZERO,
                        closing_credit_balance: Decimal::ZERO,
                    });
                },
                _ => {}
            }
        }

        // Basic tax table for Bulgaria
        let tax_table = vec![
            SafTTaxRate {
                tax_type: "VAT".to_string(),
                tax_code_details: "Standard VAT Rate".to_string(),
                tax_code: "S".to_string(),
                description: "Стандартна ставка ДДС".to_string(),
                tax_percentage: Decimal::from(20), // 20% VAT in Bulgaria
                country: "BG".to_string(),
                effective_from: NaiveDate::from_ymd_opt(2007, 1, 1).unwrap(),
                effective_to: None,
            },
            SafTTaxRate {
                tax_type: "VAT".to_string(),
                tax_code_details: "Reduced VAT Rate".to_string(),
                tax_code: "R".to_string(),
                description: "Намалена ставка ДДС".to_string(),
                tax_percentage: Decimal::from(9), // 9% VAT for specific goods
                country: "BG".to_string(),
                effective_from: NaiveDate::from_ymd_opt(2007, 1, 1).unwrap(),
                effective_to: None,
            },
            SafTTaxRate {
                tax_type: "VAT".to_string(),
                tax_code_details: "Zero VAT Rate".to_string(),
                tax_code: "Z".to_string(),
                description: "Нулева ставка ДДС".to_string(),
                tax_percentage: Decimal::ZERO,
                country: "BG".to_string(),
                effective_from: NaiveDate::from_ymd_opt(2007, 1, 1).unwrap(),
                effective_to: None,
            }
        ];

        // Basic UOM table
        let uom_table = vec![
            SafTUomEntry {
                unit_of_measure: "бр".to_string(),
                uom_description: "Броя".to_string(),
                uom_to_uom_base: Decimal::ONE,
            },
            SafTUomEntry {
                unit_of_measure: "кг".to_string(),
                uom_description: "Килограма".to_string(),
                uom_to_uom_base: Decimal::ONE,
            },
            SafTUomEntry {
                unit_of_measure: "л".to_string(),
                uom_description: "Литър".to_string(),
                uom_to_uom_base: Decimal::ONE,
            },
        ];

        Ok(MasterFiles {
            general_ledger_accounts: saft_accounts,
            customers,
            suppliers,
            tax_table,
            uom_table,
            analysis_type_table: vec![], // TODO: Implement analytical account types
            movement_type_table: vec![], // TODO: Implement movement types
            products: vec![], // TODO: Implement product catalog if needed
        })
    }

    async fn build_general_ledger(&self, request: &SafTExportRequest) -> Result<GeneralLedger, Box<dyn std::error::Error + Send + Sync>> {
        // Get all journal entries for the period
        let journal_entries = JournalEntryEntity::find()
            .filter(crate::entities::journal_entry::Column::CompanyId.eq(request.company_id))
            .filter(crate::entities::journal_entry::Column::DocumentDate.between(request.start_date, request.end_date))
            .find_with_related(EntryLineEntity)
            .all(&self.db)
            .await?;

        let mut transactions = Vec::new();
        let mut total_debit = Decimal::ZERO;
        let mut total_credit = Decimal::ZERO;

        for (entry, lines) in journal_entries {
            let mut saft_lines = Vec::new();

            for line in lines {
                let debit_amount = line.debit_amount;
                let credit_amount = line.credit_amount;

                total_debit += debit_amount;
                total_credit += credit_amount;

                saft_lines.push(SafTTransactionLine {
                    record_id: format!("{}_{}", entry.id, line.id),
                    account_id: line.account_id.to_string(), // Will need to get account code from relationship
                    analysis: None, // TODO: Implement analytical account analysis
                    value_date: entry.document_date,
                    source_document_id: entry.document_number.clone(),
                    customer_id: None, // TODO: Link to customer if applicable
                    supplier_id: None, // TODO: Link to supplier if applicable
                    description: line.description.clone().unwrap_or_default(),
                    debit_amount: if debit_amount > Decimal::ZERO { Some(debit_amount) } else { None },
                    credit_amount: if credit_amount > Decimal::ZERO { Some(credit_amount) } else { None },
                    tax_information: None, // TODO: Implement VAT information
                    reference_number: None, // entry doesn't have reference field
                    reason: None,
                    product_code: None,
                    quantity: line.quantity,
                    unit_of_measure: line.unit_of_measure_code.clone(),
                    unit_price: None, // Not available in current line model
                });
            }

            transactions.push(SafTTransaction {
                transaction_id: entry.id.to_string(),
                period: entry.document_date.month() as i32,
                period_year: entry.document_date.year(),
                transaction_date: entry.document_date,
                source_id: "1".to_string(), // Default journal ID
                description: entry.description.clone(),
                doc_archival_number: None,
                transaction_type: "Journal Entry".to_string(),
                gl_posting_date: entry.accounting_date,
                customer_id: None,
                supplier_id: None,
                lines: saft_lines,
            });
        }

        let journal = vec![SafTJournal {
            journal_id: "1".to_string(),
            description: "General Journal".to_string(),
            journal_type: "GL".to_string(),
            transaction: transactions,
        }];

        Ok(GeneralLedger {
            number_of_entries: journal.iter().map(|j| j.transaction.len() as i32).sum(),
            total_debit,
            total_credit,
            journal,
        })
    }

    async fn build_accounts_payable(&self, _request: &SafTExportRequest) -> Result<AccountsPayable, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement accounts payable logic
        Ok(AccountsPayable {
            number_of_entries: 0,
            total_debit: Decimal::ZERO,
            total_credit: Decimal::ZERO,
            supplier_transactions: vec![],
        })
    }

    async fn build_accounts_receivable(&self, _request: &SafTExportRequest) -> Result<AccountsReceivable, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement accounts receivable logic
        Ok(AccountsReceivable {
            number_of_entries: 0,
            total_debit: Decimal::ZERO,
            total_credit: Decimal::ZERO,
            customer_transactions: vec![],
        })
    }

    async fn build_fixed_assets(&self, _request: &SafTExportRequest) -> Result<FixedAssets, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement fixed assets logic
        Ok(FixedAssets {
            number_of_entries: 0,
            total_book_value: Decimal::ZERO,
            asset: vec![],
        })
    }

    fn generate_xml(&self, saft: &BulgarianSafT) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        // Write XML declaration
        writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", Some("UTF-8"), None)))?;

        // Write root element
        let mut elem = quick_xml::events::BytesStart::new("AuditFile");
        elem.push_attribute(("xmlns", "urn:StandardAuditFile-Tax:BG_2.0"));
        elem.push_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"));
        writer.write_event(Event::Start(elem))?;

        // Write Header
        self.write_header(&mut writer, &saft.header)?;

        // Write MasterFiles
        self.write_master_files(&mut writer, &saft.master_files)?;

        // Write GeneralLedger
        self.write_general_ledger(&mut writer, &saft.general_ledger)?;

        // Write optional sections
        if let Some(ref ap) = saft.accounts_payable {
            self.write_accounts_payable(&mut writer, ap)?;
        }

        if let Some(ref ar) = saft.accounts_receivable {
            self.write_accounts_receivable(&mut writer, ar)?;
        }

        if let Some(ref fa) = saft.fixed_assets {
            self.write_fixed_assets(&mut writer, fa)?;
        }

        // Close root element
        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("AuditFile")))?;

        let result = writer.into_inner().into_inner();
        Ok(String::from_utf8(result)?)
    }

    fn write_header<W: std::io::Write>(&self, writer: &mut Writer<W>, header: &SafTHeader) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(quick_xml::events::BytesStart::new("Header")))?;

        self.write_element(writer, "AuditFileVersion", &header.audit_file_version)?;
        self.write_element(writer, "AuditFileCountry", &header.audit_file_country)?;
        self.write_element(writer, "AuditFileDateCreated", &header.audit_file_date_created.to_rfc3339())?;
        self.write_element(writer, "SoftwareCompanyName", &header.software_company_name)?;
        self.write_element(writer, "SoftwareID", &header.software_id)?;
        self.write_element(writer, "SoftwareVersion", &header.software_version)?;
        self.write_element(writer, "DefaultCurrencyCode", &header.default_currency_code)?;

        // Company info
        writer.write_event(Event::Start(quick_xml::events::BytesStart::new("Company")))?;
        self.write_element(writer, "RegistrationNumber", &header.company.registration_number)?;
        self.write_element(writer, "Name", &header.company.name)?;
        self.write_element(writer, "TaxRegistrationNumber", &header.company.tax_registration_number)?;
        self.write_element(writer, "TaxAccountingBasis", &header.company.tax_accounting_basis)?;
        self.write_element(writer, "CompanyID", &header.company.company_id)?;

        // Company Address
        writer.write_event(Event::Start(quick_xml::events::BytesStart::new("Address")))?;
        if let Some(ref street) = header.company.address.street_name {
            self.write_element(writer, "StreetName", street)?;
        }
        self.write_element(writer, "City", &header.company.address.city)?;
        if let Some(ref postal) = header.company.address.postal_code {
            self.write_element(writer, "PostalCode", postal)?;
        }
        self.write_element(writer, "Country", &header.company.address.country)?;
        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("Address")))?;

        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("Company")))?;

        // Selection criteria
        writer.write_event(Event::Start(quick_xml::events::BytesStart::new("SelectionCriteria")))?;
        self.write_element(writer, "SelectionStartDate", &header.selection_criteria.selection_start_date.to_string())?;
        self.write_element(writer, "SelectionEndDate", &header.selection_criteria.selection_end_date.to_string())?;
        self.write_element(writer, "PeriodStart", &header.selection_criteria.period_start.to_string())?;
        self.write_element(writer, "PeriodEnd", &header.selection_criteria.period_end.to_string())?;
        self.write_element(writer, "PeriodType", &header.selection_criteria.period_type)?;
        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("SelectionCriteria")))?;

        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("Header")))?;
        Ok(())
    }

    fn write_master_files<W: std::io::Write>(&self, writer: &mut Writer<W>, master_files: &MasterFiles) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(quick_xml::events::BytesStart::new("MasterFiles")))?;

        // General Ledger Accounts
        writer.write_event(Event::Start(quick_xml::events::BytesStart::new("GeneralLedgerAccounts")))?;
        for account in &master_files.general_ledger_accounts {
            writer.write_event(Event::Start(quick_xml::events::BytesStart::new("Account")))?;
            self.write_element(writer, "AccountID", &account.account_id)?;
            self.write_element(writer, "AccountDescription", &account.account_description)?;
            self.write_element(writer, "StandardAccountID", &account.standard_account_id)?;
            self.write_element(writer, "GroupAccountID", &account.group_account_id)?;
            self.write_element(writer, "AccountType", &account.account_type)?;
            self.write_element(writer, "AccountCreationDate", &account.account_creation_date.to_string())?;
            self.write_element(writer, "AccountClass", &account.account_class)?;
            self.write_element(writer, "AccountNature", &account.account_nature)?;
            self.write_element(writer, "OpeningDebitBalance", &account.opening_debit_balance.to_string())?;
            self.write_element(writer, "OpeningCreditBalance", &account.opening_credit_balance.to_string())?;
            self.write_element(writer, "ClosingDebitBalance", &account.closing_debit_balance.to_string())?;
            self.write_element(writer, "ClosingCreditBalance", &account.closing_credit_balance.to_string())?;
            writer.write_event(Event::End(quick_xml::events::BytesEnd::new("Account")))?;
        }
        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("GeneralLedgerAccounts")))?;

        // Customers
        if !master_files.customers.is_empty() {
            writer.write_event(Event::Start(quick_xml::events::BytesStart::new("Customers")))?;
            for customer in &master_files.customers {
                writer.write_event(Event::Start(quick_xml::events::BytesStart::new("Customer")))?;
                self.write_element(writer, "CustomerID", &customer.customer_id)?;
                self.write_element(writer, "AccountID", &customer.account_id)?;
                self.write_element(writer, "CustomerTaxID", &customer.customer_tax_id)?;
                self.write_element(writer, "CompanyName", &customer.company_name)?;
                self.write_element(writer, "Contact", &customer.contact)?;

                // Billing Address
                writer.write_event(Event::Start(quick_xml::events::BytesStart::new("BillingAddress")))?;
                if let Some(ref street) = customer.billing_address.street_name {
                    self.write_element(writer, "StreetName", street)?;
                }
                self.write_element(writer, "City", &customer.billing_address.city)?;
                if let Some(ref postal) = customer.billing_address.postal_code {
                    self.write_element(writer, "PostalCode", postal)?;
                }
                self.write_element(writer, "Country", &customer.billing_address.country)?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("BillingAddress")))?;

                if let Some(ref phone) = customer.telephone {
                    self.write_element(writer, "Telephone", phone)?;
                }
                if let Some(ref email) = customer.email {
                    self.write_element(writer, "Email", email)?;
                }

                self.write_element(writer, "CreationDate", &customer.creation_date.to_string())?;
                self.write_element(writer, "SelfBillingIndicator", &customer.self_billing_indicator)?;
                self.write_element(writer, "OpeningDebitBalance", &customer.opening_debit_balance.to_string())?;
                self.write_element(writer, "OpeningCreditBalance", &customer.opening_credit_balance.to_string())?;
                self.write_element(writer, "ClosingDebitBalance", &customer.closing_debit_balance.to_string())?;
                self.write_element(writer, "ClosingCreditBalance", &customer.closing_credit_balance.to_string())?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("Customer")))?;
            }
            writer.write_event(Event::End(quick_xml::events::BytesEnd::new("Customers")))?;
        }

        // Suppliers
        if !master_files.suppliers.is_empty() {
            writer.write_event(Event::Start(quick_xml::events::BytesStart::new("Suppliers")))?;
            for supplier in &master_files.suppliers {
                writer.write_event(Event::Start(quick_xml::events::BytesStart::new("Supplier")))?;
                self.write_element(writer, "SupplierID", &supplier.supplier_id)?;
                self.write_element(writer, "AccountID", &supplier.account_id)?;
                self.write_element(writer, "SupplierTaxID", &supplier.supplier_tax_id)?;
                self.write_element(writer, "CompanyName", &supplier.company_name)?;
                self.write_element(writer, "Contact", &supplier.contact)?;

                // Billing Address
                writer.write_event(Event::Start(quick_xml::events::BytesStart::new("BillingAddress")))?;
                if let Some(ref street) = supplier.billing_address.street_name {
                    self.write_element(writer, "StreetName", street)?;
                }
                self.write_element(writer, "City", &supplier.billing_address.city)?;
                if let Some(ref postal) = supplier.billing_address.postal_code {
                    self.write_element(writer, "PostalCode", postal)?;
                }
                self.write_element(writer, "Country", &supplier.billing_address.country)?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("BillingAddress")))?;

                if let Some(ref phone) = supplier.telephone {
                    self.write_element(writer, "Telephone", phone)?;
                }
                if let Some(ref email) = supplier.email {
                    self.write_element(writer, "Email", email)?;
                }

                self.write_element(writer, "CreationDate", &supplier.creation_date.to_string())?;
                self.write_element(writer, "SelfBillingIndicator", &supplier.self_billing_indicator)?;
                self.write_element(writer, "OpeningDebitBalance", &supplier.opening_debit_balance.to_string())?;
                self.write_element(writer, "OpeningCreditBalance", &supplier.opening_credit_balance.to_string())?;
                self.write_element(writer, "ClosingDebitBalance", &supplier.closing_debit_balance.to_string())?;
                self.write_element(writer, "ClosingCreditBalance", &supplier.closing_credit_balance.to_string())?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("Supplier")))?;
            }
            writer.write_event(Event::End(quick_xml::events::BytesEnd::new("Suppliers")))?;
        }

        // Tax Table
        if !master_files.tax_table.is_empty() {
            writer.write_event(Event::Start(quick_xml::events::BytesStart::new("TaxTable")))?;
            for tax_rate in &master_files.tax_table {
                writer.write_event(Event::Start(quick_xml::events::BytesStart::new("TaxTableEntry")))?;
                self.write_element(writer, "TaxType", &tax_rate.tax_type)?;
                self.write_element(writer, "TaxCodeDetails", &tax_rate.tax_code_details)?;
                self.write_element(writer, "TaxCode", &tax_rate.tax_code)?;
                self.write_element(writer, "Description", &tax_rate.description)?;
                self.write_element(writer, "TaxPercentage", &tax_rate.tax_percentage.to_string())?;
                self.write_element(writer, "Country", &tax_rate.country)?;
                self.write_element(writer, "EffectiveFrom", &tax_rate.effective_from.to_string())?;
                if let Some(ref effective_to) = tax_rate.effective_to {
                    self.write_element(writer, "EffectiveTo", &effective_to.to_string())?;
                }
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("TaxTableEntry")))?;
            }
            writer.write_event(Event::End(quick_xml::events::BytesEnd::new("TaxTable")))?;
        }

        // UOM Table
        if !master_files.uom_table.is_empty() {
            writer.write_event(Event::Start(quick_xml::events::BytesStart::new("UOMTable")))?;
            for uom in &master_files.uom_table {
                writer.write_event(Event::Start(quick_xml::events::BytesStart::new("UOMTableEntry")))?;
                self.write_element(writer, "UnitOfMeasure", &uom.unit_of_measure)?;
                self.write_element(writer, "UOMDescription", &uom.uom_description)?;
                self.write_element(writer, "UOMToUOMBase", &uom.uom_to_uom_base.to_string())?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("UOMTableEntry")))?;
            }
            writer.write_event(Event::End(quick_xml::events::BytesEnd::new("UOMTable")))?;
        }

        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("MasterFiles")))?;
        Ok(())
    }

    fn write_general_ledger<W: std::io::Write>(&self, writer: &mut Writer<W>, general_ledger: &GeneralLedger) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(quick_xml::events::BytesStart::new("GeneralLedger")))?;

        self.write_element(writer, "NumberOfEntries", &general_ledger.number_of_entries.to_string())?;
        self.write_element(writer, "TotalDebit", &general_ledger.total_debit.to_string())?;
        self.write_element(writer, "TotalCredit", &general_ledger.total_credit.to_string())?;

        for journal in &general_ledger.journal {
            writer.write_event(Event::Start(quick_xml::events::BytesStart::new("Journal")))?;
            self.write_element(writer, "JournalID", &journal.journal_id)?;
            self.write_element(writer, "Description", &journal.description)?;
            self.write_element(writer, "JournalType", &journal.journal_type)?;

            for transaction in &journal.transaction {
                writer.write_event(Event::Start(quick_xml::events::BytesStart::new("Transaction")))?;
                self.write_element(writer, "TransactionID", &transaction.transaction_id)?;
                self.write_element(writer, "Period", &transaction.period.to_string())?;
                self.write_element(writer, "PeriodYear", &transaction.period_year.to_string())?;
                self.write_element(writer, "TransactionDate", &transaction.transaction_date.to_string())?;
                self.write_element(writer, "SourceID", &transaction.source_id)?;
                self.write_element(writer, "Description", &transaction.description)?;
                self.write_element(writer, "TransactionType", &transaction.transaction_type)?;
                self.write_element(writer, "GLPostingDate", &transaction.gl_posting_date.to_string())?;

                for line in &transaction.lines {
                    writer.write_event(Event::Start(quick_xml::events::BytesStart::new("Line")))?;
                    self.write_element(writer, "RecordID", &line.record_id)?;
                    self.write_element(writer, "AccountID", &line.account_id)?;
                    self.write_element(writer, "ValueDate", &line.value_date.to_string())?;
                    self.write_element(writer, "Description", &line.description)?;

                    if let Some(ref debit) = line.debit_amount {
                        self.write_element(writer, "DebitAmount", &debit.to_string())?;
                    }
                    if let Some(ref credit) = line.credit_amount {
                        self.write_element(writer, "CreditAmount", &credit.to_string())?;
                    }
                    if let Some(ref quantity) = line.quantity {
                        self.write_element(writer, "Quantity", &quantity.to_string())?;
                    }
                    if let Some(ref unit) = line.unit_of_measure {
                        self.write_element(writer, "UnitOfMeasure", unit)?;
                    }
                    if let Some(ref price) = line.unit_price {
                        self.write_element(writer, "UnitPrice", &price.to_string())?;
                    }

                    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("Line")))?;
                }

                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("Transaction")))?;
            }

            writer.write_event(Event::End(quick_xml::events::BytesEnd::new("Journal")))?;
        }

        writer.write_event(Event::End(quick_xml::events::BytesEnd::new("GeneralLedger")))?;
        Ok(())
    }

    fn write_accounts_payable<W: std::io::Write>(&self, _writer: &mut Writer<W>, _ap: &AccountsPayable) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement accounts payable XML output
        Ok(())
    }

    fn write_accounts_receivable<W: std::io::Write>(&self, _writer: &mut Writer<W>, _ar: &AccountsReceivable) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement accounts receivable XML output
        Ok(())
    }

    fn write_fixed_assets<W: std::io::Write>(&self, _writer: &mut Writer<W>, _fa: &FixedAssets) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement fixed assets XML output
        Ok(())
    }

    fn write_element<W: std::io::Write>(&self, writer: &mut Writer<W>, name: &str, content: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(quick_xml::events::BytesStart::new(name)))?;
        writer.write_event(Event::Text(quick_xml::events::BytesText::new(content)))?;
        writer.write_event(Event::End(quick_xml::events::BytesEnd::new(name)))?;
        Ok(())
    }
    */
}
