-- =======================================================================================
-- RECEIVABLES MODULE - FULL SCHEMA
-- Run this ONCE after `accounting` schema exists
-- =======================================================================================

-- Create schema
CREATE SCHEMA IF NOT EXISTS receivables;

-- ================================================================================
-- SEQUENCES
-- ================================================================================
CREATE SEQUENCE IF NOT EXISTS sales.customers_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS sales.turnover_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS sales.turnover_items_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS sales.payments_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS sales.payment_applications_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS sales.credit_notes_serial_id_seq;

-- ================================================================================
-- TABLE: customers
-- ================================================================================
CREATE TABLE IF NOT EXISTS sales.customers (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('sales.customers_serial_id_seq') NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    email text,
    phone text,
    billing_address text,
    credit_limit numeric(18, 2) DEFAULT 0,
    current_balance numeric(18, 2) DEFAULT 0,
    terms text DEFAULT 'Net 30',
    status text DEFAULT 'Active',
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT customers_pkey PRIMARY KEY (uuid),
    CONSTRAINT customers_serial_id_key UNIQUE (serial_id),
    CONSTRAINT customers_code_key UNIQUE (code)
);

-- ================================================================================
-- TABLE: invoices & cash sales
-- ================================================================================
CREATE TABLE IF NOT EXISTS sales.turnover (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('sales.turnover_serial_id_seq') NOT NULL,
    invoice_number text NOT NULL,
    customer_uuid uuid NOT NULL,
    issue_date date NOT NULL,
    due_date date NOT NULL,
    total_amount numeric(18, 2) DEFAULT 0,
    tax_amount numeric(18, 2) DEFAULT 0,
    balance_due numeric(18, 2) DEFAULT 0,
    settlement_type text DEFAULT 'credit' CHECK(settlement_type IN ('credit', 'cash')),
    status text DEFAULT DEFAULT 'unpaid' CHECK(settlement_type IN ('unpaid', 'paid', 'partial', 'cancelled')),
    posted boolean DEFAULT false,
    gl_transaction_uuid uuid,
    paid_at timestamptz,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT invoices_pkey PRIMARY KEY (uuid),
    CONSTRAINT invoices_serial_id_key UNIQUE (serial_id),
    CONSTRAINT invoices_invoice_number_key UNIQUE (invoice_number),
    CONSTRAINT invoices_customer_uuid_fkey
        FOREIGN KEY (customer_uuid) REFERENCES sales.customers(uuid) ON DELETE RESTRICT,
    CONSTRAINT invoices_gl_transaction_uuid_fkey
        FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid)
);

-- ================================================================================
-- TABLE: invoice_items
-- ================================================================================
CREATE TABLE IF NOT EXISTS sales.turnover_items (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('sales.turnover_items_serial_id_seq') NOT NULL,
    invoice_uuid uuid NOT NULL,
    stock_item_id bigint,
    description text,
    quantity numeric(12, 4) DEFAULT 1,
    unit_price numeric(18, 4) DEFAULT 0,
    tax_rate numeric(5, 2) DEFAULT 0,
    total numeric(18, 2) NOT NULL,
    CONSTRAINT invoice_items_pkey PRIMARY KEY (uuid),
    CONSTRAINT invoice_items_serial_id_key UNIQUE (serial_id),
    CONSTRAINT invoice_items_invoice_uuid_fkey
        FOREIGN KEY (invoice_uuid) REFERENCES sales.turnover(uuid) ON DELETE CASCADE
);

