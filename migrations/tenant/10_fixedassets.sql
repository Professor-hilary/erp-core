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

    tax_class VARCHAR(20) CHECK (tax_class IN ('1', '2', '3', '4', 'building', 'machinery'))

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
    financial_depreciation NUMERIC(18,2),
    accumulated_tax_depreciation NUMERIC(18,2),
    nbv NUMERIC(18,2) NOT NULL,
    nbv_financial NUMERIC(18,2),
    nbv_tax NUMERIC(18,2),
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

/*
async fn calculate_and_post_depreciation(
    &self,
    asset_id: Uuid,
    period_date: NaiveDate,
    user_id: Uuid,
) -> Result<AssetDepreciation, AppError> {
    let mut tx = self.pool.begin().await?;

    let dep_record = sqlx::query_as::<_, AssetDepreciation>(
        r#"
        SELECT * FROM calculate_depreciation_full($1, $2, $3)
        AS dep(financial_depreciation, tax_depreciation,
               accumulated_financial_dep, accumulated_tax_dep,
               nbv_financial, nbv_tax)
        "#,
    )
    .bind(user_id)
    .bind(asset_id)
    .bind(period_date)
    .fetch_one(&mut *tx)
    .await?;

    // Insert into depreciation table
    let inserted = self.repo.create_depreciation_record(&mut tx, asset_id, user_id, &dep_record).await?;

    tx.commit().await?;

    Ok(inserted)
}
*/

CREATE OR REPLACE FUNCTION calculate_depreciation_advanced(
    p_user_id UUID,
    p_asset_id UUID,
    p_period_date DATE
) RETURNS TABLE (
    depreciation_amount NUMERIC(18,2),
    accumulated_depreciation NUMERIC(18,2),
    nbv NUMERIC(18,2),
    notes TEXT
) AS $$
DECLARE
    v_cost NUMERIC(18,2);
    v_residual NUMERIC(18,2);
    v_life INTEGER;
    v_method TEXT;
    v_rate NUMERIC(8,4);
    v_start_date DATE;
    v_months_used INTEGER;
    v_total_months INTEGER;
BEGIN
    SELECT original_cost, residual_value, useful_life_years,
           depreciation_method, depreciation_rate, depreciation_start_date
    INTO v_cost, v_residual, v_life, v_method, v_rate, v_start_date
    FROM fixed_assets
    WHERE asset_id = p_asset_id AND user_id = p_user_id;

    IF v_start_date IS NULL OR p_period_date < v_start_date THEN
        RETURN QUERY SELECT 0::NUMERIC, 0::NUMERIC, v_cost, 'Depreciation not started yet';
        RETURN;
    END IF;

    v_total_months := v_life * 12;
    v_months_used := EXTRACT(YEAR FROM AGE(p_period_date, v_start_date))::int * 12
                     + EXTRACT(MONTH FROM AGE(p_period_date, v_start_date))::int + 1;

    CASE v_method
        WHEN 'Straight_Line' THEN
            depreciation_amount := ROUND((v_cost - v_residual) / v_total_months, 2);

        WHEN 'Declining_Balance' THEN
            depreciation_amount := ROUND(
                v_cost * (v_rate / 100) * (1 - POWER(1 - v_rate/100, v_months_used-1)), 2
            );

        WHEN 'Units_of_Production' THEN
            -- Requires units_used field. Simplified fallback:
            depreciation_amount := ROUND((v_cost - v_residual) * 0.08, 2); -- placeholder
        ELSE
            depreciation_amount := 0;
    END CASE;

    SELECT COALESCE(SUM(depreciation_amount), 0)
    INTO accumulated_depreciation
    FROM asset_depreciation
    WHERE asset_id = p_asset_id AND period_date <= p_period_date;

    nbv := v_cost - accumulated_depreciation;

    RETURN NEXT;
END;
$$ LANGUAGE plpgsql;

-- Function: Calculate Monthly Depreciation
CREATE OR REPLACE FUNCTION calculate_depreciation_full(
    p_user_id UUID,
    p_asset_id UUID,
    p_period_date DATE
) RETURNS TABLE (
    financial_depreciation NUMERIC(18,2),
    tax_depreciation NUMERIC(18,2),
    accumulated_financial_dep NUMERIC(18,2),
    accumulated_tax_dep NUMERIC(18,2),
    nbv_financial NUMERIC(18,2),
    nbv_tax NUMERIC(18,2)
) AS $$
DECLARE
    v_original_cost NUMERIC(18,2);
    v_residual NUMERIC(18,2);
    v_useful_life INTEGER;
    v_dep_method TEXT;
    v_tax_rate NUMERIC(8,4);
    v_tax_class TEXT;
    v_start_date DATE;
