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
-- Name: payables; Type: SCHEMA; Schema: -; Owner: chiefalry_user
--

CREATE SCHEMA payables;
ALTER SCHEMA payables OWNER TO chiefalry_user;
--
-- Name: apply_payment(bigint, bigint, numeric); Type: FUNCTION; Schema: payables; Owner: chiefalry_user
--

CREATE FUNCTION payables.apply_payment(
    p_payment_serial_id bigint,
    p_bill_serial_id bigint,
    p_amount numeric
) RETURNS void LANGUAGE plpgsql AS $$
DECLARE v_payment_uuid UUID;
v_bill_uuid UUID;
v_bill RECORD;
BEGIN -- Resolve UUIDs
SELECT uuid INTO v_payment_uuid
FROM payables.payments
WHERE serial_id = p_payment_serial_id;
SELECT uuid,
    balance_due INTO v_bill_uuid,
    v_bill.balance_due
FROM payables.bills
WHERE serial_id = p_bill_serial_id;
IF v_payment_uuid IS NULL THEN RAISE EXCEPTION 'Payment serial_id % not found',
p_payment_serial_id;
END IF;
IF v_bill_uuid IS NULL THEN RAISE EXCEPTION 'Bill serial_id % not found',
p_bill_serial_id;
END IF;
IF p_amount <= 0 THEN RAISE EXCEPTION 'Applied amount must be positive';
END IF;
-- Insert application
INSERT INTO payables.payment_applications (
        payment_uuid,
        bill_uuid,
        applied_amount,
        applied_date
    )
VALUES (
        v_payment_uuid,
        v_bill_uuid,
        p_amount,
        CURRENT_DATE
    );
-- Update bill
UPDATE payables.bills
SET balance_due = balance_due - p_amount,
    status = CASE
        WHEN balance_due - p_amount <= 0 THEN 'Paid'
        WHEN balance_due - p_amount < total_amount THEN 'Partial'
        ELSE 'Unpaid'
    END,
    updated_at = now()
WHERE uuid = v_bill_uuid;
-- Update payment
UPDATE payables.payments
SET applied_amount = applied_amount + p_amount,
    unapplied_amount = amount - (applied_amount + p_amount),
    updated_at = now()
WHERE uuid = v_payment_uuid;
END;
$$;
ALTER FUNCTION payables.apply_payment(
    p_payment_serial_id bigint,
    p_bill_serial_id bigint,
    p_amount numeric
) OWNER TO chiefalry_user;
--
-- Name: post_bill(bigint, bigint); Type: FUNCTION; Schema: payables; Owner: chiefalry_user
--

CREATE FUNCTION payables.post_bill(p_bill_serial_id bigint, p_user bigint) RETURNS void LANGUAGE plpgsql AS $$
DECLARE v_bill payables.bills %ROWTYPE;
v_txn_serial_id BIGINT;
v_txn_uuid UUID;
v_lines JSONB;
BEGIN -- Fetch bill by serial_id
SELECT * INTO v_bill
FROM payables.bills
WHERE serial_id = p_bill_serial_id;
IF NOT FOUND THEN RAISE EXCEPTION 'Bill with serial_id % not found',
p_bill_serial_id;
END IF;
IF v_bill.posted THEN RAISE NOTICE 'Bill % already posted',
v_bill.bill_number;
RETURN;
END IF;
-- GL: Debit Expense, Credit A/P
v_lines := jsonb_build_array(
    jsonb_build_object(
        'account_ref',
        '5.1.1',
        'debit',
        v_bill.total_amount,
        'credit',
        0,
        'memo',
        v_bill.bill_number
    ),
    jsonb_build_object(
        'account_ref',
        '2.1.1',
        'debit',
        0,
        'credit',
        v_bill.total_amount,
        'memo',
        v_bill.bill_number
    )
);
-- Post to GL → returns serial_id
v_txn_serial_id := accounting.post_transaction(
    v_bill.bill_date,
    v_bill.bill_number,
    'Vendor Bill Posting',
    p_user,
    'bill',
    v_lines
);
-- Get UUID of the transaction
SELECT uuid INTO v_txn_uuid
FROM accounting.transactions
WHERE serial_id = v_txn_serial_id;
IF v_txn_uuid IS NULL THEN RAISE EXCEPTION 'Failed to retrieve GL transaction UUID for serial_id %',
v_txn_serial_id;
END IF;
-- Update bill
UPDATE payables.bills
SET gl_transaction_uuid = v_txn_uuid,
    posted = TRUE,
    updated_at = now()
