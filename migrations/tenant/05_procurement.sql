-- ===============================================
-- PROCUREMENT MODULE - FULL SCHEMA
-- Run this ONCE after `accounting` schema exists
-- ===============================================

-- Create schema
CREATE SCHEMA IF NOT EXISTS procurement;

-- ========================================
-- SEQUENCES
-- ========================================
CREATE SEQUENCE IF NOT EXISTS procurement.vendors_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS procurement.purchases_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS procurement.purchase_items_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS procurement.payments_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS procurement.payment_applications_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS procurement.credit_notes_serial_id_seq;

-- ========================================
-- TABLE: vendors
-- ========================================
CREATE TABLE IF NOT EXISTS procurement.vendors (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('procurement.vendors_serial_id_seq') NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    contact_person text,
    email text,
    phone text,
    address text,
    tax_id text,                    -- TIN for URA
    payment_terms text DEFAULT 'Net 30',
    credit_limit numeric(18, 2) DEFAULT 0,
    current_balance numeric(18, 2) DEFAULT 0,
    status text DEFAULT 'Active',
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT vendors_pkey PRIMARY KEY (uuid),
    CONSTRAINT vendors_serial_id_key UNIQUE (serial_id),
    CONSTRAINT vendors_code_key UNIQUE (code)
);

-- ========================================
-- TABLE: purchases
-- ========================================
CREATE TABLE IF NOT EXISTS procurement.purchases (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('procurement.purchases_serial_id_seq') NOT NULL,
    bill_number text NOT NULL,
    vendor_uuid uuid NOT NULL,
    bill_date date NOT NULL,
    due_date date NOT NULL,
    reference text,
    total_amount numeric(18, 2) DEFAULT 0,
    tax_amount numeric(18, 2) DEFAULT 0,
    balance_due numeric(18, 2) DEFAULT 0,
    -- currency text DEFAULT 'UGX',
    settlement_type text DEFAULT 'credit' CHECK(settlement_type IN ('credit', 'cash')),
    payment_status text DEFAULT 'unpaid' CHECK(settlement_type IN ('unpaid', 'paid', 'partial', 'cancelled')),
    posted boolean DEFAULT false,
    gl_transaction_uuid uuid,
    paid_at timestamptz,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT purchases_pkey PRIMARY KEY (uuid),
    CONSTRAINT purchases_serial_id_key UNIQUE (serial_id),
    CONSTRAINT purchases_bill_number_key UNIQUE (bill_number),
    CONSTRAINT purchases_vendor_uuid_fkey
        FOREIGN KEY (vendor_uuid) REFERENCES procurement.vendors(uuid) ON DELETE RESTRICT,
    CONSTRAINT purchases_gl_transaction_uuid_fkey
        FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid)
);

-- ========================================
-- TABLE: purchase_items
-- ========================================
CREATE TABLE IF NOT EXISTS procurement.purchase_items (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('procurement.purchase_items_serial_id_seq') NOT NULL,
    bill_uuid uuid NOT NULL,
    stock_item_id bigint,
    description text,
    quantity numeric(12, 4) DEFAULT 1,
    unit_price numeric(18, 4) DEFAULT 0,
    tax_rate numeric(5, 2) DEFAULT 18,  -- 18% VAT default
    total numeric(18, 2) NOT NULL,
    CONSTRAINT purchase_items_pkey PRIMARY KEY (uuid),
    CONSTRAINT purchase_items_serial_id_key UNIQUE (serial_id),
    CONSTRAINT purchase_items_bill_uuid_fkey
        FOREIGN KEY (bill_uuid) REFERENCES procurement.purchases(uuid) ON DELETE CASCADE
);

