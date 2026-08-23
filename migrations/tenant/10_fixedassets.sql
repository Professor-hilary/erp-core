-- =============================================
-- IMPROVED FIXED ASSETS SCHEMA
-- =============================================

-- Create schema
CREATE SCHEMA IF NOT EXISTS fixedassets;
create sequence fixedassets.depreciation_reference_seq;

-- 1. Asset Classes
CREATE TABLE fixedassets.asset_classes (
    class_id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL,
    class_name VARCHAR(100) NOT NULL,
    category_type VARCHAR(50) NOT NULL CHECK (category_type IN ('PPE', 'INTANGIBLE', 'ROU', 'INVESTMENT_PROPERTY')),
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 2. Fixed Assets (Main Table)
CREATE TABLE fixedassets.fixed_assets (
    asset_id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL,
    asset_code VARCHAR(50) UNIQUE NOT NULL,
    asset_name VARCHAR(200) NOT NULL,
    description TEXT,
    class_id UUID REFERENCES fixedassets.asset_classes(class_id) ON DELETE SET NULL,

    location VARCHAR(150),
    department VARCHAR(100),
    custodian_id UUID, -- link to employees table

    tax_class VARCHAR(10) CHECK (
        tax_class IN ('Class1', 'Class2', 'Class3', 'Building')
    ),
    tax_depreciation_rate NUMERIC(8,4),

    acquisition_date DATE NOT NULL,
    supplier_id UUID,
    po_reference VARCHAR(50),

    original_cost NUMERIC(18,2) NOT NULL CHECK(original_cost > 0),
    capitalized_amount NUMERIC(18,2),
    is_capitalized BOOLEAN DEFAULT FALSE,
    capitalization_date DATE,

    accumulated_depr_acc VARCHAR(6),
    depreciation_exp_acc VARCHAR(6),

    financing_method VARCHAR(50) CHECK (financing_method IN ('Cash', 'Bank_Loan', 'Supplier_Credit', 'Finance_Lease')),

    useful_life_years INTEGER NOT NULL CHECK (useful_life_years > 0),
    residual_value NUMERIC(18,2) DEFAULT 0 CHECK(residual_value >= 0),
    depreciation_method VARCHAR(30) DEFAULT 'Straight_Line'
        CHECK (depreciation_method IN ('Straight_Line', 'Declining_Balance', 'Units_of_Production')),
    depreciation_rate NUMERIC(8,4),
    depreciation_start_date DATE,

    status VARCHAR(30) DEFAULT 'In_Use'
        CHECK (status IN ('In_Use', 'Idle', 'Under_Repair', 'Disposed', 'Impaired')),

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
-- Class    Assets Included                                                     Depr. Rate
-- Class1   = Computers and data handling equipment                             40% RBM
-- Class2   = Plant and Machinery for farming, mining, manufacturing            30% RBM
-- Class3   = Vehicles, furniture, fixtures, others not in class 1 or 2         20% RBM
-- Building = Industrial/Commercial buildings only                              05% SLM


-- 3. Capital Work in Progress (CWIP)
CREATE TABLE fixedassets.asset_cwip (
    cwip_id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL,
    project_name VARCHAR(200) NOT NULL,
    total_accumulated_cost NUMERIC(18,2) DEFAULT 0,
    start_date DATE NOT NULL,
    expected_completion_date DATE,
    status VARCHAR(30) DEFAULT 'In_Progress' CHECK (
        status IN ('Planned', 'In_Progress', 'Suspended', 'Completed', 'Capitalized', 'Cancelled', 'Impared', 'Disposed')
    ),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 4. Asset Books (Multiple depreciation books per asset)
CREATE TABLE fixedassets.asset_books (
    book_id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL,
    asset_id UUID REFERENCES fixedassets.fixed_assets(asset_id) ON DELETE CASCADE,
    book_type VARCHAR(50) NOT NULL CHECk(
        book_type IN ('INSURANCE', 'TAX', 'IFRS', 'MANAGEMENT', 'REVALUATION'
    )),
    useful_life_years INTEGER,
    depreciation_method VARCHAR(30),
    depreciation_rate NUMERIC(8,4),
    residual_value NUMERIC(18,2),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
-- alter table fixedassets.asset_depreciation add column  book_id UUID REFERENCES fixedassets.asset_books(book_id);
-- 5. Depreciation History
CREATE TABLE fixedassets.asset_depreciation (
    dep_id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL,
    asset_id UUID REFERENCES fixedassets.fixed_assets(asset_id) ON DELETE CASCADE,
    book_id UUID REFERENCES fixedassets.asset_books(book_id),
    period_date DATE NOT NULL,
    depreciation_amount NUMERIC(18,2) NOT NULL,
    accumulated_depreciation NUMERIC(18,2) NOT NULL,
    financial_depreciation NUMERIC(18,2),
    accumulated_tax_depreciation NUMERIC(18,2),
    transaction_reference varchar(50),
    nbv NUMERIC(18,2) NOT NULL,
    nbv_financial NUMERIC(18,2),
    twdv NUMERIC(18,2), -- Tax Written Down Value
    posted_to_gl BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 6. Asset Components
CREATE TABLE fixedassets.asset_components (
    component_id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL,
    asset_id UUID REFERENCES fixedassets.fixed_assets(asset_id) ON DELETE CASCADE,
    component_name VARCHAR(150) NOT NULL,
    cost NUMERIC(18,2) NOT NULL,
    useful_life_years INTEGER,
    depreciation_start_date DATE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 7. Maintenance
CREATE TABLE fixedassets.asset_maintenance (
    maintenance_id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL,
    asset_id UUID REFERENCES fixedassets.fixed_assets(asset_id) ON DELETE CASCADE,
    maintenance_type VARCHAR(50),
    description TEXT,
    cost NUMERIC(18,2),
    maintenance_date DATE NOT NULL,
    next_due_date DATE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 8. Insurance
CREATE TABLE fixedassets.asset_insurance (
    insurance_id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL,
    asset_id UUID REFERENCES fixedassets.fixed_assets(asset_id) ON DELETE CASCADE,
    policy_number VARCHAR(100),
    insurer VARCHAR(150),
    insured_amount NUMERIC(18,2),
    start_date DATE,
    expiry_date DATE,
    premium NUMERIC(18,2),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 9. Transfers, Revaluations, Disposals (Unified Transaction Log)
CREATE TABLE fixedassets.asset_transactions (
    trans_id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL,
    asset_id UUID REFERENCES fixedassets.fixed_assets(asset_id) ON DELETE CASCADE,
    transaction_type VARCHAR(50) NOT NULL, -- Transfer, Revaluation, Disposal, Impairment
    transaction_date DATE NOT NULL,
    from_location VARCHAR(150),
    to_location VARCHAR(150),
    amount NUMERIC(18,2),
    proceeds NUMERIC(18,2),
    gain_loss NUMERIC(18,2),
    reason TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_fixed_assets_user_id ON fixedassets.fixed_assets(user_id);
CREATE INDEX idx_fixed_assets_class_id ON fixedassets.fixed_assets(class_id);
CREATE INDEX idx_fixed_assets_status ON fixedassets.fixed_assets(status);
CREATE INDEX idx_asset_depreciation_asset_period ON fixedassets.asset_depreciation(asset_id, period_date);
CREATE INDEX idx_asset_transactions_asset_id ON fixedassets.asset_transactions(asset_id);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION fixedassets.update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function: Calculate Monthly Depreciation
CREATE OR REPLACE FUNCTION fixedassets.create_depreciation_entry(
    p_user_id UUID,
    p_asset_id UUID,
    p_period_date DATE
)
RETURNS TABLE (
    dep_id UUID,
    user_id UUID,
    asset_id UUID,
    book_id UUID,
    period_date DATE,
    depreciation_amount NUMERIC(18,2),
    accumulated_depreciation NUMERIC(18,2),
    nbv NUMERIC(18,2),
    posted_to_gl BOOLEAN,
    created_at TIMESTAMPTZ,
    transaction_reference varchar(50),
    financial_depreciation NUMERIC(18,2),
    accumulated_tax_depreciation NUMERIC(18,2),
    nbv_financial NUMERIC(18,2),
    twdv NUMERIC(18,2)
)
AS $$
DECLARE
    v_original_cost NUMERIC(18,2);
    v_residual NUMERIC(18,2);
    v_useful_life INTEGER;
    v_dep_method TEXT;
    v_tax_rate NUMERIC(8,4);
    v_tax_class TEXT;
    v_start_date DATE;

    v_financial_depr NUMERIC(18,2);
    v_tax_depr NUMERIC(18,2);

    v_accumulated_financial NUMERIC(18,2);
    v_accumulated_tax NUMERIC(18,2);

    v_nbv_financial NUMERIC(18,2);
    v_nbv_tax NUMERIC(18,2);
BEGIN
    /* Check that no depereciation entry is unposted */
    IF EXISTS (
        SELECT 1
        FROM fixedassets.asset_depreciation AS ad
        WHERE ad.asset_id = p_asset_id
        --   AND ad.book_id = p_book_id
          AND ad.posted_to_gl = FALSE
    ) THEN
        RAISE EXCEPTION 'Asset % already has an unposted depreciation entry',
            p_asset_id;
    END IF;

    /* Get asset information */
    SELECT
        fa.original_cost, fa.residual_value, fa.useful_life_years, fa.depreciation_method,
        fa.tax_depreciation_rate, fa.tax_class, fa.depreciation_start_date
    INTO
        v_original_cost, v_residual, v_useful_life, v_dep_method, v_tax_rate,
        v_tax_class, v_start_date
    FROM fixedassets.fixed_assets as fa
    WHERE fa.asset_id = p_asset_id
      AND fa.user_id = p_user_id;

    /* Make sure the asset actually exists. */

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Asset % does not exist for user %', p_asset_id, p_user_id;
    END IF;
    /* Financial depreciation */
    v_financial_depr := ROUND((v_original_cost - v_residual) / (v_useful_life * 12.0), 2);

    /* Tax depreciation */
    CASE
        WHEN v_tax_class IN ('Class1', 'Class2') THEN
            v_tax_depr := ROUND(v_original_cost * (v_tax_rate / 100) / 12, 2);
        WHEN v_tax_class = 'Building' THEN
            v_tax_depr := ROUND(v_original_cost * 0.02 / 12, 2);
        ELSE
            v_tax_depr := v_financial_depr;
    END CASE;

    /* Previous accumulated depreciation */
    SELECT
        COALESCE(SUM(ad.financial_depreciation), 0),
        COALESCE(SUM(ad.accumulated_tax_depreciation), 0)
    INTO
        v_accumulated_financial,
        v_accumulated_tax
    FROM fixedassets.asset_depreciation as ad
    WHERE ad.asset_id = p_asset_id AND ad.period_date < p_period_date;

    /* Add current period depreciation */
    v_accumulated_financial := v_accumulated_financial + v_financial_depr;
    v_accumulated_tax := v_accumulated_tax + v_tax_depr;

    /* Calculate NBVs */
    v_nbv_financial := v_original_cost - v_accumulated_financial;
    v_nbv_tax := v_original_cost - v_accumulated_tax;


    /* Return the shape expected by AssetDepreciation. */
    dep_id := gen_random_uuid();
    user_id := p_user_id;
    asset_id := p_asset_id;

    /*
     * Set this if the asset has a book relationship.
     * Otherwise NULL is appropriate.
     */
    SELECT ab.book_id INTO book_id FROM fixedassets.asset_books
      AS ab WHERE ab.asset_id = p_user_id;

    period_date := p_period_date;

    /*
     * Generic depreciation fields.
     *
     * Here we treat financial depreciation as
     * the primary accounting depreciation.
     */
    depreciation_amount := v_financial_depr;
    accumulated_depreciation := v_accumulated_financial;
    nbv := v_nbv_financial;

    posted_to_gl := FALSE;
    created_at := NOW();

    financial_depreciation := v_financial_depr;
    accumulated_tax_depreciation := v_accumulated_tax;
    nbv_financial := v_nbv_financial;

    /* TWDV = tax written-down value. */
    twdv := v_nbv_tax;

    RETURN NEXT;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION fixedassets.post_depreciation(
    p_user_id UUID,
    p_dep_id UUID
)
RETURNS fixedassets.asset_depreciation
LANGUAGE plpgsql
AS $$
DECLARE
    v_dep fixedassets.asset_depreciation%ROWTYPE;
    v_asset fixedassets.fixed_assets%ROWTYPE;
    v_book fixedassets.asset_books%ROWTYPE;
    v_journal_id BIGINT;
    v_reference TEXT;
    v_start_date DATE;
    v_result fixedassets.asset_depreciation%ROWTYPE;
BEGIN

    /* Get the depreciation record. */
    SELECT * INTO v_dep FROM fixedassets.asset_depreciation
    WHERE dep_id = p_dep_id AND user_id = p_user_id
    FOR UPDATE;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Depreciation record % was not found', p_dep_id;
    END IF;

    /* Prevent duplicate posting. */
    IF v_dep.posted_to_gl THEN
        RAISE EXCEPTION
            'Depreciation record % has already been posted', p_dep_id;
    END IF;

    /* Don't post before depreciation start date */
    SELECT depreciation_start_date INTO v_start_date FROM fixedassets.fixed_assets AS fa
        WHERE fa.asset_id = v_dep.asset_id;

    IF v_dep.period_date < v_start_date THEN
        RAISE EXCEPTION 'Cannot depreciate asset % before depreciation start date %',
        v_dep.asset_id, v_start_date;
    END IF;

    /* Don't post before depreciation start date */
    IF (
        SELECT 1 FROM fixedassets.asset_transactions AS at
          WHERE at.asset_id = v_dep.asset_id
          AND at.transaction_type = 'Disposal'
          AND at.transaction_date <= p_period_date
    ) THEN
        RAISE EXCEPTION 'Asset % has already been disposed and cannot be depreciated',
        v_dep.asset_id;
    END IF;

    /* Generate reference */
    v_reference := 'DEP_' || LPAD(
        nextval('fixedassets.depreciation_reference_seq')::TEXT, 6, '0'
    );

    /* Get the Asset Book. */
    SELECT * INTO v_asset FROM fixedassets.fixed_assets AS fa
    WHERE fa.asset_id = v_dep.asset_id
      AND fa.user_id = p_user_id;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Asset % was not found', v_dep.asset_id;
    END IF;

    /* Debit depreciation expense, Credit accumulated depreciation. */
    v_journal_id := accounting.post_transaction(
        v_reference,
        'Depreciation - Asset ' || v_dep.asset_id,
        p_user_id, 'depreciation', v_dep.period_date, jsonb_build_array(
            jsonb_build_object(
                'account_ref', v_asset.depreciation_exp_acc,
                'debit', v_dep.depreciation_amount,
                'credit', 0,
                'memo', 'Recognizing depreciation expenditure'
            ),
            jsonb_build_object(
                'account_ref', v_asset.accumulated_depr_acc,
                'debit', 0,
                'credit', v_dep.depreciation_amount,
                'memo', 'Acummulating depreciation'
            )
        )
    );

    /* Mark depreciation as posted. */
    UPDATE fixedassets.asset_depreciation
    SET
        posted_to_gl = TRUE, transaction_reference = v_reference
        -- journal_id = v_journal_id, posted_at = NOW(),
        -- posted_by = p_user_id
    WHERE dep_id = p_dep_id RETURNING * INTO v_result;

    RETURN v_result;
END;
$$;

CREATE OR REPLACE PROCEDURE fixedassets.dispose_asset_advanced(
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
    PERFORM * FROM fixedassets.fixed_assets
    WHERE asset_id = p_asset_id AND user_id = p_user_id FOR UPDATE;

    SELECT original_cost - COALESCE(SUM(accumulated_depreciation), 0)
    INTO v_nbv
    FROM fixedassets.fixed_assets fa
    LEFT JOIN fixedassets.asset_depreciation ad ON fa.asset_id = ad.asset_id
    WHERE fa.asset_id = p_asset_id AND fa.user_id = p_user_id;

    v_gain_loss := p_proceeds - v_nbv;

    INSERT INTO asset_transactions (
        user_id, asset_id, transaction_type, transaction_date,
        proceeds, gain_loss, reason
    ) VALUES (
        p_user_id, p_asset_id, 'Disposal', p_disposal_date,
        p_proceeds, v_gain_loss, p_reason
    );

    UPDATE fixedassets.fixed_assets
    SET status = 'Disposed'
    WHERE asset_id = p_asset_id AND user_id = p_user_id;
END;
$$;

-- Capitalize CWIP / Asset
CREATE OR REPLACE FUNCTION fixedassets.capitalize_asset(
    p_user_id UUID,
    p_asset_id UUID,
    p_cwip_id UUID DEFAULT NULL,
    p_capitalized_amount NUMERIC DEFAULT NULL,
    p_date DATE DEFAULT CURRENT_DATE,
    p_breakdown JSONB DEFAULT NULL
) RETURNS TABLE(
    asset_id            UUID,
    capitalized_amount  NUMERIC,
    breakdown           JSONB
) AS $$
    DECLARE v_final_amount NUMERIC := p_capitalized_amount;
BEGIN
    -- Calculate final capitalized amount
    IF v_final_amount IS NULL THEN
        SELECT fa.original_cost INTO v_final_amount
        FROM fixedassets.fixed_assets AS fa
        WHERE fa.asset_id = p_asset_id AND fa.user_id = p_user_id;
    END IF;

    -- If coming from CWIP
    IF p_cwip_id IS NOT NULL THEN
        UPDATE fixedassets.asset_cwip
        SET status = 'Capitalized',
            total_accumulated_cost = v_final_amount,
            updated_at = NOW()
        WHERE cwip_id = p_cwip_id AND user_id = p_user_id;
    END IF;

    UPDATE fixedassets.fixed_assets AS fa
    SET is_capitalized = TRUE,
        capitalization_date = p_date,
        capitalized_amount = v_final_amount,
        depreciation_start_date = p_date,
        status = 'In_Use',
        description = COALESCE(description, '') ||
            ' | Capitalized with breakdown: ' || p_breakdown::text
    WHERE fa.asset_id = p_asset_id
      AND fa.user_id = p_user_id;

    -- Create opening depreciation record
    INSERT INTO fixedassets.asset_depreciation (
        user_id, asset_id, period_date, depreciation_amount,
        accumulated_depreciation, nbv, posted_to_gl
    ) VALUES (
        p_user_id, p_asset_id, p_date, 0, 0, v_final_amount, FALSE
    );

    RETURN QUERY SELECT p_asset_id, v_final_amount, p_breakdown;
END;
$$ LANGUAGE plpgsql;

-- Procedure: Dispose Asset
CREATE OR REPLACE PROCEDURE fixedassets.dispose_asset(
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
-- Depreciation Schedule View
CREATE OR REPLACE VIEW fixedassets.vw_depreciation_schedule AS
SELECT
    fa.asset_code,
    fa.asset_name,
    ad.period_date,
    ad.depreciation_amount,
    ad.accumulated_depreciation,
    ad.nbv,
    ad.posted_to_gl
FROM fixedassets.asset_depreciation ad
JOIN fixedassets.fixed_assets fa ON ad.asset_id = fa.asset_id
WHERE fa.user_id = ad.user_id
ORDER BY fa.asset_code, ad.period_date ASC;

-- Main Fixed Asset Register View
CREATE OR REPLACE VIEW fixedassets.vw_fixed_asset_register AS
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
FROM fixedassets.fixed_assets fa
LEFT JOIN fixedassets.asset_classes ac ON fa.class_id = ac.class_id
LEFT JOIN fixedassets.asset_depreciation ad ON fa.asset_id = ad.asset_id
WHERE fa.user_id = ac.user_id  -- Safety
GROUP BY
    fa.asset_id, fa.asset_code, fa.asset_name, ac.class_name, ac.category_type,
    fa.location, fa.department, fa.original_cost, fa.capitalized_amount,
    fa.acquisition_date, fa.capitalization_date, fa.status,
    fa.depreciation_method, fa.useful_life_years, fa.financing_method, fa.residual_value;

-- Indexes
CREATE INDEX idx_assets_status ON fixedassets.fixed_assets(status);
CREATE INDEX idx_assets_location ON fixedassets.fixed_assets(location);
CREATE INDEX idx_depreciation_asset_period ON fixedassets.asset_depreciation(asset_id, period_date);

