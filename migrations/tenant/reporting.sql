--
-- PostgreSQL database dump
--

-- Dumped from database version 17.5 (Debian 17.5-1)
-- Dumped by pg_dump version 17.5 (Debian 17.5-1)
SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;
--
-- Name: reporting; Type: SCHEMA; Schema: -; Owner: chiefalry_user
--

CREATE SCHEMA reporting;
ALTER SCHEMA reporting OWNER TO chiefalry_user;
--
-- Name: refresh_all(boolean); Type: FUNCTION; Schema: reporting; Owner: chiefalry_user
--

CREATE FUNCTION reporting.refresh_all(p_force boolean DEFAULT false) RETURNS void LANGUAGE plpgsql AS $$
DECLARE v_start TIMESTAMPTZ := clock_timestamp();
BEGIN RAISE NOTICE 'Starting reporting refresh...';
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
RAISE NOTICE 'Reporting refresh completed in %',
clock_timestamp() - v_start;
END;
$$;
ALTER FUNCTION reporting.refresh_all(p_force boolean) OWNER TO chiefalry_user;
SET default_tablespace = '';
SET default_table_access_method = heap;
--
-- Name: ap_aging_detailed; Type: MATERIALIZED VIEW; Schema: reporting; Owner: chiefalry_user
--

CREATE MATERIALIZED VIEW reporting.ap_aging_detailed AS
SELECT v.serial_id AS vendor_serial_id,
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
    WHEN (b.due_date >= CURRENT_DATE) THEN 'Current'::text
    WHEN (b.due_date >= (CURRENT_DATE - 30)) THEN '1-30'::text
    WHEN (b.due_date >= (CURRENT_DATE - 60)) THEN '31-60'::text
    WHEN (b.due_date >= (CURRENT_DATE - 90)) THEN '61-90'::text
    ELSE 'Over 90'::text
  END AS aging_bucket
FROM (
    payables.bills b
    JOIN payables.vendors v ON ((v.uuid = b.vendor_uuid))
  )
WHERE (
    (
      b.status <> ALL (ARRAY ['Paid'::text, 'Cancelled'::text])
    )
    AND (b.balance_due > (0)::numeric)
  )
ORDER BY v.name,
  b.due_date WITH NO DATA;
ALTER MATERIALIZED VIEW reporting.ap_aging_detailed OWNER TO chiefalry_user;
--
-- Name: ar_aging_detailed; Type: MATERIALIZED VIEW; Schema: reporting; Owner: chiefalry_user
--

CREATE MATERIALIZED VIEW reporting.ar_aging_detailed AS
SELECT c.serial_id AS customer_serial_id,
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
    WHEN (i.due_date >= CURRENT_DATE) THEN 'Current'::text
    WHEN (i.due_date >= (CURRENT_DATE - 30)) THEN '1-30'::text
    WHEN (i.due_date >= (CURRENT_DATE - 60)) THEN '31-60'::text
    WHEN (i.due_date >= (CURRENT_DATE - 90)) THEN '61-90'::text
    ELSE 'Over 90'::text
  END AS aging_bucket
FROM (
    receivables.invoices i
    JOIN receivables.customers c ON ((c.uuid = i.customer_uuid))
  )
WHERE (
    (
      i.status <> ALL (ARRAY ['Paid'::text, 'Cancelled'::text])
    )
    AND (i.balance_due > (0)::numeric)
  )
ORDER BY c.name,
  i.due_date WITH NO DATA;
ALTER MATERIALIZED VIEW reporting.ar_aging_detailed OWNER TO chiefalry_user;
--
-- Name: balance_sheet; Type: MATERIALIZED VIEW; Schema: reporting; Owner: chiefalry_user
--

CREATE MATERIALIZED VIEW reporting.balance_sheet AS WITH balances AS (
  SELECT a.type,
    a.subtype,
    (
      COALESCE(sum(te.debit), (0)::numeric) - COALESCE(sum(te.credit), (0)::numeric)
    ) AS balance
  FROM (
      accounting.accounts a
      LEFT JOIN accounting.transaction_entries te ON ((te.account_uuid = a.uuid))
    )
  GROUP BY a.type,
    a.subtype
)
SELECT 'ASSETS'::text AS section,
  COALESCE(
    sum(balance) FILTER (
      WHERE (type = 'Asset'::text)
    ),
    (0)::numeric
  ) AS total_assets,
  COALESCE(
    sum(balance) FILTER (
      WHERE (
          (type = 'Asset'::text)
          AND (subtype = 'Current'::text)
        )
    ),
    (0)::numeric
  ) AS current_assets,
  COALESCE(
    sum(balance) FILTER (
      WHERE (
          (type = 'Asset'::text)
          AND (subtype = 'Non-current'::text)
        )
    ),
    (0)::numeric
  ) AS non_current_assets,
  'LIABILITIES'::text AS section2,
  COALESCE(
    sum(balance) FILTER (
      WHERE (type = 'Liability'::text)
    ),
    (0)::numeric
  ) AS total_liabilities,
  COALESCE(
    sum(balance) FILTER (
      WHERE (
          (type = 'Liability'::text)
          AND (subtype = 'Current'::text)
        )
    ),
    (0)::numeric
  ) AS current_liabilities,
  COALESCE(
    sum(balance) FILTER (
      WHERE (
          (type = 'Liability'::text)
          AND (subtype = 'Non-current'::text)
        )
    ),
    (0)::numeric
  ) AS non_current_liabilities,
  'EQUITY'::text AS section3,
  COALESCE(
    sum(balance) FILTER (
      WHERE (type = 'Equity'::text)
    ),
    (0)::numeric
  ) AS total_equity
