use chrono::{Datelike, NaiveDate, Utc};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use rust_decimal::Decimal;
use sea_orm::*;
use std::collections::HashMap;
use std::io::Cursor;

use crate::entities::{
    account::Entity as AccountEntity, company::Entity as CompanyEntity,
    counterpart::Entity as CounterpartEntity, entry_line::Entity as EntryLineEntity,
    journal_entry::Entity as JournalEntryEntity, saft::*,
};

pub struct SafTServiceV2 {
    db: DatabaseConnection,
}

impl SafTServiceV2 {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Generate SAF-T XML file v1.0.1 for a given company and period
    pub async fn generate_saft(
        &self,
        request: SafTExportRequest,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Get company information
        let company = CompanyEntity::find_by_id(request.company_id)
            .one(&self.db)
            .await?
            .ok_or("Company not found")?;

        // Build header
        let header = self.build_header(&company, &request).await?;

        // Build SAF-T structure based on file type
        let mut saft = BulgarianSafT {
            header,
            file_type: request.file_type.clone(),
            // Annual
            master_files_annual: None,
            source_documents_annual: None,
            // Monthly
            master_files_monthly: None,
            corresponding_accounts_report: None,
            general_ledger_entries: None,
            source_documents_monthly: None,
            // OnDemand
            master_files_on_demand: None,
            source_documents_on_demand: None,
        };

        match request.file_type {
            SafTFileType::Annual => {
                saft.master_files_annual = Some(self.build_master_files_annual(&request).await?);
                saft.source_documents_annual =
                    Some(self.build_source_documents_annual(&request).await?);
            }
            SafTFileType::Monthly => {
                saft.master_files_monthly = Some(self.build_master_files_monthly(&request).await?);
                saft.corresponding_accounts_report =
                    self.build_corresponding_accounts_report(&request).await?;
                saft.general_ledger_entries =
                    Some(self.build_general_ledger_entries(&request).await?);
                saft.source_documents_monthly =
                    Some(self.build_source_documents_monthly(&request).await?);
            }
            SafTFileType::OnDemand => {
                saft.master_files_on_demand =
                    Some(self.build_master_files_on_demand(&request).await?);
                saft.source_documents_on_demand =
                    Some(self.build_source_documents_on_demand(&request).await?);
            }
        }

        // Generate XML
        self.generate_xml(&saft)
    }

    async fn build_header(
        &self,
        company: &crate::entities::company::Model,
        request: &SafTExportRequest,
    ) -> Result<SafTHeader, Box<dyn std::error::Error + Send + Sync>> {
        let header_comment = match request.file_type {
            SafTFileType::Annual => "A",
            SafTFileType::Monthly => "M",
            SafTFileType::OnDemand => "O",
        }
        .to_string();

        Ok(SafTHeader {
            audit_file_version: "007".to_string(), // Version 1.0.1
            audit_file_country: "BG".to_string(),
            audit_file_region: Some("BG-22".to_string()), // Sofia region by default
            audit_file_date_created: Utc::now().date_naive(),
            software_company_name: "RS Accounting BG".to_string(),
            software_id: "RS-AC-BG".to_string(),
            software_version: "001".to_string(),
            company: CompanyInfo {
                registration_number: company.eik.clone(),
                name: company.name.clone(),
                address: Address {
                    street_name: company
                        .address
                        .clone()
                        .or_else(|| Some("неизвестен адрес".to_string())),
                    number: None,
                    additional_address_detail: None,
                    building: None,
                    city: company.city.clone().unwrap_or("София".to_string()),
                    postal_code: None,
                    region: None,
                    country: "BG".to_string(),
                    address_type: "StreetAddress".to_string(),
                },
                contact: Contact {
                    contact_person: company.contact_person.as_ref().map(|name| ContactPerson {
                        title: None,
                        first_name: Some(name.clone()),
                        initials: None,
                        last_name_prefix: None,
                        last_name: None,
                        birth_name: None,
                        salutation: None,
                        other_titles: vec![],
                    }),
                    telephone: company.phone.clone(),
                    fax: None,
                    email: company.email.clone(),
                    website: None,
                },
                tax_registration: vec![TaxRegistration {
                    tax_registration_number: company.eik.clone(),
                    tax_type: "100010".to_string(), // VAT code for Bulgaria
                    tax_number: company
                        .vat_number
                        .clone()
                        .unwrap_or_else(|| format!("BG{}", company.eik)),
                    tax_verification_date: None,
                }],
                bank_account: vec![], // TODO: Add bank accounts from company settings
            },
            ownership: None, // TODO: Add ownership information if available
            default_currency_code: "BGN".to_string(),
            selection_criteria: SelectionCriteria {
                tax_reporting_jurisdiction: "NRA".to_string(),
                company_entity: None,
                period_start: request.period_start,
                period_start_year: request.period_start_year,
                period_end: request.period_end,
                period_end_year: request.period_end_year,
                document_type: None,
                other_criteria: None,
            },
            header_comment,
            segment_index: None,
            total_segments_in_sequence: None,
            tax_accounting_basis: request.tax_accounting_basis.clone(),
            tax_entity: Some("Company".to_string()),
        })
    }

