-- ========================================
-- PAYROLL MODULE - FULL SCHEMA (GLOBAL-READY)
-- Run this ONCE after `accounting` and `hr` schemas exist
-- ========================================

-- Enable UUID extension
-- CREATE EXTENSION IF NOT EXISTS pg_uuidv7;

-- Create schema
CREATE SCHEMA IF NOT EXISTS payroll;

-- ========================================
-- SEQUENCES
-- ========================================
CREATE SEQUENCE IF NOT EXISTS payroll.payruns_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS payroll.payslips_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS payroll.payslip_items_serial_id_seq;

-- ========================================
-- TABLE: payruns
-- ========================================
CREATE TABLE IF NOT EXISTS payroll.payruns (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('payroll.payruns_serial_id_seq') NOT NULL,
    pay_period_start date NOT NULL,
    pay_period_end date NOT NULL,
    payment_date date NOT NULL,
    status varchar(20) DEFAULT 'Pending' CHECK (status IN ('Pending', 'Processed', 'Posted')),
    gl_transaction_uuid uuid,
    notes text,
	processed_at timestamptz,
	posted_at timestamptz,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT payruns_pkey PRIMARY KEY (uuid),
    CONSTRAINT payruns_serial_id_key UNIQUE (serial_id)
);

-- ========================================
-- TABLE: payslips
-- ========================================
CREATE TABLE IF NOT EXISTS payroll.payslips (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('payroll.payslips_serial_id_seq') NOT NULL,
    payrun_uuid uuid NOT NULL,
    employee_uuid uuid NOT NULL,
    gross_pay numeric(14, 2) NOT NULL,
    tax_deducted numeric(14, 2) DEFAULT 0,
    social_security numeric(14, 2) DEFAULT 0,
	employer_social_security numeric(14,2) DEFAULT 0,
    other_deductions numeric(14, 2) DEFAULT 0,
	basic_pay numeric(14,2),
    net_pay numeric(14, 2) GENERATED ALWAYS AS (
        ((gross_pay - tax_deducted) - social_security) - other_deductions
    ) STORED,
    payment_method varchar(30) DEFAULT 'Bank Transfer',
    bank_account varchar(50),
    status varchar(20) DEFAULT 'Pending' CHECK (status IN ('Pending', 'Paid')),
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT payslips_pkey PRIMARY KEY (uuid),
    CONSTRAINT payslips_serial_id_key UNIQUE (serial_id),
    CONSTRAINT payslips_payrun_uuid_fkey
        FOREIGN KEY (payrun_uuid) REFERENCES payroll.payruns(uuid) ON DELETE CASCADE,
    CONSTRAINT payslips_employee_uuid_fkey
        FOREIGN KEY (employee_uuid) REFERENCES hr.employees(uuid)
);

