-- =============================================
-- CAPITAL & TREASURY SCHEMA
-- =============================================

-- Create schema
CREATE SCHEMA IF NOT EXISTS capital;

CREATE TABLE capital.capital_instruments (
    id UUID PRIMARY KEY,
    company_id UUID NOT NULL,
    instrument_type TEXT NOT NULL CHECK(instrument_type IN(
        'DEBT', 'EQUITY', 'SHAREHOLDER_LOAN', 'BOND',
        'PREFERRED_EQUITY', 'CONVERTIBLE_DEBT', 'OTHER'
    )),
    name TEXT NOT NULL,
    currency_id UUID NOT NULL,
    original_amount NUMERIC(20, 4),
    effective_date DATE NOT NULL,
    maturity_date DATE,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.debt_facilities (
    id UUID PRIMARY KEY,
    company_id UUID NOT NULL,
    lender_id UUID NOT NULL,
    facility_name TEXT NOT NULL,
    facility_type TEXT NOT NULL,
    currency_id UUID NOT NULL,
    approved_limit NUMERIC(20,4) NOT NULL,
    interest_rate NUMERIC(10,6),
    interest_type TEXT,
    effective_date DATE NOT NULL,
    maturity_date DATE,
    collateral_required BOOLEAN NOT NULL DEFAULT false,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.debt_drawdowns (
    id UUID PRIMARY KEY,
    facility_id UUID NOT NULL REFERENCES debt_facilities(id),
    drawdown_date DATE NOT NULL,
    amount NUMERIC(20,4) NOT NULL,
    reference TEXT,
    journal_entry_id UUID,
    status TEXT NOT NULL DEFAULT 'posted',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.debt_repayment_schedules (
    id UUID PRIMARY KEY,
    facility_id UUID NOT NULL REFERENCES debt_facilities(id),
    due_date DATE NOT NULL,
    principal_due NUMERIC(20,4) NOT NULL DEFAULT 0,
    interest_due NUMERIC(20,4) NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.debt_repayments (
    id UUID PRIMARY KEY,
    facility_id UUID NOT NULL REFERENCES debt_facilities(id),
    repayment_date DATE NOT NULL,
    principal_amount NUMERIC(20,4) NOT NULL DEFAULT 0,
    interest_amount NUMERIC(20,4) NOT NULL DEFAULT 0,
    journal_entry_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.debt_interest_accruals (
    id UUID PRIMARY KEY,
    facility_id UUID NOT NULL REFERENCES debt_facilities(id),
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    principal_base NUMERIC(20,4) NOT NULL,
    annual_rate NUMERIC(10,6) NOT NULL,
    days INTEGER NOT NULL,
    interest_amount NUMERIC(20,4) NOT NULL,
    journal_entry_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.debt_covenants (
    id UUID PRIMARY KEY,
    facility_id UUID NOT NULL REFERENCES debt_facilities(id),
    covenant_type TEXT NOT NULL,
    operator TEXT NOT NULL,
    threshold NUMERIC(20,6) NOT NULL,
    measurement_frequency TEXT,
    effective_date DATE NOT NULL,
    expiry_date DATE,
    status TEXT NOT NULL DEFAULT 'active'
);

CREATE TABLE capital.share_classes (
    id UUID PRIMARY KEY,
    company_id UUID NOT NULL,
    name TEXT NOT NULL,
    class_code TEXT NOT NULL,
    authorized_shares BIGINT,
    nominal_value NUMERIC(20,4),
    currency_id UUID,
    voting_rights BOOLEAN NOT NULL DEFAULT true,
    dividend_rights BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.shareholders (
    id UUID PRIMARY KEY,
    company_id UUID NOT NULL,
    name TEXT NOT NULL,
    shareholder_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.share_transactions (
    id UUID PRIMARY KEY,
    company_id UUID NOT NULL,
    share_class_id UUID NOT NULL REFERENCES share_classes(id),
    shareholder_id UUID NOT NULL REFERENCES shareholders(id),
    transaction_type TEXT NOT NULL CHECK(transaction_type IN (
        'ISSUE', 'TRANSFER_IN', 'TRANSFER_OUT', 'BUYBACK',
        'CANCELLATION', 'BONUS_ISSUE'
    )),
    shares BIGINT NOT NULL,
    price_per_share NUMERIC(20,4),
    total_amount NUMERIC(20,4),
    transaction_date DATE NOT NULL,
    journal_entry_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.dividends (
    id UUID PRIMARY KEY,
    company_id UUID NOT NULL,
    share_class_id UUID NOT NULL,
    declaration_date DATE,
    record_date DATE,
    payment_date DATE,
    dividend_per_share NUMERIC(20,4),
    total_declared NUMERIC(20,4),
    status TEXT NOT NULL DEFAULT 'proposed',
    journal_entry_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.capital_allocations (
    id UUID PRIMARY KEY,
    company_id UUID NOT NULL,
    capital_source_type TEXT NOT NULL,
    capital_source_id UUID NOT NULL,
    allocation_type TEXT NOT NULL,
    allocation_id UUID,
    amount NUMERIC(20,4) NOT NULL,
    allocation_date DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.capital_projects (
    id UUID PRIMARY KEY,
    company_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    start_date DATE,
    expected_completion_date DATE,
    approved_budget NUMERIC(20,4),
    expected_irr NUMERIC(10,6),
    expected_npv NUMERIC(20,4),
    status TEXT NOT NULL DEFAULT 'planned',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE capital.cash_forecasts (
    id UUID PRIMARY KEY,
    company_id UUID NOT NULL,
    forecast_date DATE NOT NULL,
    opening_cash NUMERIC(20,4),
    expected_inflows NUMERIC(20,4),
    expected_outflows NUMERIC(20,4),
    debt_service NUMERIC(20,4),
    closing_cash NUMERIC(20,4),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE VIEW v_shareholder_ownership AS
SELECT
    shareholder_id,
    share_class_id,
    SUM(
        CASE
            WHEN transaction_type IN ('ISSUE', 'TRANSFER_IN')
                THEN shares
            WHEN transaction_type IN ('TRANSFER_OUT', 'BUYBACK')
                THEN -shares
            ELSE 0
        END
    ) AS shares_held
FROM share_transactions
GROUP BY shareholder_id, share_class_id;

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
      ┌──────┴───────┐
      ↓              ↓
    DEBT           EQUITY
      │              │
      ↓              ↓
 FACILITIES       SHARES
      │              │
 DRAWDOWNS       SHAREHOLDERS
      │              │
 REPAYMENTS       DIVIDENDS
      │              │
 INTEREST            │
      └───────┬──────┘
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