-- ========================================
-- TABLE: payments
-- ========================================
CREATE TABLE IF NOT EXISTS procurement.payments (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('procurement.payments_serial_id_seq') NOT NULL,
    payment_number text NOT NULL,
    vendor_uuid uuid NOT NULL,
    payment_date date NOT NULL,
    method text NOT NULL CHECK (method IN ('Cash', 'Bank Transfer', 'Mobile Money', 'Cheque')),
    reference text,
    amount numeric(18, 2) NOT NULL,
    -- currency text DEFAULT 'UGX',
    applied_amount numeric(18, 2) DEFAULT 0,
    unapplied_amount numeric(18, 2) GENERATED ALWAYS AS (amount - applied_amount) STORED,
    gl_transaction_uuid uuid,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT payments_pkey PRIMARY KEY (uuid),
    CONSTRAINT payments_serial_id_key UNIQUE (serial_id),
    CONSTRAINT payments_payment_number_key UNIQUE (payment_number),
    CONSTRAINT payments_vendor_uuid_fkey
        FOREIGN KEY (vendor_uuid) REFERENCES procurement.vendors(uuid),
    CONSTRAINT payments_gl_transaction_uuid_fkey
        FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid)
);

-- ========================================
-- TABLE: payment_applications
-- ========================================
CREATE TABLE IF NOT EXISTS procurement.payment_applications (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('procurement.payment_applications_serial_id_seq') NOT NULL,
    payment_uuid uuid NOT NULL,
    bill_uuid uuid NOT NULL,
    applied_amount numeric(18, 2) NOT NULL,
    applied_date date DEFAULT now() NOT NULL,
    CONSTRAINT payment_applications_pkey PRIMARY KEY (uuid),
    CONSTRAINT payment_applications_serial_id_key UNIQUE (serial_id),
    CONSTRAINT payment_applications_payment_uuid_bill_uuid_key UNIQUE (payment_uuid, bill_uuid),
    CONSTRAINT payment_applications_payment_uuid_fkey
        FOREIGN KEY (payment_uuid) REFERENCES procurement.payments(uuid) ON DELETE CASCADE,
    CONSTRAINT payment_applications_bill_uuid_fkey
        FOREIGN KEY (bill_uuid) REFERENCES procurement.purchases(uuid) ON DELETE CASCADE
);

-- ========================================
-- TABLE: credit_notes
-- ========================================
CREATE TABLE IF NOT EXISTS procurement.credit_notes (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('procurement.credit_notes_serial_id_seq') NOT NULL,
    credit_number text NOT NULL,
    vendor_uuid uuid NOT NULL,
    reference_bill_uuid uuid,
    issue_date date NOT NULL,
    amount numeric(18, 2) NOT NULL,
    reason text,
    gl_transaction_uuid uuid,
    status text DEFAULT 'Open',
    created_at timestamptz DEFAULT now(),
    CONSTRAINT credit_notes_pkey PRIMARY KEY (uuid),
    CONSTRAINT credit_notes_serial_id_key UNIQUE (serial_id),
    CONSTRAINT credit_notes_credit_number_key UNIQUE (credit_number),
    CONSTRAINT credit_notes_vendor_uuid_fkey
        FOREIGN KEY (vendor_uuid) REFERENCES procurement.vendors(uuid),
    CONSTRAINT credit_notes_reference_bill_uuid_fkey
        FOREIGN KEY (reference_bill_uuid) REFERENCES procurement.purchases(uuid),
    CONSTRAINT credit_notes_gl_transaction_uuid_fkey
        FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid)
);

-- ========================================
-- VIEWS
-- ========================================
CREATE OR REPLACE VIEW procurement.vendor_aging AS
SELECT
    v.serial_id AS vendor_serial_id,
    v.code AS vendor_code,
    v.name AS vendor_name,
    SUM(CASE WHEN b.due_date >= CURRENT_DATE THEN b.balance_due ELSE 0 END) AS current,
    SUM(CASE WHEN b.due_date < CURRENT_DATE AND b.due_date >= CURRENT_DATE - INTERVAL '30 days' THEN b.balance_due ELSE 0 END) AS days_1_30,
    SUM(CASE WHEN b.due_date >= CURRENT_DATE - INTERVAL '60 days' AND b.due_date < CURRENT_DATE - INTERVAL '30 days' THEN b.balance_due ELSE 0 END) AS days_31_60,
    SUM(CASE WHEN b.due_date >= CURRENT_DATE - INTERVAL '90 days' AND b.due_date < CURRENT_DATE - INTERVAL '60 days' THEN b.balance_due ELSE 0 END) AS days_61_90,
    SUM(CASE WHEN b.due_date < CURRENT_DATE - INTERVAL '90 days' THEN b.balance_due ELSE 0 END) AS days_over_90,
    SUM(b.balance_due) AS total_outstanding