WHERE serial_id = p_bill_serial_id;
-- Increase vendor balance (liability)
UPDATE payables.vendors
SET current_balance = current_balance + v_bill.total_amount,
    updated_at = now()
WHERE uuid = v_bill.vendor_uuid;
END;
$$;
ALTER FUNCTION payables.post_bill(p_bill_serial_id bigint, p_user bigint) OWNER TO chiefalry_user;
--
-- Name: post_payment(bigint, bigint); Type: FUNCTION; Schema: payables; Owner: chiefalry_user
--

CREATE FUNCTION payables.post_payment(p_payment_serial_id bigint, p_user bigint) RETURNS void LANGUAGE plpgsql AS $$
DECLARE v_payment payables.payments %ROWTYPE;
v_txn_serial_id BIGINT;
v_txn_uuid UUID;
v_lines JSONB;
BEGIN
SELECT * INTO v_payment
FROM payables.payments
WHERE serial_id = p_payment_serial_id;
IF NOT FOUND THEN RAISE EXCEPTION 'Payment with serial_id % not found',
p_payment_serial_id;
END IF;
-- GL: Debit A/P, Credit Cash/Bank
v_lines := jsonb_build_array(
    jsonb_build_object(
        'account_ref',
        '2.1.1',
        'debit',
        v_payment.amount,
        'credit',
        0,
        'memo',
        v_payment.payment_number
    ),
    jsonb_build_object(
        'account_ref',
        '1.1.02',
        'debit',
        0,
        'credit',
        v_payment.amount,
        'memo',
        v_payment.payment_number
    )
);
v_txn_serial_id := accounting.post_transaction(
    v_payment.payment_date,
    v_payment.payment_number,
    'Vendor Payment',
    p_user,
    'vendor_payment',
    v_lines
);
SELECT uuid INTO v_txn_uuid
FROM accounting.transactions
WHERE serial_id = v_txn_serial_id;
IF v_txn_uuid IS NULL THEN RAISE EXCEPTION 'Failed to retrieve GL transaction UUID';
END IF;
UPDATE payables.payments
SET gl_transaction_uuid = v_txn_uuid,
    updated_at = now()
WHERE serial_id = p_payment_serial_id;
-- Reduce vendor balance
UPDATE payables.vendors
SET current_balance = current_balance - v_payment.amount,
    updated_at = now()
WHERE uuid = v_payment.vendor_uuid;
END;
$$;
ALTER FUNCTION payables.post_payment(p_payment_serial_id bigint, p_user bigint) OWNER TO chiefalry_user;
SET default_tablespace = '';
SET default_table_access_method = heap;
--
-- Name: bill_items; Type: TABLE; Schema: payables; Owner: chiefalry_user
--

CREATE TABLE payables.bill_items (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    bill_uuid uuid NOT NULL,
    item_code text,
    description text,
    quantity numeric(12, 4) DEFAULT 1,
    unit_price numeric(18, 4) DEFAULT 0,
    tax_rate numeric(5, 2) DEFAULT 0,
    total numeric(18, 2) NOT NULL
);
ALTER TABLE payables.bill_items OWNER TO chiefalry_user;
--
-- Name: bill_items_serial_id_seq; Type: SEQUENCE; Schema: payables; Owner: chiefalry_user
--

CREATE SEQUENCE payables.bill_items_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE payables.bill_items_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: bill_items_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: payables; Owner: chiefalry_user
--

