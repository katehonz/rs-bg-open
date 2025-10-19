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

    // Legacy fields (kept for backward compatibility)
    pub output_vat_amount: Decimal,
    pub input_vat_amount: Decimal,
    pub vat_to_pay: Decimal,
    pub vat_to_refund: Decimal,
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

    // NAP DEKLAR.TXT Fields - Sales (Продажби)
    // Field 01-01: Обща данъчна основа
    pub total_sales_taxable: Decimal,
    // Field 01-20: Общо начислен ДДС
    pub total_sales_vat: Decimal,
    // Field 01-11: Данъчна основа 20% (про11)
    #[sea_orm(column_name = "sales_base20")]
    pub sales_base_20: Decimal,
    // Field 01-21: ДДС 20%
    #[sea_orm(column_name = "sales_vat20")]
    pub sales_vat_20: Decimal,
    // Field 01-12: ВОП данъчна основа (про12)
    pub sales_base_vop: Decimal,
    // Field 01-22: ВОП ДДС
    pub sales_vat_vop: Decimal,
    // Field 01-23: ДДС за лично ползване
    pub sales_vat_personal_use: Decimal,
    // Field 01-13: Данъчна основа 9% (про17)
    #[sea_orm(column_name = "sales_base9")]
    pub sales_base_9: Decimal,
    // Field 01-24: ДДС 9%
    #[sea_orm(column_name = "sales_vat9")]
    pub sales_vat_9: Decimal,
    // Field 01-14: Данъчна основа 0% чл.3 (про14)
    #[sea_orm(column_name = "sales_base0_art3")]
    pub sales_base_0_art3: Decimal,
    // Field 01-15: Данъчна основа 0% ВОД (про20)
    #[sea_orm(column_name = "sales_base0_vod")]
    pub sales_base_0_vod: Decimal,
    // Field 01-16: Данъчна основа 0% чл.140/146/173 (про19, про21)
    #[sea_orm(column_name = "sales_base0_export")]
    pub sales_base_0_export: Decimal,
    // Field 01-17: Данъчна основа услуги чл.21 (про22)
    pub sales_base_art21: Decimal,
    // Field 01-18: Данъчна основа чл.69 (про23-1, про23-2)
    pub sales_base_art69: Decimal,
    // Field 01-19: Освободени доставки (про24-1, про24-2, про24-3)
    pub sales_base_exempt: Decimal,

    // NAP DEKLAR.TXT Fields - Purchases (Покупки)
    // Field 01-30: Данъчна основа без кредит (пок09 without credit)
    pub purchase_base_no_credit: Decimal,
    // Field 01-31: Данъчна основа пълен кредит (пок09, пок10)
    pub purchase_base_full_credit: Decimal,
    // Field 01-41: ДДС пълен кредит
    pub purchase_vat_full_credit: Decimal,
    // Field 01-32: Данъчна основа частичен кредит (пок12)
    pub purchase_base_partial_credit: Decimal,
    // Field 01-42: ДДС частичен кредит
    pub purchase_vat_partial_credit: Decimal,
    // Field 01-43: Годишно коригиране (пок14)
    pub purchase_vat_annual_adjustment: Decimal,

    // NAP DEKLAR.TXT Fields - Results
    // Field 01-33: Коефициент
    pub credit_coefficient: Decimal,
    // Field 01-40: Общ данъчен кредит
    pub total_deductible_vat: Decimal,

    // NAP DEKLAR.TXT Fields - Document counts
    // Field 00-05: Брой документи продажби
    pub sales_document_count: i32,
    // Field 00-06: Брой документи покупки
    pub purchase_document_count: i32,
    // Field 00-04: Подаващо лице (ЕГН/Име)
    pub submitted_by_person: Option<String>,

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