-- ================================================================================
-- TABLE: payments
-- ================================================================================
CREATE TABLE IF NOT EXISTS sales.payments (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('sales.payments_serial_id_seq') NOT NULL,
    payment_number text NOT NULL,
    customer_uuid uuid NOT NULL,
    payment_date date NOT NULL,
    method text NOT NULL CHECK (method IN ('Cash', 'Bank Transfer', 'Mobile Money', 'Card', 'Cheque', 'Digital Payment')),
    reference text,
    amount numeric(18, 2) NOT NULL,
    applied_amount numeric(18, 2) DEFAULT 0,
    unapplied_amount numeric(18, 2) GENERATED ALWAYS AS (amount - applied_amount) STORED,
    gl_transaction_uuid uuid,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT payments_pkey PRIMARY KEY (uuid),
    CONSTRAINT payments_serial_id_key UNIQUE (serial_id),
    CONSTRAINT payments_payment_number_key UNIQUE (payment_number),
    CONSTRAINT payments_customer_uuid_fkey
        FOREIGN KEY (customer_uuid) REFERENCES sales.customers(uuid),
    CONSTRAINT payments_gl_transaction_uuid_fkey
        FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid)
);

-- ================================================================================
-- TABLE: payment_applications
-- ================================================================================
CREATE TABLE IF NOT EXISTS sales.payment_applications (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('sales.payment_applications_serial_id_seq') NOT NULL,
    payment_uuid uuid NOT NULL,
    invoice_uuid uuid NOT NULL,
    applied_amount numeric(18, 2) NOT NULL,
    applied_date date DEFAULT now() NOT NULL,
    CONSTRAINT payment_applications_pkey PRIMARY KEY (uuid),
    CONSTRAINT payment_applications_serial_id_key UNIQUE (serial_id),
    CONSTRAINT payment_applications_payment_uuid_invoice_uuid_key UNIQUE (payment_uuid, invoice_uuid),
    CONSTRAINT payment_applications_payment_uuid_fkey
        FOREIGN KEY (payment_uuid) REFERENCES sales.payments(uuid) ON DELETE CASCADE,
    CONSTRAINT payment_applications_invoice_uuid_fkey
        FOREIGN KEY (invoice_uuid) REFERENCES sales.turnover(uuid) ON DELETE CASCADE
);

-- ================================================================================
-- TABLE: credit_notes
-- ================================================================================
CREATE TABLE IF NOT EXISTS sales.credit_notes (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('sales.credit_notes_serial_id_seq') NOT NULL,
    credit_number text NOT NULL,
    customer_uuid uuid NOT NULL,
    reference_invoice_uuid uuid,
    issue_date date NOT NULL,
    amount numeric(18, 2) NOT NULL,
    reason text,
    gl_transaction_uuid uuid,
    status text DEFAULT 'Open',
    created_at timestamptz DEFAULT now(),
    CONSTRAINT credit_notes_pkey PRIMARY KEY (uuid),
    CONSTRAINT credit_notes_serial_id_key UNIQUE (serial_id),
    CONSTRAINT credit_notes_credit_number_key UNIQUE (credit_number),
    CONSTRAINT credit_notes_customer_uuid_fkey
        FOREIGN KEY (customer_uuid) REFERENCES sales.customers(uuid),
    CONSTRAINT credit_notes_reference_invoice_uuid_fkey
        FOREIGN KEY (reference_invoice_uuid) REFERENCES sales.turnover(uuid),
    CONSTRAINT credit_notes_gl_transaction_uuid_fkey
        FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid)
);

-- ================================================================================
-- VIEWS
-- ================================================================================
CREATE OR REPLACE VIEW sales.customer_aging AS
SELECT
    c.serial_id AS customer_serial_id,
    c.code AS customer_code,
    c.name AS customer_name,
    SUM(CASE WHEN i.due_date >= CURRENT_DATE THEN i.balance_due ELSE 0 END) AS current,
    SUM(CASE WHEN i.due_date < CURRENT_DATE AND i.due_date >= CURRENT_DATE - INTERVAL '30 days' THEN i.balance_due ELSE 0 END) AS days_1_30,
    SUM(CASE WHEN i.due_date >= CURRENT_DATE - INTERVAL '60 days' AND i.due_date < CURRENT_DATE - INTERVAL '30 days' THEN i.balance_due ELSE 0 END) AS days_31_60,
    SUM(CASE WHEN i.due_date >= CURRENT_DATE - INTERVAL '90 days' AND i.due_date < CURRENT_DATE - INTERVAL '60 days' THEN i.balance_due ELSE 0 END) AS days_61_90,
    SUM(CASE WHEN i.due_date < CURRENT_DATE - INTERVAL '90 days' THEN i.balance_due ELSE 0 END) AS days_over_90,
    SUM(i.balance_due) AS total_outstanding