FROM procurement.vendors v
LEFT JOIN procurement.purchases b ON b.vendor_uuid = v.uuid AND b.payment_status NOT IN ('paid', 'cancelled')
GROUP BY v.serial_id, v.code, v.name
ORDER BY total_outstanding DESC;

CREATE OR REPLACE VIEW procurement.vendor_statement_clean AS
SELECT DISTINCT ON (v.serial_id, COALESCE(b.serial_id, 0))
    v.serial_id AS vendor_serial_id,
    v.code AS vendor_code,
    v.name AS vendor_name,
    b.serial_id AS bill_serial_id,
    b.bill_number,
    b.bill_date,
    b.due_date,
    b.total_amount,
    b.balance_due,
    b.payment_status AS bill_status,
    COALESCE(p.payment_number, '') AS last_payment_number,
    COALESCE(p.amount, 0) AS last_payment_amount,
    p.payment_date AS last_payment_date
FROM procurement.vendors v
LEFT JOIN procurement.purchases b ON b.vendor_uuid = v.uuid
LEFT JOIN procurement.payments p ON p.vendor_uuid = v.uuid
ORDER BY v.serial_id, COALESCE(b.serial_id, 0), p.payment_date DESC NULLS LAST;

-- ========================================
-- FUNCTIONS
-- ========================================
-- Payables post bill function (credit purchase) V2.0
CREATE OR REPLACE FUNCTION procurement.post_purchase(
    p_bill_serial_id bigint,
    p_user uuid,
    p_vat_tax_code text,
    p_payable_code text DEFAULT null,
    p_cash_account_code text DEFAULT null
) RETURNS void LANGUAGE plpgsql
AS $$
DECLARE
    v_bill procurement.purchases%ROWTYPE;
    v_txn_serial_id bigint;
    v_txn_uuid uuid;
    v_payable_account_uuid uuid;
    v_input_vat_account_uuid uuid;
    v_cash_account_uuid uuid;
