-- ========================================
-- REPORTING MODULE - FULL SCHEMA (GLOBAL-READY)
-- Run this ONCE after all core modules exist:
--   accounting, payables, receivables, payroll, inventory
-- ========================================

-- Enable UUID extension (if not already)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

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
        REFRESH MATERIALIZED VIEW CONCURRENTLY reporting.balance_sheet;
        REFRESH MATERIALIZED VIEW CONCURRENTLY reporting.income_statement;
        REFRESH MATERIALIZED VIEW CONCURRENTLY reporting.trial_balance;
        REFRESH MATERIALIZED VIEW CONCURRENTLY reporting.cashbook;
        REFRESH MATERIALIZED VIEW CONCURRENTLY reporting.cash_flow;
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

-- 1. Balance Sheet
CREATE MATERIALIZED VIEW IF NOT EXISTS reporting.balance_sheet AS
WITH balances AS (
    SELECT
        a.type,
        a.subtype,
        COALESCE(SUM(te.debit), 0) - COALESCE(SUM(te.credit), 0) AS balance
    FROM accounting.accounts a
    LEFT JOIN accounting.transaction_entries te ON te.account_uuid = a.uuid
    GROUP BY a.type, a.subtype
)
SELECT
    'ASSETS' AS section,
    COALESCE(SUM(balance) FILTER (WHERE type = 'Asset'), 0) AS total_assets,
    COALESCE(SUM(balance) FILTER (WHERE type = 'Asset' AND subtype = 'Current'), 0) AS current_assets,
    COALESCE(SUM(balance) FILTER (WHERE type = 'Asset' AND subtype = 'Non-current'), 0) AS non_current_assets,
    'LIABILITIES' AS section2,
    COALESCE(SUM(balance) FILTER (WHERE type = 'Liability'), 0) AS total_liabilities,
    COALESCE(SUM(balance) FILTER (WHERE type = 'Liability' AND subtype = 'Current'), 0) AS current_liabilities,
    COALESCE(SUM(balance) FILTER (WHERE type = 'Liability' AND subtype = 'Non-current'), 0) AS non_current_liabilities,
    'EQUITY' AS section3,
    COALESCE(SUM(balance) FILTER (WHERE type = 'Equity'), 0) AS total_equity
FROM balances
WITH NO DATA;

-- 2. Income Statement
CREATE MATERIALIZED VIEW IF NOT EXISTS reporting.income_statement AS
WITH
rev AS (
    SELECT COALESCE(SUM(te.debit - te.credit), 0) AS revenue
    FROM accounting.transaction_entries te
    JOIN accounting.accounts a ON a.uuid = te.account_uuid
    WHERE a.type = 'Revenue'
),
cogs AS (
    SELECT COALESCE(SUM(te.debit - te.credit), 0) AS cogs
    FROM accounting.transaction_entries te
    JOIN accounting.accounts a ON a.uuid = te.account_uuid
    WHERE a.code LIKE '5.2.%'
),
opex AS (
    SELECT COALESCE(SUM(te.debit - te.credit), 0) AS opex
    FROM accounting.transaction_entries te
    JOIN accounting.accounts a ON a.uuid = te.account_uuid
    WHERE a.type = 'Expense' AND a.code NOT LIKE '5.2.%'
),
payroll AS (
    SELECT COALESCE(SUM(ps.gross_pay), 0) AS payroll_expense
    FROM payroll.payslips ps
    JOIN payroll.payruns pr ON pr.uuid = ps.payrun_uuid
    WHERE pr.status = 'Posted'
)
SELECT
    r.revenue,
    c.cogs,
    (r.revenue - c.cogs) AS gross_profit,
    (o.opex + p.payroll_expense) AS operating_expenses,
    (r.revenue - c.cogs - o.opex - p.payroll_expense) AS operating_income,
    0::numeric AS other_income,
    0::numeric AS other_expense,
    (r.revenue - c.cogs - o.opex - p.payroll_expense) AS net_income
FROM rev r, cogs c, opex o, payroll p
WITH NO DATA;

-- 3. Trial Balance
CREATE MATERIALIZED VIEW IF NOT EXISTS reporting.trial_balance AS
SELECT
    a.serial_id AS account_serial_id,
    a.code,
    a.name,
    a.type,
    COALESCE(SUM(te.debit), 0) AS total_debit,
    COALESCE(SUM(te.credit), 0) AS total_credit,
    COALESCE(SUM(te.debit - te.credit), 0) AS balance
FROM accounting.accounts a
LEFT JOIN accounting.transaction_entries te ON te.account_uuid = a.uuid
GROUP BY a.serial_id, a.code, a.name, a.type
ORDER BY a.code
WITH NO DATA;

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
WHERE a.code LIKE '1.1.%'
ORDER BY t.txn_date DESC, t.serial_id
WITH NO DATA;