ALTER SEQUENCE payables.bill_items_serial_id_seq OWNED BY payables.bill_items.serial_id;
--
-- Name: bills; Type: TABLE; Schema: payables; Owner: chiefalry_user
--

CREATE TABLE payables.bills (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    bill_number text NOT NULL,
    vendor_uuid uuid NOT NULL,
    bill_date date NOT NULL,
    due_date date NOT NULL,
    reference text,
    total_amount numeric(18, 2) NOT NULL,
    tax_amount numeric(18, 2) DEFAULT 0,
    balance_due numeric(18, 2) NOT NULL,
    currency text DEFAULT 'UGX'::text,
    status text DEFAULT 'Unpaid'::text,
    posted boolean DEFAULT false,
    gl_transaction_uuid uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);
ALTER TABLE payables.bills OWNER TO chiefalry_user;
--
-- Name: bills_serial_id_seq; Type: SEQUENCE; Schema: payables; Owner: chiefalry_user
--

CREATE SEQUENCE payables.bills_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE payables.bills_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: bills_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: payables; Owner: chiefalry_user
--

ALTER SEQUENCE payables.bills_serial_id_seq OWNED BY payables.bills.serial_id;
--
-- Name: credit_notes; Type: TABLE; Schema: payables; Owner: chiefalry_user
--

CREATE TABLE payables.credit_notes (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    credit_number text NOT NULL,
    vendor_uuid uuid NOT NULL,
    reference_bill_uuid uuid,
    issue_date date NOT NULL,
    amount numeric(18, 2) NOT NULL,
    reason text,
    gl_transaction_uuid uuid,
    status text DEFAULT 'Open'::text,
    created_at timestamp with time zone DEFAULT now()
);
ALTER TABLE payables.credit_notes OWNER TO chiefalry_user;
--
-- Name: credit_notes_serial_id_seq; Type: SEQUENCE; Schema: payables; Owner: chiefalry_user
--

CREATE SEQUENCE payables.credit_notes_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE payables.credit_notes_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: credit_notes_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: payables; Owner: chiefalry_user
--

ALTER SEQUENCE payables.credit_notes_serial_id_seq OWNED BY payables.credit_notes.serial_id;
--
-- Name: payment_applications; Type: TABLE; Schema: payables; Owner: chiefalry_user
--

CREATE TABLE payables.payment_applications (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    payment_uuid uuid NOT NULL,
    bill_uuid uuid NOT NULL,
    applied_amount numeric(18, 2) NOT NULL,
    applied_date date DEFAULT now() NOT NULL
);
ALTER TABLE payables.payment_applications OWNER TO chiefalry_user;
--
-- Name: payment_applications_serial_id_seq; Type: SEQUENCE; Schema: payables; Owner: chiefalry_user
--

CREATE SEQUENCE payables.payment_applications_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE payables.payment_applications_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: payment_applications_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: payables; Owner: chiefalry_user
--

ALTER SEQUENCE payables.payment_applications_serial_id_seq OWNED BY payables.payment_applications.serial_id;
--
-- Name: payments; Type: TABLE; Schema: payables; Owner: chiefalry_user
--

CREATE TABLE payables.payments (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    payment_number text NOT NULL,
    vendor_uuid uuid NOT NULL,
    payment_date date NOT NULL,
    method text NOT NULL,
    reference text,
    amount numeric(18, 2) NOT NULL,
    currency text DEFAULT 'UGX'::text,
    applied_amount numeric(18, 2) DEFAULT 0,
    unapplied_amount numeric(18, 2) DEFAULT 0,
    gl_transaction_uuid uuid,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);
ALTER TABLE payables.payments OWNER TO chiefalry_user;
--
-- Name: payments_serial_id_seq; Type: SEQUENCE; Schema: payables; Owner: chiefalry_user
--

CREATE SEQUENCE payables.payments_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE payables.payments_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: payments_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: payables; Owner: chiefalry_user
--

ALTER SEQUENCE payables.payments_serial_id_seq OWNED BY payables.payments.serial_id;
--
-- Name: vendors; Type: TABLE; Schema: payables; Owner: chiefalry_user
--

