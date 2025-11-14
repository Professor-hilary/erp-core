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
-- Name: receivables; Type: SCHEMA; Schema: -; Owner: chiefalry_user
--

CREATE SCHEMA receivables;
ALTER SCHEMA receivables OWNER TO chiefalry_user;
--
-- Name: apply_payment(bigint, bigint, numeric); Type: FUNCTION; Schema: receivables; Owner: chiefalry_user
--

CREATE FUNCTION receivables.apply_payment(
    p_payment_serial_id bigint,
    p_invoice_serial_id bigint,
    p_amount numeric
) RETURNS void LANGUAGE plpgsql AS $$
DECLARE v_payment_uuid UUID;
v_invoice_uuid UUID;
v_invoice receivables.invoices %ROWTYPE;
v_payment receivables.payments %ROWTYPE;
BEGIN -- Resolve UUIDs
SELECT uuid INTO v_payment_uuid
FROM receivables.payments
WHERE serial_id = p_payment_serial_id;
SELECT uuid,
    balance_due INTO v_invoice_uuid,
    v_invoice.balance_due
FROM receivables.invoices
WHERE serial_id = p_invoice_serial_id;
IF v_payment_uuid IS NULL THEN RAISE EXCEPTION 'Payment serial_id % not found',
p_payment_serial_id;
END IF;
IF v_invoice_uuid IS NULL THEN RAISE EXCEPTION 'Invoice serial_id % not found',
p_invoice_serial_id;
END IF;
IF p_amount <= 0 THEN RAISE EXCEPTION 'Applied amount must be positive';
END IF;
-- Insert application
INSERT INTO receivables.payment_applications (
        payment_uuid,
        invoice_uuid,
        applied_amount,
        applied_date
    )
VALUES (
        v_payment_uuid,
        v_invoice_uuid,
        p_amount,
        CURRENT_DATE
    );
-- Update invoice
UPDATE receivables.invoices
SET balance_due = balance_due - p_amount,
    status = CASE
        WHEN balance_due - p_amount <= 0 THEN 'Paid'
        WHEN balance_due - p_amount < total_amount THEN 'Partial'
        ELSE 'Unpaid'
    END,
    updated_at = now()
WHERE uuid = v_invoice_uuid;
-- Update payment
UPDATE receivables.payments
SET applied_amount = applied_amount + p_amount,
    unapplied_amount = amount - (applied_amount + p_amount),
    updated_at = now()
WHERE uuid = v_payment_uuid;
-- Update customer balance (only if not already adjusted via post_payment)
-- Optional: skip if you handle balance only on post
END;
$$;
ALTER FUNCTION receivables.apply_payment(
    p_payment_serial_id bigint,
    p_invoice_serial_id bigint,
    p_amount numeric
) OWNER TO chiefalry_user;
--
-- Name: post_invoice(bigint, bigint); Type: FUNCTION; Schema: receivables; Owner: chiefalry_user
--

CREATE FUNCTION receivables.post_invoice(p_invoice_serial_id bigint, p_user bigint) RETURNS void LANGUAGE plpgsql AS $$
DECLARE v_invoice receivables.invoices %ROWTYPE;
v_txn_serial_id BIGINT;
v_txn_uuid UUID;
v_lines JSONB;
BEGIN -- Fetch invoice by serial_id
SELECT * INTO v_invoice
FROM receivables.invoices
WHERE serial_id = p_invoice_serial_id;
IF NOT FOUND THEN RAISE EXCEPTION 'Invoice with serial_id % not found',
p_invoice_serial_id;
END IF;
IF v_invoice.posted THEN RAISE NOTICE 'Invoice % already posted',
v_invoice.invoice_number;
RETURN;
END IF;
-- Build GL lines: Debit A/R, Credit Revenue
v_lines := jsonb_build_array(
    jsonb_build_object(
        'account_ref',
        '1.2.1',
        'debit',
        v_invoice.total_amount,
        'credit',
        0,
        'memo',
        v_invoice.invoice_number
    ),
    jsonb_build_object(
        'account_ref',
        '4.1.1',
        'debit',
        0,
        'credit',
        v_invoice.total_amount,
        'memo',
        v_invoice.invoice_number
    )
);
-- Post to GL → returns txn serial_id, but we need uuid
v_txn_serial_id := accounting.post_transaction(
    v_invoice.issue_date,
    v_invoice.invoice_number,
    'Invoice Posting',
    p_user,
    'invoice',
    v_lines
);
-- Get the UUID of the newly created transaction
SELECT uuid INTO v_txn_uuid
FROM accounting.transactions
WHERE serial_id = v_txn_serial_id;
IF v_txn_uuid IS NULL THEN RAISE EXCEPTION 'Failed to retrieve GL transaction UUID for serial_id %',
v_txn_serial_id;
END IF;
-- Update invoice: store GL UUID and mark posted
UPDATE receivables.invoices
SET gl_transaction_uuid = v_txn_uuid,
    posted = TRUE,
    updated_at = now()
