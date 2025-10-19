use async_graphql::{Enum, InputObject, SimpleObject};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "bank_profiles")]
#[graphql(concrete(name = "BankProfile", params()))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub company_id: i32,
    pub name: String,
    pub iban: Option<String>,
    pub account_id: i32,
    pub buffer_account_id: i32,
    pub currency_code: String,
    pub import_format: String,
    pub is_active: bool,
    pub settings: Option<Value>,
    pub created_by: Option<i32>,
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
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id"
    )]
    Account,
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::BufferAccountId",
        to = "super::account::Column::Id"
    )]
    BufferAccount,
    #[sea_orm(has_many = "super::bank_import::Entity")]
    BankImports,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::bank_import::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BankImports.def()
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum BankImportFormat {
    #[graphql(name = "UNICREDIT_MT940")]
    UnicreditMt940,
    #[graphql(name = "WISE_CAMT053")]
    WiseCamt053,
    #[graphql(name = "REVOLUT_CAMT053")]
    RevolutCamt053,
    #[graphql(name = "PAYSERA_CAMT053")]
    PayseraCamt053,
    #[graphql(name = "POSTBANK_XML")]
    PostbankXml,
    #[graphql(name = "OBB_XML")]
    ObbXml,
    #[graphql(name = "CCB_CSV")]
    CcbCsv,
}

impl BankImportFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            BankImportFormat::UnicreditMt940 => "UNICREDIT_MT940",
            BankImportFormat::WiseCamt053 => "WISE_CAMT053",
            BankImportFormat::RevolutCamt053 => "REVOLUT_CAMT053",
            BankImportFormat::PayseraCamt053 => "PAYSERA_CAMT053",
            BankImportFormat::PostbankXml => "POSTBANK_XML",
            BankImportFormat::ObbXml => "OBB_XML",
            BankImportFormat::CcbCsv => "CCB_CSV",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "UNICREDIT_MT940" => Some(Self::UnicreditMt940),
            "WISE_CAMT053" => Some(Self::WiseCamt053),
            "REVOLUT_CAMT053" => Some(Self::RevolutCamt053),
            "PAYSERA_CAMT053" => Some(Self::PayseraCamt053),
            "POSTBANK_XML" => Some(Self::PostbankXml),
            "OBB_XML" => Some(Self::ObbXml),
            "CCB_CSV" => Some(Self::CcbCsv),
            _ => None,
        }
    }
}

impl FromStr for BankImportFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_code(s).ok_or_else(|| format!("Unsupported bank import format: {s}"))
    }
}

#[derive(InputObject, Deserialize, Serialize)]
pub struct CreateBankProfileInput {
    pub company_id: i32,
    pub name: String,
    pub iban: Option<String>,
    pub account_id: i32,
    pub buffer_account_id: i32,
    pub currency_code: String,
    pub import_format: BankImportFormat,
    pub is_active: Option<bool>,
    pub settings: Option<Value>,
}

#[derive(InputObject, Deserialize, Serialize)]
pub struct UpdateBankProfileInput {
    pub name: Option<String>,
    pub iban: Option<Option<String>>,
    pub account_id: Option<i32>,
    pub buffer_account_id: Option<i32>,
    pub currency_code: Option<String>,
    pub import_format: Option<BankImportFormat>,
    pub is_active: Option<bool>,
    pub settings: Option<Option<Value>>,
}

impl From<CreateBankProfileInput> for ActiveModel {
    fn from(input: CreateBankProfileInput) -> Self {
        ActiveModel {
            company_id: Set(input.company_id),
            name: Set(input.name.trim().to_string()),
            iban: Set(input.iban),
            account_id: Set(input.account_id),
            buffer_account_id: Set(input.buffer_account_id),
            currency_code: Set(input.currency_code.to_uppercase()),
            import_format: Set(input.import_format.as_str().to_string()),
            is_active: Set(input.is_active.unwrap_or(true)),
            settings: Set(input.settings),
            ..Default::default()
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