CREATE TABLE payables.vendors (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    contact_person text,
    email text,
    phone text,
    address text,
    tax_id text,
    payment_terms text DEFAULT 'Net 30'::text,
    credit_limit numeric(18, 2) DEFAULT 0,
    current_balance numeric(18, 2) DEFAULT 0,
    status text DEFAULT 'Active'::text,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);
ALTER TABLE payables.vendors OWNER TO chiefalry_user;
--
-- Name: vendor_aging; Type: VIEW; Schema: payables; Owner: chiefalry_user
--

CREATE VIEW payables.vendor_aging AS
SELECT v.serial_id AS vendor_serial_id,
    v.code AS vendor_code,
    v.name AS vendor_name,
    sum(
        CASE
            WHEN (b.due_date >= CURRENT_DATE) THEN b.balance_due
            ELSE (0)::numeric
        END
    ) AS current,
    sum(
        CASE
            WHEN (
                (b.due_date < CURRENT_DATE)
                AND (
                    b.due_date >= (CURRENT_DATE - '30 days'::interval)
                )
            ) THEN b.balance_due
            ELSE (0)::numeric
        END
    ) AS days_1_30,
    sum(
        CASE
            WHEN (
                (
                    b.due_date < (CURRENT_DATE - '30 days'::interval)
                )
                AND (
                    b.due_date >= (CURRENT_DATE - '60 days'::interval)
                )
            ) THEN b.balance_due
            ELSE (0)::numeric
        END
    ) AS days_31_60,
    sum(
        CASE
            WHEN (
                (
                    b.due_date < (CURRENT_DATE - '60 days'::interval)
                )
                AND (
                    b.due_date >= (CURRENT_DATE - '90 days'::interval)
                )
            ) THEN b.balance_due
            ELSE (0)::numeric
        END
    ) AS days_61_90,
    sum(
        CASE
            WHEN (
                b.due_date < (CURRENT_DATE - '90 days'::interval)
            ) THEN b.balance_due
            ELSE (0)::numeric
        END
    ) AS days_over_90,
    sum(b.balance_due) AS total_outstanding
FROM (
        payables.vendors v
        LEFT JOIN payables.bills b ON ((b.vendor_uuid = v.uuid))
    )
WHERE (
        (
            b.status <> ALL (ARRAY ['Paid'::text, 'Cancelled'::text])
        )
        OR (b.status IS NULL)
    )
GROUP BY v.serial_id,
    v.code,
    v.name
ORDER BY (sum(b.balance_due)) DESC;
ALTER VIEW payables.vendor_aging OWNER TO chiefalry_user;
--
-- Name: vendor_statement; Type: VIEW; Schema: payables; Owner: chiefalry_user
--

CREATE VIEW payables.vendor_statement AS
SELECT v.serial_id AS vendor_serial_id,
    v.code AS vendor_code,
    v.name AS vendor_name,
    b.serial_id AS bill_serial_id,
    b.bill_number,
    b.bill_date,
    b.due_date,
    b.total_amount,
    b.balance_due,
    b.status AS bill_status,
    COALESCE(p.payment_number, ''::text) AS last_payment_number,
    COALESCE(p.amount, (0)::numeric) AS last_payment_amount,
    COALESCE(p.payment_date, NULL::date) AS last_payment_date
FROM (
        (
            payables.vendors v
            LEFT JOIN payables.bills b ON ((b.vendor_uuid = v.uuid))
        )
        LEFT JOIN payables.payments p ON ((p.vendor_uuid = v.uuid))
    );
ALTER VIEW payables.vendor_statement OWNER TO chiefalry_user;
--
-- Name: vendor_statement_clean; Type: VIEW; Schema: payables; Owner: chiefalry_user
--