FROM sales.customers c
LEFT JOIN sales.turnover i ON i.customer_uuid = c.uuid AND i.status NOT IN ('Paid', 'Cancelled')
GROUP BY c.serial_id, c.code, c.name
ORDER BY total_outstanding DESC;

CREATE OR REPLACE VIEW sales.customer_statement AS
SELECT DISTINCT ON (c.serial_id, COALESCE(i.serial_id, 0))
    c.serial_id AS customer_serial_id,
    c.code AS customer_code,
    c.name AS customer_name,
    i.serial_id AS invoice_serial_id,
    i.invoice_number,
    i.issue_date,
    i.due_date,
    i.total_amount,
    i.balance_due,
    i.status AS invoice_status,
    COALESCE(p.payment_number, '') AS last_payment_number,
    COALESCE(p.amount, 0) AS last_payment_amount,
    p.payment_date AS last_payment_date
FROM sales.customers c
LEFT JOIN sales.turnover i ON i.customer_uuid = c.uuid
LEFT JOIN sales.payments p ON p.customer_uuid = c.uuid
ORDER BY c.serial_id, COALESCE(i.serial_id, 0), p.payment_date DESC NULLS LAST;

-- ================================================================================
-- FUNCTIONS
-- ================================================================================
-- Customer post sale function (credit or cash)
CREATE OR REPLACE FUNCTION sales.post_turnover(
    p_turnover_serial_id   bigint,
    p_user                 uuid,
    p_output_vat_code      text,
    p_receivables_code     text DEFAULT NULL,
    p_revenue_code         text DEFAULT NULL, -- fallback revenue account code
    p_cash_account_code    text DEFAULT NULL
) RETURNS sales.turnover
LANGUAGE plpgsql
AS $$
DECLARE
    v_turnover        sales.turnover%ROWTYPE;
    v_txn_serial_id   bigint;
    v_txn_uuid        uuid;
    v_main_account    uuid;
    v_output_vat_uuid uuid;
