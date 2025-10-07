use super::accounting_resolvers::AccountingQuery;
use super::admin_resolvers::AdminQuery;
use super::bank_resolvers::BankQuery;
use super::contragent_resolvers::ContragentQuery;
use super::controlisy_resolver::ControlisyQuery;
use super::currency_resolvers::CurrencyQuery;
use super::fixed_assets_resolvers::FixedAssetsQuery;
use super::maintenance_resolver::MaintenanceQuery;
use super::reports_resolvers::ReportsQuery;
use super::saft_resolvers::SafTQuery;
use super::user_resolvers::UserQuery;
use super::vat_resolvers::VatQuery;
use async_graphql::{MergedObject, Object};

#[derive(MergedObject, Default)]
pub struct Query(
    AppQuery,
    UserQuery,
    AdminQuery,
    VatQuery,
    CurrencyQuery,
    AccountingQuery,
    BankQuery,
    SafTQuery,
    FixedAssetsQuery,
    ReportsQuery,
    ControlisyQuery,
    ContragentQuery,
    MaintenanceQuery,
);

#[derive(Default)]
pub struct AppQuery;

#[Object]
impl AppQuery {
    async fn hello(&self) -> &str {
        "Hello, RS-AC-BG Bulgarian Accounting System!"
    }

    async fn version(&self) -> &str {
        "1.0.0"
    }

    async fn status(&self) -> &str {
        "Running"
    }
}