    async fn build_master_files_monthly(
        &self,
        request: &SafTExportRequest,
    ) -> Result<MasterFilesMonthly, Box<dyn std::error::Error + Send + Sync>> {
        // Get accounts
        let accounts = AccountEntity::find()
            .filter(crate::entities::account::Column::CompanyId.eq(request.company_id))
            .filter(crate::entities::account::Column::IsActive.eq(true))
            .all(&self.db)
            .await?;

        let mut saft_accounts = Vec::new();
        for account in accounts {
            // Parse account code as number
            let account_id = account.code.parse::<i32>().unwrap_or(0);

            saft_accounts.push(SafTAccount {
                account_id,
                account_description: account.name.clone(),
                taxpayer_account_id: account.code.clone(),
                grouping_category: None,
                grouping_code: None,
                account_type: "Bifunctional".to_string(), // Most common for Bulgarian accounting
                account_creation_date: Some(account.created_at.date_naive()),
                opening_debit_balance: Some(Decimal::ZERO), // TODO: Calculate from previous period
                opening_credit_balance: Some(Decimal::ZERO),
                closing_debit_balance: Some(Decimal::ZERO), // TODO: Calculate for current period
                closing_credit_balance: Some(Decimal::ZERO),
            });
        }

        // Get counterparts
        let counterparts = CounterpartEntity::find()
            .filter(crate::entities::counterpart::Column::CompanyId.eq(request.company_id))
            .all(&self.db)
            .await?;

        let mut customers = Vec::new();
        let mut suppliers = Vec::new();

        for counterpart in counterparts {
            let address = Address {
                street_name: counterpart
                    .address
                    .clone()
                    .or_else(|| Some("неизвестен адрес".to_string())),
                number: None,
                additional_address_detail: None,
                building: None,
                city: counterpart
                    .city
                    .clone()
                    .unwrap_or("неизвестен град".to_string()),
                postal_code: None,
                region: None,
                country: counterpart.country.clone().unwrap_or("BG".to_string()),
                address_type: "StreetAddress".to_string(),
            };

            let contact = Contact {
                contact_person: counterpart
                    .contact_person
                    .as_ref()
                    .map(|name| ContactPerson {
                        first_name: Some(name.clone()),
                        title: None,
                        initials: None,
                        last_name_prefix: None,
                        last_name: None,
                        birth_name: None,
                        salutation: None,
                        other_titles: vec![],
                    }),
                telephone: counterpart.phone.clone(),
                fax: None,
                email: counterpart.email.clone(),
                website: None,
            };

            let tax_reg = counterpart.vat_number.as_ref().map(|vat| TaxRegistration {
                tax_registration_number: counterpart.eik.clone().unwrap_or_default(),
                tax_type: "100010".to_string(),
                tax_number: vat.clone(),
                tax_verification_date: None,
            });

            match counterpart.counterpart_type {
                crate::entities::counterpart::CounterpartType::Customer => {
                    customers.push(SafTCustomer {
                        customer_id: counterpart.id.to_string(),
                        customer_name: counterpart.name.clone(),
                        taxpayer_customer_id: None,
                        addresses: vec![CustomerAddress { address }],
                        contact: Some(contact),
                        tax_registration: tax_reg,
                        bank_account: vec![],
                        self_billing_indicator: "0".to_string(),
                        account_id: None,
                        opening_balance: None,
                        closing_balance: None,
                        party_info: None,
                    });
                }
                crate::entities::counterpart::CounterpartType::Supplier => {
                    suppliers.push(SafTSupplier {
                        supplier_id: counterpart.id.to_string(),
                        supplier_name: counterpart.name.clone(),
                        taxpayer_supplier_id: None,
                        addresses: vec![SupplierAddress { address }],
                        contact: Some(contact),
                        tax_registration: tax_reg,
                        bank_account: vec![],
                        self_billing_indicator: "0".to_string(),
                        account_id: None,
                        opening_balance: None,
                        closing_balance: None,
                        party_info: None,
                    });
                }
                _ => {}
            }
        }

        // Tax table
        let tax_table = TaxTable {
            tax_code_details: vec![
                SafTTaxCodeDetails {
                    tax_code: "VAT20".to_string(),
                    effective_date: NaiveDate::from_ymd_opt(2007, 1, 1).unwrap(),
                    expiration_date: None,
                    description: "Стандартна ставка ДДС 20%".to_string(),
                    tax_percentage: Some(Decimal::from(20)),
                    flat_tax_rate: None,
                    country: "BG".to_string(),
                    region: None,
                    standard_tax_code: None,
                    compensation: None,
                },
                SafTTaxCodeDetails {
                    tax_code: "VAT9".to_string(),
                    effective_date: NaiveDate::from_ymd_opt(2011, 4, 1).unwrap(),
                    expiration_date: None,
                    description: "Намалена ставка ДДС 9%".to_string(),
                    tax_percentage: Some(Decimal::from(9)),
                    flat_tax_rate: None,
                    country: "BG".to_string(),
                    region: None,
                    standard_tax_code: None,
                    compensation: None,
                },
                SafTTaxCodeDetails {
                    tax_code: "VAT0".to_string(),
                    effective_date: NaiveDate::from_ymd_opt(2007, 1, 1).unwrap(),
                    expiration_date: None,
                    description: "Нулева ставка ДДС".to_string(),
                    tax_percentage: Some(Decimal::ZERO),
                    flat_tax_rate: None,
                    country: "BG".to_string(),
                    region: None,
                    standard_tax_code: None,
                    compensation: None,
                },
            ],
        };

        // UOM table
        let uom_table = UOMTable {
            uom_entry: vec![
                SafTUomEntry {
                    unit_of_measure: "бр".to_string(),
                    uom_description: "Брой".to_string(),
                    uom_to_uom_base: Decimal::ONE,
                },
                SafTUomEntry {
                    unit_of_measure: "кг".to_string(),
                    uom_description: "Килограм".to_string(),
                    uom_to_uom_base: Decimal::ONE,
                },
                SafTUomEntry {
                    unit_of_measure: "л".to_string(),
                    uom_description: "Литър".to_string(),
                    uom_to_uom_base: Decimal::ONE,
                },
            ],
        };

        Ok(MasterFilesMonthly {
            general_ledger_accounts: GeneralLedgerAccounts {
                account: saft_accounts,
            },
            taxonomies: None,
            customers: Customers {
                customer: customers,
            },
            suppliers: Suppliers {
                supplier: suppliers,
            },
            tax_table,
            uom_table,
            analysis_type_table: None,
            products: Products { product: vec![] },
            owners: None,
        })
    }

