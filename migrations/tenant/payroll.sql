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
-- Name: payroll; Type: SCHEMA; Schema: -; Owner: chiefalry_user
--

CREATE SCHEMA payroll;
ALTER SCHEMA payroll OWNER TO chiefalry_user;
--
-- Name: post_payrun(bigint); Type: FUNCTION; Schema: payroll; Owner: chiefalry_user
--

CREATE FUNCTION payroll.post_payrun(p_payrun_serial_id bigint) RETURNS void LANGUAGE plpgsql AS $$
DECLARE v_payrun payroll.payruns %ROWTYPE;
v_total_gross NUMERIC(14, 2) := 0;
v_total_tax NUMERIC(14, 2) := 0;
v_total_nssf NUMERIC(14, 2) := 0;
v_total_net NUMERIC(14, 2) := 0;
v_txn_serial_id BIGINT;
v_txn_uuid UUID;
v_lines JSONB;
BEGIN -- Fetch payrun by serial_id
SELECT * INTO v_payrun
FROM payroll.payruns
WHERE serial_id = p_payrun_serial_id;
IF NOT FOUND THEN RAISE EXCEPTION 'Payrun with serial_id % not found',
p_payrun_serial_id;
END IF;
IF v_payrun.status = 'Posted' THEN RAISE NOTICE 'Payrun % already posted',
v_payrun.serial_id;
RETURN;
END IF;
IF v_payrun.status != 'Processed' THEN RAISE EXCEPTION 'Payrun must be Processed before posting. Current status: %',
v_payrun.status;
END IF;
-- Aggregate payslip totals
SELECT COALESCE(SUM(gross_pay), 0),
    COALESCE(SUM(tax_deducted), 0),
    COALESCE(SUM(nssf), 0),
    COALESCE(SUM(net_pay), 0) INTO v_total_gross,
    v_total_tax,
    v_total_nssf,
    v_total_net
FROM payroll.payslips
WHERE payrun_uuid = v_payrun.uuid;
IF v_total_net <= 0 THEN RAISE EXCEPTION 'Net pay total is zero or negative. Cannot post.';
END IF;
-- Build GL lines using account codes (as strings)
v_lines := jsonb_build_array(
    -- Debit: Salaries Expense
    jsonb_build_object(
        'account_ref',
        '5.1.1',
        -- adjust to your chart
        'debit',
        v_total_gross,
        'credit',
        0,
        'memo',
        format('Payroll Gross - Payrun %s', v_payrun.serial_id)
    ),
    -- Credit: PAYE Payable
    jsonb_build_object(
        'account_ref',
        '2.1.2',
        'debit',
        0,
        'credit',
        v_total_tax,
        'memo',
        'PAYE Withholding'
    ),
    -- Credit: NSSF Payable
    jsonb_build_object(
        'account_ref',
        '2.1.3',
        'debit',
        0,
        'credit',
        v_total_nssf,
        'memo',
        'NSSF Contribution'
    ),
    -- Credit: Bank/Cash (net payment)
    jsonb_build_object(
        'account_ref',
        '1.1.02',
        'debit',
        0,
        'credit',
        v_total_net,
        'memo',
        format(
            'Payroll Net Pay - Payrun %s',
            v_payrun.serial_id
        )
    )
);
-- Post to GL using shared function
v_txn_serial_id := accounting.post_transaction(
    v_payrun.payment_date,
    format(
        'Payroll - %s to %s',
        v_payrun.pay_period_start,
        v_payrun.pay_period_end
    ),
    'Payroll Posting',
    NULL,
    -- created_by (optional, set via app context if needed)
    'payroll',
    v_lines
);
-- Get the UUID of the GL transaction
SELECT uuid INTO v_txn_uuid
FROM accounting.transactions
WHERE serial_id = v_txn_serial_id;
IF v_txn_uuid IS NULL THEN RAISE EXCEPTION 'Failed to retrieve GL transaction UUID for serial_id %',
v_txn_serial_id;
END IF;
-- Update payrun: mark as Posted and link GL
UPDATE payroll.payruns
SET status = 'Posted',
    gl_transaction_uuid = v_txn_uuid,
    updated_at = now()
WHERE uuid = v_payrun.uuid;
-- Optional: mark payslips as Paid
UPDATE payroll.payslips
SET status = 'Paid'
WHERE payrun_uuid = v_payrun.uuid
    AND status = 'Pending';
END;
$$;
ALTER FUNCTION payroll.post_payrun(p_payrun_serial_id bigint) OWNER TO chiefalry_user;
SET default_tablespace = '';
SET default_table_access_method = heap;
--
-- Name: payruns; Type: TABLE; Schema: payroll; Owner: chiefalry_user
--