CREATE VIEW payables.vendor_statement_clean AS
SELECT DISTINCT ON (v.serial_id, b.serial_id) v.serial_id AS vendor_serial_id,
    v.code AS vendor_code,
    v.name AS vendor_name,
    b.serial_id AS bill_serial_id,
    b.bill_number,
    b.bill_date,
    b.due_date,
    b.total_amount,
    b.balance_due,
    b.status AS bill_status,
    COALESCE(p.payment_number, ''::text) AS last_payment_number,
    COALESCE(p.amount, (0)::numeric) AS last_payment_amount,
    COALESCE(p.payment_date, NULL::date) AS last_payment_date
FROM (
        (
            payables.vendors v
            LEFT JOIN payables.bills b ON ((b.vendor_uuid = v.uuid))
        )
        LEFT JOIN payables.payments p ON ((p.vendor_uuid = v.uuid))
    )
ORDER BY v.serial_id,
    b.serial_id,
    p.payment_date DESC NULLS LAST,
    p.serial_id DESC;
ALTER VIEW payables.vendor_statement_clean OWNER TO chiefalry_user;
--
-- Name: vendors_serial_id_seq; Type: SEQUENCE; Schema: payables; Owner: chiefalry_user
--

CREATE SEQUENCE payables.vendors_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE payables.vendors_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: vendors_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: payables; Owner: chiefalry_user
--

ALTER SEQUENCE payables.vendors_serial_id_seq OWNED BY payables.vendors.serial_id;
--
-- Name: bill_items serial_id; Type: DEFAULT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.bill_items
ALTER COLUMN serial_id
SET DEFAULT nextval('payables.bill_items_serial_id_seq'::regclass);
--
-- Name: bills serial_id; Type: DEFAULT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.bills
ALTER COLUMN serial_id
SET DEFAULT nextval('payables.bills_serial_id_seq'::regclass);
--
-- Name: credit_notes serial_id; Type: DEFAULT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.credit_notes
ALTER COLUMN serial_id
SET DEFAULT nextval('payables.credit_notes_serial_id_seq'::regclass);
--
-- Name: payment_applications serial_id; Type: DEFAULT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.payment_applications
ALTER COLUMN serial_id
SET DEFAULT nextval(
        'payables.payment_applications_serial_id_seq'::regclass
    );