WHERE serial_id = p_invoice_serial_id;
-- Update customer balance
UPDATE receivables.customers
SET current_balance = current_balance + v_invoice.total_amount,
    updated_at = now()
WHERE uuid = v_invoice.customer_uuid;
END;
$$;
ALTER FUNCTION receivables.post_invoice(p_invoice_serial_id bigint, p_user bigint) OWNER TO chiefalry_user;
--
-- Name: post_payment(bigint, bigint); Type: FUNCTION; Schema: receivables; Owner: chiefalry_user
--

CREATE FUNCTION receivables.post_payment(p_payment_serial_id bigint, p_user bigint) RETURNS void LANGUAGE plpgsql AS $$
DECLARE v_payment receivables.payments %ROWTYPE;
v_txn_serial_id BIGINT;
v_txn_uuid UUID;
v_lines JSONB;
BEGIN
SELECT * INTO v_payment
FROM receivables.payments
WHERE serial_id = p_payment_serial_id;
IF NOT FOUND THEN RAISE EXCEPTION 'Payment with serial_id % not found',
p_payment_serial_id;
END IF;
-- Build GL lines: Debit Cash/Bank, Credit A/R
v_lines := jsonb_build_array(
    jsonb_build_object(
        'account_ref',
        '1.1.02',
        'debit',
        v_payment.amount,
        'credit',
        0,
        'memo',
        v_payment.payment_number
    ),
    jsonb_build_object(
        'account_ref',
        '1.2.1',
        'debit',
        0,
        'credit',
        v_payment.amount,
        'memo',
        v_payment.payment_number
    )
);
-- Post to GL
v_txn_serial_id := accounting.post_transaction(
    v_payment.payment_date,
    v_payment.payment_number,
    'Payment Receipt',
    p_user,
    'payment',
    v_lines
);
-- Get UUID
SELECT uuid INTO v_txn_uuid
FROM accounting.transactions
WHERE serial_id = v_txn_serial_id;
IF v_txn_uuid IS NULL THEN RAISE EXCEPTION 'Failed to retrieve GL transaction UUID';
END IF;
-- Update payment
UPDATE receivables.payments
SET gl_transaction_uuid = v_txn_uuid,
    updated_at = now()
WHERE serial_id = p_payment_serial_id;
-- Reduce customer balance
UPDATE receivables.customers
SET current_balance = current_balance - v_payment.amount,
    updated_at = now()
WHERE uuid = v_payment.customer_uuid;
END;
$$;
ALTER FUNCTION receivables.post_payment(p_payment_serial_id bigint, p_user bigint) OWNER TO chiefalry_user;
SET default_tablespace = '';
SET default_table_access_method = heap;
--
-- Name: credit_notes; Type: TABLE; Schema: receivables; Owner: chiefalry_user
--

CREATE TABLE receivables.credit_notes (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    credit_number text NOT NULL,
    customer_uuid uuid NOT NULL,
    reference_invoice_uuid uuid,
    issue_date date NOT NULL,
    amount numeric(18, 2) NOT NULL,
    reason text,
    gl_transaction_uuid uuid,
    status text DEFAULT 'Open'::text,
    created_at timestamp with time zone DEFAULT now()
);
ALTER TABLE receivables.credit_notes OWNER TO chiefalry_user;
--
-- Name: credit_notes_serial_id_seq; Type: SEQUENCE; Schema: receivables; Owner: chiefalry_user
--

CREATE SEQUENCE receivables.credit_notes_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE receivables.credit_notes_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: credit_notes_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: receivables; Owner: chiefalry_user
--

ALTER SEQUENCE receivables.credit_notes_serial_id_seq OWNED BY receivables.credit_notes.serial_id;
--
-- Name: customers; Type: TABLE; Schema: receivables; Owner: chiefalry_user
--

