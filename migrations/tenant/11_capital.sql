-- ================================================================
-- CAPITAL & TREASURY SCHEMA (Enterprise Grade)
-- Supports: Debt | Equity | Hybrid Instruments | Retained Earnings
--           Capital Structure | Deployment | Liquidity | Analytics
-- ================================================================

CREATE SCHEMA IF NOT EXISTS capital;

-- =============================================
-- 0. SHARED / REFERENCE
-- =============================================

CREATE TABLE capital.currencies (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id       bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    code            CHAR(3) NOT NULL UNIQUE,          -- ISO 4217
    name            TEXT NOT NULL,
    decimal_places  SMALLINT NOT NULL DEFAULT 2,
    is_active       BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE capital.parties (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id       bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id      UUID,                             -- null = external party
    party_type      TEXT NOT NULL CHECK (party_type IN (
                        'LENDER', 'INVESTOR', 'SHAREHOLDER', 'GUARANTOR',
                        'AGENT', 'TRUSTEE', 'RATING_AGENCY', 'OTHER'
                    )),
    legal_name      TEXT NOT NULL,
    role            TEXT,
    short_name      TEXT,
    registration_number TEXT,
    country_code    CHAR(2),
    is_related_party BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.exchange_rates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id       bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    from_currency_id UUID NOT NULL REFERENCES capital.currencies(id),
    to_currency_id  UUID NOT NULL REFERENCES capital.currencies(id),
    rate_date       DATE NOT NULL,
    rate            NUMERIC(18,10) NOT NULL,
    source          TEXT,
    UNIQUE (from_currency_id, to_currency_id, rate_date)
);

-- =============================================
-- 1. CAPITAL INSTRUMENTS (Unified Source of Truth)
-- =============================================

CREATE TABLE capital.capital_instruments (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id       bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    instrument_code     TEXT NOT NULL,                 -- internal identifier
    name                TEXT NOT NULL,
    instrument_family   TEXT NOT NULL CHECK (instrument_family IN (
                            'DEBT', 'EQUITY', 'HYBRID', 'MEZZANINE', 'OTHER'
                        )),
    instrument_type     TEXT NOT NULL CHECK (instrument_type IN (
                            -- Debt
                            'TERM_LOAN', 'REVOLVING_CREDIT', 'BOND', 'NOTE',
                            'COMMERCIAL_PAPER', 'SHAREHOLDER_LOAN', 'INTERCOMPANY_LOAN',
                            'LEASE_LIABILITY', 'CONVERTIBLE_DEBT',
                            -- Equity
                            'COMMON_EQUITY', 'PREFERRED_EQUITY', 'TREASURY_SHARES',
                            -- Hybrid
                            'CONVERTIBLE_PREFERRED', 'PERPETUAL_BOND', 'WARRANT',
                            'OTHER'
                        )),
    currency_id         UUID NOT NULL REFERENCES capital.currencies(id),
    original_principal  NUMERIC(24,6) NOT NULL DEFAULT 0,
    outstanding_principal NUMERIC(24,6) NOT NULL DEFAULT 0,
    face_value          NUMERIC(24,6),
    issue_price         NUMERIC(18,8),                 -- % of par or absolute
    effective_date      DATE NOT NULL,
    maturity_date       DATE,
    is_perpetual        BOOLEAN NOT NULL DEFAULT false,
    ranking             TEXT CHECK (ranking IN (
                            'SENIOR_SECURED', 'SENIOR_UNSECURED', 'SUBORDINATED',
                            'MEZZANINE', 'EQUITY', 'OTHER'
                        )),
    is_callable         BOOLEAN NOT NULL DEFAULT false,
    is_putable          BOOLEAN NOT NULL DEFAULT false,
    is_convertible      BOOLEAN NOT NULL DEFAULT false,
    conversion_ratio    NUMERIC(18,8),
    conversion_price    NUMERIC(24,6),
    status              TEXT NOT NULL DEFAULT 'active' CHECK (status IN (
                            'draft', 'active', 'matured', 'called', 'converted',
                            'repurchased', 'cancelled', 'defaulted', 'restructured'
                        )),
    accounting_treatment TEXT,                         -- IFRS 9 / ASC 470 etc.
    journal_entry_id    BIGINT,                          -- link to GL
    notes               TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (company_id, instrument_code)
);

CREATE INDEX idx_capital_instruments_company ON capital.capital_instruments(company_id);
CREATE INDEX idx_capital_instruments_type ON capital.capital_instruments(instrument_family, instrument_type);
CREATE INDEX idx_capital_instruments_status ON capital.capital_instruments(status);

-- =============================================
-- 2. CAPITAL EVENTS (Audit / Lifecycle)
-- =============================================

CREATE TABLE capital.capital_events (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    instrument_id       UUID REFERENCES capital.capital_instruments(id),
    event_type          TEXT NOT NULL CHECK (event_type IN (
                            'ISSUANCE', 'DRAWDOWN', 'REPAYMENT', 'INTEREST_ACCRUAL',
                            'INTEREST_PAYMENT', 'AMORTIZATION', 'REPRICING',
                            'REFINANCING', 'CONVERSION', 'CALL', 'PUT',
                            'BUYBACK', 'CANCELLATION', 'COVENANT_BREACH',
                            'WAIVER', 'AMENDMENT', 'RESTRUCTURE',
                            'DIVIDEND_DECLARATION', 'DIVIDEND_PAYMENT',
                            'SHARE_ISSUE', 'SHARE_TRANSFER', 'BONUS_ISSUE',
                            'STOCK_SPLIT', 'REVERSE_SPLIT', 'WRITE_OFF',
                            'FAIR_VALUE_ADJUSTMENT','EQUITY_CONTRIBUTION', 'OTHER'
                        )),
    event_date          DATE NOT NULL,
    effective_date      DATE,
    amount              NUMERIC(24,2),
    currency_id         UUID REFERENCES capital.currencies(id),
    shares              BIGINT,
    description         TEXT,
    related_party_id    UUID REFERENCES capital.parties(id),
    journal_entry_id    BIGINT,
    created_by          UUID,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_capital_events_instrument ON capital.capital_events(instrument_id);
CREATE INDEX idx_capital_events_date ON capital.capital_events(event_date);

-- =============================================
-- 3. DEBT MODULE
-- =============================================

CREATE TABLE capital.debt_facilities (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id               bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id              UUID NOT NULL,
    instrument_id           UUID REFERENCES capital.capital_instruments(id), -- optional link
    facility_code           TEXT NOT NULL,
    facility_name           TEXT NOT NULL,
    facility_type           TEXT NOT NULL CHECK (facility_type IN (
                                'TERM_LOAN', 'REVOLVER', 'BRIDGE', 'DELAYED_DRAW',
                                'LETTER_OF_CREDIT', 'BANK_GUARANTEE', 'OVERDRAFT',
                                'SYNDICATED', 'BILATERAL', 'OTHER'
                            )),
    agent_id                UUID REFERENCES capital.parties(id),
    currency_id             UUID NOT NULL REFERENCES capital.currencies(id),
    committed_amount        NUMERIC(24,6) NOT NULL,
    available_amount        NUMERIC(24,6) NOT NULL,   -- remaining undrawn
    drawn_amount            NUMERIC(24,6) NOT NULL DEFAULT 0,
    interest_type           TEXT CHECK (interest_type IN (
                                'FIXED', 'FLOATING', 'HYBRID', 'ZERO_COUPON'
                            )),
    base_rate_index         TEXT,                     -- SOFR, EURIBOR, etc.
    margin_bps              NUMERIC(10,4),            -- spread in basis points
    floor_rate              NUMERIC(10,6),
    ceiling_rate            NUMERIC(10,6),
    day_count_convention    TEXT DEFAULT 'ACT/360',   -- ACT/360, 30/360, ACT/365
    payment_frequency       TEXT CHECK (payment_frequency IN (
                                'MONTHLY', 'QUARTERLY', 'SEMI_ANNUAL', 'ANNUAL', 'BULLET'
                            )),
    amortization_type       TEXT CHECK (amortization_type IN (
                                'BULLET', 'AMORTIZING', 'MORTGAGE', 'CUSTOM'
                            )),
    effective_date          DATE NOT NULL,
    maturity_date           DATE,
    commitment_fee_bps      NUMERIC(10,4),
    utilization_fee_bps     NUMERIC(10,4),
    prepayment_penalty      TEXT,
    collateral_required     BOOLEAN NOT NULL DEFAULT false,
    is_secured              BOOLEAN NOT NULL DEFAULT false,
    ranking                 TEXT,
    status                  TEXT NOT NULL DEFAULT 'active' CHECK (status IN (
                                'committed', 'active', 'fully_drawn', 'expired',
                                'cancelled', 'defaulted', 'restructured'
                            )),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (company_id, facility_code)
);

CREATE TABLE capital.debt_facility_lenders (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    facility_id         UUID NOT NULL REFERENCES capital.debt_facilities(id) ON DELETE CASCADE,
    lender_id           UUID NOT NULL REFERENCES capital.parties(id),
    commitment_amount   NUMERIC(24,6) NOT NULL,
    participation_pct   NUMERIC(8,4),                 -- for syndicated
    is_agent            BOOLEAN NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.debt_drawdowns (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    facility_id         UUID NOT NULL REFERENCES capital.debt_facilities(id),
    instrument_id       UUID REFERENCES capital.capital_instruments(id),
    drawdown_date       DATE NOT NULL,
    value_date          DATE,
    amount              NUMERIC(24,6) NOT NULL,
    currency_id         UUID NOT NULL REFERENCES capital.currencies(id),
    reference           TEXT,
    purpose             TEXT,
    journal_entry_id    BIGINT,
    status              TEXT NOT NULL DEFAULT 'posted' CHECK (status IN (
                            'requested', 'approved', 'posted', 'reversed'
                        )),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.debt_repayment_schedules (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    facility_id         UUID NOT NULL REFERENCES capital.debt_facilities(id),
    instrument_id       UUID REFERENCES capital.capital_instruments(id),
    sequence_no         INTEGER NOT NULL,
    due_date            DATE NOT NULL,
    principal_due       NUMERIC(24,6) NOT NULL DEFAULT 0,
    interest_due        NUMERIC(24,6) NOT NULL DEFAULT 0,
    fee_due             NUMERIC(24,6) NOT NULL DEFAULT 0,
    total_due           NUMERIC(24,6) GENERATED ALWAYS AS (
                            principal_due + interest_due + fee_due
                        ) STORED,
    status              TEXT NOT NULL DEFAULT 'pending' CHECK (status IN (
                            'pending', 'partially_paid', 'paid', 'overdue', 'waived', 'rescheduled'
                        )),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (facility_id, sequence_no)
);

CREATE TABLE capital.debt_repayments (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    facility_id         UUID NOT NULL REFERENCES capital.debt_facilities(id),
    schedule_id         UUID REFERENCES capital.debt_repayment_schedules(id),
    repayment_date      DATE NOT NULL,
    principal_amount    NUMERIC(24,6) NOT NULL DEFAULT 0,
    interest_amount     NUMERIC(24,6) NOT NULL DEFAULT 0,
    fee_amount          NUMERIC(24,6) NOT NULL DEFAULT 0,
    total_amount        NUMERIC(24,6) GENERATED ALWAYS AS (
                            principal_amount + interest_amount + fee_amount
                        ) STORED,
    currency_id         UUID NOT NULL REFERENCES capital.currencies(id),
    payment_method      TEXT,
    reference           TEXT,
    journal_entry_id    BIGINT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.debt_interest_accruals (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    facility_id         UUID NOT NULL REFERENCES capital.debt_facilities(id),
    instrument_id       UUID REFERENCES capital.capital_instruments(id),
    period_start        DATE NOT NULL,
    period_end          DATE NOT NULL,
    principal_base      NUMERIC(24,6) NOT NULL,
    annual_rate         NUMERIC(12,8) NOT NULL,
    day_count           INTEGER NOT NULL,
    interest_amount     NUMERIC(24,6) NOT NULL,
    is_paid             BOOLEAN NOT NULL DEFAULT false,
    journal_entry_id    BIGINT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.debt_fees (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    facility_id         UUID NOT NULL REFERENCES capital.debt_facilities(id),
    fee_type            TEXT NOT NULL CHECK (fee_type IN (
                            'COMMITMENT', 'UTILIZATION', 'ARRANGEMENT',
                            'AGENCY', 'PREPAYMENT', 'AMENDMENT', 'OTHER'
                        )),
    fee_date            DATE NOT NULL,
    amount              NUMERIC(24,6) NOT NULL,
    currency_id         UUID NOT NULL REFERENCES capital.currencies(id),
    description         TEXT,
    journal_entry_id    BIGINT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.debt_covenants (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    facility_id         UUID NOT NULL REFERENCES capital.debt_facilities(id),
    covenant_code       TEXT NOT NULL,
    covenant_name       TEXT NOT NULL,
    covenant_type       TEXT NOT NULL CHECK (covenant_type IN (
                            'FINANCIAL', 'AFFIRMATIVE', 'NEGATIVE', 'INFORMATION', 'OTHER'
                        )),
    metric              TEXT,                         -- e.g. Net Debt / EBITDA, DSCR, Interest Cover
    operator            TEXT CHECK (operator IN ('>', '>=', '<', '<=', '=', 'BETWEEN')),
    threshold_min       NUMERIC(20,6),
    threshold_max       NUMERIC(20,6),
    measurement_frequency TEXT CHECK (measurement_frequency IN (
                            'MONTHLY', 'QUARTERLY', 'SEMI_ANNUAL', 'ANNUAL', 'CONTINUOUS'
                        )),
    testing_date_rule   TEXT,                         -- e.g. "last day of quarter"
    effective_date      DATE NOT NULL,
    expiry_date         DATE,
    cure_period_days    INTEGER,
    status              TEXT NOT NULL DEFAULT 'active',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.debt_covenant_tests (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    covenant_id         UUID NOT NULL REFERENCES capital.debt_covenants(id),
    test_date           DATE NOT NULL,
    actual_value        NUMERIC(20,6),
    is_compliant        BOOLEAN,
    headroom            NUMERIC(20,6),
    notes               TEXT,
    tested_by           UUID,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.debt_collateral (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    facility_id         UUID NOT NULL REFERENCES capital.debt_facilities(id),
    collateral_type     TEXT NOT NULL,                -- real estate, inventory, receivables, shares, etc.
    description         TEXT,
    estimated_value     NUMERIC(24,6),
    currency_id         UUID REFERENCES capital.currencies(id),
    valuation_date      DATE,
    ranking             TEXT,                         -- first, second lien
    is_perfected        BOOLEAN NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.debt_refinancings (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    old_facility_id     UUID NOT NULL REFERENCES capital.debt_facilities(id),
    new_facility_id     UUID REFERENCES capital.debt_facilities(id),
    refinancing_date    DATE NOT NULL,
    principal_refinanced NUMERIC(24,6) NOT NULL,
    costs               NUMERIC(24,6) DEFAULT 0,
    description         TEXT,
    journal_entry_id    BIGINT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =============================================
-- 4. EQUITY MODULE
-- =============================================

CREATE TABLE capital.share_classes (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    class_code          TEXT NOT NULL,
    name                TEXT NOT NULL,
    share_type          TEXT NOT NULL CHECK (share_type IN (
                            'ORDINARY', 'PREFERRED', 'REDEEMABLE', 'CONVERTIBLE',
                            'TREASURY', 'OTHER'
                        )),
    authorized_shares   BIGINT,
    issued_shares       BIGINT NOT NULL DEFAULT 0,
    outstanding_shares  BIGINT NOT NULL DEFAULT 0,    -- issued - treasury
    par_value           NUMERIC(20,8),
    currency_id         UUID REFERENCES capital.currencies(id),
    voting_rights       BOOLEAN NOT NULL DEFAULT true,
    votes_per_share     NUMERIC(10,4) DEFAULT 1,
    dividend_rights     BOOLEAN NOT NULL DEFAULT true,
    dividend_preference TEXT,                         -- cumulative / non-cumulative
    liquidation_preference NUMERIC(20,6),
    is_callable         BOOLEAN NOT NULL DEFAULT false,
    is_convertible      BOOLEAN NOT NULL DEFAULT false,
    conversion_ratio    NUMERIC(18,8),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (company_id, class_code)
);

CREATE TABLE capital.shareholders (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    party_id            UUID REFERENCES capital.parties(id),
    name                TEXT NOT NULL,
    shareholder_type    TEXT NOT NULL CHECK (shareholder_type IN (
                            'INDIVIDUAL', 'CORPORATE', 'FUND', 'TRUST',
                            'EMPLOYEE', 'FOUNDER', 'PUBLIC', 'OTHER'
                        )),
    tax_id              TEXT,
    residency_country   CHAR(2),
    is_related_party    BOOLEAN NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.shareholdings (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    share_class_id      UUID NOT NULL REFERENCES capital.share_classes(id),
    shareholder_id      UUID NOT NULL REFERENCES capital.shareholders(id),
    shares_held         BIGINT NOT NULL DEFAULT 0,
    average_cost        NUMERIC(24,8),
    acquisition_date    DATE,
    is_beneficial       BOOLEAN NOT NULL DEFAULT true,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (share_class_id, shareholder_id)
);

CREATE TABLE capital.share_transactions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    share_class_id      UUID NOT NULL REFERENCES capital.share_classes(id),
    transaction_type    TEXT NOT NULL CHECK (transaction_type IN (
                            'ISSUE', 'TRANSFER', 'BUYBACK', 'CANCELLATION',
                            'BONUS_ISSUE', 'STOCK_SPLIT', 'REVERSE_SPLIT',
                            'CONVERSION', 'EXERCISE', 'FORFEITURE', 'OTHER'
                        )),
    transaction_date    DATE NOT NULL,
    from_shareholder_id UUID REFERENCES capital.shareholders(id),
    to_shareholder_id   UUID REFERENCES capital.shareholders(id),
    shares              BIGINT NOT NULL,
    price_per_share     NUMERIC(24,8),
    total_consideration NUMERIC(24,6),
    currency_id         UUID REFERENCES capital.currencies(id),
    premium             NUMERIC(24,6),                -- share premium
    journal_entry_id    BIGINT,
    reference           TEXT,
    notes               TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_share_tx_class ON capital.share_transactions(share_class_id);
CREATE INDEX idx_share_tx_date ON capital.share_transactions(transaction_date);

CREATE TABLE capital.dividends (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    share_class_id      UUID NOT NULL REFERENCES capital.share_classes(id),
    dividend_type       TEXT NOT NULL CHECK (dividend_type IN (
                            'CASH', 'SCRIP', 'PROPERTY', 'SPECIAL', 'INTERIM', 'FINAL'
                        )),
    declaration_date    DATE NOT NULL,
    record_date         DATE,
    ex_dividend_date    DATE,
    payment_date        DATE,
    dividend_per_share  NUMERIC(20,8) NOT NULL,
    total_declared      NUMERIC(24,6),
    currency_id         UUID REFERENCES capital.currencies(id),
    status              TEXT NOT NULL DEFAULT 'proposed' CHECK (status IN (
                            'proposed', 'declared', 'record_passed', 'paid', 'cancelled'
                        )),
    journal_entry_id    BIGINT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.dividend_payments (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    dividend_id         UUID NOT NULL REFERENCES capital.dividends(id),
    shareholder_id      UUID NOT NULL REFERENCES capital.shareholders(id),
    shares_held         BIGINT NOT NULL,
    gross_amount        NUMERIC(24,6) NOT NULL,
    withholding_tax     NUMERIC(24,6) DEFAULT 0,
    net_amount          NUMERIC(24,6) GENERATED ALWAYS AS (gross_amount - withholding_tax) STORED,
    payment_date        DATE,
    payment_reference   TEXT,
    journal_entry_id    BIGINT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =============================================
-- 5. RETAINED EARNINGS & EQUITY RESERVES
-- =============================================

CREATE TABLE capital.equity_accounts (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    account_code        TEXT NOT NULL,
    account_name        TEXT NOT NULL,
    account_type        TEXT NOT NULL CHECK (account_type IN (
                            'SHARE_CAPITAL', 'SHARE_PREMIUM', 'RETAINED_EARNINGS',
                            'REVALUATION_RESERVE', 'OTHER_COMPREHENSIVE_INCOME',
                            'TREASURY_SHARES', 'OTHER_RESERVES', 'NON_CONTROLLING_INTEREST'
                        )),
    currency_id         UUID REFERENCES capital.currencies(id),
    is_distributable    BOOLEAN NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (company_id, account_code)
);

CREATE TABLE capital.equity_movements (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    equity_account_id   UUID NOT NULL REFERENCES capital.equity_accounts(id),
    movement_date       DATE NOT NULL,
    movement_type       TEXT NOT NULL CHECK (movement_type IN (
                            'PROFIT_FOR_PERIOD', 'LOSS_FOR_PERIOD',
                            'DIVIDEND_DECLARED', 'SHARE_ISSUE', 'SHARE_PREMIUM',
                            'BUYBACK', 'REVALUATION', 'OCI', 'TRANSFER',
                            'PRIOR_PERIOD_ADJUSTMENT', 'OTHER'
                        )),
    amount              NUMERIC(24,6) NOT NULL,       -- positive = increase
    description         TEXT,
    related_instrument_id UUID REFERENCES capital.capital_instruments(id),
    related_event_id    UUID REFERENCES capital.capital_events(id),
    journal_entry_id    BIGINT,
    period_year         INTEGER,
    period_month        INTEGER,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_equity_movements_account ON capital.equity_movements(equity_account_id);
CREATE INDEX idx_equity_movements_date ON capital.equity_movements(movement_date);

-- =============================================
-- 6. CAPITAL STRUCTURE & ALLOCATION
-- =============================================

CREATE TABLE capital.capital_structure_snapshots (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    as_of_date          DATE NOT NULL,
    total_debt          NUMERIC(24,6) NOT NULL DEFAULT 0,
    total_equity        NUMERIC(24,6) NOT NULL DEFAULT 0,
    total_hybrid        NUMERIC(24,6) NOT NULL DEFAULT 0,
    cash_and_equivalents NUMERIC(24,6) DEFAULT 0,
    net_debt            NUMERIC(24,6) GENERATED ALWAYS AS (total_debt - cash_and_equivalents) STORED,
    debt_to_equity      NUMERIC(12,6),
    equity_ratio        NUMERIC(12,6),
    notes               TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (company_id, as_of_date)
);

CREATE TABLE capital.capital_allocations (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    allocation_code     TEXT,
    capital_source_type TEXT NOT NULL CHECK (capital_source_type IN (
                            'DEBT_FACILITY', 'INSTRUMENT', 'EQUITY', 'RETAINED_EARNINGS',
                            'INTERNAL', 'OTHER'
                        )),
    capital_source_id   UUID NOT NULL,                -- polymorphic reference
    allocation_type     TEXT NOT NULL CHECK (allocation_type IN (
                            'PROJECT', 'ASSET', 'WORKING_CAPITAL', 'ACQUISITION',
                            'REFINANCING', 'DIVIDEND', 'SHARE_BUYBACK', 'OTHER'
                        )),
    allocation_target_id UUID,                        -- project / asset / etc.
    amount              NUMERIC(24,6) NOT NULL,
    currency_id         UUID REFERENCES capital.currencies(id),
    allocation_date     DATE NOT NULL,
    expected_return     NUMERIC(12,6),                -- IRR / ROIC target
    status              TEXT NOT NULL DEFAULT 'allocated',
    notes               TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =============================================
-- 7. DEPLOYMENT / PROJECTS / INVESTMENTS
-- =============================================

CREATE TABLE capital.capital_projects (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id               bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id              UUID NOT NULL,
    project_code            TEXT NOT NULL,
    name                    TEXT NOT NULL,
    description             TEXT,
    project_type            TEXT CHECK (project_type IN (
                                'CAPEX', 'ACQUISITION', 'DEVELOPMENT', 'R_AND_D',
                                'WORKING_CAPITAL', 'REFINANCING', 'OTHER'
                            )),
    start_date              DATE,
    expected_completion     DATE,
    actual_completion       DATE,
    approved_budget         NUMERIC(24,6),
    spent_to_date           NUMERIC(24,6) DEFAULT 0,
    currency_id             UUID REFERENCES capital.currencies(id),
    expected_irr            NUMERIC(12,6),
    expected_npv            NUMERIC(24,6),
    expected_payback_years  NUMERIC(8,2),
    risk_rating             TEXT,
    status                  TEXT NOT NULL DEFAULT 'planned' CHECK (status IN (
                                'planned', 'approved', 'in_progress', 'on_hold',
                                'completed', 'cancelled', 'abandoned'
                            )),
    owner_id                UUID,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (company_id, project_code)
);

CREATE TABLE capital.project_funding (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    project_id          UUID NOT NULL REFERENCES capital.capital_projects(id),
    funding_source_type TEXT NOT NULL,                -- mirrors capital_allocations
    funding_source_id   UUID NOT NULL,
    amount_committed    NUMERIC(24,6) NOT NULL,
    amount_drawn        NUMERIC(24,6) DEFAULT 0,
    currency_id         UUID REFERENCES capital.currencies(id),
    funding_date        DATE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.investment_positions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    position_name       TEXT NOT NULL,
    asset_class         TEXT CHECK (asset_class IN (
                            'EQUITY', 'FIXED_INCOME', 'REAL_ESTATE', 'PRIVATE_EQUITY',
                            'HEDGE_FUND', 'CASH', 'DERIVATIVE', 'OTHER'
                        )),
    instrument_id       UUID REFERENCES capital.capital_instruments(id),
    quantity            NUMERIC(24,8),
    cost_basis          NUMERIC(24,6),
    current_value       NUMERIC(24,6),
    currency_id         UUID REFERENCES capital.currencies(id),
    valuation_date      DATE,
    unrealized_pnl      NUMERIC(24,6),
    status              TEXT NOT NULL DEFAULT 'open',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =============================================
-- 8. LIQUIDITY & CASH FORECASTING
-- =============================================

CREATE TABLE capital.cash_forecasts (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    forecast_name       TEXT,
    forecast_date       DATE NOT NULL,                -- as-of date of forecast
    horizon_days        INTEGER NOT NULL DEFAULT 90,
    currency_id         UUID REFERENCES capital.currencies(id),
    opening_cash        NUMERIC(24,6),
    scenario            TEXT DEFAULT 'base' CHECK (scenario IN (
                            'base', 'optimistic', 'pessimistic', 'stress'
                        )),
    status              TEXT NOT NULL DEFAULT 'draft',
    created_by          UUID,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.cash_forecast_lines (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    line_date           DATE NOT NULL,
    category            TEXT NOT NULL CHECK (category IN (
                            'OPERATING_INFLOW', 'OPERATING_OUTFLOW',
                            'INVESTING_INFLOW', 'INVESTING_OUTFLOW',
                            'FINANCING_INFLOW', 'FINANCING_OUTFLOW',
                            'DEBT_SERVICE', 'DIVIDEND', 'TAX', 'OTHER'
                        )),
    description         TEXT,
    amount              NUMERIC(24,6) NOT NULL,       -- signed: + inflow, - outflow
    is_committed        BOOLEAN NOT NULL DEFAULT false,
    related_facility_id UUID REFERENCES capital.debt_facilities(id),
    related_project_id  UUID REFERENCES capital.capital_projects(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.funding_requirements (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    requirement_date    DATE NOT NULL,
    amount_needed       NUMERIC(24,6) NOT NULL,
    currency_id         UUID REFERENCES capital.currencies(id),
    purpose             TEXT,
    preferred_source    TEXT,                         -- debt / equity / internal
    status              TEXT NOT NULL DEFAULT 'open' CHECK (status IN (
                            'open', 'partially_funded', 'funded', 'cancelled'
                        )),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =============================================
-- 9. ANALYTICS / PERFORMANCE METRICS
-- =============================================

CREATE TABLE capital.capital_metrics (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    metric_date         DATE NOT NULL,
    metric_code         TEXT NOT NULL,                -- ROIC, ROE, DSCR, ICR, EVA, WACC, etc.
    metric_name         TEXT NOT NULL,
    value               NUMERIC(20,8) NOT NULL,
    numerator           NUMERIC(24,6),
    denominator         NUMERIC(24,6),
    currency_id         UUID REFERENCES capital.currencies(id),
    period_type         TEXT CHECK (period_type IN ('DAILY', 'MTD', 'QTD', 'YTD', 'LTM', 'CUSTOM')),
    notes               TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (company_id, metric_date, metric_code, period_type)
);

CREATE TABLE capital.wacc_components (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    serial_id           bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    company_id          UUID NOT NULL,
    as_of_date          DATE NOT NULL,
    cost_of_equity      NUMERIC(12,8),
    cost_of_debt        NUMERIC(12,8),
    tax_rate            NUMERIC(8,6),
    equity_weight       NUMERIC(8,6),
    debt_weight         NUMERIC(8,6),
    wacc                NUMERIC(12,8),
    notes               TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (company_id, as_of_date)
);

-- =============================================
-- 10. VIEWS (Key Analytics)
-- =============================================

-- Current shareholder ownership
CREATE OR REPLACE VIEW capital.v_shareholder_ownership AS
SELECT
    sh.company_id,
    st.to_shareholder_id,
    st.from_shareholder_id,
    sh.name AS shareholder_name,
    st.share_class_id,
    sc.class_code,
    sc.name AS share_class_name,
    SUM(CASE
            WHEN st.transaction_type IN (
                'ISSUE', 'TRANSFER', 'BONUS_ISSUE', 'CONVERSION', 'EXERCISE'
            ) THEN st.shares
            WHEN st.transaction_type IN (
                'TRANSFER', 'BUYBACK', 'CANCELLATION', 'FORFEITURE'
            ) THEN -st.shares
            ELSE 0
        END) AS shares_held
FROM capital.share_transactions st
JOIN capital.shareholders sh ON sh.id = st.to_shareholder_id OR sh.id = st.from_shareholder_id
JOIN capital.share_classes sc ON sc.id = st.share_class_id
GROUP BY sh.company_id, st.to_shareholder_id, st.from_shareholder_id, sh.name, st.share_class_id, sc.class_code, sc.name
HAVING SUM(
    CASE
        WHEN st.transaction_type IN (
            'ISSUE', 'TRANSFER', 'BONUS_ISSUE', 'CONVERSION', 'EXERCISE'
        ) THEN st.shares
        WHEN st.transaction_type IN (
            'TRANSFER', 'BUYBACK', 'CANCELLATION', 'FORFEITURE'
        ) THEN -st.shares
        ELSE 0
    END
    ) <> 0;

-- Facility utilization
CREATE OR REPLACE VIEW capital.v_facility_utilization AS
SELECT
    f.id AS facility_id,
    f.company_id,
    f.facility_code,
    f.facility_name,
    f.committed_amount,
    f.drawn_amount,
    f.available_amount,
    CASE WHEN f.committed_amount > 0
         THEN ROUND((f.drawn_amount / f.committed_amount) * 100, 2)
         ELSE 0 END AS utilization_pct,
    f.maturity_date,
    f.status
FROM capital.debt_facilities f;

-- Debt service upcoming
CREATE OR REPLACE VIEW capital.v_upcoming_debt_service AS
SELECT
    s.facility_id,
    f.facility_name,
    f.company_id,
    s.due_date,
    s.principal_due,
    s.interest_due,
    s.fee_due,
    s.total_due,
    s.status
FROM capital.debt_repayment_schedules s
JOIN capital.debt_facilities f ON f.id = s.facility_id
WHERE s.status IN ('pending', 'partially_paid', 'overdue')
  AND s.due_date >= CURRENT_DATE
ORDER BY s.due_date;

-- Capital structure summary (latest)
CREATE OR REPLACE VIEW capital.v_latest_capital_structure AS
SELECT DISTINCT ON (company_id)
    company_id,
    as_of_date,
    total_debt,
    total_equity,
    total_hybrid,
    cash_and_equivalents,
    net_debt,
    debt_to_equity,
    equity_ratio
FROM capital.capital_structure_snapshots
ORDER BY company_id, as_of_date DESC;

-- Retained earnings movement (simplified)
CREATE OR REPLACE VIEW capital.v_retained_earnings AS
SELECT
    ea.company_id,
    ea.id AS equity_account_id,
    ea.account_name,
    COALESCE(SUM(em.amount), 0) AS balance
FROM capital.equity_accounts ea
LEFT JOIN capital.equity_movements em ON em.equity_account_id = ea.id
WHERE ea.account_type = 'RETAINED_EARNINGS'
GROUP BY ea.company_id, ea.id, ea.account_name;

-- =============================================
-- 11. HELPER FUNCTIONS (examples)
-- =============================================
CREATE OR REPLACE FUNCTION capital.issue_shares(
    p_company_id          UUID,
    p_user_id             UUID,
    p_share_class_id      UUID,          -- existing class, or NULL → create
    p_shareholder_id      UUID,
    p_shares              BIGINT,
    p_price_per_share     NUMERIC(24,8),
    p_issue_date          DATE,
    p_currency_id         UUID,
    p_instrument_code     TEXT DEFAULT NULL,   -- only needed when creating new instrument
    p_instrument_name     TEXT DEFAULT NULL,
    p_par_value           NUMERIC(20,8) DEFAULT NULL,
    p_cash_account_code   TEXT,                -- for the GL side
    p_share_capital_code  TEXT,
    p_share_premium_code  TEXT DEFAULT NULL,   -- optional
    p_reference           TEXT DEFAULT NULL,
    p_notes               TEXT DEFAULT NULL
) RETURNS TABLE (
    instrument_id   UUID,
    event_id        UUID,
    transaction_id  BIGINT,          -- share_transactions.serial_id
    journal_serial  BIGINT           -- accounting.transactions.serial_id
)
LANGUAGE plpgsql
AS $function$
DECLARE
    v_class             capital.share_classes%ROWTYPE;
    v_instrument_id     UUID;
    v_event_id          UUID;
    v_tx_id             BIGINT;
    v_journal_serial    BIGINT;
    v_total             NUMERIC(24,6);
    v_premium           NUMERIC(24,6);
    v_par_total         NUMERIC(24,6);
    v_lines             JSONB;
BEGIN
    -- 1. Lock & load (or create) the share class
    IF p_share_class_id IS NOT NULL THEN
        SELECT * INTO v_class
        FROM capital.share_classes
        WHERE id = p_share_class_id AND company_id = p_company_id
        FOR UPDATE;
        IF NOT FOUND THEN
            RAISE EXCEPTION 'Share class not found' USING ERRCODE = 'P0002';
        END IF;
    ELSE
        -- create a new class + matching equity instrument in one go
        -- (omitted for brevity – same pattern)
        RAISE EXCEPTION 'Creating new class not implemented in this sketch';
    END IF;

    v_total     := p_shares * p_price_per_share;
    v_par_total := p_shares * COALESCE(p_par_value, v_class.par_value, 0);
    v_premium   := v_total - v_par_total;

    -- 2. Ensure / update the capital instrument
    --    (one instrument per share class is the usual 1:1)
    SELECT id INTO v_instrument_id
    FROM capital.capital_instruments
    WHERE company_id = p_company_id
      AND instrument_type IN ('COMMON_EQUITY','PREFERRED_EQUITY')
      AND instrument_code = v_class.class_code;

    IF v_instrument_id IS NULL THEN
        INSERT INTO capital.capital_instruments (
            company_id, instrument_code, name,
            instrument_family, instrument_type,
            currency_id, original_principal, outstanding_principal,
            face_value, issue_price, effective_date, status
        ) VALUES (
            p_company_id,
            COALESCE(p_instrument_code, v_class.class_code),
            COALESCE(p_instrument_name, v_class.name),
            'EQUITY',
            CASE v_class.share_type
                WHEN 'PREFERRED' THEN 'PREFERRED_EQUITY'
                ELSE 'COMMON_EQUITY'
            END,
            p_currency_id,
            v_total, v_total,
            v_par_total, p_price_per_share, p_issue_date, 'active'
        )
        RETURNING id INTO v_instrument_id;
    ELSE
        UPDATE capital.capital_instruments
        SET outstanding_principal = outstanding_principal + v_total,
            original_principal    = original_principal    + v_total,
            updated_at            = now()
        WHERE id = v_instrument_id;
    END IF;

    -- 3. Capital event (audit trail)
    INSERT INTO capital.capital_events (
        company_id, instrument_id, event_type, event_date,
        amount, currency_id, shares, description,
        related_party_id, created_by
    ) VALUES (
        p_company_id, v_instrument_id, 'SHARE_ISSUE', p_issue_date,
        v_total, p_currency_id, p_shares,
        COALESCE(p_notes, format('Issue of %s shares', p_shares)),
        p_shareholder_id, p_user_id
    )
    RETURNING id INTO v_event_id;

    -- 4. Share transaction (immutable movement)
    INSERT INTO capital.share_transactions (
        company_id, share_class_id, transaction_type, transaction_date,
        to_shareholder_id, shares, price_per_share,
        total_consideration, currency_id, premium, reference, notes
    ) VALUES (
        p_company_id, v_class.id, 'ISSUE', p_issue_date,
        p_shareholder_id, p_shares, p_price_per_share,
        v_total, p_currency_id, v_premium, p_reference, p_notes
    )
    RETURNING serial_id INTO v_tx_id;

    -- 5. Upsert shareholding
    INSERT INTO capital.shareholdings (
        company_id, share_class_id, shareholder_id,
        shares_held, average_cost, acquisition_date
    ) VALUES (
        p_company_id, v_class.id, p_shareholder_id,
        p_shares, p_price_per_share, p_issue_date
    )
    ON CONFLICT (share_class_id, shareholder_id) DO UPDATE
    SET shares_held = capital.shareholdings.shares_held + EXCLUDED.shares_held,
        average_cost = (
            (capital.shareholdings.shares_held * COALESCE(capital.shareholdings.average_cost,0)
             + EXCLUDED.shares_held * EXCLUDED.average_cost)
            / NULLIF(capital.shareholdings.shares_held + EXCLUDED.shares_held, 0)
        ),
        updated_at = now();

    -- 6. Update class counters
    UPDATE capital.share_classes
    SET issued_shares      = issued_shares      + p_shares,
        outstanding_shares = outstanding_shares + p_shares,
        updated_at         = now()
    WHERE id = v_class.id;

    -- 7. GL posting (cash/bank ← share capital + premium)
    v_lines := jsonb_build_array(
        jsonb_build_object(
            'account_ref', p_cash_account_code,
            'debit', v_total, 'credit', 0,
            'memo', format('Share issue %s', COALESCE(p_reference, v_tx_id::text))
        ),
        jsonb_build_object(
            'account_ref', p_share_capital_code,
            'debit', 0, 'credit', v_par_total,
            'memo', 'Share capital'
        )
    );
    IF v_premium > 0 AND p_share_premium_code IS NOT NULL THEN
        v_lines := v_lines || jsonb_build_object(
            'account_ref', p_share_premium_code,
            'debit', 0, 'credit', v_premium,
            'memo', 'Share premium'
        );
    END IF;

    v_journal_serial := accounting.post_transaction(
        COALESCE(p_reference, 'SI-' || v_tx_id),
        'Share issuance',
        p_user_id,
        'share_issue',
        p_issue_date,
        v_lines
    );

    -- optional: link the journal back
    UPDATE capital.capital_events
    SET journal_entry_id = v_journal_serial
    WHERE id = v_event_id;

    UPDATE capital.share_transactions
    SET journal_entry_id = v_journal_serial
    WHERE serial_id = v_tx_id;

    RETURN QUERY SELECT v_instrument_id, v_event_id, v_tx_id, v_journal_serial;
END; 
$$;

CREATE OR REPLACE FUNCTION capital.borrow(
    p_company_id          UUID,
    p_user_id             UUID,
    p_facility_id         UUID,               -- existing facility
    p_amount              NUMERIC(24,6),
    p_drawdown_date       DATE,
    p_value_date          DATE DEFAULT NULL,
    p_currency_id         UUID,
    p_cash_account_code   TEXT,               -- bank / cash GL account
    p_loan_liability_code TEXT,               -- the liability account
    p_reference           TEXT DEFAULT NULL,
    p_purpose             TEXT DEFAULT NULL
) RETURNS TABLE (
    drawdown_id     UUID,
    event_id        UUID,
    journal_serial  BIGINT
)
LANGUAGE plpgsql
AS $function$ 
DECLARE
    v_facility      capital.debt_facilities%ROWTYPE;
    v_instrument_id UUID;
    v_drawdown_id   UUID;
    v_event_id      UUID;
    v_journal       BIGINT;
    v_lines         JSONB;
BEGIN
    -- 1. Lock facility
    SELECT * INTO v_facility
    FROM capital.debt_facilities
    WHERE id = p_facility_id AND company_id = p_company_id
    FOR UPDATE;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Facility not found' USING ERRCODE = 'P0002';
    END IF;

    IF p_amount > v_facility.available_amount THEN
        RAISE EXCEPTION 'Drawdown % exceeds available %',
            p_amount, v_facility.available_amount
            USING ERRCODE = 'P0001';
    END IF;

    v_instrument_id := v_facility.instrument_id;

    -- 2. Create / update the capital instrument (if the facility is linked)
    IF v_instrument_id IS NOT NULL THEN
        UPDATE capital.capital_instruments
        SET outstanding_principal = outstanding_principal + p_amount,
            updated_at = now()
        WHERE id = v_instrument_id;
    END IF;

    -- 3. Drawdown record
    INSERT INTO capital.debt_drawdowns (
        facility_id, instrument_id, drawdown_date, value_date,
        amount, currency_id, reference, purpose, status
    ) VALUES (
        p_facility_id, v_instrument_id,
        p_drawdown_date, COALESCE(p_value_date, p_drawdown_date),
        p_amount, p_currency_id, p_reference, p_purpose, 'posted'
    )
    RETURNING id INTO v_drawdown_id;

    -- 4. Update facility balances
    UPDATE capital.debt_facilities
    SET drawn_amount     = drawn_amount + p_amount,
        available_amount = available_amount - p_amount,
        status           = CASE
                             WHEN available_amount - p_amount <= 0 THEN 'fully_drawn'
                             ELSE status
                           END,
        updated_at       = now()
    WHERE id = p_facility_id;

    -- 5. Capital event
    INSERT INTO capital.capital_events (
        company_id, instrument_id, event_type, event_date,
        amount, currency_id, description, created_by
    ) VALUES (
        p_company_id, v_instrument_id, 'DRAWDOWN', p_drawdown_date,
        p_amount, p_currency_id,
        COALESCE(p_purpose, format('Drawdown on facility %s', v_facility.facility_code)),
        p_user_id
    )
    RETURNING id INTO v_event_id;

    -- 6. GL
    v_lines := jsonb_build_array(
        jsonb_build_object(
            'account_ref', p_cash_account_code,
            'debit', p_amount, 'credit', 0,
            'memo', format('Loan drawdown %s', COALESCE(p_reference, v_drawdown_id::text))
        ),
        jsonb_build_object(
            'account_ref', p_loan_liability_code,
            'debit', 0, 'credit', p_amount,
            'memo', 'Loan liability'
        )
    );

    v_journal := accounting.post_transaction(
        COALESCE(p_reference, 'DD-' || v_drawdown_id),
        'Debt drawdown',
        p_user_id,
        'debt_drawdown',
        p_drawdown_date,
        v_lines
    );

    -- link journal
    UPDATE capital.debt_drawdowns
    SET journal_entry_id = v_journal
    WHERE id = v_drawdown_id;

    UPDATE capital.capital_events
    SET journal_entry_id = v_journal
    WHERE id = v_event_id;

    RETURN QUERY SELECT v_drawdown_id, v_event_id, v_journal;
END; 
$$;

CREATE OR REPLACE FUNCTION capital.record_equity_contribution(
    p_company_id          UUID,
    p_user_id             UUID,
    p_party_id            UUID,               -- investor / owner / partner
    p_amount              NUMERIC(24,6),
    p_contribution_date   DATE,
    p_currency_id         UUID,
    p_cash_account_code   TEXT,
    p_equity_account_code TEXT,               -- capital contribution / additional paid-in capital
    p_description         TEXT DEFAULT NULL,
    p_reference           TEXT DEFAULT NULL
) RETURNS TABLE (
    event_id       UUID,
    journal_serial BIGINT
)
LANGUAGE plpgsql
AS $function$ 
DECLARE
    v_event_id UUID;
    v_journal  BIGINT;
    v_lines    JSONB;
BEGIN
    -- 1. Event only (no instrument)
    INSERT INTO capital.capital_events (
        company_id, instrument_id, event_type, event_date,
        amount, currency_id, description,
        related_party_id, created_by
    ) VALUES (
        p_company_id,
        NULL,                               -- ← explicit: no formal instrument
        'EQUITY_CONTRIBUTION',
        p_contribution_date,
        p_amount, p_currency_id,
        COALESCE(p_description, 'Equity contribution'),
        p_party_id, p_user_id
    )
    RETURNING id INTO v_event_id;

    -- 2. Optional equity movement (if you keep equity_accounts)
    -- INSERT INTO capital.equity_movements ... (can be added later)

    -- 3. GL
    v_lines := jsonb_build_array(
        jsonb_build_object(
            'account_ref', p_cash_account_code,
            'debit', p_amount, 'credit', 0,
            'memo', COALESCE(p_reference, 'Equity contribution')
        ),
        jsonb_build_object(
            'account_ref', p_equity_account_code,
            'debit', 0, 'credit', p_amount,
            'memo', 'Capital contribution'
        )
    );

    v_journal := accounting.post_transaction(
        COALESCE(p_reference, 'EC-' || v_event_id),
        'Equity contribution',
        p_user_id,
        'equity_contribution',
        p_contribution_date,
        v_lines
    );

    UPDATE capital.capital_events
    SET journal_entry_id = v_journal
    WHERE id = v_event_id;

    RETURN QUERY SELECT v_event_id, v_journal;
END; 
$$;

-- Example: recalculate outstanding principal on an instrument
-- (implementation would live in application or trigger layer)

COMMENT ON SCHEMA capital IS
'Enterprise Capital & Treasury schema supporting Debt, Equity, Hybrids,
Retained Earnings, Capital Structure, Deployment, Liquidity Forecasting
and Capital Performance Analytics (ROIC, ROE, DSCR, WACC, EVA, etc.)';

/*
capital
│
├── capital_instruments
├── capital_events
├── capital_allocations
│
├── equity
│   ├── share_classes
│   ├── shareholders
│   ├── shareholdings
│   ├── share_transactions
│   └── dividends
│
├── debt
│   ├── facilities
│   ├── drawdowns
│   ├── repayment_schedules
│   ├── repayments
│   ├── interest_accruals
│   ├── covenants
│   └── refinancing
│
├── deployment
│   ├── capital_projects
│   ├── project_funding
│   ├── investment_positions
│   └── capital_allocations
│
├── liquidity
│   ├── cash_forecasts
│   ├── cash_forecast_lines
│   └── funding_requirements
│
└── analytics
    ├── capital_structure
    ├── debt_performance
    ├── liquidity
    ├── profitability
    └── capital_efficiency


                         COMPANY
                            │
             ┌──────────────┴──────────────┐
             ↓                             ↓
       CAPITAL SOURCES                CAPITAL STRUCTURE
             │
      ┌──────┴───────┐───────────────┐
      ↓              ↓               │
    DEBT           EQUITY       CAP-CONTRIBUTION
      │              │               │
      ↓              ↓               │
 FACILITIES       SHARES             │
      │              │               │
 DRAWDOWNS       SHAREHOLDERS        │
      │              │               │
 REPAYMENTS       DIVIDENDS          │
      │              │               │
 INTEREST            │               │
      └───────┬──────┘───────────────┘
              ↓
      CAPITAL ALLOCATIONS
              │
      ┌───────┼────────┐
      ↓       ↓        ↓
    ASSETS  PROJECTS  WORKING
                     CAPITAL
      │       │        │
      └───────┼────────┘
              ↓
        OPERATING RESULTS
              │
              ↓
       CASH FLOW / PROFIT
              │
              ↓
       CAPITAL PERFORMANCE
              │
      ┌───────┼───────────────┐
      ↓       ↓       ↓       ↓
    ROIC    ROE     DSCR     EVA
              │
              ↓
        MANAGEMENT DECISION
*/