BEGIN
    SELECT * INTO v_turnover
    FROM sales.turnover
    WHERE serial_id = p_turnover_serial_id
    FOR UPDATE;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Turnover % not found', p_turnover_serial_id;
    END IF;

    IF v_turnover.posted THEN RETURN
        RAISE EXCEPTION 'Turnover % already posted', p_turnover_serial_id;
    END IF;

    -- Resolve main account
    IF v_turnover.settlement_type = 'credit' THEN
        SELECT uuid INTO v_main_account
        FROM accounting.accounts WHERE code = p_receivables_code;
        IF v_main_account IS NULL THEN
            RAISE EXCEPTION 'Receivables account not found: %', p_receivables_code;
        END IF;
    ELSE
        SELECT uuid INTO v_main_account
        FROM accounting.accounts WHERE code = p_cash_account_code;
        IF v_main_account IS NULL THEN
            RAISE EXCEPTION 'Cash account not found: %', p_cash_account_code;
        END IF;
    END IF;

    -- Output VAT
    SELECT uuid INTO v_output_vat_uuid
    FROM accounting.accounts WHERE code = p_output_vat_code;
    IF v_output_vat_uuid IS NULL THEN
        RAISE EXCEPTION 'Output VAT account not found: %', p_output_vat_code;
    END IF;

    -- Create GL transaction
    v_txn_serial_id := accounting.post_transaction(
        v_turnover.invoice_number,
        CASE WHEN v_turnover.settlement_type = 'cash' THEN 'Cash Sale' ELSE 'Credit Sale' END,
        p_user, 'turnover', v_turnover.issue_date, '[]'::jsonb
    );

    SELECT uuid INTO v_txn_uuid
    FROM accounting.transactions WHERE serial_id = v_txn_serial_id;

    -- Journal lines: Revenue + main + VAT only
    WITH revenue_grouped AS (
        SELECT
            a.uuid AS account_uuid,
            SUM(ti.quantity*ti.unit_price)::numeric(18,2) AS net_amount
        FROM sales.turnover_items ti
        JOIN inventory.items i ON i.serial_id = ti.stock_item_id
        JOIN accounting.accounts a
            ON a.code = COALESCE(i.income_account, p_revenue_code)
        WHERE ti.invoice_uuid = v_turnover.uuid
        GROUP BY a.uuid
    ),
    all_lines AS (
        -- Debit: gross amount
        SELECT v_main_account AS account_uuid,
               (v_turnover.total_amount + v_turnover.tax_amount)::numeric(18,2) AS amount,
               'debit' AS side,
               CASE WHEN v_turnover.settlement_type = 'cash' THEN 'Cash received – sale' ELSE 'Accounts receivable – sale' END AS memo

        UNION ALL
        -- Credit: revenue (net, grouped if multiple income accounts)
        SELECT account_uuid, net_amount, 'credit', 'Sales revenue'
        FROM revenue_grouped

        UNION ALL
        -- Credit: output VAT
        SELECT v_output_vat_uuid, v_turnover.tax_amount::numeric(18,2), 'credit', 'Output VAT collected'
    )
    INSERT INTO accounting.transaction_entries (
        transaction_uuid, account_uuid, line_no, amount, debit, credit, memo
    )
    SELECT
        v_txn_uuid,
        account_uuid,
        ROW_NUMBER() OVER (
            ORDER BY CASE side WHEN 'debit' THEN 0 ELSE 1 END, memo, account_uuid
        )::integer AS line_no,
        CASE WHEN side = 'debit' THEN amount ELSE -amount END,
        CASE WHEN side = 'debit' THEN amount ELSE 0 END,
        CASE WHEN side = 'credit' THEN amount ELSE 0 END,
        memo
    FROM all_lines;

    -- Post COGS & inventory reduction (per item, already correct)
    PERFORM inventory.post_sale(
        ti.stock_item_id,
        i.warehouse_serial,
        ti.quantity,
        'turnover',
        p_turnover_serial_id,
        p_user,
        v_txn_uuid
    )
    FROM sales.turnover_items ti
    JOIN inventory.items i ON i.serial_id = ti.stock_item_id
    WHERE ti.invoice_uuid = v_turnover.uuid;

    -- Mark posted & update customer balance
    UPDATE sales.turnover
    SET posted = TRUE,
        gl_transaction_uuid = v_txn_uuid,
        status = CASE WHEN settlement_type = 'cash' THEN 'paid' ELSE status END,
        balance_due = CASE WHEN settlement_type = 'cash' THEN 0 ELSE (total_amount + tax_amount) END,
        paid_at = CASE WHEN settlement_type = 'cash' THEN now() ELSE NULL END,
        updated_at = now()
    WHERE uuid = v_turnover.uuid;

    IF v_turnover.settlement_type = 'credit' THEN
        UPDATE sales.customers
        SET current_balance = current_balance + v_turnover.total_amount + v_turnover.tax_amount,
            updated_at = now()
        WHERE uuid = v_turnover.customer_uuid;
    END IF;

    RETURN v_turnover;
END;
$$;

-- Invoice payment function (payment for credit sale)
CREATE OR REPLACE FUNCTION sales.post_payment(
    p_payment_serial_id bigint,
    p_user uuid,
    p_cash_account_code TEXT,
    p_receivables_code TEXT
)
RETURNS void LANGUAGE plpgsql AS $$
DECLARE
    v_payment sales.payments%ROWTYPE;
    v_txn_serial_id bigint;
    v_txn_uuid uuid;
    v_lines jsonb;
