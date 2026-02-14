-- ========================================
-- REPORTING MODULE - FULL SCHEMA (GLOBAL-READY)
-- Run this ONCE after all core modules exist:
--   accounting, procurement, receivables, payroll, inventory
-- ========================================

-- Enable UUID extension (if not already)
-- CREATE EXTENSION IF NOT EXISTS pg_uuidv7;

-- Create schema
CREATE SCHEMA IF NOT EXISTS reporting;

-- ========================================
-- FUNCTION: refresh_all
-- ========================================
CREATE OR REPLACE FUNCTION reporting.refresh_all(p_force boolean DEFAULT false)
RETURNS void LANGUAGE plpgsql AS $$
DECLARE
    v_start timestamptz := clock_timestamp();
BEGIN
    RAISE NOTICE 'Starting reporting refresh at %...', v_start;

    -- Only refresh if data exists OR force is true
    IF p_force OR EXISTS (SELECT 1 FROM accounting.accounts LIMIT 1) THEN
        REFRESH MATERIALIZED VIEW CONCURRENTLY reporting.cashbook;
        REFRESH MATERIALIZED VIEW CONCURRENTLY reporting.ar_aging_detailed;
        REFRESH MATERIALIZED VIEW CONCURRENTLY reporting.ap_aging_detailed;
        REFRESH MATERIALIZED VIEW CONCURRENTLY reporting.payroll_summary;
        REFRESH MATERIALIZED VIEW CONCURRENTLY reporting.inventory_valuation;
        REFRESH MATERIALIZED VIEW CONCURRENTLY reporting.customer_statement;
    END IF;

    RAISE NOTICE 'Reporting refresh completed in %', clock_timestamp() - v_start;
END;
$$;

-- ========================================
-- MATERIALIZED VIEWS
-- ========================================

-- Optional helper view: transactions with txn_date for easy filtering
CREATE OR REPLACE VIEW reporting.transactions_for_reporting AS
SELECT t.uuid, t.serial_id, t.txn_date, te.account_uuid, te.debit, te.credit, t.reference, te.memo
FROM accounting.transactions t
JOIN accounting.transaction_entries te ON te.transaction_uuid = t.uuid;

-- trigger function that NOTIFYs
CREATE OR REPLACE FUNCTION reporting.notify_reporting_changes()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    PERFORM pg_notify('reporting_changes', TG_TABLE_NAME || ':' || TG_OP || ':' || NEW::text);
    RETURN NEW;
EXCEPTION WHEN others THEN
    -- keep DB changes robust; swallow notify errors but log
    RAISE NOTICE 'notify failure: %', SQLERRM;
    RETURN NEW;
END;
$$;

-- 4. Cashbook
CREATE MATERIALIZED VIEW IF NOT EXISTS reporting.cashbook AS
SELECT
    t.serial_id AS txn_serial_id,
    t.txn_date,
    t.reference,
    te.account_uuid,
    a.code AS account_code,
    a.name AS account_name,
    te.debit,
    te.credit,
    te.memo
FROM accounting.transactions t
JOIN accounting.transaction_entries te ON te.transaction_uuid = t.uuid
JOIN accounting.accounts a ON a.uuid = te.account_uuid
WHERE a.code LIKE '11%'
ORDER BY t.txn_date DESC, t.serial_id
WITH NO DATA;

-- 6. AR Aging Detailed
CREATE MATERIALIZED VIEW IF NOT EXISTS reporting.ar_aging_detailed AS
SELECT
    c.serial_id AS customer_serial_id,
    c.code,
    c.name,
    i.serial_id AS invoice_serial_id,
    i.invoice_number,
    i.issue_date,
    i.due_date,
    i.total_amount,
    i.balance_due,
    (CURRENT_DATE - i.due_date) AS days_overdue,
    CASE
        WHEN i.due_date >= CURRENT_DATE THEN 'Current'
        WHEN i.due_date >= CURRENT_DATE - 30 THEN '1-30'
        WHEN i.due_date >= CURRENT_DATE - 60 THEN '31-60'
        WHEN i.due_date >= CURRENT_DATE - 90 THEN '61-90'
        ELSE 'Over 90'
    END AS aging_bucket
FROM sales.turnover i
JOIN sales.customers c ON c.uuid = i.customer_uuid
WHERE i.status NOT IN ('Paid', 'Cancelled') AND i.balance_due > 0
ORDER BY c.name, i.due_date
WITH NO DATA;

