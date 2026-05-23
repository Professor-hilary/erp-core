CREATE OR REPLACE FUNCTION accounting.post_transaction(
    p_reference TEXT,
    p_description TEXT,
    p_created_by UUID,
    p_module TEXT,
    p_txn_date DATE DEFAULT NULL,
    p_lines JSONB DEFAULT NULL,
    p_cash_flow_section VARCHAR(32) DEFAULT NULL,
    p_cash_flow_activity VARCHAR(64) DEFAULT NULL,
    p_cash_flow_description TEXT DEFAULT NULL
)
RETURNS BIGINT
LANGUAGE plpgsql
AS $$
DECLARE
    v_txn_uuid UUID;
    v_txn_serial_id BIGINT;
    v_total_debits NUMERIC(18,2) := 0;
    v_total_credits NUMERIC(18,2) := 0;
    v_line RECORD;
    v_account_uuid UUID;
    v_line_no INT := 0;
BEGIN
    -- Validate transaction is balanced
    FOR v_line IN
        SELECT * FROM jsonb_to_recordset(p_lines)
        AS t(account_ref JSONB, debit NUMERIC, credit NUMERIC, memo TEXT)
    LOOP
        v_total_debits := v_total_debits + COALESCE(v_line.debit, 0);
        v_total_credits := v_total_credits + COALESCE(v_line.credit, 0);
    END LOOP;

    IF v_total_debits <> v_total_credits THEN
        RAISE EXCEPTION 'Unbalanced transaction: debits (%) != credits (%)',
            v_total_debits, v_total_credits;
    END IF;

    -- Insert transaction header
    INSERT INTO accounting.transactions (
        txn_date, reference, description, created_by, module
    )
    VALUES (
        p_txn_date, p_reference, p_description, p_created_by, p_module
    )
    RETURNING uuid, serial_id INTO v_txn_uuid, v_txn_serial_id;

    -- Insert journal entries
    FOR v_line IN
        SELECT * FROM jsonb_to_recordset(p_lines)
        AS t(account_ref JSONB, debit NUMERIC, credit NUMERIC, memo TEXT)
    LOOP
        v_line_no := v_line_no + 1;

        -- Resolve account (support code, uuid, or serial_id)
        IF jsonb_typeof(v_line.account_ref) = 'string' THEN
            DECLARE
                v_ref_text TEXT := v_line.account_ref #>> '{}';
            BEGIN
                BEGIN
                    v_account_uuid := v_ref_text::UUID;
                    IF NOT EXISTS (SELECT 1 FROM accounting.accounts WHERE uuid = v_account_uuid) THEN
                        RAISE EXCEPTION 'Account not found: %', v_ref_text;
                    END IF;
                EXCEPTION WHEN others THEN
                    SELECT uuid INTO v_account_uuid
                    FROM accounting.accounts
                    WHERE code = v_ref_text;
                    IF v_account_uuid IS NULL THEN
                        RAISE EXCEPTION 'Account not found: %', v_ref_text;
                    END IF;
                END;
            END;
        ELSIF jsonb_typeof(v_line.account_ref) = 'number' THEN
            SELECT uuid INTO v_account_uuid
            FROM accounting.accounts
            WHERE serial_id = (v_line.account_ref)::BIGINT;
            IF v_account_uuid IS NULL THEN
                RAISE EXCEPTION 'Account not found for serial_id: %', v_line.account_ref;
            END IF;
        ELSE
            RAISE EXCEPTION 'Invalid account_ref type';
        END IF;

        -- Insert entry
        INSERT INTO accounting.transaction_entries (
            transaction_uuid, account_uuid, line_no,
            amount, debit, credit, memo
        ) VALUES (
            v_txn_uuid, v_account_uuid, v_line_no,
            COALESCE(v_line.debit, 0) - COALESCE(v_line.credit, 0),
            COALESCE(v_line.debit, 0),
            COALESCE(v_line.credit, 0),
            v_line.memo
        );
    END LOOP;

    -- === Cash Flow Entries (Multiple rows supported) ===
    IF p_cash_flow_section IS NOT NULL AND p_cash_flow_activity IS NOT NULL THEN
        INSERT INTO accounting.cash_flow_entries (
            transaction_uuid,
            transaction_entry_uuid,
            activity_section,
            activity_type,
            direction,
            amount,
            description
        )
        SELECT
            v_txn_uuid,
            te.uuid,
            p_cash_flow_section,
            p_cash_flow_activity,
            CASE WHEN (te.debit - te.credit) > 0 THEN 'inflow' ELSE 'outflow' END,
            ABS(te.debit - te.credit),
            COALESCE(p_cash_flow_description, p_description)
        FROM accounting.transaction_entries te
        JOIN accounting.accounts a ON a.uuid = te.account_uuid
        WHERE te.transaction_uuid = v_txn_uuid
          AND a.cash_flow_category = 'cash'
          AND (te.debit - te.credit) <> 0;
    END IF;

    RETURN v_txn_serial_id;
