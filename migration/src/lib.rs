pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_users_table;
// mod m20240101_000002_create_posts_table; // Not needed for accounting system
mod m20240101_000003_create_companies;
mod m20240101_000004_create_user_groups;
mod m20240101_000005_create_accounts;
mod m20240101_000006_create_journal_entries;
mod m20240101_000007_create_entry_lines;
mod m20240101_000008_create_counterparts;
mod m20240101_000009_create_vat_rates;
mod m20240101_000010_create_vat_returns;
mod m20240101_000011_create_currencies;
mod m20240101_000012_create_exchange_rates;
mod m20240101_000013_add_counterpart_fields;
mod m20240101_000014_add_vat_codes;
mod m20240101_000015_add_company_eik;
mod m20240101_000016_add_vat_declaration_fields;
mod m20240101_000017_make_vat_date_optional;
mod m20240101_000018_add_quantity_support_to_accounts;
mod m20240101_000019_create_fixed_assets;
mod m20240101_000020_seed_demo_company;
mod m20240101_000021_load_full_chart;
mod m20240101_000022_create_controlisy_imports;
mod m20240912_000001_extend_counterparts_saft;
mod m20240912_000002_create_packages;
mod m20240914_000001_create_user_companies;
mod m20240915_000001_add_customer_supplier_flags;
mod m20240917_000001_add_contragent_api_settings;
mod m20240917_000002_add_contragent_api_key;
mod m20241005_000001_create_global_contragents;
mod m20241005_000002_create_contragent_settings;
mod m20241007_000001_alter_contragent_settings_timestamps;
mod m20241009_000001_alter_global_contragents_timestamps;
mod m20241012_000001_add_counterpart_address_details;
mod m20241015_000001_create_bank_modules;
mod m20251011_000001_add_base_currency_to_companies;
mod m20250101_000001_add_nap_fields_to_vat_returns;
mod m20251015_000001_create_inventory_management;
mod m20251017_000001_create_ai_accounting_settings;
mod m20251017_000002_create_ai_bank_accounting_settings;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // User groups must be created before users (FK dependency)
            Box::new(m20240101_000004_create_user_groups::Migration),
            Box::new(m20240101_000001_create_users_table::Migration),
            Box::new(m20240101_000003_create_companies::Migration),
            Box::new(m20240101_000008_create_counterparts::Migration),
            Box::new(m20240101_000005_create_accounts::Migration),
            Box::new(m20240101_000009_create_vat_rates::Migration),
            Box::new(m20240101_000006_create_journal_entries::Migration),
            Box::new(m20240101_000007_create_entry_lines::Migration),
            Box::new(m20240101_000010_create_vat_returns::Migration),
            Box::new(m20240101_000011_create_currencies::Migration),
            Box::new(m20240101_000012_create_exchange_rates::Migration),
            Box::new(m20240101_000013_add_counterpart_fields::Migration),
            Box::new(m20240101_000014_add_vat_codes::Migration),
            Box::new(m20240101_000015_add_company_eik::Migration),
            Box::new(m20240101_000016_add_vat_declaration_fields::Migration),
            Box::new(m20240101_000017_make_vat_date_optional::Migration),
            Box::new(m20240101_000018_add_quantity_support_to_accounts::Migration),
            Box::new(m20240101_000019_create_fixed_assets::Migration),
            Box::new(m20240101_000020_seed_demo_company::Migration),
            Box::new(m20240101_000021_load_full_chart::Migration),
            Box::new(m20240101_000022_create_controlisy_imports::Migration),
            Box::new(m20240912_000001_extend_counterparts_saft::Migration),
            Box::new(m20240912_000002_create_packages::Migration),
            Box::new(m20240914_000001_create_user_companies::Migration),
            Box::new(m20240915_000001_add_customer_supplier_flags::Migration),
            Box::new(m20240917_000001_add_contragent_api_settings::Migration),
            Box::new(m20240917_000002_add_contragent_api_key::Migration),
            Box::new(m20241005_000001_create_global_contragents::Migration),
            Box::new(m20241005_000002_create_contragent_settings::Migration),
            Box::new(m20241007_000001_alter_contragent_settings_timestamps::Migration),
            Box::new(m20241009_000001_alter_global_contragents_timestamps::Migration),
            Box::new(m20241012_000001_add_counterpart_address_details::Migration),
            Box::new(m20241015_000001_create_bank_modules::Migration),
            Box::new(m20251011_000001_add_base_currency_to_companies::Migration),
            Box::new(m20250101_000001_add_nap_fields_to_vat_returns::Migration),
            Box::new(m20251015_000001_create_inventory_management::Migration),
            Box::new(m20251017_000001_create_ai_accounting_settings::Migration),
            Box::new(m20251017_000002_create_ai_bank_accounting_settings::Migration),
            // Box::new(m20240101_000002_create_posts_table::Migration), // Not needed
        ]
    }
}
