use super::accounting_resolvers::AccountingQuery;
use super::admin_resolvers::AdminQuery;
use super::ai_accounting_settings_resolvers::AiAccountingSettingsQuery;
use super::ai_bank_accounting_settings_resolvers::AiBankAccountingSettingsQuery;
use super::bank_resolvers::BankQuery;
use super::contragent_resolvers::ContragentQuery;
use super::controlisy_resolver::ControlisyQuery;
use super::currency_resolvers::CurrencyQuery;
use super::fixed_assets_resolvers::FixedAssetsQuery;
use super::inventory_resolvers::InventoryQuery;
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
    AiAccountingSettingsQuery,
    AiBankAccountingSettingsQuery,
    BankQuery,
    SafTQuery,
    FixedAssetsQuery,
    InventoryQuery,
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