FROM balances WITH NO DATA;
ALTER MATERIALIZED VIEW reporting.balance_sheet OWNER TO chiefalry_user;
--
-- Name: cash_flow; Type: MATERIALIZED VIEW; Schema: reporting; Owner: chiefalry_user
--

CREATE MATERIALIZED VIEW reporting.cash_flow AS WITH op AS (
  SELECT COALESCE(sum((te.debit - te.credit)), (0)::numeric) AS net_income
  FROM (
      accounting.transaction_entries te
      JOIN accounting.accounts a ON ((a.uuid = te.account_uuid))
    )
  WHERE (
      a.type = ANY (ARRAY ['Revenue'::text, 'Expense'::text])
    )
),
dep AS (
  SELECT COALESCE(sum(te.debit), (0)::numeric) AS depreciation
  FROM (
      accounting.transaction_entries te
      JOIN accounting.accounts a ON ((a.uuid = te.account_uuid))
    )
  WHERE (a.code = '5.3.1'::text)
),
ar_change AS (
  SELECT COALESCE(
      sum((i.total_amount - i.balance_due)),
      (0)::numeric
    ) AS ar_increase
  FROM receivables.invoices i
  WHERE (
      i.status <> ALL (ARRAY ['Paid'::text, 'Cancelled'::text])
    )
),
ap_change AS (
  SELECT COALESCE(
      sum((b.total_amount - b.balance_due)),
      (0)::numeric
    ) AS ap_increase
  FROM payables.bills b
  WHERE (
      b.status <> ALL (ARRAY ['Paid'::text, 'Cancelled'::text])
    )
),
inv_change AS (
  SELECT COALESCE(
      sum((i.quantity_on_hand * i.cost_price)),
      (0)::numeric
    ) AS inventory_value
  FROM inventory.items i
)
SELECT (o.net_income + d.depreciation) AS cash_from_operations,
  d.depreciation AS add_back_depreciation,
  (ar_change.ar_increase * ('-1'::integer)::numeric) AS decrease_in_ar,
  ap_change.ap_increase AS increase_in_ap,
  (
    inv_change.inventory_value * ('-1'::integer)::numeric
  ) AS increase_in_inventory,
  (0)::numeric AS capex,
  (0)::numeric AS financing,
  (
    (
      (
        (o.net_income + d.depreciation) - ar_change.ar_increase
      ) + ap_change.ap_increase
    ) - inv_change.inventory_value
  ) AS net_cash_flow
FROM op o,
  dep d,
  ar_change,
  ap_change,
  inv_change WITH NO DATA;
ALTER MATERIALIZED VIEW reporting.cash_flow OWNER TO chiefalry_user;
--
-- Name: cashbook; Type: MATERIALIZED VIEW; Schema: reporting; Owner: chiefalry_user
--

CREATE MATERIALIZED VIEW reporting.cashbook AS
SELECT t.serial_id AS txn_serial_id,
  t.txn_date,
  t.reference,
  te.account_uuid,
  a.code AS account_code,
  a.name AS account_name,
  te.debit,
  te.credit,
  te.memo
FROM (
    (
      accounting.transactions t
      JOIN accounting.transaction_entries te ON ((te.transaction_uuid = t.uuid))
    )
    JOIN accounting.accounts a ON ((a.uuid = te.account_uuid))
  )
WHERE (a.code ~~ '1.1.%'::text)
ORDER BY t.txn_date DESC,
  t.serial_id WITH NO DATA;
ALTER MATERIALIZED VIEW reporting.cashbook OWNER TO chiefalry_user;
--
-- Name: customer_statement; Type: MATERIALIZED VIEW; Schema: reporting; Owner: chiefalry_user
--

CREATE MATERIALIZED VIEW reporting.customer_statement AS
SELECT c.serial_id AS customer_serial_id,
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
FROM (
    (
      (
        receivables.customers c
        LEFT JOIN receivables.invoices i ON ((i.customer_uuid = c.uuid))
      )
      LEFT JOIN receivables.payment_applications pa ON ((pa.invoice_uuid = i.uuid))
    )
    LEFT JOIN receivables.payments p ON ((p.uuid = pa.payment_uuid))
  ) WITH NO DATA;
ALTER MATERIALIZED VIEW reporting.customer_statement OWNER TO chiefalry_user;
--
-- Name: income_statement; Type: MATERIALIZED VIEW; Schema: reporting; Owner: chiefalry_user
--

