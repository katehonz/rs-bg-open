//! GraphQL Resolvers for Production Module
//!
//! Handles CRUD operations for technology cards and production batches

use async_graphql::{Context, FieldResult, InputObject, Object, SimpleObject};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use std::sync::Arc;

use crate::entities::{
    technology_card, technology_card_stage, production_batch, production_batch_stage,
    CreateTechnologyCardInput, CreateTechnologyCardStageInput, TechnologyCard, TechnologyCardStage,
    ProductionBatch, ProductionBatchStage, UpdateTechnologyCardInput,
};

// Extended input for creating technology card with stages
#[derive(InputObject, Debug, Clone)]
pub struct CreateTechnologyCardWithStagesInput {
    pub company_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub output_unit: String,
    pub stages: Vec<StageInput>,
}

#[derive(InputObject, Debug, Clone)]
pub struct StageInput {
    pub stage_number: i32,
    pub name: String,
    pub debit_account_id: i32,
    pub credit_account_id: i32,
    pub quantity_formula: Option<String>,
    pub unit_of_measure: Option<String>,
    pub amount_formula: Option<String>,
    pub description: Option<String>,
}

// Output type with stages
#[derive(SimpleObject)]
pub struct TechnologyCardWithStages {
    #[graphql(flatten)]
    pub card: technology_card::Model,
    pub stages: Vec<technology_card_stage::Model>,
}

#[derive(Default)]
pub struct ProductionQuery;

#[Object]
impl ProductionQuery {
    /// Get all technology cards for a company
    async fn technology_cards(
        &self,
        ctx: &Context<'_>,
        company_id: i32,
    ) -> FieldResult<Vec<technology_card::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        let cards = TechnologyCard::find()
            .filter(technology_card::Column::CompanyId.eq(company_id))
            .order_by_desc(technology_card::Column::CreatedAt)
            .all(db.as_ref())
            .await?;

        Ok(cards)
    }

    /// Get a single technology card with its stages
    async fn technology_card(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> FieldResult<Option<TechnologyCardWithStages>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        let card = TechnologyCard::find_by_id(id).one(db.as_ref()).await?;

        match card {
            Some(card_model) => {
                let stages = TechnologyCardStage::find()
                    .filter(technology_card_stage::Column::TechnologyCardId.eq(id))
                    .order_by_asc(technology_card_stage::Column::StageNumber)
                    .all(db.as_ref())
                    .await?;

                Ok(Some(TechnologyCardWithStages {
                    card: card_model,
                    stages,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get stages for a technology card
    async fn technology_card_stages(
        &self,
        ctx: &Context<'_>,
        technology_card_id: i32,
    ) -> FieldResult<Vec<technology_card_stage::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        let stages = TechnologyCardStage::find()
            .filter(technology_card_stage::Column::TechnologyCardId.eq(technology_card_id))
            .order_by_asc(technology_card_stage::Column::StageNumber)
            .all(db.as_ref())
            .await?;

        Ok(stages)
    }
}

#[derive(Default)]
pub struct ProductionMutation;

#[Object]
impl ProductionMutation {
    /// Create a new technology card with stages
    async fn create_technology_card_with_stages(
        &self,
        ctx: &Context<'_>,
        input: CreateTechnologyCardWithStagesInput,
    ) -> FieldResult<TechnologyCardWithStages> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        // Create the technology card
        let now = Utc::now();
        let card = technology_card::ActiveModel {
            company_id: Set(input.company_id),
            name: Set(input.name),
            description: Set(input.description),
            output_unit: Set(input.output_unit),
            is_active: Set(true),
            created_by: Set(None), // TODO: Get from auth context
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let card_result = card.insert(db.as_ref()).await?;

        // Create the stages
        let mut stages_results = Vec::new();
        for stage_input in input.stages {
            let stage = technology_card_stage::ActiveModel {
                technology_card_id: Set(card_result.id),
                stage_number: Set(stage_input.stage_number),
                name: Set(stage_input.name),
                debit_account_id: Set(stage_input.debit_account_id),
                credit_account_id: Set(stage_input.credit_account_id),
                quantity_formula: Set(stage_input.quantity_formula),
                unit_of_measure: Set(stage_input.unit_of_measure),
                amount_formula: Set(stage_input.amount_formula),
                description: Set(stage_input.description),
                created_at: Set(now),
                ..Default::default()
            };

            let stage_result = stage.insert(db.as_ref()).await?;
            stages_results.push(stage_result);
        }

        Ok(TechnologyCardWithStages {
            card: card_result,
            stages: stages_results,
        })
    }

    /// Update a technology card
    async fn update_technology_card(
        &self,
        ctx: &Context<'_>,
        input: UpdateTechnologyCardInput,
    ) -> FieldResult<technology_card::Model> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        let card = TechnologyCard::find_by_id(input.id)
            .one(db.as_ref())
            .await?
            .ok_or("Technology card not found")?;

        let mut card: technology_card::ActiveModel = card.into();

        if let Some(name) = input.name {
            card.name = Set(name);
        }
        if let Some(description) = input.description {
            card.description = Set(Some(description));
        }
        if let Some(output_unit) = input.output_unit {
            card.output_unit = Set(output_unit);
        }
        if let Some(is_active) = input.is_active {
            card.is_active = Set(is_active);
        }

        card.updated_at = Set(Utc::now());

        let updated_card = card.update(db.as_ref()).await?;
        Ok(updated_card)
    }

    /// Delete a technology card and its stages
    async fn delete_technology_card(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> FieldResult<bool> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        let card = TechnologyCard::find_by_id(id)
            .one(db.as_ref())
            .await?
            .ok_or("Technology card not found")?;

        // Delete stages first (cascade should handle this, but being explicit)
        TechnologyCardStage::delete_many()
            .filter(technology_card_stage::Column::TechnologyCardId.eq(id))
            .exec(db.as_ref())
            .await?;

        // Delete the card
        let card: technology_card::ActiveModel = card.into();
        card.delete(db.as_ref()).await?;

        Ok(true)
    }

    /// Update stages for a technology card
    async fn update_technology_card_stages(
        &self,
        ctx: &Context<'_>,
        technology_card_id: i32,
        stages: Vec<StageInput>,
    ) -> FieldResult<Vec<technology_card_stage::Model>> {
        let db = ctx.data::<Arc<DatabaseConnection>>()?;

        // Verify card exists
        TechnologyCard::find_by_id(technology_card_id)
            .one(db.as_ref())
            .await?
            .ok_or("Technology card not found")?;

        // Delete existing stages
        TechnologyCardStage::delete_many()
            .filter(technology_card_stage::Column::TechnologyCardId.eq(technology_card_id))
            .exec(db.as_ref())
            .await?;

        // Create new stages
        let now = Utc::now();
        let mut stages_results = Vec::new();

        for stage_input in stages {
            let stage = technology_card_stage::ActiveModel {
                technology_card_id: Set(technology_card_id),
                stage_number: Set(stage_input.stage_number),
                name: Set(stage_input.name),
                debit_account_id: Set(stage_input.debit_account_id),
                credit_account_id: Set(stage_input.credit_account_id),
                quantity_formula: Set(stage_input.quantity_formula),
                unit_of_measure: Set(stage_input.unit_of_measure),
                amount_formula: Set(stage_input.amount_formula),
                description: Set(stage_input.description),
                created_at: Set(now),
                ..Default::default()
            };

            let stage_result = stage.insert(db.as_ref()).await?;
            stages_results.push(stage_result);
        }

        Ok(stages_results)
    }
}
