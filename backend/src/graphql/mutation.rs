use super::accounting_resolvers::AccountingMutation;
use super::admin_resolvers::AdminMutation;
use super::ai_accounting_settings_resolvers::AiAccountingSettingsMutation;
use super::ai_bank_accounting_settings_resolvers::AiBankAccountingSettingsMutation;
use super::bank_resolvers::BankMutation;
use super::contragent_resolvers::ContragentMutation;
use super::controlisy_resolver::ControlisyMutation;
use super::currency_resolvers::CurrencyMutation;
use super::fixed_assets_resolvers::FixedAssetsMutation;
use super::inventory_resolvers::InventoryMutation;
use super::invoice_resolver::InvoiceMutation;
use super::maintenance_resolver::MaintenanceMutation;
use super::reports_resolvers::ReportsMutation;
use super::saft_resolvers::SafTMutation;
use super::user_resolvers::UserMutation;
use super::vat_resolvers::VatMutation;
use async_graphql::{MergedObject, Object};

#[derive(MergedObject, Default)]
pub struct Mutation(
    AppMutation,
    UserMutation,
    AdminMutation,
    VatMutation,
    CurrencyMutation,
    AccountingMutation,
    AiAccountingSettingsMutation,
    AiBankAccountingSettingsMutation,
    BankMutation,
    SafTMutation,
    FixedAssetsMutation,
    InventoryMutation,
    ReportsMutation,
    ControlisyMutation,
    ContragentMutation,
    MaintenanceMutation,
    InvoiceMutation,
);

#[derive(Default)]
pub struct AppMutation;

#[Object]
impl AppMutation {
    async fn ping(&self) -> &str {
        "pong"
    }
}