CREATE TABLE payroll.payruns (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    pay_period_start date NOT NULL,
    pay_period_end date NOT NULL,
    payment_date date NOT NULL,
    status character varying(20) DEFAULT 'Pending'::character varying,
    gl_transaction_uuid uuid,
    notes text,
    created_at timestamp with time zone DEFAULT now(),
    CONSTRAINT payruns_status_check CHECK (
        (
            (status)::text = ANY (
                (
                    ARRAY ['Pending'::character varying, 'Processed'::character varying, 'Posted'::character varying]
                )::text []
            )
        )
    )
);
ALTER TABLE payroll.payruns OWNER TO chiefalry_user;
--
-- Name: payruns_serial_id_seq; Type: SEQUENCE; Schema: payroll; Owner: chiefalry_user
--

CREATE SEQUENCE payroll.payruns_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE payroll.payruns_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: payruns_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: payroll; Owner: chiefalry_user
--

ALTER SEQUENCE payroll.payruns_serial_id_seq OWNED BY payroll.payruns.serial_id;
--
-- Name: payslip_items; Type: TABLE; Schema: payroll; Owner: chiefalry_user
--

CREATE TABLE payroll.payslip_items (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    payslip_uuid uuid NOT NULL,
    item_type character varying(20),
    description character varying(100),
    amount numeric(14, 2) NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    CONSTRAINT payslip_items_item_type_check CHECK (
        (
            (item_type)::text = ANY (
                (
                    ARRAY ['Allowance'::character varying, 'Deduction'::character varying]
                )::text []
            )
        )
    )
);
ALTER TABLE payroll.payslip_items OWNER TO chiefalry_user;
--
-- Name: payslip_items_serial_id_seq; Type: SEQUENCE; Schema: payroll; Owner: chiefalry_user
--

CREATE SEQUENCE payroll.payslip_items_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE payroll.payslip_items_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: payslip_items_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: payroll; Owner: chiefalry_user
--

ALTER SEQUENCE payroll.payslip_items_serial_id_seq OWNED BY payroll.payslip_items.serial_id;
--
-- Name: payslips; Type: TABLE; Schema: payroll; Owner: chiefalry_user
--

CREATE TABLE payroll.payslips (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    payrun_uuid uuid NOT NULL,
    employee_uuid uuid NOT NULL,
    gross_pay numeric(14, 2) NOT NULL,
    tax_deducted numeric(14, 2) DEFAULT 0,
    nssf numeric(14, 2) DEFAULT 0,
    other_deductions numeric(14, 2) DEFAULT 0,
    net_pay numeric(14, 2) GENERATED ALWAYS AS (
        (
            ((gross_pay - tax_deducted) - nssf) - other_deductions
        )
    ) STORED,
    payment_method character varying(30) DEFAULT 'Bank Transfer'::character varying,
    bank_account character varying(50),
    status character varying(20) DEFAULT 'Pending'::character varying,
    created_at timestamp with time zone DEFAULT now(),
    CONSTRAINT payslips_status_check CHECK (
        (
            (status)::text = ANY (
                (
                    ARRAY ['Pending'::character varying, 'Paid'::character varying]
                )::text []
            )
        )
    )
);
ALTER TABLE payroll.payslips OWNER TO chiefalry_user;
--
-- Name: payslips_serial_id_seq; Type: SEQUENCE; Schema: payroll; Owner: chiefalry_user
--

CREATE SEQUENCE payroll.payslips_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE payroll.payslips_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: payslips_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: payroll; Owner: chiefalry_user
--

ALTER SEQUENCE payroll.payslips_serial_id_seq OWNED BY payroll.payslips.serial_id;
--
-- Name: payruns serial_id; Type: DEFAULT; Schema: payroll; Owner: chiefalry_user
--

ALTER TABLE ONLY payroll.payruns
ALTER COLUMN serial_id
SET DEFAULT nextval('payroll.payruns_serial_id_seq'::regclass);
--
-- Name: payslip_items serial_id; Type: DEFAULT; Schema: payroll; Owner: chiefalry_user
--

ALTER TABLE ONLY payroll.payslip_items
ALTER COLUMN serial_id
SET DEFAULT nextval('payroll.payslip_items_serial_id_seq'::regclass);
--
-- Name: payslips serial_id; Type: DEFAULT; Schema: payroll; Owner: chiefalry_user
--

ALTER TABLE ONLY payroll.payslips
ALTER COLUMN serial_id
SET DEFAULT nextval('payroll.payslips_serial_id_seq'::regclass);
--
-- Name: payruns payruns_pkey; Type: CONSTRAINT; Schema: payroll; Owner: chiefalry_user
--

ALTER TABLE ONLY payroll.payruns
ADD CONSTRAINT payruns_pkey PRIMARY KEY (uuid);
--
-- Name: payruns payruns_serial_id_key; Type: CONSTRAINT; Schema: payroll; Owner: chiefalry_user
--

ALTER TABLE ONLY payroll.payruns
ADD CONSTRAINT payruns_serial_id_key UNIQUE (serial_id);
--
-- Name: payslip_items payslip_items_pkey; Type: CONSTRAINT; Schema: payroll; Owner: chiefalry_user
--