CREATE TABLE receivables.customers (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    email text,
    phone text,
    billing_address text,
    credit_limit numeric(18, 2) DEFAULT 0,
    current_balance numeric(18, 2) DEFAULT 0,
    terms text DEFAULT 'Net 30'::text,
    status text DEFAULT 'Active'::text,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);
ALTER TABLE receivables.customers OWNER TO chiefalry_user;
--
-- Name: invoices; Type: TABLE; Schema: receivables; Owner: chiefalry_user
--

CREATE TABLE receivables.invoices (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    invoice_number text NOT NULL,
    customer_uuid uuid NOT NULL,
    issue_date date NOT NULL,
    due_date date NOT NULL,
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
ALTER TABLE receivables.invoices OWNER TO chiefalry_user;
--
-- Name: customer_aging; Type: VIEW; Schema: receivables; Owner: chiefalry_user
--

CREATE VIEW receivables.customer_aging AS
SELECT c.serial_id AS customer_serial_id,
    c.code AS customer_code,
    c.name AS customer_name,
    sum(
        CASE
            WHEN (i.due_date >= CURRENT_DATE) THEN i.balance_due
            ELSE (0)::numeric
        END
    ) AS current,
    sum(
        CASE
            WHEN (
                (i.due_date < CURRENT_DATE)
                AND (
                    i.due_date >= (CURRENT_DATE - '30 days'::interval)
                )
            ) THEN i.balance_due
            ELSE (0)::numeric
        END
    ) AS days_1_30,
    sum(
        CASE
            WHEN (
                (
                    i.due_date < (CURRENT_DATE - '30 days'::interval)
                )
                AND (
                    i.due_date >= (CURRENT_DATE - '60 days'::interval)
                )
            ) THEN i.balance_due
            ELSE (0)::numeric
        END
    ) AS days_31_60,
    sum(
        CASE
            WHEN (
                (
                    i.due_date < (CURRENT_DATE - '60 days'::interval)
                )
                AND (
                    i.due_date >= (CURRENT_DATE - '90 days'::interval)
                )
            ) THEN i.balance_due
            ELSE (0)::numeric
        END
    ) AS days_61_90,
    sum(
        CASE
            WHEN (
                i.due_date < (CURRENT_DATE - '90 days'::interval)
            ) THEN i.balance_due
            ELSE (0)::numeric
        END
    ) AS days_over_90,
    sum(i.balance_due) AS total_outstanding
FROM (
        receivables.customers c
        LEFT JOIN receivables.invoices i ON ((i.customer_uuid = c.uuid))
    )
WHERE (
        (
            i.status <> ALL (ARRAY ['Paid'::text, 'Cancelled'::text])
        )
        OR (i.status IS NULL)
    )
GROUP BY c.serial_id,
    c.code,
    c.name;
ALTER VIEW receivables.customer_aging OWNER TO chiefalry_user;
--
-- Name: payments; Type: TABLE; Schema: receivables; Owner: chiefalry_user
--

CREATE TABLE receivables.payments (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    payment_number text NOT NULL,
    customer_uuid uuid NOT NULL,
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
ALTER TABLE receivables.payments OWNER TO chiefalry_user;
--
-- Name: customer_statement; Type: VIEW; Schema: receivables; Owner: chiefalry_user
--

CREATE VIEW receivables.customer_statement AS
SELECT DISTINCT ON (c.serial_id, i.serial_id) c.serial_id AS customer_serial_id,
    c.code AS customer_code,
    c.name AS customer_name,
    i.serial_id AS invoice_serial_id,
    i.invoice_number,
    i.issue_date,
    i.due_date,
    i.total_amount,
    i.balance_due,
    i.status AS invoice_status,
    COALESCE(p.payment_number, ''::text) AS last_payment_number,
    COALESCE(p.amount, (0)::numeric) AS last_payment_amount,
    COALESCE(p.payment_date, NULL::date) AS last_payment_date
FROM (
        (
            receivables.customers c
            LEFT JOIN receivables.invoices i ON ((i.customer_uuid = c.uuid))
        )
        LEFT JOIN receivables.payments p ON ((p.customer_uuid = c.uuid))
    )
ORDER BY c.serial_id,
    i.serial_id,
    p.payment_date DESC NULLS LAST,
    p.serial_id DESC;
ALTER VIEW receivables.customer_statement OWNER TO chiefalry_user;
--
-- Name: customers_serial_id_seq; Type: SEQUENCE; Schema: receivables; Owner: chiefalry_user
--

CREATE SEQUENCE receivables.customers_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE receivables.customers_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: customers_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: receivables; Owner: chiefalry_user
--

ALTER SEQUENCE receivables.customers_serial_id_seq OWNED BY receivables.customers.serial_id;
--
-- Name: invoice_items; Type: TABLE; Schema: receivables; Owner: chiefalry_user
--

CREATE TABLE receivables.invoice_items (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    invoice_uuid uuid NOT NULL,
    item_code text,
    description text,
    quantity numeric(12, 4) DEFAULT 1,
    unit_price numeric(18, 4) DEFAULT 0,
    tax_rate numeric(5, 2) DEFAULT 0,
    total numeric(18, 2) NOT NULL
);
ALTER TABLE receivables.invoice_items OWNER TO chiefalry_user;
--
-- Name: invoice_items_serial_id_seq; Type: SEQUENCE; Schema: receivables; Owner: chiefalry_user
--

CREATE SEQUENCE receivables.invoice_items_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE receivables.invoice_items_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: invoice_items_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: receivables; Owner: chiefalry_user
--

ALTER SEQUENCE receivables.invoice_items_serial_id_seq OWNED BY receivables.invoice_items.serial_id;
--
-- Name: invoices_serial_id_seq; Type: SEQUENCE; Schema: receivables; Owner: chiefalry_user
--

CREATE SEQUENCE receivables.invoices_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE receivables.invoices_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: invoices_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: receivables; Owner: chiefalry_user
--

ALTER SEQUENCE receivables.invoices_serial_id_seq OWNED BY receivables.invoices.serial_id;
--
-- Name: payment_applications; Type: TABLE; Schema: receivables; Owner: chiefalry_user
--

CREATE TABLE receivables.payment_applications (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    payment_uuid uuid NOT NULL,
    invoice_uuid uuid NOT NULL,
    applied_amount numeric(18, 2) NOT NULL,
    applied_date date DEFAULT now() NOT NULL
);
ALTER TABLE receivables.payment_applications OWNER TO chiefalry_user;
--
-- Name: payment_applications_serial_id_seq; Type: SEQUENCE; Schema: receivables; Owner: chiefalry_user
--

CREATE SEQUENCE receivables.payment_applications_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE receivables.payment_applications_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: payment_applications_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: receivables; Owner: chiefalry_user
--

ALTER SEQUENCE receivables.payment_applications_serial_id_seq OWNED BY receivables.payment_applications.serial_id;
--
-- Name: payments_serial_id_seq; Type: SEQUENCE; Schema: receivables; Owner: chiefalry_user
--

CREATE SEQUENCE receivables.payments_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE receivables.payments_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: payments_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: receivables; Owner: chiefalry_user
--

ALTER SEQUENCE receivables.payments_serial_id_seq OWNED BY receivables.payments.serial_id;
--
-- Name: credit_notes serial_id; Type: DEFAULT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.credit_notes
ALTER COLUMN serial_id
SET DEFAULT nextval(
        'receivables.credit_notes_serial_id_seq'::regclass
    );
--
-- Name: customers serial_id; Type: DEFAULT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.customers
ALTER COLUMN serial_id
SET DEFAULT nextval('receivables.customers_serial_id_seq'::regclass);
--
-- Name: invoice_items serial_id; Type: DEFAULT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.invoice_items
ALTER COLUMN serial_id
SET DEFAULT nextval(
        'receivables.invoice_items_serial_id_seq'::regclass
    );
--
-- Name: invoices serial_id; Type: DEFAULT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.invoices
ALTER COLUMN serial_id
SET DEFAULT nextval('receivables.invoices_serial_id_seq'::regclass);
--
-- Name: payment_applications serial_id; Type: DEFAULT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.payment_applications
ALTER COLUMN serial_id
SET DEFAULT nextval(
        'receivables.payment_applications_serial_id_seq'::regclass
    );
--
-- Name: payments serial_id; Type: DEFAULT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.payments
ALTER COLUMN serial_id
SET DEFAULT nextval('receivables.payments_serial_id_seq'::regclass);
--
-- Name: credit_notes credit_notes_credit_number_key; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.credit_notes
ADD CONSTRAINT credit_notes_credit_number_key UNIQUE (credit_number);
--
-- Name: credit_notes credit_notes_pkey; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.credit_notes
ADD CONSTRAINT credit_notes_pkey PRIMARY KEY (uuid);
--
-- Name: credit_notes credit_notes_serial_id_key; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.credit_notes
ADD CONSTRAINT credit_notes_serial_id_key UNIQUE (serial_id);
--
-- Name: customers customers_code_key; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.customers
ADD CONSTRAINT customers_code_key UNIQUE (code);
--
-- Name: customers customers_pkey; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.customers
ADD CONSTRAINT customers_pkey PRIMARY KEY (uuid);
--
-- Name: customers customers_serial_id_key; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.customers
ADD CONSTRAINT customers_serial_id_key UNIQUE (serial_id);
--
-- Name: invoice_items invoice_items_pkey; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.invoice_items
ADD CONSTRAINT invoice_items_pkey PRIMARY KEY (uuid);
--
-- Name: invoice_items invoice_items_serial_id_key; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.invoice_items
ADD CONSTRAINT invoice_items_serial_id_key UNIQUE (serial_id);
--
-- Name: invoices invoices_invoice_number_key; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.invoices
ADD CONSTRAINT invoices_invoice_number_key UNIQUE (invoice_number);
--
-- Name: invoices invoices_pkey; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.invoices
ADD CONSTRAINT invoices_pkey PRIMARY KEY (uuid);
--
-- Name: invoices invoices_serial_id_key; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.invoices
ADD CONSTRAINT invoices_serial_id_key UNIQUE (serial_id);
--
-- Name: payment_applications payment_applications_payment_uuid_invoice_uuid_key; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.payment_applications
ADD CONSTRAINT payment_applications_payment_uuid_invoice_uuid_key UNIQUE (payment_uuid, invoice_uuid);
--
-- Name: payment_applications payment_applications_pkey; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.payment_applications
ADD CONSTRAINT payment_applications_pkey PRIMARY KEY (uuid);
--
-- Name: payment_applications payment_applications_serial_id_key; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.payment_applications
ADD CONSTRAINT payment_applications_serial_id_key UNIQUE (serial_id);
--
-- Name: payments payments_payment_number_key; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.payments
ADD CONSTRAINT payments_payment_number_key UNIQUE (payment_number);
--
-- Name: payments payments_pkey; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.payments
ADD CONSTRAINT payments_pkey PRIMARY KEY (uuid);
--
-- Name: payments payments_serial_id_key; Type: CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.payments
ADD CONSTRAINT payments_serial_id_key UNIQUE (serial_id);
--
-- Name: credit_notes_credit_number_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX credit_notes_credit_number_idx ON receivables.credit_notes USING btree (credit_number);
--
-- Name: credit_notes_customer_uuid_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX credit_notes_customer_uuid_idx ON receivables.credit_notes USING btree (customer_uuid);
--
-- Name: credit_notes_reference_invoice_uuid_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX credit_notes_reference_invoice_uuid_idx ON receivables.credit_notes USING btree (reference_invoice_uuid);
--
-- Name: credit_notes_serial_id_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX credit_notes_serial_id_idx ON receivables.credit_notes USING btree (serial_id);
--
-- Name: customers_code_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX customers_code_idx ON receivables.customers USING btree (code);
--
-- Name: customers_name_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX customers_name_idx ON receivables.customers USING btree (name);
--
-- Name: customers_serial_id_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX customers_serial_id_idx ON receivables.customers USING btree (serial_id);
--
-- Name: idx_invoices_customer_uuid; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX idx_invoices_customer_uuid ON receivables.invoices USING btree (customer_uuid);
--
-- Name: idx_payments_customer_uuid; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX idx_payments_customer_uuid ON receivables.payments USING btree (customer_uuid);
--
-- Name: idx_payments_date_desc; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX idx_payments_date_desc ON receivables.payments USING btree (payment_date DESC);
--
-- Name: invoice_items_invoice_uuid_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX invoice_items_invoice_uuid_idx ON receivables.invoice_items USING btree (invoice_uuid);
--
-- Name: invoice_items_serial_id_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX invoice_items_serial_id_idx ON receivables.invoice_items USING btree (serial_id);
--
-- Name: invoices_customer_uuid_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX invoices_customer_uuid_idx ON receivables.invoices USING btree (customer_uuid);
--
-- Name: invoices_due_date_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX invoices_due_date_idx ON receivables.invoices USING btree (due_date);
--
-- Name: invoices_invoice_number_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX invoices_invoice_number_idx ON receivables.invoices USING btree (invoice_number);
--
-- Name: invoices_issue_date_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX invoices_issue_date_idx ON receivables.invoices USING btree (issue_date);
--
-- Name: invoices_serial_id_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX invoices_serial_id_idx ON receivables.invoices USING btree (serial_id);
--
-- Name: invoices_status_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX invoices_status_idx ON receivables.invoices USING btree (status);
--
-- Name: payment_applications_invoice_uuid_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX payment_applications_invoice_uuid_idx ON receivables.payment_applications USING btree (invoice_uuid);
--
-- Name: payment_applications_payment_uuid_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX payment_applications_payment_uuid_idx ON receivables.payment_applications USING btree (payment_uuid);
--
-- Name: payment_applications_serial_id_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX payment_applications_serial_id_idx ON receivables.payment_applications USING btree (serial_id);
--
-- Name: payments_customer_uuid_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX payments_customer_uuid_idx ON receivables.payments USING btree (customer_uuid);
--
-- Name: payments_payment_date_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX payments_payment_date_idx ON receivables.payments USING btree (payment_date);
--
-- Name: payments_payment_number_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX payments_payment_number_idx ON receivables.payments USING btree (payment_number);
--
-- Name: payments_serial_id_idx; Type: INDEX; Schema: receivables; Owner: chiefalry_user
--

CREATE INDEX payments_serial_id_idx ON receivables.payments USING btree (serial_id);
--
-- Name: credit_notes credit_notes_customer_uuid_fkey; Type: FK CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.credit_notes
ADD CONSTRAINT credit_notes_customer_uuid_fkey FOREIGN KEY (customer_uuid) REFERENCES receivables.customers(uuid);
--
-- Name: credit_notes credit_notes_gl_transaction_uuid_fkey; Type: FK CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.credit_notes
ADD CONSTRAINT credit_notes_gl_transaction_uuid_fkey FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid);
--
-- Name: credit_notes credit_notes_reference_invoice_uuid_fkey; Type: FK CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.credit_notes
ADD CONSTRAINT credit_notes_reference_invoice_uuid_fkey FOREIGN KEY (reference_invoice_uuid) REFERENCES receivables.invoices(uuid);
--
-- Name: invoice_items invoice_items_invoice_uuid_fkey; Type: FK CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.invoice_items
ADD CONSTRAINT invoice_items_invoice_uuid_fkey FOREIGN KEY (invoice_uuid) REFERENCES receivables.invoices(uuid) ON DELETE CASCADE;
--
-- Name: invoices invoices_customer_uuid_fkey; Type: FK CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.invoices
ADD CONSTRAINT invoices_customer_uuid_fkey FOREIGN KEY (customer_uuid) REFERENCES receivables.customers(uuid) ON DELETE RESTRICT;
--
-- Name: invoices invoices_gl_transaction_uuid_fkey; Type: FK CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.invoices
ADD CONSTRAINT invoices_gl_transaction_uuid_fkey FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid);
--
-- Name: payment_applications payment_applications_invoice_uuid_fkey; Type: FK CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.payment_applications
ADD CONSTRAINT payment_applications_invoice_uuid_fkey FOREIGN KEY (invoice_uuid) REFERENCES receivables.invoices(uuid) ON DELETE CASCADE;
--
-- Name: payment_applications payment_applications_payment_uuid_fkey; Type: FK CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.payment_applications
ADD CONSTRAINT payment_applications_payment_uuid_fkey FOREIGN KEY (payment_uuid) REFERENCES receivables.payments(uuid) ON DELETE CASCADE;
--
-- Name: payments payments_customer_uuid_fkey; Type: FK CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.payments
ADD CONSTRAINT payments_customer_uuid_fkey FOREIGN KEY (customer_uuid) REFERENCES receivables.customers(uuid);
--
-- Name: payments payments_gl_transaction_uuid_fkey; Type: FK CONSTRAINT; Schema: receivables; Owner: chiefalry_user
--

ALTER TABLE ONLY receivables.payments
ADD CONSTRAINT payments_gl_transaction_uuid_fkey FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid);
--
-- PostgreSQL database dump complete
--