-- 7. AP Aging Detailed
CREATE MATERIALIZED VIEW IF NOT EXISTS reporting.ap_aging_detailed AS
SELECT
    v.serial_id AS vendor_serial_id,
    v.code,
    v.name,
    b.serial_id AS bill_serial_id,
    b.bill_number,
    b.bill_date,
    b.due_date,
    b.total_amount,
    b.balance_due,
    (CURRENT_DATE - b.due_date) AS days_overdue,
    CASE
        WHEN b.due_date >= CURRENT_DATE THEN 'Current'
        WHEN b.due_date >= CURRENT_DATE - 30 THEN '1-30'
        WHEN b.due_date >= CURRENT_DATE - 60 THEN '31-60'
        WHEN b.due_date >= CURRENT_DATE - 90 THEN '61-90'
        ELSE 'Over 90'
    END AS aging_bucket
FROM procurement.purchases b
JOIN procurement.vendors v ON v.uuid = b.vendor_uuid
WHERE b.payment_status NOT IN ('paid', 'cancelled') AND b.balance_due > 0
ORDER BY v.name, b.due_date
WITH NO DATA;

-- 8. Payroll Summary
CREATE MATERIALIZED VIEW IF NOT EXISTS reporting.payroll_summary AS
SELECT
    pr.serial_id AS payrun_serial_id,
    pr.pay_period_start,
    pr.pay_period_end,
    pr.payment_date,
    COUNT(ps.uuid) AS employees_paid,
    SUM(ps.gross_pay) AS total_gross,
    SUM(ps.tax_deducted) AS total_tax,
    SUM(ps.social_security) AS total_social_security,
    SUM(ps.other_deductions) AS total_other_deductions,
    SUM(ps.net_pay) AS total_net,
    pr.status
FROM payroll.payruns pr
LEFT JOIN payroll.payslips ps ON ps.payrun_uuid = pr.uuid
GROUP BY pr.serial_id, pr.pay_period_start, pr.pay_period_end, pr.payment_date, pr.status
ORDER BY pr.pay_period_start DESC
WITH NO DATA;

-- 9. Inventory Valuation
--CREATE MATERIALIZED VIEW IF NOT EXISTS reporting.inventory_valuation AS
--SELECT
--    i.serial_id AS item_serial_id,
--    i.sku,
--    i.name,
--    c.name AS category,
--    i.unit,
--    (i.quantity_on_hand * i.cost_price) AS total_value,
--    i.reorder_level,
--    (i.quantity_on_hand <= i.reorder_level) AS needs_reorder
--FROM inventory.items i
--LEFT JOIN inventory.item_categories c ON c.uuid = i.category_uuid
--WHERE i.track_quantity = true
--ORDER BY total_value DESC
--WITH NO DATA;

-- 10. Customer Statement (Detailed)
CREATE MATERIALIZED VIEW IF NOT EXISTS reporting.customer_statement AS
SELECT
    c.serial_id AS customer_serial_id,
    c.code,
    c.name,
    i.serial_id AS invoice_serial_id,
    i.invoice_number,
    i.issue_date,
    i.due_date,
    i.total_amount,
    i.balance_due,
    i.status AS invoice_status,
    p.serial_id AS payment_serial_id,
    p.payment_number,
    p.amount AS payment_amount,
    p.payment_date
FROM sales.customers c
LEFT JOIN sales.turnover i ON i.customer_uuid = c.uuid
LEFT JOIN sales.payment_applications pa ON pa.invoice_uuid = i.uuid
LEFT JOIN sales.payments p ON p.uuid = pa.payment_uuid
WITH NO DATA;

-- ===============================================================
-- MATERIALIZE VIEWS BEFORE ANY REFRESH MATERIALIZED VIEW COMMAND
-- ===============================================================
REFRESH MATERIALIZED VIEW reporting.cashbook;
REFRESH MATERIALIZED VIEW reporting.ar_aging_detailed;
REFRESH MATERIALIZED VIEW reporting.ap_aging_detailed;
REFRESH MATERIALIZED VIEW reporting.payroll_summary;
--REFRESH MATERIALIZED VIEW reporting.inventory_valuation;
REFRESH MATERIALIZED VIEW reporting.customer_statement;