END;
$$;
=============================================================================================================
-- Backfill cash_flow_entries from existing data
INSERT INTO accounting.cash_flow_entries (
    transaction_uuid,
    transaction_entry_uuid,
    activity_section,
    activity_type,
    direction,
    amount,
    description
)
SELECT
    t.uuid,
    te.uuid,
    -- You can improve classification logic here based on module / accounts
    COALESCE(t.module_mapping.section, 'operating') AS activity_section,
    COALESCE(t.module_mapping.activity, 'other_cash_movements') AS activity_type,
    CASE WHEN (te.debit - te.credit) > 0 THEN 'inflow' ELSE 'outflow' END,
    ABS(te.debit - te.credit),
    COALESCE(t.description, 'Cash movement - ' || t.module)
FROM accounting.transactions t
JOIN accounting.transaction_entries te ON te.transaction_uuid = t.uuid
JOIN accounting.accounts a ON a.uuid = te.account_uuid
CROSS JOIN LATERAL (
    VALUES (
        CASE
            WHEN t.module IN ('sales', 'receipt', 'customer_payment') THEN ('operating', 'customer_receipts')
            WHEN t.module IN ('payment', 'supplier_payment', 'expense') THEN ('operating', 'supplier_payments')
            WHEN t.module = 'payroll' THEN ('operating', 'payroll_payments')
            WHEN t.module IN ('asset_purchase', 'fixed_asset') THEN ('investing', 'fixed_asset_purchase')
            WHEN t.module IN ('loan', 'loan_disbursement') THEN ('financing', 'loan_proceeds')
            WHEN t.module IN ('loan_repayment') THEN ('financing', 'loan_repayments')
            ELSE ('operating', 'other_cash_movements')
        END
    )
) AS t(module_mapping(section, activity))
WHERE a.cash_flow_category = 'cash'
  AND (te.debit - te.credit) <> 0
  AND NOT EXISTS (
        SELECT 1
        FROM accounting.cash_flow_entries cfe
        WHERE cfe.transaction_uuid = t.uuid
          AND cfe.transaction_entry_uuid = te.uuid
  );

-- migration: 20260521_001_populate_cash_flow_mapping.sql
=============================================================================================================