BEGIN
    SELECT original_cost, residual_value, useful_life_years,
           depreciation_method, tax_depreciation_rate, tax_class, depreciation_start_date
    INTO v_original_cost, v_residual, v_useful_life, v_dep_method,
         v_tax_rate, v_tax_class, v_start_date
    FROM fixed_assets
    WHERE asset_id = p_asset_id AND user_id = p_user_id;

    -- Financial Depreciation (Straight Line by default)
    financial_depreciation := ROUND((v_original_cost - v_residual) / (v_useful_life * 12.0), 2);

    -- Tax Depreciation (Common URA rules)
    CASE
        WHEN v_tax_class IN ('Class1', 'Class2') THEN
            tax_depreciation := ROUND(v_original_cost * (v_tax_rate / 100) / 12, 2);
        WHEN v_tax_class = 'Building' THEN
            tax_depreciation := ROUND(v_original_cost * 0.02 / 12, 2); -- 2% straight line
        ELSE
            tax_depreciation := financial_depreciation; -- fallback
    END CASE;

    -- Accumulated values
    SELECT
        COALESCE(SUM(financial_depreciation), 0),
        COALESCE(SUM(tax_depreciation), 0)
    INTO accumulated_financial_dep, accumulated_tax_dep
    FROM asset_depreciation
    WHERE asset_id = p_asset_id AND period_date < p_period_date;

    accumulated_financial_dep := accumulated_financial_dep + financial_depreciation;
    accumulated_tax_dep := accumulated_tax_dep + tax_depreciation;

    nbv_financial := v_original_cost - accumulated_financial_dep;
    nbv_tax := v_original_cost - accumulated_tax_dep;

    RETURN NEXT;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE PROCEDURE dispose_asset_advanced(
    p_user_id UUID,
    p_asset_id UUID,
    p_disposal_date DATE,
    p_proceeds NUMERIC(18,2),
    p_reason TEXT DEFAULT NULL,
    p_disposal_type TEXT DEFAULT 'Sale'
)
LANGUAGE plpgsql AS $$
DECLARE
    v_nbv NUMERIC(18,2);
    v_gain_loss NUMERIC(18,2);
BEGIN
    -- Lock row to prevent race conditions
    PERFORM * FROM fixed_assets
    WHERE asset_id = p_asset_id AND user_id = p_user_id FOR UPDATE;

    SELECT original_cost - COALESCE(SUM(accumulated_depreciation), 0)
    INTO v_nbv
    FROM fixed_assets fa
    LEFT JOIN asset_depreciation ad ON fa.asset_id = ad.asset_id
    WHERE fa.asset_id = p_asset_id AND fa.user_id = p_user_id;

    v_gain_loss := p_proceeds - v_nbv;

    INSERT INTO asset_transactions (
        user_id, asset_id, transaction_type, transaction_date,
        proceeds, gain_loss, reason
    ) VALUES (
        p_user_id, p_asset_id, 'Disposal', p_disposal_date,
        p_proceeds, v_gain_loss, p_reason
    );

    UPDATE fixed_assets
    SET status = 'Disposed'
    WHERE asset_id = p_asset_id AND user_id = p_user_id;
END;
$$;

CREATE OR REPLACE FUNCTION capitalize_asset_advanced(
    p_user_id UUID,
    p_asset_id UUID,
    p_cwip_id UUID DEFAULT NULL,
    p_capitalized_amount NUMERIC DEFAULT NULL,
    p_date DATE DEFAULT CURRENT_DATE
) RETURNS VOID AS $$
BEGIN
    -- If coming from CWIP
    IF p_cwip_id IS NOT NULL THEN
        UPDATE asset_cwip
        SET status = 'Completed', total_accumulated_cost = p_capitalized_amount
        WHERE cwip_id = p_cwip_id AND user_id = p_user_id;
    END IF;

    UPDATE fixed_assets
    SET is_capitalized = TRUE,
        capitalization_date = p_date,
        capitalized_amount = COALESCE(p_capitalized_amount, original_cost),
        depreciation_start_date = p_date,
        status = 'In_Use'
    WHERE asset_id = p_asset_id AND user_id = p_user_id;

    -- Create opening depreciation record
    INSERT INTO asset_depreciation (
        user_id, asset_id, period_date, depreciation_amount,
        accumulated_depreciation, nbv, posted_to_gl
    )
    VALUES (p_user_id, p_asset_id, p_date, 0, 0,
            (SELECT capitalized_amount FROM fixed_assets WHERE asset_id = p_asset_id),
            FALSE);