    async fn build_general_ledger_entries(
        &self,
        request: &SafTExportRequest,
    ) -> Result<GeneralLedgerEntries, Box<dyn std::error::Error + Send + Sync>> {
        // Calculate date range from period
        let start_date =
            NaiveDate::from_ymd_opt(request.period_start_year, request.period_start as u32, 1)
                .unwrap();
        let end_date = if request.period_end == 12 {
            NaiveDate::from_ymd_opt(request.period_end_year, 12, 31).unwrap()
        } else {
            NaiveDate::from_ymd_opt(request.period_end_year, (request.period_end + 1) as u32, 1)
                .unwrap()
                .pred_opt()
                .unwrap()
        };

        // Get journal entries
        let journal_entries = JournalEntryEntity::find()
            .filter(crate::entities::journal_entry::Column::CompanyId.eq(request.company_id))
            .filter(
                crate::entities::journal_entry::Column::DocumentDate.between(start_date, end_date),
            )
            .find_with_related(EntryLineEntity)
            .all(&self.db)
            .await?;

        let mut transactions = Vec::new();
        let mut total_debit = Decimal::ZERO;
        let mut total_credit = Decimal::ZERO;

        for (entry, lines) in journal_entries {
            let mut debit_lines = Vec::new();
            let mut credit_lines = Vec::new();

            for line in lines {
                // Get account code
                let account = AccountEntity::find_by_id(line.account_id)
                    .one(&self.db)
                    .await?
                    .ok_or("Account not found")?;

                let account_id = account.code.parse::<i32>().unwrap_or(0);

                total_debit += line.debit_amount;
                total_credit += line.credit_amount;

                let line_data = SafTTransactionLine {
                    record_id: format!("L{}", line.id),
                    account_id,
                    taxpayer_account_id: Some(account.code),
                    analysis: None,
                    value_date: Some(entry.document_date),
                    source_document_id: entry.document_number.clone(),
                    customer_id: None,
                    supplier_id: None,
                    description: line.description.clone().unwrap_or_default(),
                    debit_amount: if line.debit_amount > Decimal::ZERO {
                        Some(AmountStructure {
                            amount: line.debit_amount,
                            currency_code: None,
                            currency_amount: None,
                            exchange_rate: None,
                        })
                    } else {
                        None
                    },
                    credit_amount: if line.credit_amount > Decimal::ZERO {
                        Some(AmountStructure {
                            amount: line.credit_amount,
                            currency_code: None,
                            currency_amount: None,
                            exchange_rate: None,
                        })
                    } else {
                        None
                    },
                    tax_information: None,
                    reference_number: None,
                    cid: None,
                    quantity: line.quantity,
                    cross_reference: None,
                    system_entry_time: None,
                };

                if line.debit_amount > Decimal::ZERO {
                    debit_lines.push(line_data);
                } else if line.credit_amount > Decimal::ZERO {
                    credit_lines.push(line_data);
                }
            }

            transactions.push(SafTTransaction {
                transaction_id: entry.id.to_string(),
                period: entry.document_date.month() as i32,
                period_year: entry.document_date.year(),
                transaction_date: entry.document_date,
                source_id: Some("1".to_string()),
                description: entry.description.clone(),
                doc_archival_number: None,
                transaction_type: Some("Normal".to_string()),
                system_entry_date: entry.created_at.date_naive(),
                gl_posting_date: entry.accounting_date,
                customer_id: None,
                supplier_id: None,
                system_id: None,
                lines: Lines {
                    debit_line: debit_lines,
                    credit_line: credit_lines,
                },
            });
        }

        let journal = vec![SafTJournal {
            journal_id: "1".to_string(),
            description: "Главна книга".to_string(),
            journal_type: "GL".to_string(),
            transaction: transactions,
        }];

        Ok(GeneralLedgerEntries {
            number_of_entries: journal.iter().map(|j| j.transaction.len() as i32).sum(),
            total_debit,
            total_credit,
            journal,
        })
    }

    async fn build_corresponding_accounts_report(
        &self,
        _request: &SafTExportRequest,
    ) -> Result<Option<CorrespondingAccountsReport>, Box<dyn std::error::Error + Send + Sync>> {
        // This is optional, return None for now
        Ok(None)
    }