ALTER TABLE ONLY payroll.payslip_items
ADD CONSTRAINT payslip_items_pkey PRIMARY KEY (uuid);
--
-- Name: payslip_items payslip_items_serial_id_key; Type: CONSTRAINT; Schema: payroll; Owner: chiefalry_user
--

ALTER TABLE ONLY payroll.payslip_items
ADD CONSTRAINT payslip_items_serial_id_key UNIQUE (serial_id);
--
-- Name: payslips payslips_pkey; Type: CONSTRAINT; Schema: payroll; Owner: chiefalry_user
--

ALTER TABLE ONLY payroll.payslips
ADD CONSTRAINT payslips_pkey PRIMARY KEY (uuid);
--
-- Name: payslips payslips_serial_id_key; Type: CONSTRAINT; Schema: payroll; Owner: chiefalry_user
--

ALTER TABLE ONLY payroll.payslips
ADD CONSTRAINT payslips_serial_id_key UNIQUE (serial_id);
--
-- Name: payruns_pay_period_start_pay_period_end_idx; Type: INDEX; Schema: payroll; Owner: chiefalry_user
--

CREATE INDEX payruns_pay_period_start_pay_period_end_idx ON payroll.payruns USING btree (pay_period_start, pay_period_end);
--
-- Name: payruns_payment_date_idx; Type: INDEX; Schema: payroll; Owner: chiefalry_user
--

CREATE INDEX payruns_payment_date_idx ON payroll.payruns USING btree (payment_date);
--
-- Name: payruns_serial_id_idx; Type: INDEX; Schema: payroll; Owner: chiefalry_user
--

CREATE INDEX payruns_serial_id_idx ON payroll.payruns USING btree (serial_id);
--
-- Name: payruns_status_idx; Type: INDEX; Schema: payroll; Owner: chiefalry_user
--

CREATE INDEX payruns_status_idx ON payroll.payruns USING btree (status);
--
-- Name: payslip_items_item_type_idx; Type: INDEX; Schema: payroll; Owner: chiefalry_user
--

CREATE INDEX payslip_items_item_type_idx ON payroll.payslip_items USING btree (item_type);
--
-- Name: payslip_items_payslip_uuid_idx; Type: INDEX; Schema: payroll; Owner: chiefalry_user
--

CREATE INDEX payslip_items_payslip_uuid_idx ON payroll.payslip_items USING btree (payslip_uuid);
--
-- Name: payslip_items_serial_id_idx; Type: INDEX; Schema: payroll; Owner: chiefalry_user
--

CREATE INDEX payslip_items_serial_id_idx ON payroll.payslip_items USING btree (serial_id);
--
-- Name: payslips_employee_uuid_idx; Type: INDEX; Schema: payroll; Owner: chiefalry_user
--

CREATE INDEX payslips_employee_uuid_idx ON payroll.payslips USING btree (employee_uuid);
--
-- Name: payslips_payrun_uuid_idx; Type: INDEX; Schema: payroll; Owner: chiefalry_user
--

CREATE INDEX payslips_payrun_uuid_idx ON payroll.payslips USING btree (payrun_uuid);
--
-- Name: payslips_serial_id_idx; Type: INDEX; Schema: payroll; Owner: chiefalry_user
--

CREATE INDEX payslips_serial_id_idx ON payroll.payslips USING btree (serial_id);
--
-- Name: payslips_status_idx; Type: INDEX; Schema: payroll; Owner: chiefalry_user
--

CREATE INDEX payslips_status_idx ON payroll.payslips USING btree (status);
--
-- Name: payruns payruns_gl_transaction_uuid_fkey; Type: FK CONSTRAINT; Schema: payroll; Owner: chiefalry_user
--

ALTER TABLE ONLY payroll.payruns
ADD CONSTRAINT payruns_gl_transaction_uuid_fkey FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid);
--
-- Name: payslip_items payslip_items_payslip_uuid_fkey; Type: FK CONSTRAINT; Schema: payroll; Owner: chiefalry_user
--

ALTER TABLE ONLY payroll.payslip_items
ADD CONSTRAINT payslip_items_payslip_uuid_fkey FOREIGN KEY (payslip_uuid) REFERENCES payroll.payslips(uuid) ON DELETE CASCADE;
--
-- Name: payslips payslips_employee_uuid_fkey; Type: FK CONSTRAINT; Schema: payroll; Owner: chiefalry_user
--

ALTER TABLE ONLY payroll.payslips
ADD CONSTRAINT payslips_employee_uuid_fkey FOREIGN KEY (employee_uuid) REFERENCES hr.employees(uuid);
--
-- Name: payslips payslips_payrun_uuid_fkey; Type: FK CONSTRAINT; Schema: payroll; Owner: chiefalry_user
--

ALTER TABLE ONLY payroll.payslips
ADD CONSTRAINT payslips_payrun_uuid_fkey FOREIGN KEY (payrun_uuid) REFERENCES payroll.payruns(uuid) ON DELETE CASCADE;
--
-- PostgreSQL database dump complete
--