-- ========================================
-- INDEXES ON MATERIALIZED VIEWS
-- ========================================
CREATE INDEX IF NOT EXISTS idx_ar_aging_customer ON reporting.ar_aging_detailed(customer_serial_id);
CREATE INDEX IF NOT EXISTS idx_ap_aging_vendor ON reporting.ap_aging_detailed(vendor_serial_id);
CREATE INDEX IF NOT EXISTS idx_cashbook_date ON reporting.cashbook(txn_date DESC);

-- ==========================================
-- FUNCTIONS FOR BUILDING REPORTS DYNAMICALLY
-- ==========================================
-- 1 Balancesheet function
CREATE OR REPLACE FUNCTION reporting.get_balance_sheet(as_of DATE)
RETURNS TABLE (
    code TEXT,
    name TEXT,
    category TEXT,
    depth INT,
    path TEXT[],
    balance NUMERIC
) AS
$$
WITH RECURSIVE

-- 1. Compute raw balances for all relevant accounts
account_balances AS (
    SELECT
        acc.uuid,
        acc.code,
        acc.name,
        acc.category,
        acc.parent_code,
        acc.normal_balance,
        acc.is_contra,
        COALESCE(SUM(te.debit) FILTER (WHERE te.created_at <= as_of), 0)
      - COALESCE(SUM(te.credit) FILTER (WHERE te.created_at <= as_of), 0) AS raw_balance
    FROM accounting.accounts acc
    LEFT JOIN accounting.transaction_entries te
        ON te.account_uuid = acc.uuid
    WHERE acc.category IN ('asset', 'equity', 'liability')
    GROUP BY acc.uuid, acc.code, acc.name, acc.category, acc.parent_code, acc.normal_balance, acc.is_contra
),

-- 2. Include parent accounts that may have zero balances
all_relevant_accounts AS (
    SELECT * FROM account_balances
    UNION
    SELECT
        acc.uuid,
        acc.code,
        acc.name,
        acc.category,
        acc.parent_code,
        acc.normal_balance,
        acc.is_contra,
        0 AS raw_balance
    FROM accounting.accounts acc
    WHERE acc.code IN (
        SELECT parent_code FROM account_balances WHERE parent_code IS NOT NULL
    )
    AND acc.code NOT IN (SELECT code FROM account_balances)
),

-- 3. Normalize signs: positive for assets/expenses, negative for liabilities/equity
signed_accounts AS (
    SELECT
        code,
        name,
        category,
        parent_code,
        CASE
            WHEN is_contra THEN -raw_balance
            WHEN normal_balance = 'cr' THEN -raw_balance
            ELSE raw_balance
        END AS balance
    FROM all_relevant_accounts
),

-- 4. Recursive roll-up tree
tree AS (
    SELECT
        code,
        name,
        category,
        parent_code,
        balance,
        balance AS total_balance,
        0 AS depth,
        ARRAY[code] AS path
    FROM signed_accounts

    UNION ALL

    SELECT
        p.code,
        p.name,
        p.category,
        p.parent_code,
        p.balance,
        p.balance + c.total_balance,
        c.depth + 1,
        p.code || c.path
    FROM tree c
    JOIN signed_accounts p
        ON p.code = c.parent_code
)

-- 5. Select one row per account with rolled-up total balance
SELECT DISTINCT ON (code)
    code,
    name,
    category,
    depth,
    path,
    total_balance AS balance
FROM tree
ORDER BY code, depth DESC;
$$ LANGUAGE sql STABLE;

-- ===================================================
-- 2 Trial Balance Statement
-- ===================================================
CREATE OR REPLACE FUNCTION reporting.get_trial_balance(as_of DATE)
RETURNS TABLE (
    code TEXT,
    name TEXT,
    category TEXT,
    debit NUMERIC,
    credit NUMERIC,
    balance NUMERIC
) AS
$$
WITH signed AS (
    SELECT
        acc.code,
        acc.name,
        acc.category,

        -- Universal signed balance: + means debit, - means credit
        COALESCE(SUM(te.debit) FILTER (WHERE te.created_at <= as_of), 0)
      - COALESCE(SUM(te.credit) FILTER (WHERE te.created_at <= as_of), 0)
        AS balance
    FROM accounting.accounts acc
    LEFT JOIN accounting.transaction_entries te
        ON te.account_uuid = acc.uuid
    GROUP BY acc.code, acc.name, acc.category
)

