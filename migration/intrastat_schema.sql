-- INTRASTAT Nomenclature table
CREATE TABLE IF NOT EXISTS intrastat_nomenclature (
    id SERIAL PRIMARY KEY,
    cn_code VARCHAR(10) NOT NULL UNIQUE,
    description_bg TEXT NOT NULL,
    description_en TEXT,
    unit_of_measure VARCHAR(20) NOT NULL,
    unit_description VARCHAR(100) NOT NULL,
    parent_code VARCHAR(10),
    level INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- INTRASTAT Account Mapping table
CREATE TABLE IF NOT EXISTS intrastat_account_mapping (
    id SERIAL PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    nomenclature_id INTEGER NOT NULL REFERENCES intrastat_nomenclature(id) ON DELETE CASCADE,
    flow_direction VARCHAR(10) NOT NULL CHECK (flow_direction IN ('ARRIVAL', 'DISPATCH')),
    transaction_nature_code VARCHAR(5) NOT NULL,
    is_quantity_tracked BOOLEAN NOT NULL DEFAULT true,
    default_country_code VARCHAR(2),
    default_transport_mode INTEGER,
    is_optional BOOLEAN NOT NULL DEFAULT false,
    min_threshold_bgn DECIMAL(15, 2),
    company_id INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(account_id, nomenclature_id, flow_direction, company_id)
);

-- INTRASTAT Settings table
CREATE TABLE IF NOT EXISTS intrastat_settings (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    is_enabled BOOLEAN NOT NULL DEFAULT false,
    arrival_threshold_bgn DECIMAL(15, 2) NOT NULL DEFAULT 400000.00,
    dispatch_threshold_bgn DECIMAL(15, 2) NOT NULL DEFAULT 400000.00,
    current_arrival_threshold_bgn DECIMAL(15, 2) NOT NULL DEFAULT 0.00,
    current_dispatch_threshold_bgn DECIMAL(15, 2) NOT NULL DEFAULT 0.00,
    auto_generate_declarations BOOLEAN NOT NULL DEFAULT false,
    default_transport_mode INTEGER,
    default_delivery_terms VARCHAR(10),
    default_transaction_nature VARCHAR(5),
    responsible_person_name VARCHAR(200),
    responsible_person_phone VARCHAR(50),
    responsible_person_email VARCHAR(200),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(company_id)
);

-- INTRASTAT Declaration table
CREATE TABLE IF NOT EXISTS intrastat_declaration (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    declaration_type VARCHAR(10) NOT NULL CHECK (declaration_type IN ('ARRIVAL', 'DISPATCH')),
    reference_period VARCHAR(6) NOT NULL,
    year INTEGER NOT NULL,
    month INTEGER NOT NULL CHECK (month >= 1 AND month <= 12),
    declaration_number VARCHAR(50),
    declarant_eik VARCHAR(20) NOT NULL,
    declarant_name VARCHAR(200) NOT NULL,
    contact_person VARCHAR(200) NOT NULL,
    contact_phone VARCHAR(50) NOT NULL,
    contact_email VARCHAR(200) NOT NULL,
    total_items INTEGER NOT NULL DEFAULT 0,
    total_statistical_value DECIMAL(15, 2) NOT NULL DEFAULT 0.00,
    total_invoice_value DECIMAL(15, 2) NOT NULL DEFAULT 0.00,
    status VARCHAR(15) NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'SUBMITTED', 'ACCEPTED', 'REJECTED')),
    submission_date TIMESTAMP WITH TIME ZONE,
    xml_file_path TEXT,
    created_by INTEGER NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(company_id, declaration_type, year, month)
);

-- INTRASTAT Declaration Items table
CREATE TABLE IF NOT EXISTS intrastat_declaration_item (
    id SERIAL PRIMARY KEY,
    declaration_id INTEGER NOT NULL REFERENCES intrastat_declaration(id) ON DELETE CASCADE,
    item_number INTEGER NOT NULL,
    cn_code VARCHAR(10) NOT NULL,
    nomenclature_id INTEGER REFERENCES intrastat_nomenclature(id),
    country_of_origin VARCHAR(2) NOT NULL,
    country_of_consignment VARCHAR(2) NOT NULL,
    transaction_nature_code VARCHAR(5) NOT NULL,
    transport_mode INTEGER NOT NULL,
    delivery_terms VARCHAR(10) NOT NULL,
    statistical_procedure VARCHAR(10),
    net_mass_kg DECIMAL(15, 3) NOT NULL,
    supplementary_unit DECIMAL(15, 3),
    invoice_value DECIMAL(15, 2) NOT NULL,
    statistical_value DECIMAL(15, 2) NOT NULL,
    currency_code VARCHAR(3) NOT NULL DEFAULT 'BGN',
    description TEXT NOT NULL,
    region_code VARCHAR(10),
    port_code VARCHAR(10),
    journal_entry_id INTEGER REFERENCES journal_entries(id),
    entry_line_id INTEGER REFERENCES entry_lines(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(declaration_id, item_number)
);

-- Indexes for better performance
CREATE INDEX idx_intrastat_nomenclature_cn_code ON intrastat_nomenclature(cn_code);
CREATE INDEX idx_intrastat_nomenclature_parent ON intrastat_nomenclature(parent_code);
CREATE INDEX idx_intrastat_account_mapping_account ON intrastat_account_mapping(account_id);
CREATE INDEX idx_intrastat_account_mapping_nomenclature ON intrastat_account_mapping(nomenclature_id);
CREATE INDEX idx_intrastat_declaration_company ON intrastat_declaration(company_id);
CREATE INDEX idx_intrastat_declaration_period ON intrastat_declaration(year, month);
CREATE INDEX idx_intrastat_declaration_item_declaration ON intrastat_declaration_item(declaration_id);
CREATE INDEX idx_intrastat_declaration_item_journal ON intrastat_declaration_item(journal_entry_id);

-- Add triggers for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_intrastat_nomenclature_updated_at BEFORE UPDATE ON intrastat_nomenclature
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_intrastat_account_mapping_updated_at BEFORE UPDATE ON intrastat_account_mapping
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_intrastat_settings_updated_at BEFORE UPDATE ON intrastat_settings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_intrastat_declaration_updated_at BEFORE UPDATE ON intrastat_declaration
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_intrastat_declaration_item_updated_at BEFORE UPDATE ON intrastat_declaration_item
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();