BEGIN
    -- Lock bill row to prevent race conditions
    SELECT * INTO v_bill FROM procurement.purchases
        WHERE serial_id = p_bill_serial_id
        FOR UPDATE;

    -- Idempotency guard
    IF v_bill.posted THEN
        RAISE EXCEPTION 'Bill already posted';
    END IF;

    -- Resolve procurement account
    SELECT uuid INTO v_payable_account_uuid FROM accounting.accounts
        WHERE code = p_payable_code;

    IF v_payable_account_uuid IS NULL THEN
        RAISE EXCEPTION 'Payables account not found for code: %', p_payable_code;
    END IF;

    -- Resolve cash account for cash purchases
    IF v_bill.settlement_type = 'cash' THEN
        SELECT uuid INTO v_cash_account_uuid FROM accounting.accounts
            WHERE code = p_cash_account_code;

        IF v_cash_account_uuid IS NULL THEN
            RAISE EXCEPTION 'Cash account not found for code: %', p_cash_account_code;
        END IF;
    END IF;

    -- Resolve tax account
    SELECT uuid INTO v_input_vat_account_uuid FROM accounting.accounts
        WHERE code = p_vat_tax_code;

    IF v_input_vat_account_uuid IS NULL THEN
        RAISE EXCEPTION 'VAT tax account not found for code: %', p_vat_tax_code;
    END IF;

    -- Create empty GL transaction shell
    v_txn_serial_id := accounting.post_transaction(
        v_bill.bill_number, 'Vendor Bill', p_user, 'bill', v_bill.bill_date, '[]'::jsonb
    );

    SELECT uuid INTO v_txn_uuid FROM accounting.transactions
        WHERE serial_id = v_txn_serial_id;

    -------------------------------------------------------------------------------------
    -- Insert all accounting lines in ONE atomic statement
    -------------------------------------------------------------------------------------
    WITH asset_totals AS (
        SELECT
            a.uuid AS account_uuid,
            -- SUM(bi.total) AS amount
            SUM(bi.quantity * bi.unit_price) as amount
            FROM procurement.purchase_items bi
            JOIN inventory.items i ON i.serial_id = bi.stock_item_id
            JOIN accounting.accounts a ON a.code = i.asset_account
            WHERE bi.bill_uuid = v_bill.uuid
            GROUP BY a.uuid
    ),
    all_lines AS (
        -- Debit asset accounts (net, i.e., cost excluding tax)
        SELECT account_uuid, amount, 'debit' AS entry_type FROM asset_totals

        UNION ALL

        -- Debit input VAT from vendor
        SELECT v_input_vat_account_uuid, v_bill.tax_amount, 'debit'

        UNION ALL

        -- Credit payables or cash based on purchase type
        SELECT
            CASE
                WHEN v_bill.settlement_type = 'cash' THEN v_cash_account_uuid
                ELSE v_payable_account_uuid
            END,
            (v_bill.total_amount + v_bill.tax_amount), 'credit'
    )
    INSERT INTO accounting.transaction_entries (
        transaction_uuid, account_uuid, line_no, amount, debit, credit, memo
    )
    SELECT
        v_txn_uuid, account_uuid,
        ROW_NUMBER() OVER (
            ORDER BY
                CASE WHEN entry_type = 'debit' THEN 1 ELSE 2 END,
                account_uuid
        ) AS line_no,
        CASE WHEN entry_type = 'debit' THEN amount ELSE -amount END,
        CASE WHEN entry_type = 'debit' THEN amount ELSE 0 END,
        CASE WHEN entry_type = 'credit' THEN amount ELSE 0 END,
        CASE
            WHEN entry_type = 'debit' THEN 'Inventory billed on credit'
            ELSE 'Account payable credit purchase'
        END
    FROM all_lines;

    -------------------------------------------------------------------------------------
    -- Inventory qauntity movement (no GL touch here)
    -------------------------------------------------------------------------------------
    PERFORM inventory.post_purchase(
        bi.stock_item_id::bigint, i.warehouse_serial, bi.quantity, bi.unit_price, 'bill',
        p_bill_serial_id, p_user, p_payable_code, null, v_txn_uuid
    )
    FROM procurement.purchase_items bi
    JOIN inventory.items i ON i.serial_id = bi.stock_item_id
    WHERE bi.bill_uuid = v_bill.uuid;

    -------------------------------------------------------------------------------------
    -- Mark bill as posted
    -------------------------------------------------------------------------------------
    UPDATE procurement.purchases
    SET
        posted = TRUE,
        gl_transaction_uuid = v_txn_uuid,
        payment_status = CASE
            WHEN settlement_type = 'cash' THEN 'paid'
            ELSE payment_status
        END,
        balance_due = CASE
            WHEN settlement_type = 'cash' THEN 0
            ELSE (total_amount + tax_amount)
        END,
        paid_at = CASE
            WHEN settlement_type = 'cash' THEN now()
            ELSE NULL
        END
    WHERE serial_id = p_bill_serial_id;

    -------------------------------------------------------------------------------------
    -- Update vendor balance
    -------------------------------------------------------------------------------------
    IF v-bill.settlement_type = 'credit' THEN
        UPDATE procurement.vendors
        SET current_balance = current_balance + v_bill.total_amount + v_bill.tax_amount
        WHERE uuid = v_bill.vendor_uuid;
    END IF;

END;
$$;

-- Bill payment function (payment for credit purchase)
CREATE OR REPLACE FUNCTION procurement.post_payment(
    p_payment_serial_id bigint,
    p_user bigint,
    p_cash_account_code TEXT,
    p_payable_code TEXT
)
RETURNS void LANGUAGE plpgsql AS $$
DECLARE
    v_payment procurement.payments%ROWTYPE;
    v_txn_serial_id BIGINT;
    v_txn_uuid UUID;
    v_lines JSONB;