--
-- Name: payments serial_id; Type: DEFAULT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.payments
ALTER COLUMN serial_id
SET DEFAULT nextval('payables.payments_serial_id_seq'::regclass);
--
-- Name: vendors serial_id; Type: DEFAULT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.vendors
ALTER COLUMN serial_id
SET DEFAULT nextval('payables.vendors_serial_id_seq'::regclass);
--
-- Name: bill_items bill_items_pkey; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.bill_items
ADD CONSTRAINT bill_items_pkey PRIMARY KEY (uuid);
--
-- Name: bill_items bill_items_serial_id_key; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.bill_items
ADD CONSTRAINT bill_items_serial_id_key UNIQUE (serial_id);
--
-- Name: bills bills_bill_number_key; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.bills
ADD CONSTRAINT bills_bill_number_key UNIQUE (bill_number);
--
-- Name: bills bills_pkey; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.bills
ADD CONSTRAINT bills_pkey PRIMARY KEY (uuid);
--
-- Name: bills bills_serial_id_key; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.bills
ADD CONSTRAINT bills_serial_id_key UNIQUE (serial_id);
--
-- Name: credit_notes credit_notes_credit_number_key; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.credit_notes
ADD CONSTRAINT credit_notes_credit_number_key UNIQUE (credit_number);
--
-- Name: credit_notes credit_notes_pkey; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.credit_notes
ADD CONSTRAINT credit_notes_pkey PRIMARY KEY (uuid);
--
-- Name: credit_notes credit_notes_serial_id_key; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.credit_notes
ADD CONSTRAINT credit_notes_serial_id_key UNIQUE (serial_id);
--
-- Name: payment_applications payment_applications_payment_uuid_bill_uuid_key; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.payment_applications
ADD CONSTRAINT payment_applications_payment_uuid_bill_uuid_key UNIQUE (payment_uuid, bill_uuid);
--
-- Name: payment_applications payment_applications_pkey; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.payment_applications
ADD CONSTRAINT payment_applications_pkey PRIMARY KEY (uuid);
--
-- Name: payment_applications payment_applications_serial_id_key; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.payment_applications
ADD CONSTRAINT payment_applications_serial_id_key UNIQUE (serial_id);
--
-- Name: payments payments_payment_number_key; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.payments
ADD CONSTRAINT payments_payment_number_key UNIQUE (payment_number);
--
-- Name: payments payments_pkey; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.payments
ADD CONSTRAINT payments_pkey PRIMARY KEY (uuid);
--
-- Name: payments payments_serial_id_key; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.payments
ADD CONSTRAINT payments_serial_id_key UNIQUE (serial_id);
--
-- Name: vendors vendors_code_key; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.vendors
ADD CONSTRAINT vendors_code_key UNIQUE (code);
--
-- Name: vendors vendors_pkey; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.vendors
ADD CONSTRAINT vendors_pkey PRIMARY KEY (uuid);
--
-- Name: vendors vendors_serial_id_key; Type: CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.vendors
ADD CONSTRAINT vendors_serial_id_key UNIQUE (serial_id);
--
-- Name: bill_items_bill_uuid_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX bill_items_bill_uuid_idx ON payables.bill_items USING btree (bill_uuid);
--
-- Name: bill_items_serial_id_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX bill_items_serial_id_idx ON payables.bill_items USING btree (serial_id);
--
-- Name: bills_bill_date_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX bills_bill_date_idx ON payables.bills USING btree (bill_date);
--
-- Name: bills_bill_number_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX bills_bill_number_idx ON payables.bills USING btree (bill_number);
--
-- Name: bills_due_date_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX bills_due_date_idx ON payables.bills USING btree (due_date);
--
-- Name: bills_serial_id_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX bills_serial_id_idx ON payables.bills USING btree (serial_id);
--
-- Name: bills_status_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX bills_status_idx ON payables.bills USING btree (status);
--
-- Name: bills_vendor_uuid_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX bills_vendor_uuid_idx ON payables.bills USING btree (vendor_uuid);
--
-- Name: credit_notes_credit_number_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX credit_notes_credit_number_idx ON payables.credit_notes USING btree (credit_number);
--
-- Name: credit_notes_reference_bill_uuid_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX credit_notes_reference_bill_uuid_idx ON payables.credit_notes USING btree (reference_bill_uuid);
--
-- Name: credit_notes_serial_id_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX credit_notes_serial_id_idx ON payables.credit_notes USING btree (serial_id);
--
-- Name: credit_notes_vendor_uuid_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX credit_notes_vendor_uuid_idx ON payables.credit_notes USING btree (vendor_uuid);
--
-- Name: idx_bills_balance_due; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX idx_bills_balance_due ON payables.bills USING btree (balance_due)
WHERE (balance_due > (0)::numeric);
--
-- Name: idx_bills_due_date; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX idx_bills_due_date ON payables.bills USING btree (due_date);
--
-- Name: idx_bills_serial_id; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX idx_bills_serial_id ON payables.bills USING btree (serial_id);
--
-- Name: idx_bills_status; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX idx_bills_status ON payables.bills USING btree (status);
--
-- Name: idx_bills_vendor_uuid; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX idx_bills_vendor_uuid ON payables.bills USING btree (vendor_uuid);
--
-- Name: idx_payments_date_desc; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX idx_payments_date_desc ON payables.payments USING btree (payment_date DESC);
--
-- Name: idx_payments_serial_id; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX idx_payments_serial_id ON payables.payments USING btree (serial_id);
--
-- Name: idx_payments_vendor_uuid; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX idx_payments_vendor_uuid ON payables.payments USING btree (vendor_uuid);
--
-- Name: idx_vendors_code; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX idx_vendors_code ON payables.vendors USING btree (code);
--
-- Name: idx_vendors_name; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX idx_vendors_name ON payables.vendors USING btree (name);
--
-- Name: idx_vendors_serial_id; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX idx_vendors_serial_id ON payables.vendors USING btree (serial_id);
--
-- Name: payment_applications_bill_uuid_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX payment_applications_bill_uuid_idx ON payables.payment_applications USING btree (bill_uuid);
--
-- Name: payment_applications_payment_uuid_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX payment_applications_payment_uuid_idx ON payables.payment_applications USING btree (payment_uuid);
--
-- Name: payment_applications_serial_id_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX payment_applications_serial_id_idx ON payables.payment_applications USING btree (serial_id);
--
-- Name: payments_payment_date_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX payments_payment_date_idx ON payables.payments USING btree (payment_date);
--
-- Name: payments_payment_number_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX payments_payment_number_idx ON payables.payments USING btree (payment_number);
--
-- Name: payments_serial_id_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX payments_serial_id_idx ON payables.payments USING btree (serial_id);
--
-- Name: payments_vendor_uuid_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX payments_vendor_uuid_idx ON payables.payments USING btree (vendor_uuid);
--
-- Name: vendors_code_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX vendors_code_idx ON payables.vendors USING btree (code);
--
-- Name: vendors_name_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX vendors_name_idx ON payables.vendors USING btree (name);
--
-- Name: vendors_serial_id_idx; Type: INDEX; Schema: payables; Owner: chiefalry_user
--

