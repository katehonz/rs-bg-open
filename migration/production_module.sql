-- Production Module Schema
-- Technology Cards (Технологични карти/Рецепти)

CREATE TABLE IF NOT EXISTS technology_cards (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    output_unit VARCHAR(50) NOT NULL, -- единица мярка на крайния продукт (кг, бр, л)
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by INTEGER REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_technology_cards_company ON technology_cards(company_id);
CREATE INDEX idx_technology_cards_active ON technology_cards(is_active);

-- Technology Card Stages (Етапи на технологична карта)
CREATE TABLE IF NOT EXISTS technology_card_stages (
    id SERIAL PRIMARY KEY,
    technology_card_id INTEGER NOT NULL REFERENCES technology_cards(id) ON DELETE CASCADE,
    stage_number INTEGER NOT NULL, -- ред на етапа (1, 2, 3...)
    name VARCHAR(255) NOT NULL,
    debit_account_id INTEGER NOT NULL REFERENCES accounts(id),
    credit_account_id INTEGER NOT NULL REFERENCES accounts(id),
    quantity_formula TEXT, -- формула за количество (напр. "input_quantity", "input_quantity * 1.05")
    unit_of_measure VARCHAR(50), -- единица мярка за този етап
    amount_formula TEXT, -- формула за сума (напр. "quantity * unit_price", "previous_stage_amount")
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(technology_card_id, stage_number)
);

CREATE INDEX idx_tech_card_stages_card ON technology_card_stages(technology_card_id);

-- Production Batches (Производствени партиди)
CREATE TABLE IF NOT EXISTS production_batches (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    technology_card_id INTEGER NOT NULL REFERENCES technology_cards(id),
    batch_number VARCHAR(100) NOT NULL, -- номер на партида
    input_quantity NUMERIC(15, 4) NOT NULL, -- входно количество за производство
    production_date DATE NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'draft', -- draft, in_progress, completed, cancelled
    notes TEXT,
    created_by INTEGER REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(company_id, batch_number)
);

CREATE INDEX idx_production_batches_company ON production_batches(company_id);
CREATE INDEX idx_production_batches_status ON production_batches(status);
CREATE INDEX idx_production_batches_date ON production_batches(production_date);

-- Production Batch Stages (Изпълнени етапи на партида)
CREATE TABLE IF NOT EXISTS production_batch_stages (
    id SERIAL PRIMARY KEY,
    production_batch_id INTEGER NOT NULL REFERENCES production_batches(id) ON DELETE CASCADE,
    stage_number INTEGER NOT NULL,
    technology_card_stage_id INTEGER NOT NULL REFERENCES technology_card_stages(id),
    journal_entry_id INTEGER REFERENCES journal_entries(id) ON DELETE SET NULL, -- автоматично създадена операция
    quantity NUMERIC(15, 4), -- фактическо количество
    amount NUMERIC(15, 2), -- фактическа сума
    unit_of_measure VARCHAR(50),
    completed_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, completed, cancelled
    notes TEXT,
    UNIQUE(production_batch_id, stage_number)
);

CREATE INDEX idx_batch_stages_batch ON production_batch_stages(production_batch_id);
CREATE INDEX idx_batch_stages_journal ON production_batch_stages(journal_entry_id);

-- Comments
COMMENT ON TABLE technology_cards IS 'Технологични карти (рецепти) за производство';
COMMENT ON TABLE technology_card_stages IS 'Етапи на технологична карта със счетоводни сметки и формули';
COMMENT ON TABLE production_batches IS 'Производствени партиди - конкретни изпълнения на технологични карти';
COMMENT ON TABLE production_batch_stages IS 'Изпълнени етапи на производствена партида с автоматични счетоводни операции';