BEGIN
    SELECT * INTO v_payment FROM procurement.payments WHERE serial_id = p_payment_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Payment serial_id % not found', p_payment_serial_id; END IF;

    v_lines := jsonb_build_array(
        jsonb_build_object('account_ref', p_payable_code, 'debit', v_payment.amount, 'credit', 0, 'memo', v_payment.payment_number),
        jsonb_build_object('account_ref', p_cash_account_code, 'debit', 0, 'credit', v_payment.amount, 'memo', v_payment.payment_number)
    );

    v_txn_serial_id := accounting.post_transaction(
        v_payment.payment_number, 'Vendor Payment', p_user, 'vendor_payment', v_payment.payment_date, v_lines
    );

    SELECT uuid INTO v_txn_uuid FROM accounting.transactions WHERE serial_id = v_txn_serial_id;

    UPDATE procurement.payments
    SET gl_transaction_uuid = v_txn_uuid, updated_at = now()
    WHERE serial_id = p_payment_serial_id;

    UPDATE procurement.vendors
    SET current_balance = current_balance - v_payment.amount, updated_at = now()
    WHERE uuid = v_payment.vendor_uuid;
END;
$$;

-- Bill payment application function (payment for credit purchase)
CREATE OR REPLACE FUNCTION procurement.apply_payment(
    p_payment_serial_id bigint,
    p_bill_serial_id bigint,
    p_amount numeric
) RETURNS void LANGUAGE plpgsql AS $$
DECLARE
    v_payment_uuid UUID;
    v_bill_uuid UUID;
    v_balance_due numeric;
BEGIN
    SELECT uuid INTO v_payment_uuid FROM procurement.payments WHERE serial_id = p_payment_serial_id;
    SELECT uuid, balance_due INTO v_bill_uuid, v_balance_due FROM procurement.purchases WHERE serial_id = p_bill_serial_id;

    IF v_payment_uuid IS NULL THEN RAISE EXCEPTION 'Payment not found'; END IF;
    IF v_bill_uuid IS NULL THEN RAISE EXCEPTION 'Bill not found'; END IF;
    IF p_amount <= 0 THEN RAISE EXCEPTION 'Amount must be positive'; END IF;

    INSERT INTO procurement.payment_applications (payment_uuid, bill_uuid, applied_amount, applied_date)
    VALUES (v_payment_uuid, v_bill_uuid, p_amount, CURRENT_DATE);

    UPDATE procurement.purchases
    SET balance_due = balance_due - p_amount,
        payment_status = CASE
            WHEN balance_due - p_amount <= 0 THEN 'paid'
            WHEN balance_due - p_amount < total_amount THEN 'partial'
            ELSE 'unpaid'
        END,
        updated_at = now()
    WHERE uuid = v_bill_uuid;

    UPDATE procurement.payments
    SET applied_amount = applied_amount + p_amount,
        updated_at = now()
    WHERE uuid = v_payment_uuid;
END;
$$;

-- ========================================
-- INDEXES
-- ========================================
CREATE INDEX IF NOT EXISTS idx_purchases_vendor ON procurement.purchases(vendor_uuid);
CREATE INDEX IF NOT EXISTS idx_purchases_due_date ON procurement.purchases(due_date);
CREATE INDEX IF NOT EXISTS idx_purchases_status ON procurement.purchases(payment_status);
CREATE INDEX IF NOT EXISTS idx_purchases_balance ON procurement.purchases(balance_due) WHERE balance_due > 0;
CREATE INDEX IF NOT EXISTS idx_payments_vendor ON procurement.payments(vendor_uuid);
CREATE INDEX IF NOT EXISTS idx_payments_date ON procurement.payments(payment_date DESC);
CREATE INDEX IF NOT EXISTS idx_vendors_code ON procurement.vendors(code);
