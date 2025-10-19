use anyhow::Result;
use chrono::Utc;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use sea_orm::{QueryFilter, *};
use std::io::Cursor;

use crate::entities::{company, intrastat_declaration, intrastat_declaration_item};

#[allow(dead_code)]
pub struct IntrastatXmlExporter {
    db: DatabaseConnection,
}

impl IntrastatXmlExporter {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn export_declaration(&self, declaration_id: i32) -> Result<String> {
        let declaration = intrastat_declaration::Entity::find_by_id(declaration_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Declaration not found"))?;

        let company = company::Entity::find_by_id(declaration.company_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Company not found"))?;

        let items = intrastat_declaration_item::Entity::find()
            .filter(intrastat_declaration_item::Column::DeclarationId.eq(declaration_id))
            .order_by_asc(intrastat_declaration_item::Column::ItemNumber)
            .all(&self.db)
            .await?;

        let mut writer = Writer::new(Cursor::new(Vec::new()));

        writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            None,
        )))?;

        let mut instat_elem = BytesStart::new("INSTAT");
        instat_elem.push_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"));
        instat_elem.push_attribute((
            "xsi:schemaLocation",
            "http://www.intrastat.bg/instat/INSTAT.xsd",
        ));
        writer.write_event(Event::Start(instat_elem))?;

        self.write_envelope(&mut writer, &declaration, &company)?;

        let declaration_type = match declaration.declaration_type {
            intrastat_declaration::DeclarationType::Arrival => "INSTAT_A",
            intrastat_declaration::DeclarationType::Dispatch => "INSTAT_D",
        };

        writer.write_event(Event::Start(BytesStart::new(declaration_type)))?;

        for item in items {
            self.write_item(&mut writer, &item, &declaration)?;
        }

        writer.write_event(Event::End(BytesEnd::new(declaration_type)))?;

        writer.write_event(Event::End(BytesEnd::new("INSTAT")))?;