CREATE MATERIALIZED VIEW reporting.income_statement AS WITH rev AS (
  SELECT COALESCE(sum((te.debit - te.credit)), (0)::numeric) AS revenue
  FROM (
      accounting.transaction_entries te
      JOIN accounting.accounts a ON ((a.uuid = te.account_uuid))
    )
  WHERE (a.type = 'Revenue'::text)
),
cogs AS (
  SELECT COALESCE(sum((te.debit - te.credit)), (0)::numeric) AS cogs
  FROM (
      accounting.transaction_entries te
      JOIN accounting.accounts a ON ((a.uuid = te.account_uuid))
    )
  WHERE (a.code ~~ '5.2.%'::text)
),
opex AS (
  SELECT COALESCE(sum((te.debit - te.credit)), (0)::numeric) AS opex
  FROM (
      accounting.transaction_entries te
      JOIN accounting.accounts a ON ((a.uuid = te.account_uuid))
    )
  WHERE (
      (a.type = 'Expense'::text)
      AND (a.code !~~ '5.2.%'::text)
    )
),
payroll AS (
  SELECT COALESCE(sum(ps.gross_pay), (0)::numeric) AS payroll_expense
  FROM (
      payroll.payslips ps
      JOIN payroll.payruns pr ON ((pr.uuid = ps.payrun_uuid))
    )
  WHERE ((pr.status)::text = 'Posted'::text)
)
SELECT r.revenue,
  c.cogs,
  (r.revenue - c.cogs) AS gross_profit,
  (o.opex + p.payroll_expense) AS operating_expenses,
  (
    (r.revenue - c.cogs) - (o.opex + p.payroll_expense)
  ) AS operating_income,
  (0)::numeric AS other_income,
  (0)::numeric AS other_expense,
  (
    (r.revenue - c.cogs) - (o.opex + p.payroll_expense)
  ) AS net_income
FROM rev r,
  cogs c,
  opex o,
  payroll p WITH NO DATA;
ALTER MATERIALIZED VIEW reporting.income_statement OWNER TO chiefalry_user;
--
-- Name: inventory_valuation; Type: MATERIALIZED VIEW; Schema: reporting; Owner: chiefalry_user
--

CREATE MATERIALIZED VIEW reporting.inventory_valuation AS
SELECT i.serial_id AS item_serial_id,
  i.sku,
  i.name,
  c.name AS category,
  i.unit,
  i.quantity_on_hand,
  i.cost_price,
  (i.quantity_on_hand * i.cost_price) AS total_value,
  i.reorder_level,
  (i.quantity_on_hand <= i.reorder_level) AS needs_reorder
FROM (
    inventory.items i
    LEFT JOIN inventory.item_categories c ON ((c.uuid = i.category_uuid))
  )
WHERE (i.track_quantity = true)
ORDER BY (i.quantity_on_hand * i.cost_price) DESC WITH NO DATA;
ALTER MATERIALIZED VIEW reporting.inventory_valuation OWNER TO chiefalry_user;
--
-- Name: payroll_summary; Type: MATERIALIZED VIEW; Schema: reporting; Owner: chiefalry_user
--

CREATE MATERIALIZED VIEW reporting.payroll_summary AS
SELECT pr.serial_id AS payrun_serial_id,
  pr.pay_period_start,
  pr.pay_period_end,
  pr.payment_date,
  count(ps.uuid) AS employees_paid,
  sum(ps.gross_pay) AS total_gross,
  sum(ps.tax_deducted) AS total_tax,
  sum(ps.nssf) AS total_nssf,
  sum(ps.other_deductions) AS total_other_deductions,
  sum(ps.net_pay) AS total_net,
  pr.status
FROM (
    payroll.payruns pr
    LEFT JOIN payroll.payslips ps ON ((ps.payrun_uuid = pr.uuid))
  )
GROUP BY pr.serial_id,
  pr.pay_period_start,
  pr.pay_period_end,
  pr.payment_date,
  pr.status
ORDER BY pr.pay_period_start DESC WITH NO DATA;
ALTER MATERIALIZED VIEW reporting.payroll_summary OWNER TO chiefalry_user;
--
-- Name: trial_balance; Type: MATERIALIZED VIEW; Schema: reporting; Owner: chiefalry_user
--

CREATE MATERIALIZED VIEW reporting.trial_balance AS
SELECT a.serial_id AS account_serial_id,
  a.code,
  a.name,
  a.type,
  COALESCE(sum(te.debit), (0)::numeric) AS total_debit,
  COALESCE(sum(te.credit), (0)::numeric) AS total_credit,
  COALESCE(sum((te.debit - te.credit)), (0)::numeric) AS balance
FROM (
    accounting.accounts a
    LEFT JOIN accounting.transaction_entries te ON ((te.account_uuid = a.uuid))
  )
GROUP BY a.serial_id,
  a.code,
  a.name,
  a.type
ORDER BY a.code WITH NO DATA;
ALTER MATERIALIZED VIEW reporting.trial_balance OWNER TO chiefalry_user;
--
-- PostgreSQL database dump complete
--