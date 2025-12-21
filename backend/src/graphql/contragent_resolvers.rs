use std::sync::Arc;

use async_graphql::{Context, Enum, FieldResult, Object, SimpleObject};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::entities::{
    contragent_setting, global_contragent, ContragentSetting, ContragentSettingModel,
    GlobalContragent, GlobalContragentFilter, GlobalContragentModel, GlobalContragentSummary,
    UpsertContragentSettingInput,
};
use crate::services::contragent::{AddressComponents, ContragentDataSource, ContragentService};

#[derive(Default)]
pub struct ContragentQuery;

#[Object]
impl ContragentQuery {
    async fn global_contragents(
        &self,
        ctx: &Context<'_>,
        filter: Option<GlobalContragentFilter>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> FieldResult<Vec<GlobalContragentModel>> {
        let db = ctx.data::<Arc<sea_orm::DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut query = GlobalContragent::find();

        if let Some(f) = filter {
            let mut condition = Condition::all();

            if let Some(vat) = f.vat_number {
                condition = condition.add(global_contragent::Column::VatNumber.eq(vat));
            }

            if let Some(eik) = f.eik {
                condition = condition.add(global_contragent::Column::Eik.eq(eik));
            }

            if let Some(vat_valid) = f.vat_valid {
                condition = condition.add(global_contragent::Column::VatValid.eq(vat_valid));
            }

            if let Some(eik_valid) = f.eik_valid {
                condition = condition.add(global_contragent::Column::EikValid.eq(eik_valid));
            }

            if let Some(valid) = f.valid {
                condition = condition.add(global_contragent::Column::Valid.eq(valid));
            }

            if let Some(search) = f.search {
                let search = search.trim();
                if !search.is_empty() {
                    let mut search_condition = Condition::any();
                    search_condition = search_condition
                        .add(global_contragent::Column::CompanyName.contains(search));
                    search_condition = search_condition
                        .add(global_contragent::Column::CompanyNameBg.contains(search));
                    search_condition =
                        search_condition.add(global_contragent::Column::VatNumber.contains(search));
                    search_condition =
                        search_condition.add(global_contragent::Column::Eik.contains(search));
                    condition = condition.add(search_condition);
                }
            }

            query = query.filter(condition);
        }

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let results = query
            .order_by_asc(global_contragent::Column::CompanyName)
            .order_by_asc(global_contragent::Column::CompanyNameBg)
            .all(db)
            .await?;

        Ok(results)
    }

    async fn global_contragent_by_vat(
        &self,
        ctx: &Context<'_>,
        vat_number: String,
    ) -> FieldResult<Option<GlobalContragentModel>> {
        let db = ctx.data::<Arc<sea_orm::DatabaseConnection>>()?;
        let db = db.as_ref();

        let contragent = GlobalContragent::find()
            .filter(global_contragent::Column::VatNumber.eq(vat_number))
            .one(db)
            .await?;

        Ok(contragent)
    }

    async fn global_contragent_summary(
        &self,
        ctx: &Context<'_>,
    ) -> FieldResult<GlobalContragentSummary> {
        let db = ctx.data::<Arc<sea_orm::DatabaseConnection>>()?;
        let service = ctx.data::<Arc<ContragentService>>()?;
        let summary = service.get_summary(db.as_ref()).await?;
        Ok(summary)
    }

    async fn contragent_settings(
        &self,
        ctx: &Context<'_>,
    ) -> FieldResult<Vec<ContragentSettingModel>> {
        let db = ctx.data::<Arc<sea_orm::DatabaseConnection>>()?;
        let db = db.as_ref();
        let settings = ContragentSetting::find()
            .order_by_asc(contragent_setting::Column::Key)
            .all(db)
            .await?;
        Ok(settings)
    }
}

#[derive(Default)]
pub struct ContragentMutation;

#[Object]
impl ContragentMutation {
    async fn validate_vat(
        &self,
        ctx: &Context<'_>,
        vat_number: String,
    ) -> FieldResult<ContragentValidationPayload> {
        let db = ctx.data::<Arc<sea_orm::DatabaseConnection>>()?;
        let service = ctx.data::<Arc<ContragentService>>()?;
        let outcome = service.validate_vat(db.as_ref(), &vat_number).await?;
        Ok(ContragentValidationPayload::from(outcome))
    }

    async fn refresh_vat(
        &self,
        ctx: &Context<'_>,
        vat_number: String,
    ) -> FieldResult<ContragentValidationPayload> {
        let db = ctx.data::<Arc<sea_orm::DatabaseConnection>>()?;
        let service = ctx.data::<Arc<ContragentService>>()?;
        let outcome = service.refresh_vat(db.as_ref(), &vat_number).await?;
        Ok(ContragentValidationPayload::from(outcome))
    }

    async fn validate_eik(
        &self,
        ctx: &Context<'_>,
        eik: String,
    ) -> FieldResult<GlobalContragentModel> {
        let db = ctx.data::<Arc<sea_orm::DatabaseConnection>>()?;
        let service = ctx.data::<Arc<ContragentService>>()?;
        let contragent = service.validate_eik(db.as_ref(), &eik).await?;
        Ok(contragent)
    }

    async fn process_global_contragents(
        &self,
        ctx: &Context<'_>,
    ) -> FieldResult<ProcessContragentsResult> {
        let db = ctx.data::<Arc<sea_orm::DatabaseConnection>>()?;
        let service = ctx.data::<Arc<ContragentService>>()?;
        let (processed, failed) = service.process_existing_addresses(db.as_ref()).await?;
        Ok(ProcessContragentsResult { processed, failed })
    }

    async fn upsert_contragent_setting(
        &self,
        ctx: &Context<'_>,
        input: UpsertContragentSettingInput,
    ) -> FieldResult<ContragentSettingModel> {
        let db = ctx.data::<Arc<sea_orm::DatabaseConnection>>()?;
        let service = ctx.data::<Arc<ContragentService>>()?;
        let setting = service.upsert_setting(db.as_ref(), input).await?;
        Ok(setting)
    }

    async fn parse_address(
        &self,
        ctx: &Context<'_>,
        address: String,
    ) -> FieldResult<Option<AddressComponents>> {
        let db = ctx.data::<Arc<sea_orm::DatabaseConnection>>()?;
        let service = ctx.data::<Arc<ContragentService>>()?;
        let components = service.parse_address(db.as_ref(), &address).await?;
        Ok(components)
    }
}

#[derive(SimpleObject)]
pub struct ContragentValidationPayload {
    pub contragent: GlobalContragentModel,
    pub existed_in_database: bool,
    pub source: ContragentSource,
}

impl From<crate::services::contragent::ValidationOutcome> for ContragentValidationPayload {
    fn from(outcome: crate::services::contragent::ValidationOutcome) -> Self {
        Self {
            contragent: outcome.contragent,
            existed_in_database: outcome.existed_in_database,
            source: ContragentSource::from(outcome.source),
        }
    }
}

#[derive(Enum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ContragentSource {
    Database,
    ViesRest,
    ViesSoap,
}

impl From<ContragentDataSource> for ContragentSource {
    fn from(value: ContragentDataSource) -> Self {
        match value {
            ContragentDataSource::Database => ContragentSource::Database,
            ContragentDataSource::ViesRest => ContragentSource::ViesRest,
            ContragentDataSource::ViesSoap => ContragentSource::ViesSoap,
        }
    }
}

#[derive(SimpleObject)]
pub struct ProcessContragentsResult {
    pub processed: i64,
    pub failed: i64,
}
