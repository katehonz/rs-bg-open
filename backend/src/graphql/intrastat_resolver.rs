use crate::entities::{
    intrastat_account_mapping, intrastat_declaration, intrastat_declaration_item,
    intrastat_nomenclature, intrastat_settings,
};
use crate::services::intrastat_service::IntrastatService;
use crate::services::intrastat_xml_export::IntrastatXmlExporter;
use async_graphql::{Context, Object, Result};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
};

pub struct IntrastatQuery;

#[Object]
impl IntrastatQuery {
    async fn intrastat_settings(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
    ) -> Result<Option<intrastat_settings::Model>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let service = IntrastatService::new(db.clone());
        Ok(service.get_company_settings(company_id).await?)
    }

    async fn intrastat_nomenclatures(
        &self,
        ctx: &Context<'_>,
        search: Option<String>,
        limit: Option<i32>,
    ) -> Result<Vec<intrastat_nomenclature::Model>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let mut query = intrastat_nomenclature::Entity::find();

        if let Some(search_term) = search {
            query = query.filter(
                intrastat_nomenclature::Column::CnCode
                    .contains(&search_term)
                    .or(intrastat_nomenclature::Column::DescriptionBg.contains(&search_term)),
            );
        }

        if let Some(limit_val) = limit {
            query = query.limit(limit_val as u64);
        }

        Ok(query.all(db).await?)
    }

    async fn intrastat_account_mappings(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
    ) -> Result<Vec<intrastat_account_mapping::Model>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let service = IntrastatService::new(db.clone());
        Ok(service.get_account_mappings(company_id).await?)
    }

    async fn intrastat_declarations(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        year: Option<i32>,
        month: Option<i32>,
    ) -> Result<Vec<intrastat_declaration::Model>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let service = IntrastatService::new(db.clone());
        Ok(service.get_declarations(company_id, year, month).await?)
    }

    async fn intrastat_declaration_items(
        &self,
        ctx: &Context<'_>,
        declaration_id: i32,
    ) -> Result<Vec<intrastat_declaration_item::Model>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let items = intrastat_declaration_item::Entity::find()
            .filter(intrastat_declaration_item::Column::DeclarationId.eq(declaration_id))
            .all(db)
            .await?;
        Ok(items)
    }

    async fn check_intrastat_threshold(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
    ) -> Result<ThresholdStatus> {
        let db = ctx.data::<DatabaseConnection>()?;
        let service = IntrastatService::new(db.clone());
        let (arrival_exceeded, dispatch_exceeded) =
            service.check_threshold_exceeded(company_id).await?;

        Ok(ThresholdStatus {
            arrival_exceeded,
            dispatch_exceeded,
        })
    }
}

pub struct IntrastatMutation;

#[Object]
impl IntrastatMutation {
    async fn initialize_intrastat_settings(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
    ) -> Result<intrastat_settings::Model> {
        let db = ctx.data::<DatabaseConnection>()?;
        let service = IntrastatService::new(db.clone());
        Ok(service.initialize_company_settings(company_id).await?)
    }

    async fn update_intrastat_settings(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: intrastat_settings::UpdateIntrastatSettingsInput,
    ) -> Result<intrastat_settings::Model> {
        let db = ctx.data::<DatabaseConnection>()?;

        let mut settings: intrastat_settings::ActiveModel =
            intrastat_settings::Entity::find_by_id(id)
                .one(db)
                .await?
                .ok_or("Settings not found")?
                .into();

        if let Some(enabled) = input.is_enabled {
            settings.is_enabled = Set(enabled);
        }
        if let Some(threshold) = input.arrival_threshold_bgn {
            settings.arrival_threshold_bgn = Set(threshold);
        }
        if let Some(threshold) = input.dispatch_threshold_bgn {
            settings.dispatch_threshold_bgn = Set(threshold);
        }
        if let Some(auto) = input.auto_generate_declarations {
            settings.auto_generate_declarations = Set(auto);
        }
        if let Some(transport) = input.default_transport_mode {
            settings.default_transport_mode = Set(Some(transport));
        }
        if let Some(terms) = input.default_delivery_terms {
            settings.default_delivery_terms = Set(Some(terms));
        }
        if let Some(nature) = input.default_transaction_nature {
            settings.default_transaction_nature = Set(Some(nature));
        }
        if let Some(name) = input.responsible_person_name {
            settings.responsible_person_name = Set(Some(name));
        }
        if let Some(phone) = input.responsible_person_phone {
            settings.responsible_person_phone = Set(Some(phone));
        }
        if let Some(email) = input.responsible_person_email {
            settings.responsible_person_email = Set(Some(email));
        }

        Ok(settings.update(&*db).await?)
    }