    async fn build_source_documents_monthly(
        &self,
        _request: &SafTExportRequest,
    ) -> Result<SourceDocumentsMonthly, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement based on invoices and other documents
        Ok(SourceDocumentsMonthly {
            sales_invoices: None,
            purchase_invoices: None,
            payments: None,
            movement_of_goods: None,
            asset_transactions: None,
        })
    }

    async fn build_master_files_annual(
        &self,
        _request: &SafTExportRequest,
    ) -> Result<MasterFilesAnnual, Box<dyn std::error::Error + Send + Sync>> {
        Ok(MasterFilesAnnual {
            owners: None,
            assets: Assets {},
        })
    }

    async fn build_source_documents_annual(
        &self,
        _request: &SafTExportRequest,
    ) -> Result<SourceDocumentsAnnual, Box<dyn std::error::Error + Send + Sync>> {
        Ok(SourceDocumentsAnnual {})
    }

    async fn build_master_files_on_demand(
        &self,
        _request: &SafTExportRequest,
    ) -> Result<MasterFilesOnDemand, Box<dyn std::error::Error + Send + Sync>> {
        Ok(MasterFilesOnDemand {
            tax_table: None,
            movement_type_table: MovementTypeTable {
                movement_type_entry: vec![],
            },
            uom_table: UOMTable { uom_entry: vec![] },
            products: Products { product: vec![] },
            physical_stock: PhysicalStock {},
            owners: None,
        })
    }

    async fn build_source_documents_on_demand(
        &self,
        _request: &SafTExportRequest,
    ) -> Result<SourceDocumentsOnDemand, Box<dyn std::error::Error + Send + Sync>> {
        Ok(SourceDocumentsOnDemand {
            sales_invoices: None,
            purchase_invoices: None,
            movement_of_goods: None,
        })
    }

    fn generate_xml(
        &self,
        saft: &BulgarianSafT,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

        // Write XML declaration
        writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new(
            "1.0",
            Some("utf-8"),
            None,
        )))?;

        // Start AuditFile element with namespace
        let mut audit_file_elem = BytesStart::new("nsSAFT:AuditFile");
        audit_file_elem.push_attribute((
            "xmlns:doc",
            "urn:schemas-OECD:schema-extensions:documentation xml:lang=en",
        ));
        audit_file_elem.push_attribute(("xmlns:nsSAFT", "mf:nra:dgti:dxxxx:declaration:v1"));
        audit_file_elem.push_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"));
        writer.write_event(Event::Start(audit_file_elem))?;

        // Write Header
        self.write_header(&mut writer, &saft.header)?;

        // Write content based on file type
        match saft.file_type {
            SafTFileType::Monthly => {
                if let Some(ref master_files) = saft.master_files_monthly {
                    self.write_master_files_monthly(&mut writer, master_files)?;
                }
                if let Some(ref corr_report) = saft.corresponding_accounts_report {
                    self.write_corresponding_accounts_report(&mut writer, corr_report)?;
                }
                if let Some(ref gl_entries) = saft.general_ledger_entries {
                    self.write_general_ledger_entries(&mut writer, gl_entries)?;
                }
                if let Some(ref source_docs) = saft.source_documents_monthly {
                    self.write_source_documents_monthly(&mut writer, source_docs)?;
                }
            }
            SafTFileType::Annual => {
                if let Some(ref master_files) = saft.master_files_annual {
                    self.write_master_files_annual(&mut writer, master_files)?;
                }
                if let Some(ref source_docs) = saft.source_documents_annual {
                    self.write_source_documents_annual(&mut writer, source_docs)?;
                }
            }
            SafTFileType::OnDemand => {
                if let Some(ref master_files) = saft.master_files_on_demand {
                    self.write_master_files_on_demand(&mut writer, master_files)?;
                }
                if let Some(ref source_docs) = saft.source_documents_on_demand {
                    self.write_source_documents_on_demand(&mut writer, source_docs)?;
                }
            }
        }

        // End AuditFile
        writer.write_event(Event::End(BytesEnd::new("nsSAFT:AuditFile")))?;

        // Get the result
        let result = writer.into_inner().into_inner();
        Ok(String::from_utf8(result)?)
    }

    fn write_header(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        header: &SafTHeader,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:Header")))?;

        self.write_element(
            writer,
            "nsSAFT:AuditFileVersion",
            &header.audit_file_version,
        )?;
        self.write_element(
            writer,
            "nsSAFT:AuditFileCountry",
            &header.audit_file_country,
        )?;
        if let Some(ref region) = header.audit_file_region {
            self.write_element(writer, "nsSAFT:AuditFileRegion", region)?;
        }
        self.write_element(
            writer,
            "nsSAFT:AuditFileDateCreated",
            &header.audit_file_date_created.to_string(),
        )?;
        self.write_element(
            writer,
            "nsSAFT:SoftwareCompanyName",
            &header.software_company_name,
        )?;
        self.write_element(writer, "nsSAFT:SoftwareID", &header.software_id)?;
        self.write_element(writer, "nsSAFT:SoftwareVersion", &header.software_version)?;

        // Write Company
        self.write_company(writer, &header.company)?;

        // Write Ownership if present
        if let Some(ref ownership) = header.ownership {
            self.write_ownership(writer, ownership)?;
        }

        self.write_element(
            writer,
            "nsSAFT:DefaultCurrencyCode",
            &header.default_currency_code,
        )?;

        // Write Selection Criteria
        self.write_selection_criteria(writer, &header.selection_criteria)?;

        self.write_element(writer, "nsSAFT:HeaderComment", &header.header_comment)?;
        self.write_element(
            writer,
            "nsSAFT:TaxAccountingBasis",
            &header.tax_accounting_basis,
        )?;

        if let Some(ref entity) = header.tax_entity {
            self.write_element(writer, "nsSAFT:TaxEntity", entity)?;
        }

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:Header")))?;
        Ok(())
    }

    fn write_company(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        company: &CompanyInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:Company")))?;

        self.write_element(
            writer,
            "nsSAFT:RegistrationNumber",
            &company.registration_number,
        )?;
        self.write_element(writer, "nsSAFT:Name", &company.name)?;

        // Write Address
        self.write_address(writer, &company.address)?;

        // Write Contact
        self.write_contact(writer, &company.contact)?;

        // Write Tax Registrations
        for tax_reg in &company.tax_registration {
            self.write_tax_registration(writer, tax_reg)?;
        }

        // Write Bank Accounts
        for bank_acc in &company.bank_account {
            self.write_bank_account(writer, bank_acc)?;
        }

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:Company")))?;
        Ok(())
    }

    fn write_address(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        address: &Address,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:Address")))?;

        if let Some(ref street) = address.street_name {
            self.write_element(writer, "nsSAFT:StreetName", street)?;
        }
        if let Some(ref number) = address.number {
            self.write_element(writer, "nsSAFT:Number", number)?;
        }
        if let Some(ref detail) = address.additional_address_detail {
            self.write_element(writer, "nsSAFT:AdditionalAddressDetail", detail)?;
        }
        if let Some(ref building) = address.building {
            self.write_element(writer, "nsSAFT:Building", building)?;
        }
        self.write_element(writer, "nsSAFT:City", &address.city)?;
        if let Some(ref postal) = address.postal_code {
            self.write_element(writer, "nsSAFT:PostalCode", postal)?;
        }
        if let Some(ref region) = address.region {
            self.write_element(writer, "nsSAFT:Region", region)?;
        }
        self.write_element(writer, "nsSAFT:Country", &address.country)?;
        self.write_element(writer, "nsSAFT:AddressType", &address.address_type)?;

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:Address")))?;
        Ok(())
    }

    fn write_contact(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        contact: &Contact,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:Contact")))?;

        if let Some(ref person) = contact.contact_person {
            self.write_contact_person(writer, person)?;
        }
        if let Some(ref phone) = contact.telephone {
            self.write_element(writer, "nsSAFT:Telephone", phone)?;
        }
        if let Some(ref fax) = contact.fax {
            self.write_element(writer, "nsSAFT:Fax", fax)?;
        }
        if let Some(ref email) = contact.email {
            self.write_element(writer, "nsSAFT:Email", email)?;
        }
        if let Some(ref website) = contact.website {
            self.write_element(writer, "nsSAFT:Website", website)?;
        }

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:Contact")))?;
        Ok(())
    }

    fn write_contact_person(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        person: &ContactPerson,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:ContactPerson")))?;

        if let Some(ref title) = person.title {
            self.write_element(writer, "nsSAFT:Title", title)?;
        }
        if let Some(ref first_name) = person.first_name {
            self.write_element(writer, "nsSAFT:FirstName", first_name)?;
        }
        if let Some(ref initials) = person.initials {
            self.write_element(writer, "nsSAFT:Initials", initials)?;
        }
        if let Some(ref prefix) = person.last_name_prefix {
            self.write_element(writer, "nsSAFT:LastNamePrefix", prefix)?;
        }
        if let Some(ref last_name) = person.last_name {
            self.write_element(writer, "nsSAFT:LastName", last_name)?;
        }
        if let Some(ref birth_name) = person.birth_name {
            self.write_element(writer, "nsSAFT:BirthName", birth_name)?;
        }
        if let Some(ref salutation) = person.salutation {
            self.write_element(writer, "nsSAFT:Salutation", salutation)?;
        }
        for other_title in &person.other_titles {
            self.write_element(writer, "nsSAFT:OtherTitles", other_title)?;
        }

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:ContactPerson")))?;
        Ok(())
    }

    fn write_tax_registration(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        tax_reg: &TaxRegistration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:TaxRegistration")))?;

        self.write_element(
            writer,
            "nsSAFT:TaxRegistrationNumber",
            &tax_reg.tax_registration_number,
        )?;
        self.write_element(writer, "nsSAFT:TaxType", &tax_reg.tax_type)?;
        self.write_element(writer, "nsSAFT:TaxNumber", &tax_reg.tax_number)?;
        if let Some(ref date) = tax_reg.tax_verification_date {
            self.write_element(writer, "nsSAFT:TaxVerificationDate", &date.to_string())?;
        }

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:TaxRegistration")))?;
        Ok(())
    }

    fn write_bank_account(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        bank_acc: &BankAccount,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:BankAccount")))?;

        self.write_element(writer, "nsSAFT:IBANNumber", &bank_acc.iban_number)?;

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:BankAccount")))?;
        Ok(())
    }

    fn write_ownership(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        ownership: &Ownership,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:Ownership")))?;

        self.write_element(writer, "nsSAFT:IsPartOfGroup", &ownership.is_part_of_group)?;
        // Add other ownership fields as needed

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:Ownership")))?;
        Ok(())
    }

    fn write_selection_criteria(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        criteria: &SelectionCriteria,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:SelectionCriteria")))?;

        self.write_element(
            writer,
            "nsSAFT:TaxReportingJurisdiction",
            &criteria.tax_reporting_jurisdiction,
        )?;
        if let Some(ref entity) = criteria.company_entity {
            self.write_element(writer, "nsSAFT:CompanyEntity", entity)?;
        } else {
            self.write_element(writer, "nsSAFT:CompanyEntity", "")?;
        }
        self.write_element(
            writer,
            "nsSAFT:PeriodStart",
            &criteria.period_start.to_string(),
        )?;
        self.write_element(
            writer,
            "nsSAFT:PeriodStartYear",
            &criteria.period_start_year.to_string(),
        )?;
        self.write_element(writer, "nsSAFT:PeriodEnd", &criteria.period_end.to_string())?;
        self.write_element(
            writer,
            "nsSAFT:PeriodEndYear",
            &criteria.period_end_year.to_string(),
        )?;
        if let Some(ref doc_type) = criteria.document_type {
            self.write_element(writer, "nsSAFT:DocumentType", doc_type)?;
        } else {
            self.write_element(writer, "nsSAFT:DocumentType", "")?;
        }
        if let Some(ref other) = criteria.other_criteria {
            self.write_element(writer, "nsSAFT:OtherCriteria", other)?;
        } else {
            self.write_element(writer, "nsSAFT:OtherCriteria", "Няма")?;
        }

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:SelectionCriteria")))?;
        Ok(())
    }

    // Helper methods for writing other sections
    fn write_master_files_monthly(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        master_files: &MasterFilesMonthly,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:MasterFilesMonthly")))?;

        // Write GeneralLedgerAccounts
        self.write_general_ledger_accounts(writer, &master_files.general_ledger_accounts)?;

        // Write Customers
        self.write_customers(writer, &master_files.customers)?;

        // Write Suppliers
        self.write_suppliers(writer, &master_files.suppliers)?;

        // Write TaxTable
        self.write_tax_table(writer, &master_files.tax_table)?;

        // Write UOMTable
        self.write_uom_table(writer, &master_files.uom_table)?;

        // Write Products
        self.write_products(writer, &master_files.products)?;

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:MasterFilesMonthly")))?;
        Ok(())
    }

    fn write_general_ledger_accounts(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        accounts: &GeneralLedgerAccounts,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new(
            "nsSAFT:GeneralLedgerAccounts",
        )))?;

        for account in &accounts.account {
            writer.write_event(Event::Start(BytesStart::new("nsSAFT:Account")))?;

            self.write_element(writer, "nsSAFT:AccountID", &account.account_id.to_string())?;
            self.write_element(
                writer,
                "nsSAFT:AccountDescription",
                &account.account_description,
            )?;
            self.write_element(
                writer,
                "nsSAFT:TaxpayerAccountID",
                &account.taxpayer_account_id,
            )?;
            if let Some(ref category) = account.grouping_category {
                self.write_element(writer, "nsSAFT:GroupingCategory", category)?;
            } else {
                self.write_element(writer, "nsSAFT:GroupingCategory", "")?;
            }
            if let Some(ref code) = account.grouping_code {
                self.write_element(writer, "nsSAFT:GroupingCode", code)?;
            } else {
                self.write_element(writer, "nsSAFT:GroupingCode", "")?;
            }
            self.write_element(writer, "nsSAFT:AccountType", &account.account_type)?;
            if let Some(ref date) = account.account_creation_date {
                self.write_element(writer, "nsSAFT:AccountCreationDate", &date.to_string())?;
            }
            if let Some(ref balance) = account.opening_debit_balance {
                self.write_element(writer, "nsSAFT:OpeningDebitBalance", &balance.to_string())?;
            }
            if let Some(ref balance) = account.opening_credit_balance {
                self.write_element(writer, "nsSAFT:OpeningCreditBalance", &balance.to_string())?;
            }
            if let Some(ref balance) = account.closing_debit_balance {
                self.write_element(writer, "nsSAFT:ClosingDebitBalance", &balance.to_string())?;
            }
            if let Some(ref balance) = account.closing_credit_balance {
                self.write_element(writer, "nsSAFT:ClosingCreditBalance", &balance.to_string())?;
            }

            writer.write_event(Event::End(BytesEnd::new("nsSAFT:Account")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:GeneralLedgerAccounts")))?;
        Ok(())
    }

    fn write_customers(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        customers: &Customers,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:Customers")))?;

        for customer in &customers.customer {
            writer.write_event(Event::Start(BytesStart::new("nsSAFT:Customer")))?;

            self.write_element(writer, "nsSAFT:CustomerID", &customer.customer_id)?;
            self.write_element(writer, "nsSAFT:CustomerName", &customer.customer_name)?;

            // Write addresses
            for addr_wrapper in &customer.addresses {
                writer.write_event(Event::Start(BytesStart::new("nsSAFT:Addresses")))?;
                self.write_address(writer, &addr_wrapper.address)?;
                writer.write_event(Event::End(BytesEnd::new("nsSAFT:Addresses")))?;
            }

            // Write contact
            if let Some(ref contact) = customer.contact {
                self.write_contact(writer, contact)?;
            }

            // Write tax registration
            if let Some(ref tax_reg) = customer.tax_registration {
                self.write_tax_registration(writer, tax_reg)?;
            }

            self.write_element(
                writer,
                "nsSAFT:SelfBillingIndicator",
                &customer.self_billing_indicator,
            )?;

            writer.write_event(Event::End(BytesEnd::new("nsSAFT:Customer")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:Customers")))?;
        Ok(())
    }

    fn write_suppliers(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        suppliers: &Suppliers,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:Suppliers")))?;

        for supplier in &suppliers.supplier {
            writer.write_event(Event::Start(BytesStart::new("nsSAFT:Supplier")))?;

            self.write_element(writer, "nsSAFT:SupplierID", &supplier.supplier_id)?;
            self.write_element(writer, "nsSAFT:SupplierName", &supplier.supplier_name)?;

            // Write addresses
            for addr_wrapper in &supplier.addresses {
                writer.write_event(Event::Start(BytesStart::new("nsSAFT:Addresses")))?;
                self.write_address(writer, &addr_wrapper.address)?;
                writer.write_event(Event::End(BytesEnd::new("nsSAFT:Addresses")))?;
            }

            // Write contact
            if let Some(ref contact) = supplier.contact {
                self.write_contact(writer, contact)?;
            }

            // Write tax registration
            if let Some(ref tax_reg) = supplier.tax_registration {
                self.write_tax_registration(writer, tax_reg)?;
            }

            self.write_element(
                writer,
                "nsSAFT:SelfBillingIndicator",
                &supplier.self_billing_indicator,
            )?;

            writer.write_event(Event::End(BytesEnd::new("nsSAFT:Supplier")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:Suppliers")))?;
        Ok(())
    }

    fn write_tax_table(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        tax_table: &TaxTable,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:TaxTable")))?;

        for tax_code in &tax_table.tax_code_details {
            writer.write_event(Event::Start(BytesStart::new("nsSAFT:TaxCodeDetails")))?;

            self.write_element(writer, "nsSAFT:TaxCode", &tax_code.tax_code)?;
            self.write_element(
                writer,
                "nsSAFT:EffectiveDate",
                &tax_code.effective_date.to_string(),
            )?;
            if let Some(ref date) = tax_code.expiration_date {
                self.write_element(writer, "nsSAFT:ExpirationDate", &date.to_string())?;
            }
            self.write_element(writer, "nsSAFT:Description", &tax_code.description)?;
            if let Some(ref percentage) = tax_code.tax_percentage {
                self.write_element(writer, "nsSAFT:TaxPercentage", &percentage.to_string())?;
            }
            self.write_element(writer, "nsSAFT:Country", &tax_code.country)?;

            writer.write_event(Event::End(BytesEnd::new("nsSAFT:TaxCodeDetails")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:TaxTable")))?;
        Ok(())
    }

    fn write_uom_table(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        uom_table: &UOMTable,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:UOMTable")))?;

        for uom in &uom_table.uom_entry {
            writer.write_event(Event::Start(BytesStart::new("nsSAFT:UOMEntry")))?;

            self.write_element(writer, "nsSAFT:UnitOfMeasure", &uom.unit_of_measure)?;
            self.write_element(writer, "nsSAFT:UOMDescription", &uom.uom_description)?;

            writer.write_event(Event::End(BytesEnd::new("nsSAFT:UOMEntry")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:UOMTable")))?;
        Ok(())
    }

    fn write_products(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        products: &Products,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:Products")))?;

        for product in &products.product {
            writer.write_event(Event::Start(BytesStart::new("nsSAFT:Product")))?;

            self.write_element(writer, "nsSAFT:ProductType", &product.product_type)?;
            self.write_element(writer, "nsSAFT:ProductCode", &product.product_code)?;
            self.write_element(writer, "nsSAFT:Description", &product.description)?;

            writer.write_event(Event::End(BytesEnd::new("nsSAFT:Product")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:Products")))?;
        Ok(())
    }

    fn write_general_ledger_entries(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        gl_entries: &GeneralLedgerEntries,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:GeneralLedgerEntries")))?;

        self.write_element(
            writer,
            "nsSAFT:NumberOfEntries",
            &gl_entries.number_of_entries.to_string(),
        )?;
        self.write_element(
            writer,
            "nsSAFT:TotalDebit",
            &gl_entries.total_debit.to_string(),
        )?;
        self.write_element(
            writer,
            "nsSAFT:TotalCredit",
            &gl_entries.total_credit.to_string(),
        )?;

        for journal in &gl_entries.journal {
            writer.write_event(Event::Start(BytesStart::new("nsSAFT:Journal")))?;

            self.write_element(writer, "nsSAFT:JournalID", &journal.journal_id)?;
            self.write_element(writer, "nsSAFT:Description", &journal.description)?;
            self.write_element(writer, "nsSAFT:Type", &journal.journal_type)?;

            for transaction in &journal.transaction {
                writer.write_event(Event::Start(BytesStart::new("nsSAFT:Transaction")))?;

                self.write_element(writer, "nsSAFT:TransactionID", &transaction.transaction_id)?;
                self.write_element(writer, "nsSAFT:Period", &transaction.period.to_string())?;
                self.write_element(
                    writer,
                    "nsSAFT:PeriodYear",
                    &transaction.period_year.to_string(),
                )?;
                self.write_element(
                    writer,
                    "nsSAFT:TransactionDate",
                    &transaction.transaction_date.to_string(),
                )?;
                if let Some(ref source_id) = transaction.source_id {
                    self.write_element(writer, "nsSAFT:SourceID", source_id)?;
                }
                self.write_element(writer, "nsSAFT:Description", &transaction.description)?;
                self.write_element(
                    writer,
                    "nsSAFT:SystemEntryDate",
                    &transaction.system_entry_date.to_string(),
                )?;
                self.write_element(
                    writer,
                    "nsSAFT:GLPostingDate",
                    &transaction.gl_posting_date.to_string(),
                )?;

                // Write Lines
                writer.write_event(Event::Start(BytesStart::new("nsSAFT:Lines")))?;

                // Write Debit Lines
                for line in &transaction.lines.debit_line {
                    writer.write_event(Event::Start(BytesStart::new("nsSAFT:DebitLine")))?;
                    self.write_transaction_line(writer, line)?;
                    writer.write_event(Event::End(BytesEnd::new("nsSAFT:DebitLine")))?;
                }

                // Write Credit Lines
                for line in &transaction.lines.credit_line {
                    writer.write_event(Event::Start(BytesStart::new("nsSAFT:CreditLine")))?;
                    self.write_transaction_line(writer, line)?;
                    writer.write_event(Event::End(BytesEnd::new("nsSAFT:CreditLine")))?;
                }

                writer.write_event(Event::End(BytesEnd::new("nsSAFT:Lines")))?;
                writer.write_event(Event::End(BytesEnd::new("nsSAFT:Transaction")))?;
            }

            writer.write_event(Event::End(BytesEnd::new("nsSAFT:Journal")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("nsSAFT:GeneralLedgerEntries")))?;
        Ok(())
    }

    fn write_transaction_line(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        line: &SafTTransactionLine,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.write_element(writer, "nsSAFT:RecordID", &line.record_id)?;
        self.write_element(writer, "nsSAFT:AccountID", &line.account_id.to_string())?;

        if let Some(ref taxpayer_id) = line.taxpayer_account_id {
            self.write_element(writer, "nsSAFT:TaxpayerAccountID", taxpayer_id)?;
        }

        if let Some(ref date) = line.value_date {
            self.write_element(writer, "nsSAFT:ValueDate", &date.to_string())?;
        }

        if let Some(ref doc_id) = line.source_document_id {
            self.write_element(writer, "nsSAFT:SourceDocumentID", doc_id)?;
        }

        if let Some(ref customer_id) = line.customer_id {
            self.write_element(writer, "nsSAFT:CustomerID", customer_id)?;
        }

        if let Some(ref supplier_id) = line.supplier_id {
            self.write_element(writer, "nsSAFT:SupplierID", supplier_id)?;
        }

        self.write_element(writer, "nsSAFT:Description", &line.description)?;

        if let Some(ref amount) = line.debit_amount {
            self.write_amount_structure(writer, "nsSAFT:DebitAmount", amount)?;
        }

        if let Some(ref amount) = line.credit_amount {
            self.write_amount_structure(writer, "nsSAFT:CreditAmount", amount)?;
        }

        Ok(())
    }

    fn write_amount_structure(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        tag: &str,
        amount: &AmountStructure,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new(tag)))?;

        self.write_element(writer, "nsSAFT:Amount", &amount.amount.to_string())?;

        if let Some(ref currency) = amount.currency_code {
            self.write_element(writer, "nsSAFT:CurrencyCode", currency)?;
        }

        if let Some(ref curr_amount) = amount.currency_amount {
            self.write_element(writer, "nsSAFT:CurrencyAmount", &curr_amount.to_string())?;
        }

        if let Some(ref rate) = amount.exchange_rate {
            self.write_element(writer, "nsSAFT:ExchangeRate", &rate.to_string())?;
        }

        writer.write_event(Event::End(BytesEnd::new(tag)))?;
        Ok(())
    }

    fn write_corresponding_accounts_report(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        _report: &CorrespondingAccountsReport,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Implementation for Corresponding Accounts Report
        writer.write_event(Event::Start(BytesStart::new(
            "nsSAFT:CorrespondingAccountsReport",
        )))?;
        // TODO: Write report entries
        writer.write_event(Event::End(BytesEnd::new(
            "nsSAFT:CorrespondingAccountsReport",
        )))?;
        Ok(())
    }

    fn write_source_documents_monthly(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        _docs: &SourceDocumentsMonthly,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new(
            "nsSAFT:SourceDocumentsMonthly",
        )))?;
        // TODO: Write source documents
        writer.write_event(Event::End(BytesEnd::new("nsSAFT:SourceDocumentsMonthly")))?;
        Ok(())
    }

    fn write_master_files_annual(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        _master_files: &MasterFilesAnnual,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:MasterFilesAnnual")))?;
        // TODO: Write annual master files
        writer.write_event(Event::End(BytesEnd::new("nsSAFT:MasterFilesAnnual")))?;
        Ok(())
    }

    fn write_source_documents_annual(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        _docs: &SourceDocumentsAnnual,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new(
            "nsSAFT:SourceDocumentsAnnual",
        )))?;
        // TODO: Write annual source documents
        writer.write_event(Event::End(BytesEnd::new("nsSAFT:SourceDocumentsAnnual")))?;
        Ok(())
    }

    fn write_master_files_on_demand(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        _master_files: &MasterFilesOnDemand,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new("nsSAFT:MasterFilesOnDemand")))?;
        // TODO: Write on-demand master files
        writer.write_event(Event::End(BytesEnd::new("nsSAFT:MasterFilesOnDemand")))?;
        Ok(())
    }

    fn write_source_documents_on_demand(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        _docs: &SourceDocumentsOnDemand,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new(
            "nsSAFT:SourceDocumentsOnDemand",
        )))?;
        // TODO: Write on-demand source documents
        writer.write_event(Event::End(BytesEnd::new("nsSAFT:SourceDocumentsOnDemand")))?;
        Ok(())
    }

    fn write_element(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        tag: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        writer.write_event(Event::Start(BytesStart::new(tag)))?;
        writer.write_event(Event::Text(BytesText::new(value)))?;
        writer.write_event(Event::End(BytesEnd::new(tag)))?;
        Ok(())
    }
}