SELECT
    code,
    name,
    category,
    GREATEST(balance, 0) AS debit,
    GREATEST(-balance, 0) AS credit,
    balance
FROM signed
WHERE balance <> 0
ORDER BY code;
$$ LANGUAGE sql STABLE;

-- ===================================================
-- 3 Income Statement
-- ===================================================
CREATE OR REPLACE FUNCTION reporting.get_income_statement(
    period_start DATE,
    period_end   DATE
)
RETURNS TABLE (
    code TEXT,
    name TEXT,
    category TEXT,
    depth INT,
    path TEXT[],
    balance NUMERIC
) AS
$$
WITH RECURSIVE

account_balances AS (
    SELECT
        acc.uuid,
        acc.code,
        acc.name,
        acc.category,
        acc.parent_code,
        acc.normal_balance,
        acc.is_contra,
        COALESCE(
            SUM(te.debit) FILTER (
                WHERE te.created_at BETWEEN period_start AND period_end
            ), 0
        )
      - COALESCE(
            SUM(te.credit) FILTER (
                WHERE te.created_at BETWEEN period_start AND period_end
            ), 0
        ) AS raw_balance
    FROM accounting.accounts acc
    LEFT JOIN accounting.transaction_entries te
        ON te.account_uuid = acc.uuid
    WHERE acc.category IN ('income', 'expense')
    GROUP BY acc.uuid, acc.code, acc.name, acc.category,
             acc.parent_code, acc.normal_balance, acc.is_contra
),

all_relevant_accounts AS (
    SELECT * FROM account_balances

    UNION

    SELECT
        acc.uuid,
        acc.code,
        acc.name,
        acc.category,
        acc.parent_code,
        acc.normal_balance,
        acc.is_contra,
        0 AS raw_balance
    FROM accounting.accounts acc
    WHERE acc.code IN (
        SELECT parent_code
        FROM account_balances
        WHERE parent_code IS NOT NULL
    )
    AND acc.code NOT IN (
        SELECT code FROM account_balances
    )
),

signed_accounts AS (
    SELECT
        code,
        name,
        category,
        parent_code,
        CASE
            WHEN is_contra THEN -raw_balance
            WHEN normal_balance = 'cr' THEN -raw_balance
            ELSE raw_balance
        END AS balance
    FROM all_relevant_accounts
),

tree AS (
    SELECT
        code,
        name,
        category,
        parent_code,
        balance,
        balance AS total_balance,
        0 AS depth,
        ARRAY[code] AS path
    FROM signed_accounts

    UNION ALL

    SELECT
        p.code,
        p.name,
        p.category,
        p.parent_code,
        p.balance,
        p.balance + c.total_balance,
        c.depth + 1,
        p.code || c.path
    FROM tree c
    JOIN signed_accounts p
        ON p.code = c.parent_code
)

SELECT DISTINCT ON (code)
    code,
    name,
    category,
    depth,
    path,
    total_balance AS balance
FROM tree
ORDER BY code, depth DESC;

$$ LANGUAGE sql STABLE;

-- ===================================================
-- 4 Cash Flow Statement - Indirect
-- ===================================================
CREATE OR REPLACE FUNCTION reporting.get_cash_flow_statement(
    start_date DATE,
    end_date DATE
)
RETURNS TABLE (
    section TEXT,
    description TEXT,
    amount NUMERIC
) AS
$$
WITH

-- Net Income pulled from income statement
net_income AS (
    SELECT SUM(balance) AS ni
    FROM reporting.get_income_statement(start_date, end_date)
),

-- Account changes in the period
account_changes AS (
    SELECT
        acc.code,
        acc.name,
        acc.category,
        COALESCE(SUM(te.debit) FILTER (WHERE te.created_at <= end_date), 0)
      - COALESCE(SUM(te.credit) FILTER (WHERE te.created_at <= end_date), 0)
        -
        (
            COALESCE(SUM(te.debit) FILTER (WHERE te.created_at < start_date), 0)
          - COALESCE(SUM(te.credit) FILTER (WHERE te.created_at < start_date), 0)
        ) AS delta
    FROM accounting.accounts acc
    LEFT JOIN accounting.transaction_entries te
        ON te.account_uuid = acc.uuid
    GROUP BY acc.code, acc.name, acc.category
),

