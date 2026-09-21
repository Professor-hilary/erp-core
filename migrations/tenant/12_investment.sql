-- =====================================================================
-- INVESTMENT APPRAISAL SUBMODULE (investment)
-- =====================================================================
CREATE SCHEMA IF NOT EXISTS investment;

CREATE TABLE investment.investment_programs (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    serial_id       BIGINT GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id      UUID NOT NULL,
    program_code    TEXT NOT NULL,
    name            TEXT NOT NULL,
    description     TEXT,
    horizon_start   DATE,
    horizon_end     DATE,
    owner_id        UUID,
    status          TEXT NOT NULL DEFAULT 'active'
                    CHECK (status IN ('draft','active','closed','cancelled')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (company_id, program_code)
);

CREATE TABLE investment.investment_cases (
    id                  UUID PRIMARY KEY DEFAULT uuidv7(),
    serial_id           BIGINT GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    case_code           TEXT NOT NULL,
    title               TEXT NOT NULL,
    description         TEXT,

    -- 1. Idea & classification
    investment_type     TEXT NOT NULL CHECK (investment_type IN (
                            'REPLACEMENT', 'EXPANSION', 'NEW_PRODUCT',
                            'REGULATORY', 'INFRASTRUCTURE', 'OTHER'
                        )),
    program_id          UUID REFERENCES investment.investment_programs(id),
    strategic_score     NUMERIC(5,2),          -- 0–100 or 1–5 scaled
    strategic_notes     TEXT,
    risk_rating         TEXT CHECK (risk_rating IN ('LOW','MEDIUM','HIGH','CRITICAL')),

    -- Lifecycle stage-gate
    stage               TEXT NOT NULL DEFAULT 'IDEA' CHECK (stage IN (
                            'IDEA', 'FEASIBILITY', 'APPROVAL', 'BUDGETING',
                            'EXECUTION', 'POST_AUDIT', 'REJECTED', 'CANCELLED'
                        )),
    stage_changed_at    TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Links (filled as stage advances)
    project_id          UUID REFERENCES capital.capital_projects(id),  -- at approval/budgeting
    currency_id         UUID NOT NULL REFERENCES accounting.currencies(id),
    hurdle_rate         NUMERIC(12,8),         -- override; else WACC
    wacc_as_of          DATE,                  -- which WACC snapshot used

    -- Cached base-case metrics (from last calc)
    npv                 NUMERIC(24,6),
    irr                 NUMERIC(12,8),
    mirr                NUMERIC(12,8),
    payback_years       NUMERIC(8,2),
    discounted_payback  NUMERIC(8,2),
    profitability_index NUMERIC(12,6),
    arr                 NUMERIC(12,8),
    roce                NUMERIC(12,8),
    metrics_calculated_at TIMESTAMPTZ,

    -- Post-audit
    actual_irr          NUMERIC(12,8),
    actual_npv          NUMERIC(24,6),
    audit_completed_at  DATE,
    lessons_learned     TEXT,

    initiator_id        UUID,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (company_id, case_code)
);

CREATE INDEX idx_inv_cases_company_stage ON investment.investment_cases(company_id, stage);
CREATE INDEX idx_inv_cases_program ON investment.investment_cases(program_id);

-- Scenarios: Base / Best / Worst (extensible)
CREATE TABLE investment.investment_scenarios (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    case_id         UUID NOT NULL REFERENCES investment.investment_cases(id) ON DELETE CASCADE,
    scenario_code   TEXT NOT NULL CHECK (scenario_code IN ('BASE','BEST','WORST','CUSTOM')),
    name            TEXT NOT NULL,
    probability     NUMERIC(8,6) CHECK (probability IS NULL OR (probability >= 0 AND probability <= 1)),
    inflation_rate  NUMERIC(12,8) DEFAULT 0,   -- annual
    is_primary      BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (case_id, scenario_code)
);

-- Cash-flow builder lines (driver-level, not only totals)
CREATE TABLE investment.investment_cashflow_lines (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    serial_id       BIGINT GENERATED ALWAYS AS IDENTITY NOT NULL,
    scenario_id     UUID NOT NULL REFERENCES investment.investment_scenarios(id) ON DELETE CASCADE,
    period_no       INTEGER NOT NULL CHECK (period_no >= 0),  -- 0 = initial
    period_date     DATE,                                     -- optional calendar anchor
    line_type       TEXT NOT NULL CHECK (line_type IN (
                        'CAPEX', 'OPEX', 'WORKING_CAPITAL', 'REVENUE',
                        'SALVAGE', 'TAX', 'DEPRECIATION', 'OTHER_INFLOW', 'OTHER_OUTFLOW'
                    )),
    description     TEXT,
    amount          NUMERIC(24,6) NOT NULL,  -- signed: + inflow, - outflow (dep often 0 cash)
    is_cash         BOOLEAN NOT NULL DEFAULT true,  -- false for depreciation (ARR only)
    tax_deductible  BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (scenario_id, period_no, line_type, description)
);

-- Depreciation assumptions for modelling (not FA books)
CREATE TABLE investment.investment_dep_assumptions (
    id                  UUID PRIMARY KEY DEFAULT uuidv7(),
    scenario_id         UUID NOT NULL REFERENCES investment.investment_scenarios(id) ON DELETE CASCADE,
    asset_label         TEXT NOT NULL,              -- e.g. "CNC"
    cost                NUMERIC(24,6) NOT NULL,
    method              TEXT NOT NULL CHECK (method IN ('STRAIGHT_LINE','REDUCING_BALANCE')),
    life_years          INTEGER NOT NULL CHECK (life_years > 0),
    residual_value      NUMERIC(24,6) NOT NULL DEFAULT 0,
    tax_class           TEXT,                       -- Class1/2/3/Building (URA)
    tax_rate            NUMERIC(8,4),               -- annual tax dep rate %
    start_period        INTEGER NOT NULL DEFAULT 1,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Stored metric results per scenario (audit + speed)
CREATE TABLE investment.investment_metrics (
    id                  UUID PRIMARY KEY DEFAULT uuidv7(),
    scenario_id         UUID NOT NULL REFERENCES investment.investment_scenarios(id) ON DELETE CASCADE,
    discount_rate       NUMERIC(12,8) NOT NULL,
    npv                 NUMERIC(24,6),
    irr                 NUMERIC(12,8),
    mirr                NUMERIC(12,8),
    payback_years       NUMERIC(8,2),
    discounted_payback  NUMERIC(8,2),
    profitability_index NUMERIC(12,6),
    arr                 NUMERIC(12,8),
    roce                NUMERIC(12,8),
    total_capex         NUMERIC(24,6),
    total_pv_inflows    NUMERIC(24,6),
    calculated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    calculated_by       UUID,
    UNIQUE (scenario_id, calculated_at)  -- or keep one row + history table
);

CREATE TABLE investment.investment_sensitivity_runs (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    case_id         UUID NOT NULL REFERENCES investment.investment_cases(id) ON DELETE CASCADE,
    scenario_id     UUID REFERENCES investment.investment_scenarios(id),
    name            TEXT,
    created_by      UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tornado inputs: one variable shocked at a time
CREATE TABLE investment.investment_sensitivity_results (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    run_id          UUID NOT NULL REFERENCES investment.investment_sensitivity_runs(id) ON DELETE CASCADE,
    variable_name   TEXT NOT NULL,           -- e.g. CAPEX, REVENUE, OPEX, WACC
    shock_pct       NUMERIC(8,4) NOT NULL,   -- e.g. 0.20 = +20%
    npv_result      NUMERIC(24,6),
    irr_result      NUMERIC(12,8),
    base_npv        NUMERIC(24,6),
    delta_npv       NUMERIC(24,6)
);

CREATE TABLE investment.investment_monte_carlo_runs (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    case_id         UUID NOT NULL REFERENCES investment.investment_cases(id) ON DELETE CASCADE,
    scenario_id     UUID REFERENCES investment.investment_scenarios(id),
    iterations      INTEGER NOT NULL CHECK (iterations > 0),
    npv_mean        NUMERIC(24,6),
    npv_p5          NUMERIC(24,6),
    npv_p50         NUMERIC(24,6),
    npv_p95         NUMERIC(24,6),
    prob_npv_positive NUMERIC(8,6),
    payload         JSONB,                   -- distribution histogram optional
    created_by      UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE investment.investment_risks (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    case_id         UUID NOT NULL REFERENCES investment.investment_cases(id) ON DELETE CASCADE,
    risk_code       TEXT,
    title           TEXT NOT NULL,
    category        TEXT,                    -- MARKET, TECH, REGULATORY, OPS, FINANCIAL
    likelihood      TEXT CHECK (likelihood IN ('RARE','UNLIKELY','POSSIBLE','LIKELY','ALMOST_CERTAIN')),
    impact          TEXT CHECK (impact IN ('INSIGNIFICANT','MINOR','MODERATE','MAJOR','SEVERE')),
    score           INTEGER,                 -- derived 1–25
    mitigation      TEXT,
    owner_id        UUID,
    status          TEXT NOT NULL DEFAULT 'open'
                    CHECK (status IN ('open','mitigating','closed','accepted')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Phased budget (annual / quarterly) — can mirror budget-control module later
CREATE TABLE investment.investment_budget_phases (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    case_id         UUID NOT NULL REFERENCES investment.investment_cases(id) ON DELETE CASCADE,
    phase_no        INTEGER NOT NULL,
    fiscal_year     INTEGER,
    period_label    TEXT,                    -- FY2026-Q1
    amount          NUMERIC(24,6) NOT NULL,
    amount_spent    NUMERIC(24,6) NOT NULL DEFAULT 0,
    carry_forward   NUMERIC(24,6) NOT NULL DEFAULT 0,
    UNIQUE (case_id, phase_no)
);

CREATE TABLE investment.investment_funding_plan (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    case_id         UUID NOT NULL REFERENCES investment.investment_cases(id) ON DELETE CASCADE,
    source_type     TEXT NOT NULL CHECK (source_type IN (
                        'EQUITY', 'LOAN', 'GRANT', 'INTERNAL', 'OTHER'
                    )),
    source_id       UUID,                    -- instrument / facility / grant ref
    amount          NUMERIC(24,6) NOT NULL,
    currency_id     UUID REFERENCES accounting.currencies(id),
    -- loan helpers (optional)
    interest_rate   NUMERIC(12,8),
    tenor_months    INTEGER,
    amortization_json JSONB,                 -- precomputed schedule snapshot
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE investment.investment_authority_rules (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    company_id      UUID NOT NULL,
    min_amount      NUMERIC(24,6) NOT NULL DEFAULT 0,
    max_amount      NUMERIC(24,6),           -- NULL = no upper bound
    currency_id     UUID REFERENCES accounting.currencies(id),
    role_required   TEXT NOT NULL,           -- CFO, BOARD, CEO, FINANCE_MANAGER
    stage           TEXT NOT NULL DEFAULT 'APPROVAL',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE investment.investment_approvals (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    case_id         UUID NOT NULL REFERENCES investment.investment_cases(id) ON DELETE CASCADE,
    stage           TEXT NOT NULL,
    actor_id        UUID NOT NULL,
    action          TEXT NOT NULL CHECK (action IN (
                        'SUBMIT', 'APPROVE', 'REJECT', 'RETURN', 'COMMENT'
                    )),
    role_at_action  TEXT,
    comment         TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Assumption / cashflow change audit
CREATE TABLE investment.investment_change_log (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    case_id         UUID NOT NULL REFERENCES investment.investment_cases(id) ON DELETE CASCADE,
    entity_type     TEXT NOT NULL,           -- CASE, CASHFLOW_LINE, SCENARIO, RISK
    entity_id       UUID,
    field_name      TEXT,
    old_value       TEXT,
    new_value       TEXT,
    changed_by      UUID NOT NULL,
    changed_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE investment.investment_documents (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    case_id         UUID NOT NULL REFERENCES investment.investment_cases(id) ON DELETE CASCADE,
    doc_type        TEXT NOT NULL,           -- FEASIBILITY, CONTRACT, BOARD_PACK, OTHER
    file_name       TEXT NOT NULL,
    storage_key     TEXT NOT NULL,           -- S3/path
    uploaded_by     UUID,
    uploaded_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION investment.required_approval_role(
    p_company_id UUID,
    p_amount NUMERIC,
    p_currency_id UUID
) RETURNS TEXT
LANGUAGE sql STABLE AS $$
    SELECT role_required
    FROM investment.investment_authority_rules
    WHERE company_id = p_company_id
      AND (currency_id IS NULL OR currency_id = p_currency_id)
      AND p_amount >= min_amount
      AND (max_amount IS NULL OR p_amount < max_amount)
    ORDER BY min_amount DESC
    LIMIT 1;
$$;

-- Snapshot actuals vs plan (periodic)
CREATE TABLE investment.investment_execution_snapshots (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    case_id         UUID NOT NULL REFERENCES investment.investment_cases(id) ON DELETE CASCADE,
    as_of_date      DATE NOT NULL,
    budget_total    NUMERIC(24,6),
    committed       NUMERIC(24,6),          -- POs etc. if available
    actual_spent    NUMERIC(24,6),          -- from project.spent_to_date / costs
    forecast_at_completion NUMERIC(24,6),
    -- light EVM (optional)
    planned_value   NUMERIC(24,6),
    earned_value    NUMERIC(24,6),
    spi             NUMERIC(12,6),
    cpi             NUMERIC(12,6),
    notes           TEXT,
    UNIQUE (case_id, as_of_date)
);

-- Capex → FA link when asset goes live
CREATE TABLE investment.investment_asset_links (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    case_id         UUID NOT NULL REFERENCES investment.investment_cases(id) ON DELETE CASCADE,
    project_id      UUID REFERENCES capital.capital_projects(id),
    fixed_asset_id  UUID NOT NULL,          -- fixedassets.fixed_assets.asset_id
    capitalized_amount NUMERIC(24,6),
    capitalized_at  DATE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE investment.investment_post_audits (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    case_id         UUID NOT NULL REFERENCES investment.investment_cases(id) ON DELETE CASCADE,
    audit_date      DATE NOT NULL,
    promised_npv    NUMERIC(24,6),
    promised_irr    NUMERIC(12,8),
    actual_npv      NUMERIC(24,6),
    actual_irr      NUMERIC(12,8),
    actual_payback  NUMERIC(8,2),
    variance_notes  TEXT,
    lessons_learned TEXT,
    performance_link JSONB,                 -- e.g. { "downtime_hours_before": 12, "after": 3 }
    audited_by      UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Aggregate net cash by period for a scenario
-- CREATE OR REPLACE FUNCTION investment.scenario_cashflow_array(p_scenario_id UUID)
-- RETURNS NUMERIC[]
-- LANGUAGE sql STABLE AS $$
--     SELECT COALESCE(array_agg(net_cash ORDER BY period_no), ARRAY[]::NUMERIC[])
--     FROM investment.v_scenario_period_cashflows
--     WHERE scenario_id = p_scenario_id;
-- $$;

-- NPV: CF[0] + CF[1]/(1+r) + ...
CREATE OR REPLACE FUNCTION investment.calc_npv(p_rate NUMERIC, p_cfs NUMERIC[])
RETURNS NUMERIC
LANGUAGE plpgsql IMMUTABLE AS $$
DECLARE
    v_npv NUMERIC := 0;
    i INT;
BEGIN
    IF p_cfs IS NULL OR array_length(p_cfs, 1) IS NULL THEN
        RETURN 0;
    END IF;
    FOR i IN 1 .. array_length(p_cfs, 1) LOOP
        v_npv := v_npv + p_cfs[i] / POWER(1 + p_rate, i - 1);
    END LOOP;
    RETURN ROUND(v_npv, 6);
END;
$$;

-- Payback (undiscounted): first T where cumulative >= 0
CREATE OR REPLACE FUNCTION investment.calc_payback(p_cfs NUMERIC[])
RETURNS NUMERIC
LANGUAGE plpgsql IMMUTABLE AS $$
DECLARE
    cum NUMERIC := 0;
    i INT;
    prev NUMERIC;
BEGIN
    IF p_cfs IS NULL OR array_length(p_cfs, 1) IS NULL THEN
        RETURN NULL;
    END IF;
    FOR i IN 1 .. array_length(p_cfs, 1) LOOP
        prev := cum;
        cum := cum + p_cfs[i];
        IF cum >= 0 AND i > 1 THEN
            -- linear interpolate within period
            IF p_cfs[i] = 0 THEN
                RETURN (i - 1)::NUMERIC;
            END IF;
            RETURN (i - 2) + (ABS(prev) / p_cfs[i]);
        END IF;
    END LOOP;
    RETURN NULL; -- never recovers
END;
$$;

-- Profitability index = PV inflows / PV outflows (simplified: (NPV + |CF0|) / |CF0|)
CREATE OR REPLACE FUNCTION investment.calc_pi(p_rate NUMERIC, p_cfs NUMERIC[])
RETURNS NUMERIC
LANGUAGE plpgsql IMMUTABLE AS $$
DECLARE
    v_npv NUMERIC;
    v_init NUMERIC;
BEGIN
    IF p_cfs IS NULL OR array_length(p_cfs, 1) IS NULL OR p_cfs[1] = 0 THEN
        RETURN NULL;
    END IF;
    v_npv := investment.calc_npv(p_rate, p_cfs);
    v_init := ABS(p_cfs[1]);
    RETURN ROUND((v_npv + v_init) / v_init, 6);
END;
$$;

CREATE OR REPLACE FUNCTION investment.recalculate_scenario_metrics(
    p_scenario_id UUID,
    p_discount_rate NUMERIC,
    p_user_id UUID DEFAULT NULL
) RETURNS investment.investment_metrics
LANGUAGE plpgsql AS $$
DECLARE
    v_cfs NUMERIC[];
    v_npv NUMERIC;
    v_payback NUMERIC;
    v_pi NUMERIC;
    v_row investment.investment_metrics%ROWTYPE;
    v_case_id UUID;
BEGIN
    SELECT case_id INTO v_case_id FROM investment.investment_scenarios WHERE id = p_scenario_id;
    v_cfs := investment.scenario_cashflow_array(p_scenario_id);
    v_npv := investment.calc_npv(p_discount_rate, v_cfs);
    v_payback := investment.calc_payback(v_cfs);
    v_pi := investment.calc_pi(p_discount_rate, v_cfs);

    INSERT INTO investment.investment_metrics (
        scenario_id, discount_rate, npv, payback_years, profitability_index,
        calculated_at, calculated_by
    ) VALUES (
        p_scenario_id, p_discount_rate, v_npv, v_payback, v_pi, now(), p_user_id
    )
    RETURNING * INTO v_row;

    -- refresh case cache if this is primary BASE scenario
    UPDATE investment.investment_cases c
    SET npv = v_npv,
        payback_years = v_payback,
        profitability_index = v_pi,
        metrics_calculated_at = now(),
        updated_at = now()
    FROM investment.investment_scenarios s
    WHERE s.id = p_scenario_id
      AND s.case_id = c.id
      AND s.is_primary = true;

    RETURN v_row;
END;
$$;

CREATE OR REPLACE FUNCTION investment.advance_investment_case(
    p_case_id UUID,
    p_to_stage TEXT,
    p_actor_id UUID,
    p_comment TEXT DEFAULT NULL
) RETURNS investment.investment_cases
LANGUAGE plpgsql AS $$
DECLARE
    v_case investment.investment_cases%ROWTYPE;
    v_project_id UUID;
    v_budget NUMERIC;
BEGIN
    SELECT * INTO v_case FROM investment.investment_cases WHERE id = p_case_id FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Case not found' USING ERRCODE = 'P0002';
    END IF;

    INSERT INTO investment.investment_approvals (case_id, stage, actor_id, action, comment)
    VALUES (p_case_id, p_to_stage, p_actor_id, 'APPROVE', p_comment);

    -- On entering BUDGETING/EXECUTION from APPROVAL: ensure capital project exists
    IF p_to_stage IN ('BUDGETING', 'EXECUTION') AND v_case.project_id IS NULL THEN
        SELECT COALESCE(SUM(amount), 0) INTO v_budget
        FROM investment.investment_budget_phases WHERE case_id = p_case_id;

        IF v_budget = 0 THEN
            -- fallback: sum CAPEX from primary scenario
            SELECT COALESCE(SUM(l.amount), 0) INTO v_budget
            FROM investment.investment_scenarios s
            JOIN investment.investment_cashflow_lines l ON l.scenario_id = s.id
            WHERE s.case_id = p_case_id AND s.is_primary AND l.line_type = 'CAPEX';
            v_budget := ABS(v_budget);
        END IF;

        INSERT INTO capital.capital_projects (
            company_id, project_code, name, description, project_type,
            approved_budget, currency_id, expected_irr, expected_npv,
            expected_payback_years, risk_rating, status
        ) VALUES (
            v_case.company_id,
            v_case.case_code,
            v_case.title,
            v_case.description,
            'CAPEX',
            v_budget,
            v_case.currency_id,
            v_case.irr,
            v_case.npv,
            v_case.payback_years,
            v_case.risk_rating,
            'approved'
        )
        RETURNING id INTO v_project_id;

        v_case.project_id := v_project_id;
    END IF;

    UPDATE investment.investment_cases
    SET stage = p_to_stage,
        stage_changed_at = now(),
        project_id = COALESCE(v_project_id, project_id),
        updated_at = now()
    WHERE id = p_case_id
    RETURNING * INTO v_case;

    RETURN v_case;
END;
$$;

CREATE OR REPLACE VIEW investment.v_portfolio_board AS
SELECT
    c.company_id,
    p.program_code,
    p.name AS program_name,
    c.id AS case_id,
    c.case_code,
    c.title,
    c.investment_type,
    c.stage,
    c.risk_rating,
    c.strategic_score,
    c.npv,
    c.irr,
    c.payback_years,
    c.project_id,
    pr.approved_budget,
    pr.spent_to_date,
    CASE WHEN pr.approved_budget > 0
         THEN ROUND(pr.spent_to_date / pr.approved_budget, 4)
         ELSE NULL END AS budget_burn_pct
FROM investment.investment_cases c
LEFT JOIN investment.investment_programs p ON p.id = c.program_id
LEFT JOIN capital.capital_projects pr ON pr.id = c.project_id
WHERE c.stage NOT IN ('REJECTED', 'CANCELLED');
