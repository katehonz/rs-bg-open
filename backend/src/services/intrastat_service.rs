use anyhow::Result;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, QueryFilter, QueryOrder, *};

use crate::entities::{
    company, entry_line, intrastat_account_mapping, intrastat_declaration,
    intrastat_declaration_item, intrastat_nomenclature, intrastat_settings, journal_entry,
};

#[allow(dead_code)]
pub struct IntrastatService {
    db: DatabaseConnection,
}

impl IntrastatService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_company_settings(
        &self,
        company_id: i32,
    ) -> Result<Option<intrastat_settings::Model>> {
        let settings = intrastat_settings::Entity::find()
            .filter(intrastat_settings::Column::CompanyId.eq(company_id))
            .one(&self.db)
            .await?;

        Ok(settings)
    }

    pub async fn initialize_company_settings(
        &self,
        company_id: i32,
    ) -> Result<intrastat_settings::Model> {
        let existing = self.get_company_settings(company_id).await?;

        if let Some(settings) = existing {
            return Ok(settings);
        }

        let new_settings = intrastat_settings::ActiveModel {
            company_id: Set(company_id),
            is_enabled: Set(false),
            arrival_threshold_bgn: Set("400000.00".parse().unwrap()),
            dispatch_threshold_bgn: Set("400000.00".parse().unwrap()),
            current_arrival_threshold_bgn: Set("0.00".parse().unwrap()),
            current_dispatch_threshold_bgn: Set("0.00".parse().unwrap()),
            auto_generate_declarations: Set(false),
            ..Default::default()
        };

        let settings = new_settings.insert(&self.db).await?;
        Ok(settings)
    }

    pub async fn import_nomenclature_from_csv(
        &self,
        csv_data: &str,
    ) -> Result<Vec<intrastat_nomenclature::Model>> {
        let mut imported = Vec::new();
        let mut reader = csv::Reader::from_reader(csv_data.as_bytes());

        for result in reader.records() {
            let record = result?;
            if record.len() >= 4 {
                let cn_code = record.get(0).unwrap_or("").trim();
                let description = record.get(1).unwrap_or("").trim();
                let unit = record.get(2).unwrap_or("").trim();
                let unit_desc = record.get(3).unwrap_or("").trim();

                if !cn_code.is_empty() && !description.is_empty() {
                    let nomenclature = intrastat_nomenclature::ActiveModel {
                        cn_code: Set(cn_code.to_string()),
                        description_bg: Set(description.to_string()),
                        unit_of_measure: Set(unit.to_string()),
                        unit_description: Set(unit_desc.to_string()),
                        level: Set(cn_code.len() as i32 / 2),
                        is_active: Set(true),
                        ..Default::default()
                    };

                    let inserted = nomenclature.insert(&self.db).await?;
                    imported.push(inserted);
                }
            }
        }

        Ok(imported)
    }

    pub async fn create_account_mapping(
        &self,
        account_id: i32,
        nomenclature_id: i32,
        flow_direction: intrastat_account_mapping::FlowDirection,
        transaction_nature_code: String,
        company_id: i32,
    ) -> Result<intrastat_account_mapping::Model> {
        let mapping = intrastat_account_mapping::ActiveModel {
            account_id: Set(account_id),
            nomenclature_id: Set(nomenclature_id),
            flow_direction: Set(flow_direction),
            transaction_nature_code: Set(transaction_nature_code),
            is_quantity_tracked: Set(true),
            is_optional: Set(true),
            company_id: Set(company_id),
            ..Default::default()
        };

        let result = mapping.insert(&self.db).await?;
        Ok(result)
    }

    pub async fn get_account_mappings(
        &self,
        company_id: i32,
    ) -> Result<Vec<intrastat_account_mapping::Model>> {
        let mappings = intrastat_account_mapping::Entity::find()
            .filter(intrastat_account_mapping::Column::CompanyId.eq(company_id))
            .all(&self.db)
            .await?;

        Ok(mappings)
    }

    pub async fn create_declaration(
        &self,
        company_id: i32,
        declaration_type: intrastat_declaration::DeclarationType,
        year: i32,
        month: i32,
        user_id: i32,
    ) -> Result<intrastat_declaration::Model> {
        let company = company::Entity::find_by_id(company_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Company not found"))?;

        let reference_period = format!("{:04}{:02}", year, month);

        let declaration = intrastat_declaration::ActiveModel {
            company_id: Set(company_id),
            declaration_type: Set(declaration_type),
            reference_period: Set(reference_period),
            year: Set(year),
            month: Set(month),
            declarant_eik: Set(company.vat_number.unwrap_or_default()),
            declarant_name: Set(company.name),
            contact_person: Set("".to_string()),
            contact_phone: Set("".to_string()),
            contact_email: Set("".to_string()),
            status: Set(intrastat_declaration::DeclarationStatus::Draft),
            created_by: Set(user_id),
            ..Default::default()
        };

        let result = declaration.insert(&self.db).await?;
        Ok(result)
    }

    pub async fn get_declarations(
        &self,
        company_id: i32,
        year: Option<i32>,
        month: Option<i32>,
    ) -> Result<Vec<intrastat_declaration::Model>> {
        let mut query = intrastat_declaration::Entity::find()
            .filter(intrastat_declaration::Column::CompanyId.eq(company_id));

        if let Some(y) = year {
            query = query.filter(intrastat_declaration::Column::Year.eq(y));
        }

        if let Some(m) = month {
            query = query.filter(intrastat_declaration::Column::Month.eq(m));
        }

        let declarations = query
            .order_by_desc(intrastat_declaration::Column::Year)
            .order_by_desc(intrastat_declaration::Column::Month)
            .all(&self.db)
            .await?;

        Ok(declarations)
    }

    pub async fn generate_declaration_items_from_entries(
        &self,
        declaration_id: i32,
        year: i32,
        month: i32,
    ) -> Result<Vec<intrastat_declaration_item::Model>> {
        let declaration = intrastat_declaration::Entity::find_by_id(declaration_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Declaration not found"))?;

        let start_date = NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap();
        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1).unwrap()
        };

        let journal_entries = journal_entry::Entity::find()
            .filter(journal_entry::Column::CompanyId.eq(declaration.company_id))
            .filter(journal_entry::Column::AccountingDate.gte(start_date))
            .filter(journal_entry::Column::AccountingDate.lt(end_date))
            .all(&self.db)
            .await?;

        let mut items = Vec::new();
        let mut item_number = 1;

        for entry in journal_entries {
            let entry_lines = entry_line::Entity::find()
                .filter(entry_line::Column::JournalEntryId.eq(entry.id))
                .all(&self.db)
                .await?;

            for line in entry_lines {
                let account_id = line.account_id;
                let mappings = intrastat_account_mapping::Entity::find()
                    .filter(intrastat_account_mapping::Column::AccountId.eq(account_id))
                    .filter(intrastat_account_mapping::Column::CompanyId.eq(declaration.company_id))
                    .all(&self.db)
                    .await?;

                for mapping in mappings {
                    let flow_matches = match declaration.declaration_type {
                        intrastat_declaration::DeclarationType::Arrival => {
                            mapping.flow_direction
                                == intrastat_account_mapping::FlowDirection::Arrival
                        }
                        intrastat_declaration::DeclarationType::Dispatch => {
                            mapping.flow_direction
                                == intrastat_account_mapping::FlowDirection::Dispatch
                        }
                    };

                    if flow_matches {
                        let nomenclature =
                            intrastat_nomenclature::Entity::find_by_id(mapping.nomenclature_id)
                                .one(&self.db)
                                .await?;

                        if let Some(nom) = nomenclature {
                            let item = intrastat_declaration_item::ActiveModel {
                                declaration_id: Set(declaration_id),
                                item_number: Set(item_number),
                                cn_code: Set(nom.cn_code.clone()),
                                nomenclature_id: Set(Some(nom.id)),
                                country_of_origin: Set(mapping
                                    .default_country_code
                                    .clone()
                                    .unwrap_or("BG".to_string())),
                                country_of_consignment: Set("BG".to_string()),
                                transaction_nature_code: Set(mapping
                                    .transaction_nature_code
                                    .clone()),
                                transport_mode: Set(mapping.default_transport_mode.unwrap_or(3)),
                                delivery_terms: Set("EXW".to_string()),
                                net_mass_kg: Set("0.000".parse().unwrap()),
                                invoice_value: Set(line.debit_amount),
                                statistical_value: Set(line.debit_amount),
                                currency_code: Set("BGN".to_string()),
                                description: Set(nom.description_bg),
                                journal_entry_id: Set(Some(entry.id)),
                                entry_line_id: Set(Some(line.id)),
                                ..Default::default()
                            };

                            let inserted = item.insert(&self.db).await?;
                            items.push(inserted);
                            item_number += 1;
                        }
                    }
                }
            }
        }

        self.update_declaration_totals(declaration_id).await?;

        Ok(items)
    }

    pub async fn update_declaration_totals(&self, declaration_id: i32) -> Result<()> {
        let items = intrastat_declaration_item::Entity::find()
            .filter(intrastat_declaration_item::Column::DeclarationId.eq(declaration_id))
            .all(&self.db)
            .await?;

        let total_items = items.len() as i32;
        let total_invoice: Decimal = items.iter().map(|i| i.invoice_value).sum();
        let total_statistical: Decimal = items.iter().map(|i| i.statistical_value).sum();

        let mut declaration: intrastat_declaration::ActiveModel =
            intrastat_declaration::Entity::find_by_id(declaration_id)
                .one(&self.db)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Declaration not found"))?
                .into();

        declaration.total_items = Set(total_items);
        declaration.total_invoice_value = Set(total_invoice);
        declaration.total_statistical_value = Set(total_statistical);

        declaration.update(&self.db).await?;

        Ok(())
    }

    pub async fn check_threshold_exceeded(&self, company_id: i32) -> Result<(bool, bool)> {
        let settings = self
            .get_company_settings(company_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Settings not found"))?;

        let arrival_exceeded =
            settings.current_arrival_threshold_bgn >= settings.arrival_threshold_bgn;
        let dispatch_exceeded =
            settings.current_dispatch_threshold_bgn >= settings.dispatch_threshold_bgn;

        Ok((arrival_exceeded, dispatch_exceeded))
    }
}