        let result = writer.into_inner().into_inner();
        Ok(String::from_utf8(result)?)
    }

    fn write_envelope(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        declaration: &intrastat_declaration::Model,
        company: &company::Model,
    ) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("Envelope")))?;

        self.write_element(
            writer,
            "envelopeId",
            &format!(
                "BG{}{}",
                company.vat_number.as_ref().unwrap_or(&"".to_string()),
                declaration.reference_period
            ),
        )?;

        let now = Utc::now();
        self.write_element(
            writer,
            "DateTime",
            &now.format("%Y-%m-%dT%H:%M:%S").to_string(),
        )?;

        writer.write_event(Event::Start(BytesStart::new("Party")))?;
        self.write_element(writer, "partyType", "PSI")?;
        self.write_element(
            writer,
            "partyId",
            company.vat_number.as_ref().unwrap_or(&"".to_string()),
        )?;
        self.write_element(writer, "partyName", &company.name)?;
        writer.write_event(Event::End(BytesEnd::new("Party")))?;

        self.write_element(writer, "softwareUsed", "Accounting System v1.0")?;

        writer.write_event(Event::Start(BytesStart::new("Declaration")))?;
        self.write_element(
            writer,
            "declarationId",
            &format!("{}_{}", declaration.id, declaration.reference_period),
        )?;
        self.write_element(writer, "referencePeriod", &declaration.reference_period)?;
        self.write_element(
            writer,
            "PSIId",
            company.vat_number.as_ref().unwrap_or(&"".to_string()),
        )?;

        let function_code = match declaration.declaration_type {
            intrastat_declaration::DeclarationType::Arrival => "A",
            intrastat_declaration::DeclarationType::Dispatch => "D",
        };
        self.write_element(writer, "Function", &format!("O+{}", function_code))?;

        self.write_element(
            writer,
            "declarationTypeCode",
            if declaration.declaration_type == intrastat_declaration::DeclarationType::Arrival {
                "19"
            } else {
                "29"
            },
        )?;
        self.write_element(writer, "flowCode", function_code)?;
        self.write_element(writer, "currencyCode", "BGN")?;
        self.write_element(
            writer,
            "totalInvoicedAmount",
            &declaration.total_invoice_value.to_string(),
        )?;
        self.write_element(
            writer,
            "totalStatisticalValue",
            &declaration.total_statistical_value.to_string(),
        )?;
        self.write_element(writer, "totalItems", &declaration.total_items.to_string())?;

        writer.write_event(Event::End(BytesEnd::new("Declaration")))?;

        writer.write_event(Event::End(BytesEnd::new("Envelope")))?;

        Ok(())
    }

    fn write_item(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        item: &intrastat_declaration_item::Model,
        declaration: &intrastat_declaration::Model,
    ) -> Result<()> {
        let item_tag =
            if declaration.declaration_type == intrastat_declaration::DeclarationType::Arrival {
                "ITEM_A"
            } else {
                "ITEM_D"
            };

        writer.write_event(Event::Start(BytesStart::new(item_tag)))?;

        self.write_element(writer, "itemNumber", &item.item_number.to_string())?;

        writer.write_event(Event::Start(BytesStart::new("CN8")))?;
        writer.write_event(Event::Start(BytesStart::new("CN8Code")))?;
        writer.write_event(Event::Text(BytesText::new(&item.cn_code)))?;
        writer.write_event(Event::End(BytesEnd::new("CN8Code")))?;
        writer.write_event(Event::End(BytesEnd::new("CN8")))?;

        if declaration.declaration_type == intrastat_declaration::DeclarationType::Arrival {
            self.write_element(writer, "MSConsDestCode", &item.country_of_consignment)?;
        } else {
            self.write_element(writer, "MSConsDestCode", &item.country_of_consignment)?;
        }

        self.write_element(writer, "countryOfOriginCode", &item.country_of_origin)?;
        self.write_element(writer, "netMass", &item.net_mass_kg.to_string())?;

        if let Some(sup_unit) = &item.supplementary_unit {
            self.write_element(writer, "supplementaryUnit", &sup_unit.to_string())?;
        }

        self.write_element(writer, "invoicedAmount", &item.invoice_value.to_string())?;
        self.write_element(
            writer,
            "statisticalValue",
            &item.statistical_value.to_string(),
        )?;

        writer.write_event(Event::Start(BytesStart::new("NatureOfTransaction")))?;
        self.write_element(
            writer,
            "natureOfTransactionACode",
            &item
                .transaction_nature_code
                .chars()
                .take(1)
                .collect::<String>(),
        )?;
        if item.transaction_nature_code.len() > 1 {
            self.write_element(
                writer,
                "natureOfTransactionBCode",
                &item
                    .transaction_nature_code
                    .chars()
                    .skip(1)
                    .take(1)
                    .collect::<String>(),
            )?;
        }
        writer.write_event(Event::End(BytesEnd::new("NatureOfTransaction")))?;

        self.write_element(
            writer,
            "modeOfTransportCode",
            &item.transport_mode.to_string(),
        )?;
        self.write_element(writer, "deliveryTerms", &item.delivery_terms)?;

        if let Some(region) = &item.region_code {
            self.write_element(writer, "regionCode", region)?;
        }

        writer.write_event(Event::End(BytesEnd::new(item_tag)))?;

        Ok(())
    }

    fn write_element(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        name: &str,
        value: &str,
    ) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new(name)))?;
        writer.write_event(Event::Text(BytesText::new(value)))?;
        writer.write_event(Event::End(BytesEnd::new(name)))?;
        Ok(())
    }

    pub async fn validate_declaration(&self, declaration_id: i32) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        let declaration = intrastat_declaration::Entity::find_by_id(declaration_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Declaration not found"))?;

        if declaration.declarant_eik.is_empty() {
            errors.push("ЕИК на декларатора липсва".to_string());
        }

        if declaration.contact_person.is_empty() {
            errors.push("Лице за контакт липсва".to_string());
        }

        if declaration.contact_phone.is_empty() {
            errors.push("Телефон за контакт липсва".to_string());
        }

        let items = intrastat_declaration_item::Entity::find()
            .filter(intrastat_declaration_item::Column::DeclarationId.eq(declaration_id))
            .all(&self.db)
            .await?;

        if items.is_empty() {
            errors.push("Декларацията няма артикули".to_string());
        }

        for (idx, item) in items.iter().enumerate() {
            if item.cn_code.len() != 8 {
                errors.push(format!("Артикул {}: CN кодът трябва да е 8 цифри", idx + 1));
            }

            if item.country_of_origin.len() != 2 {
                errors.push(format!(
                    "Артикул {}: Кодът на страната на произход трябва да е 2 символа",
                    idx + 1
                ));
            }

            if item.net_mass_kg <= "0".parse().unwrap() {
                errors.push(format!(
                    "Артикул {}: Нетното тегло трябва да е положително",
                    idx + 1
                ));
            }

            if item.invoice_value <= "0".parse().unwrap() {
                errors.push(format!(
                    "Артикул {}: Фактурната стойност трябва да е положителна",
                    idx + 1
                ));
            }
        }

        Ok(errors)
    }
}
