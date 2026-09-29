-- ========================================
-- SYSTEM MODULE - FULL SCHEMA (GLOBAL-READY)
-- Run this LAST — after all other modules
-- Provides: Universal Audit Trail + App Context
-- ========================================

-- Enable UUID extension
-- CREATE EXTENSION IF NOT EXISTS pg_uuidv7;

-- Create schema
CREATE SCHEMA IF NOT EXISTS system;

-- ========================================
-- SEQUENCE
-- ========================================
CREATE SEQUENCE IF NOT EXISTS system.audit_logs_serial_id_seq;

-- ========================================
-- TABLE: audit_logs
-- ========================================
CREATE TABLE IF NOT EXISTS system.audit_logs (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('system.audit_logs_serial_id_seq') NOT NULL,
    schema_name text NOT NULL,
    table_name text NOT NULL,
    record_uuid uuid NOT NULL,
    record_serial_id bigint,
    operation text NOT NULL CHECK (operation IN ('INSERT', 'UPDATE', 'DELETE')),
    changed_by bigint,
    changed_at timestamptz DEFAULT now() NOT NULL,
    old_data jsonb,
    new_data jsonb,
    CONSTRAINT audit_logs_pkey PRIMARY KEY (uuid),
    CONSTRAINT audit_logs_serial_id_key UNIQUE (serial_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_audit_schema_table ON system.audit_logs(schema_name, table_name);
CREATE INDEX IF NOT EXISTS idx_audit_record_uuid ON system.audit_logs(record_uuid);
CREATE INDEX IF NOT EXISTS idx_audit_changed_by ON system.audit_logs(changed_by);
CREATE INDEX IF NOT EXISTS idx_audit_changed_at ON system.audit_logs(changed_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_operation ON system.audit_logs(operation);

-- ========================================
-- FUNCTION: audit_trigger_fn()
-- ========================================
CREATE OR REPLACE FUNCTION system.audit_trigger_fn()
RETURNS trigger LANGUAGE plpgsql AS $fn$
DECLARE
    v_record_uuid uuid;
    v_record_serial_id bigint;
    v_changed_by bigint;
    v_row record;
BEGIN
    -- Get current user from app context
    BEGIN
        v_changed_by := NULLIF(current_setting('app.current_user_id', true), '')::bigint;
    EXCEPTION WHEN OTHERS THEN
        v_changed_by := NULL;
    END;

    -- Select correct row
    v_row := CASE WHEN TG_OP = 'DELETE' THEN OLD ELSE NEW END;

    -- Extract UUID (assumes primary key column is 'uuid')
    BEGIN
        EXECUTE format('SELECT $1.%I', 'uuid') USING v_row INTO v_record_uuid;
    EXCEPTION WHEN OTHERS THEN
        v_record_uuid := NULL;
    END;

    -- Extract serial_id if exists
    BEGIN
        EXECUTE format('SELECT ($1::jsonb)->>%L', 'serial_id') USING v_row INTO v_record_serial_id;
    EXCEPTION WHEN OTHERS THEN
        v_record_serial_id := NULL;
    END;

    -- Fallback: try common patterns
    IF v_record_uuid IS NULL THEN
        BEGIN
            v_record_uuid := (v_row::jsonb)->>'uuid';
        EXCEPTION WHEN OTHERS THEN
            v_record_uuid := NULL;
        END;
    END IF;

    -- Insert audit log
    INSERT INTO system.audit_logs (
        schema_name,
        table_name,
        record_uuid,
        record_serial_id,
        operation,
        changed_by,
        old_data,
        new_data
    ) VALUES (
        TG_TABLE_SCHEMA,
        TG_TABLE_NAME,
        v_record_uuid,
        v_record_serial_id,
        TG_OP,
        v_changed_by,
        CASE WHEN TG_OP IN ('UPDATE', 'DELETE') THEN to_jsonb(OLD) END,
        CASE WHEN TG_OP IN ('INSERT', 'UPDATE') THEN to_jsonb(NEW) END
    );

    RETURN NULL;
END;
$fn$;

-- ========================================
-- HELPER: Set current user (call from app)
-- ========================================
CREATE OR REPLACE FUNCTION system.set_current_user(p_user_id bigint)
RETURNS void LANGUAGE sql AS $$
    SELECT set_config('app.current_user_id', p_user_id::text, false);
$$;

-- ========================================
-- HELPER: Clear current user
-- ========================================
CREATE OR REPLACE FUNCTION system.clear_current_user()
RETURNS void LANGUAGE sql AS $$
    SELECT set_config('app.current_user_id', '', false);
$$;


-- =============================================================================
-- Open period  →  activity updates current_balance
-- Close period →  1. Read current balances of all accounts (or just the ones that moved)
--                2. Write a snapshot row for each account for this period
--                3. Zero the nominal accounts (income + expense)
--                4. Lock period, open next period
CREATE TABLE accounting.period_account_balances (
    period_id       UUID NOT NULL REFERENCES accounting.financial_periods(uuid),
    account_uuid    UUID NOT NULL REFERENCES accounting.accounts(uuid),
    debit_total     NUMERIC(18,2) NOT NULL DEFAULT 0,
    credit_total    NUMERIC(18,2) NOT NULL DEFAULT 0,
    net_balance     NUMERIC(18,2) NOT NULL DEFAULT 0,  -- debit − credit (or signed by normal balance)
    category        TEXT NOT NULL,                     -- denormalised for convenience
    PRIMARY KEY (period_id, account_uuid)
);

CREATE INDEX ON accounting.period_account_balances (period_id);
CREATE INDEX ON accounting.period_account_balances (account_uuid);

-- At close time you simply:
INSERT INTO accounting.period_account_balances (...)
SELECT
    $period_id,
    a.uuid,
    COALESCE(SUM(te.debit), 0),
    COALESCE(SUM(te.credit), 0),
    COALESCE(SUM(te.debit - te.credit), 0),
    a.category
FROM accounting.accounts a
LEFT JOIN accounting.transaction_entries te ON te.account_uuid = a.uuid
LEFT JOIN accounting.transactions t ON t.uuid = te.transaction_uuid
    AND t.posted = true
    AND t.txn_date BETWEEN $start AND $end
GROUP BY a.uuid, a.category;

-- What reports look like afterwards March vs April P&L
SELECT account_uuid, category, net_balance
FROM accounting.period_account_balances
WHERE period_id IN ($march_id, $april_id);

-- // src/models/dto.rs
#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct FinancialPeriodDto {
    pub uuid: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub name: Option<String>,
    pub is_open: bool,
    pub is_locked: bool,
}
