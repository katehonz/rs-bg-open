-- Migration: Add currency revaluation settings table
-- Description: Stores company-level settings for currency revaluation (accounts 624 and 724)

CREATE TABLE IF NOT EXISTS currency_revaluation_settings (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    expense_account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    revenue_account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT unique_company_revaluation_settings UNIQUE(company_id)
);

CREATE INDEX idx_currency_revaluation_settings_company
    ON currency_revaluation_settings(company_id);

COMMENT ON TABLE currency_revaluation_settings IS
    'Company-level settings for currency revaluation of receivables (411) and payables (401)';

COMMENT ON COLUMN currency_revaluation_settings.expense_account_id IS
    'Account 624 - Expenses from currency operations (used when foreign currency strengthens)';

COMMENT ON COLUMN currency_revaluation_settings.revenue_account_id IS
    'Account 724 - Revenue from currency operations (used when foreign currency weakens)';