    async fn import_intrastat_nomenclature(
        &self,
        ctx: &Context<'_>,
        csv_data: String,
    ) -> Result<ImportResult> {
        let db = ctx.data::<DatabaseConnection>()?;
        let service = IntrastatService::new(db.clone());
        let imported = service.import_nomenclature_from_csv(&csv_data).await?;

        Ok(ImportResult {
            success: true,
            imported_count: imported.len() as i32,
            message: format!("Импортирани {} номенклатури", imported.len()),
        })
    }

    async fn create_intrastat_account_mapping(
        &self,
        ctx: &Context<'_>,
        input: intrastat_account_mapping::CreateIntrastatAccountMappingInput,
    ) -> Result<intrastat_account_mapping::Model> {
        let db = ctx.data::<DatabaseConnection>()?;
        let service = IntrastatService::new(db.clone());
        Ok(service
            .create_account_mapping(
                input.account_id,
                input.nomenclature_id,
                input.flow_direction,
                input.transaction_nature_code,
                input.company_id,
            )
            .await?)
    }

    async fn delete_intrastat_account_mapping(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let db = ctx.data::<DatabaseConnection>()?;
        intrastat_account_mapping::Entity::delete_by_id(id)
            .exec(db)
            .await?;
        Ok(true)
    }

    async fn create_intrastat_declaration(
        &self,
        ctx: &Context<'_>,
        input: intrastat_declaration::CreateIntrastatDeclarationInput,
    ) -> Result<intrastat_declaration::Model> {
        let db = ctx.data::<DatabaseConnection>()?;
        let service = IntrastatService::new(db.clone());

        let user_id = 1;

        Ok(service
            .create_declaration(
                input.company_id,
                input.declaration_type,
                input.year,
                input.month,
                user_id,
            )
            .await?)
    }

    async fn update_intrastat_declaration(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: intrastat_declaration::UpdateIntrastatDeclarationInput,
    ) -> Result<intrastat_declaration::Model> {
        let db = ctx.data::<DatabaseConnection>()?;

        let mut declaration: intrastat_declaration::ActiveModel =
            intrastat_declaration::Entity::find_by_id(id)
                .one(db)
                .await?
                .ok_or("Declaration not found")?
                .into();

        if let Some(name) = input.declarant_name {
            declaration.declarant_name = Set(name);
        }
        if let Some(person) = input.contact_person {
            declaration.contact_person = Set(person);
        }
        if let Some(phone) = input.contact_phone {
            declaration.contact_phone = Set(phone);
        }
        if let Some(email) = input.contact_email {
            declaration.contact_email = Set(email);
        }
        if let Some(status) = input.status {
            declaration.status = Set(status);
        }

        Ok(declaration.update(&*db).await?)
    }

    async fn generate_declaration_items(
        &self,
        ctx: &Context<'_>,
        declaration_id: i32,
    ) -> Result<GenerateResult> {
        let db = ctx.data::<DatabaseConnection>()?;
        let service = IntrastatService::new(db.clone());

        let declaration = intrastat_declaration::Entity::find_by_id(declaration_id)
            .one(db)
            .await?
            .ok_or("Declaration not found")?;

        let items = service
            .generate_declaration_items_from_entries(
                declaration_id,
                declaration.year,
                declaration.month as i32,
            )
            .await?;

        Ok(GenerateResult {
            success: true,
            items_count: items.len() as i32,
            message: format!("Генерирани {} артикула", items.len()),
        })
    }

    async fn export_intrastat_xml(&self, ctx: &Context<'_>, declaration_id: i32) -> Result<String> {
        let db = ctx.data::<DatabaseConnection>()?;
        let exporter = IntrastatXmlExporter::new(db.clone());
        Ok(exporter.export_declaration(declaration_id).await?)
    }

    async fn validate_intrastat_declaration(
        &self,
        ctx: &Context<'_>,
        declaration_id: i32,
    ) -> Result<ValidationResult> {
        let db = ctx.data::<DatabaseConnection>()?;
        let exporter = IntrastatXmlExporter::new(db.clone());
        let errors = exporter.validate_declaration(declaration_id).await?;

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
        })
    }
}

#[derive(async_graphql::SimpleObject)]
struct ThresholdStatus {
    arrival_exceeded: bool,
    dispatch_exceeded: bool,
}

#[derive(async_graphql::SimpleObject)]
struct ImportResult {
    success: bool,
    imported_count: i32,
    message: String,
}

#[derive(async_graphql::SimpleObject)]
struct GenerateResult {
    success: bool,
    items_count: i32,
    message: String,
}

#[derive(async_graphql::SimpleObject)]
struct ValidationResult {
    is_valid: bool,
    errors: Vec<String>,
}