-- 5. Cash Flow (Simplified)
CREATE MATERIALIZED VIEW IF NOT EXISTS reporting.cash_flow AS
WITH
op AS (
    SELECT COALESCE(SUM(te.debit - te.credit), 0) AS net_income
    FROM accounting.transaction_entries te
    JOIN accounting.accounts a ON a.uuid = te.account_uuid
    WHERE a.type IN ('Revenue', 'Expense')
),
dep AS (
    SELECT COALESCE(SUM(te.debit), 0) AS depreciation
    FROM accounting.transaction_entries te
    JOIN accounting.accounts a ON a.uuid = te.account_uuid
    WHERE a.code = '5.3.1'
),
ar_change AS (
    SELECT COALESCE(SUM(i.total_amount - i.balance_due), 0) AS ar_increase
    FROM receivables.invoices i
    WHERE i.status NOT IN ('Paid', 'Cancelled')
),
ap_change AS (
    SELECT COALESCE(SUM(b.total_amount - b.balance_due), 0) AS ap_increase
    FROM payables.bills b
    WHERE b.status NOT IN ('Paid', 'Cancelled')
),
inv_change AS (
    SELECT COALESCE(SUM(i.quantity_on_hand * i.cost_price), 0) AS inventory_value
    FROM inventory.items i
    WHERE i.track_quantity = true
)
SELECT
    (o.net_income + d.depreciation) AS cash_from_operations,
    d.depreciation AS add_back_depreciation,
    (-ar_change.ar_increase) AS decrease_in_ar,
    ap_change.ap_increase AS increase_in_ap,
    (-inv_change.inventory_value) AS increase_in_inventory,
    0::numeric AS capex,
    0::numeric AS financing,
    (o.net_income + d.depreciation - ar_change.ar_increase + ap_change.ap_increase - inv_change.inventory_value) AS net_cash_flow
FROM op o, dep d, ar_change, ap_change, inv_change
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
FROM receivables.invoices i
JOIN receivables.customers c ON c.uuid = i.customer_uuid
WHERE i.status NOT IN ('Paid', 'Cancelled') AND i.balance_due > 0
ORDER BY c.name, i.due_date
WITH NO DATA;

-- 7. AP Aging Detailed
CREATE MATERIALIZED VIEW IF NOT EXISTS reporting.ap_aging_detailed AS
SELECT
    v.serial_id AS vendor_serial_id,
    v.code,
    v.name致力于,
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
FROM payables.bills b
JOIN payables.vendors v ON v.uuid = b.vendor_uuid
WHERE b.status NOT IN ('Paid', 'Cancelled') AND b.balance_due > 0
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
    SUM(ps.nssf) AS total_nssf,
    SUM(ps.other_deductions) AS total_other_deductions,
    SUM(ps.net_pay) AS total_net,
    pr.status
FROM payroll.payruns pr
LEFT JOIN payroll.payslips ps ON ps.payrun_uuid = pr.uuid
GROUP BY pr.serial_id, pr.pay_period_start, pr.pay_period_end, pr.payment_date, pr.status
ORDER BY pr.pay_period_start DESC
WITH NO DATA;

-- 9. Inventory Valuation
CREATE MATERIALIZED VIEW IF NOT EXISTS reporting.inventory_valuation AS
SELECT
    i.serial_id AS item_serial_id,
    i.sku,
    i.name,
    c.name AS category,
    i.unit,
    i.quantity_on_hand,
    i.cost_price,
    (i.quantity_on_hand * i.cost_price) AS total_value,
    i.reorder_level,
    (i.quantity_on_hand <= i.reorder_level) AS needs_reorder
FROM inventory.items i
LEFT JOIN inventory.item_categories c ON c.uuid = i.category_uuid
WHERE i.track_quantity = true
ORDER BY total_value DESC
WITH NO DATA;

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
FROM receivables.customers c
LEFT JOIN receivables.invoices i ON i.customer_uuid = c.uuid
LEFT JOIN receivables.payment_applications pa ON pa.invoice_uuid = i.uuid
LEFT JOIN receivables.payments p ON p.uuid = pa.payment_uuid
WITH NO DATA;

-- ========================================
-- INDEXES ON MATERIALIZED VIEWS (Optional but recommended)
-- ========================================
CREATE INDEX IF NOT EXISTS idx_ar_aging_customer ON reporting.ar_aging_detailed(customer_serial_id);
CREATE INDEX IF NOT EXISTS idx_ap_aging_vendor ON reporting.ap_aging_detailed(vendor_serial_id);
CREATE INDEX IF NOT EXISTS idx_cashbook_date ON reporting.cashbook(txn_date DESC);
CREATE INDEX IF NOT EXISTS idx_trial_balance_code ON reporting.trial_balance(code);