-- Operating adjustments (working capital)
working_capital AS (
    SELECT
        'Operating Activities' AS section,
        name AS description,
        CASE
            WHEN category = 'asset' THEN -delta   -- increase in assets = cash out
            WHEN category = 'liability' THEN delta  -- increase in liability = cash in
        END AS amount
    FROM account_changes
    WHERE category IN ('asset', 'liability')
      AND code NOT LIKE '12%'   -- exclude non-current assets? optional
),

-- Investing activities (usually long-term assets)
investing AS (
    SELECT
        'Investing Activities',
        name,
        -delta AS amount
    FROM account_changes
    WHERE category = 'asset'
      AND code LIKE '12%'       -- long-term assets by code pattern
),

-- Financing (equity + long-term liabilities)
financing AS (
    SELECT
        'Financing Activities',
        name,
        delta AS amount
    FROM account_changes
    WHERE category IN ('equity', 'liability')
      AND code LIKE '22%'       -- long-term liabilities & equity
)

SELECT * FROM (
    SELECT 'Operating Activities', 'Net Income', ni FROM net_income
    UNION ALL
    SELECT * FROM working_capital
    UNION ALL
    SELECT * FROM investing
    UNION ALL
    SELECT * FROM financing
) x;
-- WHERE amount IS NOT NULL;
$$ LANGUAGE sql STABLE;

-- ===================================================
-- 4 Statement of Cash Flow - v2
-- ===================================================
CREATE OR REPLACE FUNCTION reporting.get_cashflow_direct_full(
    start_date DATE,
    end_date DATE
)
RETURNS TABLE (
    code TEXT,
    name TEXT,
    activity_group TEXT,
    depth INT,
    path TEXT[],
    inflow NUMERIC,
    outflow NUMERIC,
    net_cash NUMERIC
) AS
$$
WITH RECURSIVE

-- Step 1: Select all cash-impacting transactions
cash_txns AS (
    SELECT
        acc.uuid,
        acc.code,
        acc.name,
        acc.parent_code,
        CASE
            WHEN acc.code LIKE '11%' THEN 'Operating'
            WHEN acc.code LIKE '12%' THEN 'Investing'
            WHEN acc.code LIKE '21%' OR acc.code LIKE '22%' OR acc.category IN ('equity') THEN 'Financing'
            ELSE 'Other'
        END AS activity_group,
        SUM(te.debit) AS total_debit,
        SUM(te.credit) AS total_credit
    FROM accounting.transaction_entries te
    JOIN accounting.accounts acc ON acc.uuid = te.account_uuid
    WHERE te.created_at BETWEEN start_date AND end_date
      -- Include all accounts that actually affect cash
      AND (acc.code LIKE '11%' OR acc.code LIKE '12%' OR acc.code LIKE '21%' OR acc.code LIKE '22%' OR acc.category = 'equity')
    GROUP BY acc.uuid, acc.code, acc.name, acc.parent_code, acc.category
),

-- Step 2: Include parent accounts for roll-up
all_relevant_accounts AS (
    SELECT * FROM cash_txns

    UNION

    SELECT
        acc.uuid,
        acc.code,
        acc.name,
        acc.parent_code,
        CASE
            WHEN acc.code LIKE '11%' THEN 'Operating'
            WHEN acc.code LIKE '12%' THEN 'Investing'
            WHEN acc.code LIKE '21%' OR acc.code LIKE '22%' OR acc.category IN ('equity') THEN 'Financing'
            ELSE 'Other'
        END AS activity_group,
        0 AS total_debit,
        0 AS total_credit
    FROM accounting.accounts acc
    WHERE acc.code IN (
        SELECT parent_code FROM cash_txns WHERE parent_code IS NOT NULL
    )
    AND acc.code NOT IN (SELECT code FROM cash_txns)
),

-- Step 3: Recursive roll-up
tree AS (
    SELECT
        code,
        name,
        activity_group,
        parent_code,
        total_debit AS inflow,
        total_credit AS outflow,
        total_debit - total_credit AS net_cash,
        0 AS depth,
        ARRAY[code] AS path
    FROM all_relevant_accounts

    UNION ALL

    SELECT
        p.code,
        p.name,
        p.activity_group,
        p.parent_code,
        p.total_debit + c.inflow,
        p.total_credit + c.outflow,
        (p.total_debit + c.inflow) - (p.total_credit + c.outflow),
        c.depth + 1,
        p.code || c.path
    FROM tree c
    JOIN all_relevant_accounts p ON p.code = c.parent_code
)

