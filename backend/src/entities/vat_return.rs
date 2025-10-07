use async_graphql::{Enum, InputObject, SimpleObject};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use sea_orm::{sea_query::StringLen, Set};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Enum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(10))")]
pub enum VatReturnStatus {
    #[sea_orm(string_value = "DRAFT")]
    Draft,
    #[sea_orm(string_value = "SUBMITTED")]
    Submitted,
    #[sea_orm(string_value = "APPROVED")]
    Approved,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "vat_returns")]
#[graphql(concrete(name = "VatReturn", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub period_year: i32,
    pub period_month: i32,
    pub period_from: Date,
    pub period_to: Date,
    // VAT amounts
    pub output_vat_amount: Decimal,
    pub input_vat_amount: Decimal,
    pub vat_to_pay: Decimal,
    pub vat_to_refund: Decimal,
    // Base amounts by VAT rate
    #[sea_orm(column_name = "base_amount20")]
    pub base_amount_20: Decimal,
    #[sea_orm(column_name = "vat_amount20")]
    pub vat_amount_20: Decimal,
    #[sea_orm(column_name = "base_amount9")]
    pub base_amount_9: Decimal,
    #[sea_orm(column_name = "vat_amount9")]
    pub vat_amount_9: Decimal,
    #[sea_orm(column_name = "base_amount0")]
    pub base_amount_0: Decimal,
    pub exempt_amount: Decimal,
    // Status and metadata
    pub status: VatReturnStatus,
    pub submitted_at: Option<DateTimeUtc>,
    pub submitted_by: Option<i32>,
    pub due_date: Date,
    pub notes: Option<String>,
    pub company_id: i32,
    pub created_by: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::CompanyId",
        to = "super::company::Column::Id"
    )]
    Company,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::CreatedBy",
        to = "super::user::Column::Id"
    )]
    CreatedByUser,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::SubmittedBy",
        to = "super::user::Column::Id"
    )]
    SubmittedByUser,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

// Input types for GraphQL mutations
#[derive(InputObject, Deserialize, Serialize)]
pub struct CreateVatReturnInput {
    pub period_year: i32,
    pub period_month: i32,
    pub company_id: i32,
    pub notes: Option<String>,
}

#[derive(InputObject, Deserialize, Serialize)]
pub struct UpdateVatReturnInput {
    pub output_vat_amount: Option<Decimal>,
    pub input_vat_amount: Option<Decimal>,
    pub base_amount_20: Option<Decimal>,
    pub vat_amount_20: Option<Decimal>,
    pub base_amount_9: Option<Decimal>,
    pub vat_amount_9: Option<Decimal>,
    pub base_amount_0: Option<Decimal>,
    pub exempt_amount: Option<Decimal>,
    pub notes: Option<String>,
}

impl From<CreateVatReturnInput> for ActiveModel {
    fn from(input: CreateVatReturnInput) -> Self {
        // Calculate period dates
        let period_from =
            chrono::NaiveDate::from_ymd_opt(input.period_year, input.period_month as u32, 1)
                .unwrap_or_else(|| chrono::Utc::now().date_naive());

        let period_to = if input.period_month == 12 {
            chrono::NaiveDate::from_ymd_opt(input.period_year + 1, 1, 1)
        } else {
            chrono::NaiveDate::from_ymd_opt(input.period_year, input.period_month as u32 + 1, 1)
        }
        .unwrap_or_else(|| chrono::Utc::now().date_naive())
        .pred_opt()
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

        ActiveModel {
            period_year: Set(input.period_year),
            period_month: Set(input.period_month),
            period_from: Set(period_from),
            period_to: Set(period_to),
            company_id: Set(input.company_id),
            notes: Set(input.notes),
            ..Default::default()
        }
    }
}

// Filter input for queries
#[derive(InputObject, Deserialize)]
pub struct VatReturnFilter {
    pub company_id: Option<i32>,
    pub period_year: Option<i32>,
    pub period_month: Option<i32>,
    pub status: Option<VatReturnStatus>,
    pub overdue_only: Option<bool>,
}

impl Model {
    /// Check if VAT return is overdue
    pub fn is_overdue(&self) -> bool {
        self.status == VatReturnStatus::Draft && chrono::Utc::now().date_naive() > self.due_date
    }

    /// Get period description in Bulgarian
    pub fn get_period_description(&self) -> String {
        let month_names = [
            "Януари",
            "Февруари",
            "Март",
            "Април",
            "Май",
            "Юни",
            "Юли",
            "Август",
            "Септември",
            "Октомври",
            "Ноември",
            "Декември",
        ];

        let month_name = month_names
            .get((self.period_month - 1) as usize)
            .unwrap_or(&"Неизвестен");

        format!("{} {}", month_name, self.period_year)
    }

    /// Calculate total turnover subject to VAT
    pub fn get_total_taxable_turnover(&self) -> Decimal {
        self.base_amount_20 + self.base_amount_9 + self.base_amount_0
    }

    /// Calculate total VAT collected
    pub fn get_total_vat_collected(&self) -> Decimal {
        self.vat_amount_20 + self.vat_amount_9
    }

    /// Check if return can be submitted
    pub fn can_be_submitted(&self) -> bool {
        self.status == VatReturnStatus::Draft && !self.is_overdue()
    }

    /// Get status description in Bulgarian
    pub fn get_status_description(&self) -> &'static str {
        match self.status {
            VatReturnStatus::Draft => "Чернова",
            VatReturnStatus::Submitted => "Подадена",
            VatReturnStatus::Approved => "Одобрена",
        }
    }
}

// Helper structs for VAT reporting
#[derive(SimpleObject, Serialize)]
pub struct VatReturnSummary {
    pub vat_return: Model,
    pub total_taxable_turnover: Decimal,
    pub total_vat_collected: Decimal,
    pub is_overdue: bool,
    pub days_until_due: i64,
}

#[derive(SimpleObject, Serialize)]
pub struct MonthlyVatSummary {
    pub period_year: i32,
    pub period_month: i32,
    pub period_description: String,
    pub output_vat: Decimal,
    pub input_vat: Decimal,
    pub net_vat: Decimal,
    pub status: VatReturnStatus,
    pub due_date: Date,
    pub is_overdue: bool,
}
impl ActiveModelBehavior for ActiveModel {}