CREATE TABLE IF NOT EXISTS accounting.cash_flow_mapping (
    module VARCHAR(64) PRIMARY KEY,
    activity_section VARCHAR(32) NOT NULL CHECK (activity_section IN ('operating', 'investing', 'financing')),
    activity_type VARCHAR(64) NOT NULL,
    default_description_template TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

WITH cash_entries AS (
    SELECT
        te.transaction_uuid,
        SUM(te.debit - te.credit) AS cash_delta   -- Total cash movement in this transaction
    FROM accounting.transaction_entries te
    JOIN accounting.accounts acc ON acc.uuid = te.account_uuid
    WHERE acc.cash_flow_category = 'cash'
      AND te.created_at >= $1
      AND te.created_at < ($2 + INTERVAL '1 day')
    GROUP BY te.transaction_uuid
)
SELECT
    acc.code,
    acc.name,
    acc.parent_code,
    acc.cash_flow_category AS category,
    SUM(te2.debit - te2.credit) AS balance
FROM cash_entries ce
JOIN accounting.transaction_entries te2
    ON te2.transaction_uuid = ce.transaction_uuid
JOIN accounting.accounts acc
    ON acc.uuid = te2.account_uuid
WHERE acc.cash_flow_category IS NOT NULL
  AND acc.cash_flow_category != 'cash'
  AND acc.cash_flow_category != 'non-cash'
GROUP BY acc.code, acc.name, acc.parent_code, acc.cash_flow_category
ORDER BY acc.code;

=============================================================================================================
INSERT INTO accounting.cash_flow_mapping
    (module, activity_section, activity_type, default_description_template)
VALUES
    -- === Operating Activities ===
    ('sales', 'operating', 'customer_receipts', 'Cash sale'),
    ('cash_sale', 'operating', 'customer_receipts', 'Cash sale'),
    ('customer_payment', 'operating', 'customer_receipts', 'Receipt from customer'),
    ('receipt', 'operating', 'customer_receipts', 'Customer receipt'),

    ('purchases', 'operating', 'supplier_payments', 'Supplier payment'),
    ('supplier_payment', 'operating', 'supplier_payments', 'Payment to supplier'),
    ('payment', 'operating', 'supplier_payments', 'Cash payment'),
    ('expense', 'operating', 'supplier_payments', 'Expense payment'),

    ('payroll', 'operating', 'payroll_payments', 'Payroll payment'),
    ('salary', 'operating', 'payroll_payments', 'Salary payment'),

    ('rent', 'operating', 'rent_paid', 'Rent payment'),
    ('utility', 'operating', 'utilities_paid', 'Utility payment'),
    ('utilities', 'operating', 'utilities_paid', 'Utility payment'),

    ('tax', 'operating', 'tax_payments', 'Tax payment'),
    ('vat_payment', 'operating', 'vat_paid', 'VAT payment'),
    ('vat_refund', 'operating', 'vat_received', 'VAT refund received'),

    -- === Investing Activities ===
    ('asset_purchase', 'investing', 'fixed_asset_purchase', 'Fixed asset purchase'),
    ('fixed_asset', 'investing', 'fixed_asset_purchase', 'Fixed asset purchase'),
    ('equipment_purchase', 'investing', 'fixed_asset_purchase', 'Equipment purchase'),
    ('vehicle_purchase', 'investing', 'fixed_asset_purchase', 'Vehicle purchase'),

    ('asset_sale', 'investing', 'fixed_asset_sale', 'Fixed asset sale'),
    ('investment_purchase', 'investing', 'investment_purchase', 'Investment purchase'),
    ('investment_sale', 'investing', 'investment_sale', 'Investment sale'),

    -- === Financing Activities ===
    ('loan_disbursement', 'financing', 'loan_proceeds', 'Loan received'),
    ('loan_received', 'financing', 'loan_proceeds', 'Loan received'),
    ('loan', 'financing', 'loan_proceeds', 'Loan proceeds'),

    ('loan_repayment', 'financing', 'loan_repayments', 'Loan repayment'),
    ('loan_payment', 'financing', 'loan_repayments', 'Loan repayment'),

    ('owner_capital', 'financing', 'capital_contributions', 'Owner capital contribution'),
    ('capital', 'financing', 'capital_contributions', 'Capital injection'),
    ('dividend', 'financing', 'dividends_paid', 'Dividend payment'),
    ('dividends', 'financing', 'dividends_paid', 'Dividend payment'),

    -- Fallback / Generic
    ('journal', 'operating', 'other_cash_movements', 'Journal cash adjustment'),
    ('adjustment', 'operating', 'other_cash_movements', 'Cash adjustment'),
    ('transfer', 'operating', 'other_cash_movements', 'Cash transfer')   -- you can make internal transfers non-cash later
ON CONFLICT (module) DO NOTHING;
