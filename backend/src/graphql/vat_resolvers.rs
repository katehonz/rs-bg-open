use crate::entities::vat_rate::{
    CreateVatRateInput, UpdateVatRateInput, VatCalculation, VatRateFilter,
};
use crate::entities::vat_return::{
    CreateVatReturnInput, MonthlyVatSummary, UpdateVatReturnInput, VatReturnFilter,
    VatReturnStatus, VatReturnSummary,
};
use crate::entities::{vat_rate, vat_return};
use async_graphql::{Context, FieldResult, Object};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    Set,
};
use std::sync::Arc;

#[derive(Default)]
pub struct VatQuery;

#[Object]
impl VatQuery {
    /// Get all VAT rates with optional filtering
    async fn vat_rates(
        &self,
        ctx: &Context<'_>,
        filter: Option<VatRateFilter>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> FieldResult<Vec<vat_rate::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut query = vat_rate::Entity::find();

        if let Some(f) = filter {
            let mut condition = Condition::all();

            if let Some(company_id) = f.company_id {
                condition = condition.add(vat_rate::Column::CompanyId.eq(company_id));
            }

            if let Some(is_active) = f.is_active {
                condition = condition.add(vat_rate::Column::IsActive.eq(is_active));
            }

            if let Some(vat_direction) = f.vat_direction {
                condition = condition.add(vat_rate::Column::VatDirection.eq(vat_direction));
            }

            if let Some(valid_date) = f.valid_on_date {
                condition = condition
                    .add(vat_rate::Column::ValidFrom.lte(valid_date))
                    .add(
                        Condition::any()
                            .add(vat_rate::Column::ValidTo.is_null())
                            .add(vat_rate::Column::ValidTo.gte(valid_date)),
                    );
            }

            if let Some(code_search) = f.code_contains {
                condition = condition.add(vat_rate::Column::Code.contains(&code_search));
            }

            query = query.filter(condition);
        }

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let rates = query.order_by_asc(vat_rate::Column::Rate).all(db).await?;

        Ok(rates)
    }

    /// Get VAT rate by ID
    async fn vat_rate(&self, ctx: &Context<'_>, id: i32) -> FieldResult<Option<vat_rate::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let rate = vat_rate::Entity::find_by_id(id).one(db).await?;
        Ok(rate)
    }