SELECT DISTINCT ON (code)
    code,
    name,
    activity_group,
    depth,
    path,
    inflow,
    outflow,
    net_cash
FROM tree
ORDER BY code, depth DESC;
$$ LANGUAGE sql STABLE;

-- ===================================================
-- 5 Statement of Change of Equity
-- ===================================================
CREATE OR REPLACE FUNCTION reporting.get_statement_of_changes_in_equity(
    start_date DATE,
    end_date DATE
)
RETURNS TABLE (
    code TEXT,
    name TEXT,
    description TEXT,
    amount NUMERIC
) AS
$$
WITH opening AS (
    SELECT
        acc.code,
        acc.name,
        'Opening Balance' AS description,
        COALESCE(SUM(te.debit) FILTER (WHERE te.created_at < start_date), 0)
      - COALESCE(SUM(te.credit) FILTER (WHERE te.created_at < start_date), 0)
        AS amount
    FROM accounting.accounts acc
    LEFT JOIN accounting.transaction_entries te
        ON te.account_uuid = acc.uuid
    WHERE acc.category = 'equity'
    GROUP BY acc.code, acc.name
),

changes AS (
    SELECT
        acc.code,
        acc.name,
        'Movements During Period' AS description,
        COALESCE(SUM(te.debit) FILTER (WHERE te.created_at BETWEEN start_date AND end_date), 0)
      - COALESCE(SUM(te.credit) FILTER (WHERE te.created_at BETWEEN start_date AND end_date), 0)
        AS amount
    FROM accounting.accounts acc
    LEFT JOIN accounting.transaction_entries te
        ON te.account_uuid = acc.uuid
    WHERE acc.category = 'equity'
    GROUP BY acc.code, acc.name
),

net_income AS (
    SELECT
        '999999'::text AS code,
        'Net Income' AS name,
        'Net Income for Period' AS description,
        SUM(balance) AS amount
    FROM reporting.get_income_statement(start_date, end_date)
)

SELECT * FROM opening
UNION ALL
SELECT * FROM changes
UNION ALL
SELECT * FROM net_income
ORDER BY code, description;
$$ LANGUAGE sql STABLE;

-- ===================================================
-- 6 Balancesheet Compare Multiple Periods
-- ===================================================
CREATE OR REPLACE FUNCTION reporting.get_balance_sheet_compare(
    as_of_1 DATE,
    as_of_2 DATE
)
RETURNS TABLE (
    code TEXT,
    name TEXT,
    category TEXT,
    depth INT,
    path TEXT[],
    balance_1 NUMERIC,
    balance_2 NUMERIC,
    delta NUMERIC
) AS
$$
WITH
bs1 AS (
    SELECT * FROM reporting.get_balance_sheet(as_of_1)
),
bs2 AS (
    SELECT * FROM reporting.get_balance_sheet(as_of_2)
)
SELECT
    COALESCE(bs1.code, bs2.code) AS code,
    COALESCE(bs1.name, bs2.name) AS name,
    COALESCE(bs1.category, bs2.category) AS category,
    COALESCE(bs1.depth, bs2.depth) AS depth,
    COALESCE(bs1.path, bs2.path) AS path,
    COALESCE(bs1.balance, 0) AS balance_1,
    COALESCE(bs2.balance, 0) AS balance_2,
    COALESCE(bs2.balance, 0) - COALESCE(bs1.balance, 0) AS delta
FROM bs1
FULL OUTER JOIN bs2 USING (code)
ORDER BY path;
$$ LANGUAGE sql STABLE;


-- Example of uses
-- SELECT * FROM reporting.get_balance_sheet('2024-12-31');
-- SELECT * FROM reporting.get_trial_balance('2025-12-31') ORDER BY code;
-- SELECT * FROM reporting.get_income_statement('2025-01-01', '2025-12-31');
-- SELECT * FROM reporting.get_cash_flow_statement('2025-01-01', '2025-12-31');
-- SELECT * FROM reporting.get_statement_of_changes_in_equity('2025-01-01', '2025-12-31');
-- SELECT * FROM reporting.get_income_statement_v2('2025-01-01', '2025-12-31');
-- SELECT * FROM reporting.get_cashflow_direct_v2('2025-01-01', '2025-12-31');