-- ========================================
-- TABLE: payslip_items
-- ========================================
CREATE TABLE IF NOT EXISTS payroll.payslip_items (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('payroll.payslip_items_serial_id_seq') NOT NULL,
    payslip_uuid uuid NOT NULL,
    item_type varchar(20) CHECK (item_type IN ('Allowance', 'Deduction')),
    description varchar(100),
	component_uuid uuid REFERENCES payroll.pay_components(uuid),
    amount numeric(14, 2) NOT NULL,
    created_at timestamptz DEFAULT now(),
    CONSTRAINT payslip_items_pkey PRIMARY KEY (uuid),
    CONSTRAINT payslip_items_serial_id_key UNIQUE (serial_id),
    CONSTRAINT payslip_items_payslip_uuid_fkey
        FOREIGN KEY (payslip_uuid) REFERENCES payroll.payslips(uuid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS payroll.pay_components (
  uuid uuid PRIMARY KEY DEFAULT uuidv7(),
  code varchar(30) NOT NULL UNIQUE,
  name varchar(100) NOT NULL,
  component_kind varchar(20) NOT NULL
    CHECK (component_kind IN ('EARNING','DEDUCTION','EMPLOYER_COST')),
  calc_method varchar(30) NOT NULL
    CHECK (calc_method IN ('FIXED','PERCENT_OF_BASIC','PERCENT_OF_GROSS','MANUAL','EXTERNAL')),
  default_rate numeric(12,6),
  is_taxable boolean NOT NULL DEFAULT true,
  statutory_role varchar(40),
  is_active boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now()
);

INSERT INTO payroll.pay_components (code, name, component_kind, calc_method, statutory_role) VALUES
  ('BASIC', 		'Basic salary', 				'EARNING', 		 'FIXED', NULL),
  ('LEAVE_ENCASH', 	'Leave encashment', 			'EARNING', 		 'MANUAL', NULL),
  ('TAX_WH', 		'Income tax withheld', 			'DEDUCTION', 	 'EXTERNAL', 'income_tax_employee'),
  ('SS_EE', 		'Social security (employee)', 	'DEDUCTION', 	 'EXTERNAL', 'social_security_employee'),
  ('SS_ER', 		'Social security (employer)', 	'EMPLOYER_COST', 'EXTERNAL', 'social_security_employer')
ON CONFLICT (code) DO NOTHING;

-- ========================================
-- VIEWS
-- ========================================
CREATE OR REPLACE VIEW payroll.payrun_summary AS
SELECT
    pr.serial_id AS payrun_id,
    pr.pay_period_start,
    pr.pay_period_end,
    pr.payment_date,
    pr.status,
    COUNT(ps.uuid) AS employee_count,
    SUM(ps.gross_pay) AS total_gross,
    SUM(ps.tax_deducted) AS total_tax,
    SUM(ps.social_security) AS total_social_security,
    SUM(ps.net_pay) AS total_net
FROM payroll.payruns pr
LEFT JOIN payroll.payslips ps ON ps.payrun_uuid = pr.uuid
GROUP BY pr.uuid, pr.serial_id;

CREATE OR REPLACE VIEW hr.v_employee_payroll_snapshot AS
SELECT
  e.uuid AS employee_uuid,
  e.status,
  e.hire_date,
  lc.termination_date,
  lc.last_working_date,
  c.basic_salary,
  COALESCE(c.pay_frequency, e.pay_frequency) AS pay_frequency,
  pp.bank_account_number,
  pp.bank_name,
  pp.payment_method,
  sp.tax_id,
  sp.social_security_number
FROM hr.employees e
LEFT JOIN hr.employee_lifecycle lc ON lc.employee_uuid = e.uuid
LEFT JOIN hr.employment_contracts c
  ON c.employee_uuid = e.uuid AND c.status = 'Active'
LEFT JOIN hr.employee_payment_profiles pp ON pp.employee_uuid = e.uuid
LEFT JOIN hr.employee_statutory_profiles sp ON sp.employee_uuid = e.uuid;

-- ========================================
-- FUNCTION: post_payrun
-- ========================================
CREATE OR REPLACE FUNCTION payroll.post_payrun(
    p_payrun_serial_id bigint,
    p_payrun_labor_expense_id uuid,
    p_payrun_tax_id uuid,
    p_payrun_social_sec_id uuid,
    p_payrun_cash_id uuid,
    p_payrun_user_uuid uuid,
    p_payrun_payroll_payable_id uuid
)
RETURNS void LANGUAGE plpgsql AS $$
DECLARE
    v_payrun payroll.payruns%ROWTYPE;
    v_total_gross numeric(14, 2) := 0;
    v_total_tax numeric(14, 2) := 0;
    v_total_social_security numeric(14, 2) := 0;
    v_total_net numeric(14, 2) := 0;
    v_txn_serial_id bigint;
    v_txn_uuid uuid;
    v_lines jsonb;
BEGIN
    -- Fetch payrun
    SELECT * INTO v_payrun FROM payroll.payruns WHERE serial_id = p_payrun_serial_id;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Payrun with serial_id % not found', p_payrun_serial_id;
    END IF;

    IF v_payrun.status = 'Posted' THEN
        RAISE NOTICE 'Payrun % already posted', v_payrun.serial_id;
        RETURN;
    END IF;

    IF v_payrun.status != 'Processed' THEN
        RAISE EXCEPTION 'Payrun must be Processed before posting. Current status: %', v_payrun.status;
    END IF;

    -- Aggregate totals
    SELECT
        COALESCE(SUM(gross_pay), 0),
        COALESCE(SUM(tax_deducted), 0),
        COALESCE(SUM(social_security), 0),
        COALESCE(SUM(net_pay), 0)
    INTO v_total_gross, v_total_tax, v_total_social_security, v_total_net
    FROM payroll.payslips
    WHERE payrun_uuid = v_payrun.uuid;

    IF v_total_net <= 0 THEN
        RAISE EXCEPTION 'Net pay total is zero or negative. Cannot post.';
    END IF;

    -- Build GL lines
    -- v_lines := jsonb_build_array(
    --     jsonb_build_object('account_ref', p_payrun_labor_expense_id, 'debit', v_total_gross, 'credit', 0,
    --         'memo', format('Payroll Gross - Payrun %s', v_payrun.serial_id)),
    --     jsonb_build_object('account_ref', p_payrun_tax_id, 'debit', 0, 'credit', v_total_tax,
    --         'memo', 'Income Tax Withholding'),
    --     jsonb_build_object('account_ref', p_payrun_social_sec_id, 'debit', 0, 'credit', v_total_social_security,
    --         'memo', 'Social Security Contribution'),
    --     jsonb_build_object('account_ref', p_payrun_cash_id, 'debit', 0, 'credit', v_total_net,
    --         'memo', format('Payroll Net Pay - Payrun %s', v_payrun.serial_id))
    -- );
    v_lines := jsonb_build_array(
        -- 1. Recognize gross payroll
        jsonb_build_object(
            'account_ref', p_payrun_labor_expense_id,
            'debit', v_total_gross,
            'credit', 0,
            'memo', format('Payroll Gross - Payrun %s', v_payrun.serial_id)
        ),

        -- 2. Create payroll payable (control)
        jsonb_build_object(
            'account_ref', p_payrun_payroll_payable_id,
            'debit', 0,
            'credit', v_total_gross,
            'memo', 'Payroll Payable'
        ),

        -- 3. Settle payroll payable, typically means payroll liability payment
        jsonb_build_object(
            'account_ref', p_payrun_payroll_payable_id,
            'debit', v_total_gross,
            'credit', 0,
            'memo', 'Payroll Payable Settlement'
        ),

		-- 4. Settle witholding which should be liability
        jsonb_build_object(
            'account_ref', p_payrun_tax_id,
            'debit', 0,
            'credit', v_total_tax,
            'memo', 'Income Tax Withholding'
        ),

		-- 5. Settle social security witholding, another liability
        jsonb_build_object(
            'account_ref', p_payrun_social_sec_id,
            'debit', 0,
            'credit', v_total_social_security,
            'memo', 'Social Security Withholding'
        ),

		-- 6. Recognize cash/bank payment for hr benefits & renumeration
        jsonb_build_object(
            'account_ref', p_payrun_cash_id,
            'debit', 0,
            'credit', v_total_net,
            'memo', format('Payroll Net Pay - Payrun %s', v_payrun.serial_id)
        )
    );

    -- Post to GL
    v_txn_serial_id := accounting.post_transaction(
        format('Payroll - %s to %s', v_payrun.pay_period_start, v_payrun.pay_period_end),
        'Payroll Posting',
        p_payrun_user_uuid,
        'payroll',
        v_payrun.payment_date,
        v_lines
    );

    SELECT uuid INTO v_txn_uuid FROM accounting.transactions WHERE serial_id = v_txn_serial_id;
    IF v_txn_uuid IS NULL THEN
        RAISE EXCEPTION 'Failed to retrieve GL transaction UUID';
    END IF;

    -- Update payrun
    UPDATE payroll.payruns
    SET status = 'Posted',
        gl_transaction_uuid = v_txn_uuid,
        updated_at = now()
    WHERE uuid = v_payrun.uuid;

    -- Mark payslips as Paid
    UPDATE payroll.payslips
    SET status = 'Paid', updated_at = now()
    WHERE payrun_uuid = v_payrun.uuid AND status = 'Pending';
END;
$$;

-- ========================================
-- INDEXES
-- ========================================
CREATE INDEX IF NOT EXISTS idx_payruns_period ON payroll.payruns(pay_period_start, pay_period_end);
CREATE INDEX IF NOT EXISTS idx_payruns_payment_date ON payroll.payruns(payment_date);
CREATE INDEX IF NOT EXISTS idx_payruns_status ON payroll.payruns(status);

CREATE INDEX IF NOT EXISTS idx_payslips_payrun ON payroll.payslips(payrun_uuid);
CREATE INDEX IF NOT EXISTS idx_payslips_employee ON payroll.payslips(employee_uuid);
CREATE INDEX IF NOT EXISTS idx_payslips_status ON payroll.payslips(status);

CREATE INDEX IF NOT EXISTS idx_payslip_items_payslip ON payroll.payslip_items(payslip_uuid);
CREATE INDEX IF NOT EXISTS idx_payslip_items_type ON payroll.payslip_items(item_type);