    /// Calculate VAT for given base amount and rate
    async fn calculate_vat(
        &self,
        ctx: &Context<'_>,
        base_amount: Decimal,
        vat_rate_id: i32,
    ) -> FieldResult<VatCalculation> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        if let Some(rate) = vat_rate::Entity::find_by_id(vat_rate_id).one(db).await? {
            let vat_amount = rate.calculate_vat_amount(base_amount);
            let total_amount = base_amount + vat_amount;

            Ok(VatCalculation {
                base_amount,
                vat_rate: rate.rate,
                vat_amount,
                total_amount,
                rate_code: rate.code,
                rate_name: rate.name,
            })
        } else {
            Err("VAT rate not found".into())
        }
    }

    /// Get all VAT returns with optional filtering
    async fn vat_returns(
        &self,
        ctx: &Context<'_>,
        filter: Option<VatReturnFilter>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> FieldResult<Vec<vat_return::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut query = vat_return::Entity::find();

        if let Some(f) = filter {
            let mut condition = Condition::all();

            if let Some(company_id) = f.company_id {
                condition = condition.add(vat_return::Column::CompanyId.eq(company_id));
            }

            if let Some(year) = f.period_year {
                condition = condition.add(vat_return::Column::PeriodYear.eq(year));
            }

            if let Some(month) = f.period_month {
                condition = condition.add(vat_return::Column::PeriodMonth.eq(month));
            }

            if let Some(status) = f.status {
                condition = condition.add(vat_return::Column::Status.eq(status));
            }

            if let Some(true) = f.overdue_only {
                let today = chrono::Utc::now().date_naive();
                condition = condition
                    .add(vat_return::Column::Status.eq(VatReturnStatus::Draft))
                    .add(vat_return::Column::DueDate.lt(today));
            }

            query = query.filter(condition);
        }

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let returns = query
            .order_by_desc(vat_return::Column::PeriodYear)
            .order_by_desc(vat_return::Column::PeriodMonth)
            .all(db)
            .await?;

        Ok(returns)
    }

    /// Get VAT return by ID
    async fn vat_return(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> FieldResult<Option<vat_return::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let vat_return = vat_return::Entity::find_by_id(id).one(db).await?;
        Ok(vat_return)
    }

    /// Get VAT return summary with calculations
    async fn vat_return_summary(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> FieldResult<Option<VatReturnSummary>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        if let Some(vat_return) = vat_return::Entity::find_by_id(id).one(db).await? {
            let total_taxable_turnover = vat_return.get_total_taxable_turnover();
            let total_vat_collected = vat_return.get_total_vat_collected();
            let is_overdue = vat_return.is_overdue();

            let today = chrono::Utc::now().date_naive();
            let days_until_due = (vat_return.due_date - today).num_days();

            Ok(Some(VatReturnSummary {
                vat_return,
                total_taxable_turnover,
                total_vat_collected,
                is_overdue,
                days_until_due,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get overdue VAT returns for a company
    async fn overdue_vat_returns(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
    ) -> FieldResult<Vec<vat_return::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();
        let today = chrono::Utc::now().date_naive();

        let returns = vat_return::Entity::find()
            .filter(vat_return::Column::CompanyId.eq(company_id))
            .filter(vat_return::Column::Status.eq(VatReturnStatus::Draft))
            .filter(vat_return::Column::DueDate.lt(today))
            .order_by_asc(vat_return::Column::DueDate)
            .all(db)
            .await?;

        Ok(returns)
    }
}

#[derive(Default)]
pub struct VatMutation;

#[Object]
impl VatMutation {
    /// Create a new VAT rate
    async fn create_vat_rate(
        &self,
        ctx: &Context<'_>,
        input: CreateVatRateInput,
    ) -> FieldResult<vat_rate::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Check if code already exists for this company
        let existing = vat_rate::Entity::find()
            .filter(vat_rate::Column::CompanyId.eq(input.company_id))
            .filter(vat_rate::Column::Code.eq(&input.code))
            .one(db)
            .await?;

        if existing.is_some() {
            return Err("VAT rate code already exists for this company".into());
        }

        let rate_model = vat_rate::ActiveModel::from(input);
        let rate = vat_rate::Entity::insert(rate_model)
            .exec_with_returning(db)
            .await?;

        Ok(rate)
    }

    /// Update VAT rate
    async fn update_vat_rate(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateVatRateInput,
    ) -> FieldResult<vat_rate::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut rate: vat_rate::ActiveModel =
            if let Some(rate) = vat_rate::Entity::find_by_id(id).one(db).await? {
                rate.into()
            } else {
                return Err("VAT rate not found".into());
            };

        if let Some(code) = input.code {
            rate.code = Set(code);
        }

        if let Some(name) = input.name {
            rate.name = Set(name);
        }

        if let Some(rate_value) = input.rate {
            rate.rate = Set(rate_value);
        }

        if let Some(vat_direction) = input.vat_direction {
            rate.vat_direction = Set(vat_direction);
        }

        if let Some(valid_from) = input.valid_from {
            rate.valid_from = Set(valid_from);
        }

        if let Some(valid_to) = input.valid_to {
            rate.valid_to = Set(Some(valid_to));
        }

        if let Some(is_active) = input.is_active {
            rate.is_active = Set(is_active);
        }

        let updated_rate = vat_rate::Entity::update(rate).exec(db).await?;
        Ok(updated_rate)
    }

    /// Create a new VAT return
    async fn create_vat_return(
        &self,
        ctx: &Context<'_>,
        input: CreateVatReturnInput,
    ) -> FieldResult<vat_return::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        // Check if VAT return already exists for this period
        let existing = vat_return::Entity::find()
            .filter(vat_return::Column::CompanyId.eq(input.company_id))
            .filter(vat_return::Column::PeriodYear.eq(input.period_year))
            .filter(vat_return::Column::PeriodMonth.eq(input.period_month))
            .one(db)
            .await?;

        if existing.is_some() {
            return Err("VAT return already exists for this period".into());
        }

        let mut vat_return_model = vat_return::ActiveModel::from(input);
        // Set created_by - should come from authentication context
        vat_return_model.created_by = Set(1); // TODO: Get from auth context

        let vat_return = vat_return::Entity::insert(vat_return_model)
            .exec_with_returning(db)
            .await?;

        Ok(vat_return)
    }

    /// Update VAT return amounts
    async fn update_vat_return(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateVatReturnInput,
    ) -> FieldResult<vat_return::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut vat_return: vat_return::ActiveModel =
            if let Some(vat_return) = vat_return::Entity::find_by_id(id).one(db).await? {
                vat_return.into()
            } else {
                return Err("VAT return not found".into());
            };

        // Update amounts
        if let Some(output_vat) = input.output_vat_amount {
            vat_return.output_vat_amount = Set(output_vat);
        }

        if let Some(input_vat) = input.input_vat_amount {
            vat_return.input_vat_amount = Set(input_vat);
        }

        if let Some(base_20) = input.base_amount_20 {
            vat_return.base_amount_20 = Set(base_20);
        }

        if let Some(vat_20) = input.vat_amount_20 {
            vat_return.vat_amount_20 = Set(vat_20);
        }

        if let Some(base_9) = input.base_amount_9 {
            vat_return.base_amount_9 = Set(base_9);
        }

        if let Some(vat_9) = input.vat_amount_9 {
            vat_return.vat_amount_9 = Set(vat_9);
        }

        if let Some(base_0) = input.base_amount_0 {
            vat_return.base_amount_0 = Set(base_0);
        }

        if let Some(exempt) = input.exempt_amount {
            vat_return.exempt_amount = Set(exempt);
        }

        if let Some(notes) = input.notes {
            vat_return.notes = Set(Some(notes));
        }

        let updated_return = vat_return::Entity::update(vat_return).exec(db).await?;
        Ok(updated_return)
    }

    /// Submit VAT return
    async fn submit_vat_return(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> FieldResult<vat_return::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut vat_return: vat_return::ActiveModel =
            if let Some(vat_return) = vat_return::Entity::find_by_id(id).one(db).await? {
                if !vat_return.can_be_submitted() {
                    return Err("VAT return cannot be submitted".into());
                }
                vat_return.into()
            } else {
                return Err("VAT return not found".into());
            };

        vat_return.status = Set(VatReturnStatus::Submitted);
        vat_return.submitted_at = Set(Some(chrono::Utc::now()));
        vat_return.submitted_by = Set(Some(1)); // TODO: Get from auth context

        let updated_return = vat_return::Entity::update(vat_return).exec(db).await?;
        Ok(updated_return)
    }

    /// Generate monthly VAT returns for a year
    async fn generate_monthly_returns(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
        year: i32,
    ) -> FieldResult<Vec<vat_return::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;
        let db = db.as_ref();

        let mut generated_returns = Vec::new();

        for month in 1..=12 {
            // Check if already exists
            let existing = vat_return::Entity::find()
                .filter(vat_return::Column::CompanyId.eq(company_id))
                .filter(vat_return::Column::PeriodYear.eq(year))
                .filter(vat_return::Column::PeriodMonth.eq(month))
                .one(db)
                .await?;

            if existing.is_none() {
                let input = CreateVatReturnInput {
                    period_year: year,
                    period_month: month,
                    company_id,
                    notes: None,
                };

                let mut vat_return_model = vat_return::ActiveModel::from(input);
                vat_return_model.created_by = Set(1); // TODO: Get from auth context

                let vat_return = vat_return::Entity::insert(vat_return_model)
                    .exec_with_returning(db)
                    .await?;
                generated_returns.push(vat_return);
            }
        }

        Ok(generated_returns)
    }
}