END;
$$ LANGUAGE plpgsql;


-- Capitalize CWIP / Asset
CREATE OR REPLACE FUNCTION capitalize_asset(
    p_user_id UUID,
    p_asset_id UUID,
    p_capitalized_amount NUMERIC,
    p_date DATE DEFAULT CURRENT_DATE
) RETURNS VOID AS $$
BEGIN
    UPDATE fixed_assets
    SET is_capitalized = TRUE,
        capitalization_date = p_date,
        capitalized_amount = p_capitalized_amount,
        depreciation_start_date = p_date,
        status = 'In_Use'
    WHERE asset_id = p_asset_id
      AND user_id = p_user_id;

    -- Optionally create initial depreciation record
    IF NOT EXISTS (
        SELECT 1 FROM asset_depreciation
        WHERE asset_id = p_asset_id AND period_date = p_date
    ) THEN
        INSERT INTO asset_depreciation (
            user_id, asset_id, period_date,
            depreciation_amount, accumulated_depreciation, nbv
        )
        SELECT
            p_user_id, p_asset_id, p_date,
            0, 0, p_capitalized_amount
        FROM fixed_assets
        WHERE asset_id = p_asset_id AND user_id = p_user_id;
    END IF;
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

-- =============================================
-- VIEWS
-- =============================================

-- Main Fixed Asset Register View
CREATE OR REPLACE VIEW vw_fixed_asset_register AS
SELECT
    fa.asset_id,
    fa.asset_code,
    fa.asset_name,
    ac.class_name,
    ac.category_type,
    fa.location,
    fa.department,
    fa.original_cost,
    fa.capitalized_amount,
    fa.acquisition_date,
    fa.capitalization_date,
    fa.status,
    fa.depreciation_method,
    fa.useful_life_years,
    COALESCE(SUM(ad.accumulated_depreciation), 0) AS accumulated_depreciation,
    fa.original_cost - COALESCE(SUM(ad.accumulated_depreciation), 0) AS nbv,
    fa.financing_method,
    fa.residual_value
FROM fixed_assets fa
LEFT JOIN asset_classes ac ON fa.class_id = ac.class_id
LEFT JOIN asset_depreciation ad ON fa.asset_id = ad.asset_id
WHERE fa.user_id = ac.user_id  -- Safety
GROUP BY
    fa.asset_id, fa.asset_code, fa.asset_name, ac.class_name, ac.category_type,
    fa.location, fa.department, fa.original_cost, fa.capitalized_amount,
    fa.acquisition_date, fa.capitalization_date, fa.status,
    fa.depreciation_method, fa.useful_life_years, fa.financing_method, fa.residual_value;

-- Depreciation Schedule View
CREATE OR REPLACE VIEW vw_depreciation_schedule AS
SELECT
    fa.asset_code,
    fa.asset_name,
    ad.period_date,
    ad.depreciation_amount,
    ad.accumulated_depreciation,
    ad.nbv,
    ad.posted_to_gl
FROM asset_depreciation ad
JOIN fixed_assets fa ON ad.asset_id = fa.asset_id
WHERE fa.user_id = ad.user_id
ORDER BY fa.asset_code, ad.period_date ASC;

-- Indexes
CREATE INDEX idx_assets_status ON fixed_assets(status);
CREATE INDEX idx_assets_location ON fixed_assets(location);
CREATE INDEX idx_depreciation_asset_period ON asset_depreciation(asset_id, period_date);

-- Trigger for Updated At
CREATE TRIGGER tg_fixed_assets_updated
BEFORE UPDATE ON fixed_assets
FOR EACH ROW EXECUTE FUNCTION update_timestamp();