BEGIN
    SELECT * INTO v_payment FROM sales.payments WHERE serial_id = p_payment_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Payment serial_id % not found', p_payment_serial_id; END IF;
    v_lines := jsonb_build_array(
        jsonb_build_object('account_ref', p_cash_account_code, 'debit', v_payment.amount, 'credit', 0, 'memo', v_payment.payment_number),
        jsonb_build_object('account_ref', p_receivables_code, 'debit', 0, 'credit', v_payment.amount, 'memo', v_payment.payment_number)
    );
    v_txn_serial_id := accounting.post_transaction(
        v_payment.payment_number, 'Payment Receipt', p_user, 'payment', v_payment.payment_date, v_lines
    );

    -- Get gl uuid for posted transaction
    SELECT uuid INTO v_txn_uuid FROM accounting.transactions WHERE serial_id = v_txn_serial_id;

    -- Panick if gl uuid does not exist
    IF v_txn_uuid IS NULL THEN
        RAISE EXCEPTION 'Failed to retrieve GL transaction UUID';
    END IF;

    UPDATE sales.payments
        SET gl_transaction_uuid = v_txn_uuid, updated_at = now()
        WHERE serial_id = p_payment_serial_id;

    UPDATE sales.customers
        SET current_balance = current_balance - v_payment.amount, updated_at = now()
        WHERE uuid = v_payment.customer_uuid;
END;
$$;

-- Invoice payment application function (payment for credit sale)
CREATE OR REPLACE FUNCTION sales.apply_payment(
    p_payment_serial_id bigint,
    p_invoice_serial_id bigint,
    p_amount numeric
)
RETURNS void LANGUAGE plpgsql AS $$
DECLARE
    v_payment_uuid uuid;
    v_invoice_uuid uuid;
    v_balance_due numeric;
BEGIN
    SELECT uuid INTO v_payment_uuid FROM sales.payments WHERE serial_id = p_payment_serial_id;
    SELECT uuid, balance_due INTO v_invoice_uuid, v_balance_due FROM sales.turnover WHERE serial_id = p_invoice_serial_id;
    IF v_payment_uuid IS NULL THEN RAISE EXCEPTION 'Payment not found'; END IF;
    IF v_invoice_uuid IS NULL THEN RAISE EXCEPTION 'Invoice not found'; END IF;
    IF p_amount <= 0 THEN RAISE EXCEPTION 'Amount must be positive'; END IF;
    INSERT INTO sales.payment_applications (payment_uuid, invoice_uuid, applied_amount, applied_date)
    VALUES (v_payment_uuid, v_invoice_uuid, p_amount, CURRENT_DATE);
    UPDATE sales.turnover
    SET balance_due = balance_due - p_amount,
        status = CASE
            WHEN balance_due - p_amount <= 0 THEN 'paid'
            WHEN balance_due - p_amount < total_amount THEN 'partial'
            ELSE 'unpaid'
        END,
        updated_at = now()
    WHERE uuid = v_invoice_uuid;
    UPDATE sales.payments
    SET applied_amount = applied_amount + p_amount,
        updated_at = now()
    WHERE uuid = v_payment_uuid;
END;
$$;

-- ================================================================================
-- INDEXES
-- ================================================================================
CREATE INDEX IF NOT EXISTS idx_invoices_customer ON sales.turnover(customer_uuid);
CREATE INDEX IF NOT EXISTS idx_invoices_due_date ON sales.turnover(due_date);
CREATE INDEX IF NOT EXISTS idx_invoices_status ON sales.turnover(status);
CREATE INDEX IF NOT EXISTS idx_invoices_balance ON sales.turnover(balance_due) WHERE balance_due > 0;

CREATE INDEX IF NOT EXISTS idx_payments_customer ON sales.payments(customer_uuid);
CREATE INDEX IF NOT EXISTS idx_payments_date ON sales.payments(payment_date DESC);

CREATE INDEX IF NOT EXISTS idx_customers_code ON sales.customers(code);