CREATE INDEX vendors_serial_id_idx ON payables.vendors USING btree (serial_id);
--
-- Name: bill_items bill_items_bill_uuid_fkey; Type: FK CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.bill_items
ADD CONSTRAINT bill_items_bill_uuid_fkey FOREIGN KEY (bill_uuid) REFERENCES payables.bills(uuid) ON DELETE CASCADE;
--
-- Name: bills bills_gl_transaction_uuid_fkey; Type: FK CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.bills
ADD CONSTRAINT bills_gl_transaction_uuid_fkey FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid);
--
-- Name: bills bills_vendor_uuid_fkey; Type: FK CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.bills
ADD CONSTRAINT bills_vendor_uuid_fkey FOREIGN KEY (vendor_uuid) REFERENCES payables.vendors(uuid) ON DELETE RESTRICT;
--
-- Name: credit_notes credit_notes_gl_transaction_uuid_fkey; Type: FK CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.credit_notes
ADD CONSTRAINT credit_notes_gl_transaction_uuid_fkey FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid);
--
-- Name: credit_notes credit_notes_reference_bill_uuid_fkey; Type: FK CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.credit_notes
ADD CONSTRAINT credit_notes_reference_bill_uuid_fkey FOREIGN KEY (reference_bill_uuid) REFERENCES payables.bills(uuid);
--
-- Name: credit_notes credit_notes_vendor_uuid_fkey; Type: FK CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.credit_notes
ADD CONSTRAINT credit_notes_vendor_uuid_fkey FOREIGN KEY (vendor_uuid) REFERENCES payables.vendors(uuid);
--
-- Name: payment_applications payment_applications_bill_uuid_fkey; Type: FK CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.payment_applications
ADD CONSTRAINT payment_applications_bill_uuid_fkey FOREIGN KEY (bill_uuid) REFERENCES payables.bills(uuid) ON DELETE CASCADE;
--
-- Name: payment_applications payment_applications_payment_uuid_fkey; Type: FK CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.payment_applications
ADD CONSTRAINT payment_applications_payment_uuid_fkey FOREIGN KEY (payment_uuid) REFERENCES payables.payments(uuid) ON DELETE CASCADE;
--
-- Name: payments payments_gl_transaction_uuid_fkey; Type: FK CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.payments
ADD CONSTRAINT payments_gl_transaction_uuid_fkey FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid);
--
-- Name: payments payments_vendor_uuid_fkey; Type: FK CONSTRAINT; Schema: payables; Owner: chiefalry_user
--

ALTER TABLE ONLY payables.payments
ADD CONSTRAINT payments_vendor_uuid_fkey FOREIGN KEY (vendor_uuid) REFERENCES payables.vendors(uuid);
--
-- PostgreSQL database dump complete
--