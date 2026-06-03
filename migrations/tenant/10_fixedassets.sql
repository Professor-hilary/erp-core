-- =============================================
-- IMPROVED FIXED ASSETS SCHEMA
-- =============================================

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 1. Asset Classes
CREATE TABLE asset_classes (
    class_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    class_name VARCHAR(100) NOT NULL,
    category_type VARCHAR(50) NOT NULL CHECK (category_type IN ('PPE', 'INTANGIBLE', 'ROU', 'INVESTMENT_PROPERTY')),
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 2. Fixed Assets (Main Table)
CREATE TABLE fixed_assets (
    asset_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,

    asset_code VARCHAR(50) UNIQUE NOT NULL,
    asset_name VARCHAR(200) NOT NULL,
    description TEXT,

    class_id UUID REFERENCES asset_classes(class_id),
    location VARCHAR(150),
    department VARCHAR(100),
    custodian_id UUID, -- link to employees table

    acquisition_date DATE NOT NULL,
    supplier_id UUID,
    po_reference VARCHAR(50),

    original_cost NUMERIC(18,2) NOT NULL,
    capitalized_amount NUMERIC(18,2),
    is_capitalized BOOLEAN DEFAULT FALSE,
    capitalization_date DATE,

    financing_method VARCHAR(50) CHECK (financing_method IN ('Cash', 'Bank_Loan', 'Supplier_Credit', 'Finance_Lease')),

    useful_life_years INTEGER NOT NULL CHECK (useful_life_years > 0),
    residual_value NUMERIC(18,2) DEFAULT 0,
    depreciation_method VARCHAR(30) DEFAULT 'Straight_Line'
        CHECK (depreciation_method IN ('Straight_Line', 'Declining_Balance', 'Units_of_Production')),
    depreciation_rate NUMERIC(8,4),
    depreciation_start_date DATE,

    status VARCHAR(30) DEFAULT 'In_Use'
        CHECK (status IN ('In_Use', 'Idle', 'Under_Repair', 'Disposed', 'Impaired')),

    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 3. Capital Work in Progress (CWIP)
CREATE TABLE asset_cwip (
    cwip_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    project_name VARCHAR(200) NOT NULL,
    total_accumulated_cost NUMERIC(18,2) DEFAULT 0,
    start_date DATE NOT NULL,
    expected_completion_date DATE,
    status VARCHAR(30) DEFAULT 'In_Progress',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 4. Asset Books (Multiple depreciation books per asset)
CREATE TABLE asset_books (
    book_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    asset_id UUID REFERENCES fixed_assets(asset_id) ON DELETE CASCADE,
    book_type VARCHAR(50) NOT NULL, -- 'FINANCIAL', 'TAX', 'IFRS', 'MANAGEMENT'
    useful_life_years INTEGER,
    depreciation_method VARCHAR(30),
    depreciation_rate NUMERIC(8,4),
    residual_value NUMERIC(18,2),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 5. Depreciation History
CREATE TABLE asset_depreciation (
    dep_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    asset_id UUID REFERENCES fixed_assets(asset_id) ON DELETE CASCADE,
    book_id UUID REFERENCES asset_books(book_id),
    period_date DATE NOT NULL,
    depreciation_amount NUMERIC(18,2) NOT NULL,
    accumulated_depreciation NUMERIC(18,2) NOT NULL,
    nbv NUMERIC(18,2) NOT NULL,
    posted_to_gl BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 6. Asset Components
CREATE TABLE asset_components (
    component_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    asset_id UUID REFERENCES fixed_assets(asset_id) ON DELETE CASCADE,
    component_name VARCHAR(150) NOT NULL,
    cost NUMERIC(18,2) NOT NULL,
    useful_life_years INTEGER,
    depreciation_start_date DATE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 7. Maintenance
CREATE TABLE asset_maintenance (
    maintenance_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    asset_id UUID REFERENCES fixed_assets(asset_id) ON DELETE CASCADE,
    maintenance_type VARCHAR(50),
    description TEXT,
    cost NUMERIC(18,2),
    maintenance_date DATE NOT NULL,
    next_due_date DATE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 8. Insurance
CREATE TABLE asset_insurance (
    insurance_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    asset_id UUID REFERENCES fixed_assets(asset_id) ON DELETE CASCADE,
    policy_number VARCHAR(100),
    insurer VARCHAR(150),
    insured_amount NUMERIC(18,2),
    start_date DATE,
    expiry_date DATE,
    premium NUMERIC(18,2),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 9. Transfers, Revaluations, Disposals (Unified Transaction Log)
CREATE TABLE asset_transactions (
    trans_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    asset_id UUID REFERENCES fixed_assets(asset_id) ON DELETE CASCADE,
    transaction_type VARCHAR(50) NOT NULL, -- Transfer, Revaluation, Disposal, Impairment
    transaction_date DATE NOT NULL,
    from_location VARCHAR(150),
    to_location VARCHAR(150),
    amount NUMERIC(18,2),
    proceeds NUMERIC(18,2),
    gain_loss NUMERIC(18,2),
    reason TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_fixed_assets_user_id ON fixed_assets(user_id);
CREATE INDEX idx_fixed_assets_class_id ON fixed_assets(class_id);
CREATE INDEX idx_fixed_assets_status ON fixed_assets(status);
CREATE INDEX idx_asset_depreciation_asset_period ON asset_depreciation(asset_id, period_date);
CREATE INDEX idx_asset_transactions_asset_id ON asset_transactions(asset_id);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER tg_asset_classes_updated BEFORE UPDATE ON asset_classes
FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER tg_fixed_assets_updated BEFORE UPDATE ON fixed_assets
FOR EACH ROW EXECUTE FUNCTION update_timestamp();


-- Asset Register View (Main Report)
CREATE VIEW vw_fixed_asset_register AS
SELECT
    fa.asset_code,
    fa.asset_name,
    ac.class_name,
    fa.location,
    fa.original_cost,
    fa.acquisition_date,
    fa.status,
    COALESCE(SUM(ad.accumulated_financial_dep), 0) AS accumulated_dep,
    fa.original_cost - COALESCE(SUM(ad.accumulated_financial_dep), 0) AS nbv_financial,
    fa.financing_method,
    fa.useful_life_years
FROM fixed_assets fa
LEFT JOIN asset_classes ac ON fa.class_id = ac.class_id
LEFT JOIN asset_depreciation ad ON fa.asset_id = ad.asset_id
GROUP BY fa.asset_id, fa.asset_code, fa.asset_name, ac.class_name,
         fa.location, fa.original_cost, fa.acquisition_date,
         fa.status, fa.financing_method, fa.useful_life_years;

-- Depreciation Schedule View
CREATE VIEW vw_depreciation_schedule AS
SELECT
    fa.asset_code,
    fa.asset_name,
    ad.period_date,
    ad.financial_depreciation,
    ad.tax_depreciation,
    ad.nbv_financial,
    ad.nbv_tax
FROM asset_depreciation ad
JOIN fixed_assets fa ON ad.asset_id = fa.asset_id
ORDER BY fa.asset_code, ad.period_date;

-- Function: Capitalize an Asset
CREATE OR REPLACE FUNCTION capitalize_asset(
    p_asset_id INTEGER,
    p_capitalized_amount NUMERIC,
    p_date DATE
) RETURNS VOID AS $$
BEGIN
    UPDATE fixed_assets
    SET is_capitalized = TRUE,
        capitalization_date = p_date,
        capitalized_amount = p_capitalized_amount,
        depreciation_start_date = p_date
    WHERE asset_id = p_asset_id;

    -- Create first depreciation record if needed
END;
$$ LANGUAGE plpgsql;

-- Function: Calculate Monthly Depreciation
CREATE OR REPLACE FUNCTION calculate_depreciation(
    p_asset_id INTEGER,
    p_period_date DATE
) RETURNS TABLE (
    financial_dep NUMERIC,
    tax_dep NUMERIC,
    nbv_financial NUMERIC,
    nbv_tax NUMERIC
) AS $$
DECLARE
    v_cost NUMERIC;
    v_residual NUMERIC;
    v_life INTEGER;
    v_method TEXT;
    v_rate NUMERIC;
BEGIN
    SELECT original_cost, residual_value, useful_life_years,
           depreciation_method, tax_depreciation_rate
    INTO v_cost, v_residual, v_life, v_method, v_rate
    FROM fixed_assets WHERE asset_id = p_asset_id;

    -- Financial Depreciation Logic (example for Straight Line)
    IF v_method = 'Straight_Line' THEN
        financial_dep := (v_cost - v_residual) / v_life / 12;
    END IF;

    -- Add logic for Declining Balance, Units of Production...

    -- Tax Depreciation (Declining Balance per URA)
    tax_dep := v_cost * (v_rate / 100) / 12;   -- Simplified

    RETURN NEXT;
END;
$$ LANGUAGE plpgsql;

-- Procedure: Dispose Asset
CREATE OR REPLACE PROCEDURE dispose_asset(
    p_asset_id INTEGER,
    p_disposal_date DATE,
    p_proceeds NUMERIC,
    p_reason TEXT
)
LANGUAGE plpgsql AS $$
DECLARE
    v_nbv NUMERIC;
BEGIN
    -- Calculate current NBV
    SELECT original_cost - COALESCE(SUM(accumulated_financial_dep), 0)
    INTO v_nbv
    FROM fixed_assets fa
    LEFT JOIN asset_depreciation ad ON fa.asset_id = ad.asset_id
    WHERE fa.asset_id = p_asset_id;

    INSERT INTO asset_transactions (
        asset_id, transaction_type, transaction_date,
        proceeds, gain_loss, reason
    ) VALUES (
        p_asset_id, 'Disposal', p_disposal_date,
        p_proceeds, p_proceeds - v_nbv, p_reason
    );

    UPDATE fixed_assets
    SET status = 'Disposed'
    WHERE asset_id = p_asset_id;
END;
$$;

-- Indexes
CREATE INDEX idx_assets_status ON fixed_assets(status);
CREATE INDEX idx_assets_location ON fixed_assets(location);
CREATE INDEX idx_depreciation_asset_period ON asset_depreciation(asset_id, period_date);

-- Trigger for Updated At
CREATE TRIGGER tg_fixed_assets_updated
BEFORE UPDATE ON fixed_assets
FOR EACH ROW EXECUTE FUNCTION update_timestamp();
