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
-- Name: accounting; Type: SCHEMA; Schema: -; Owner: chiefalry_user
--

CREATE SCHEMA accounting;
ALTER SCHEMA accounting OWNER TO chiefalry_user;
--
-- Name: post_transaction(date, text, text, bigint, text, jsonb); Type: FUNCTION; Schema: accounting; Owner: chiefalry_user
--

CREATE FUNCTION accounting.post_transaction(
    p_txn_date date,
    p_reference text,
    p_description text,
    p_created_by bigint,
    p_module text,
    p_lines jsonb
) RETURNS bigint LANGUAGE plpgsql AS $$
DECLARE v_txn_uuid UUID;
v_txn_serial_id BIGINT;
v_total_debits NUMERIC(18, 2) := 0;
v_total_credits NUMERIC(18, 2) := 0;
v_line RECORD;
v_account_uuid UUID;
v_account_serial_id BIGINT;
v_line_no INT := 0;
BEGIN -- Validate balance from JSON
FOR v_line IN
SELECT *
FROM jsonb_to_recordset(p_lines) AS t(
        account_ref JSONB,
        debit NUMERIC,
        credit NUMERIC,
        memo TEXT
    ) LOOP v_total_debits := v_total_debits + COALESCE(v_line.debit, 0);
v_total_credits := v_total_credits + COALESCE(v_line.credit, 0);
END LOOP;
IF v_total_debits <> v_total_credits THEN RAISE EXCEPTION 'Unbalanced transaction: debits (%) != credits (%)',
v_total_debits,
v_total_credits;
END IF;
-- Insert transaction header
INSERT INTO accounting.transactions(
        txn_date,
        reference,
        description,
        created_by,
        module
    )
VALUES (
        p_txn_date,
        p_reference,
        p_description,
        p_created_by,
        p_module
    )
RETURNING uuid,
    serial_id INTO v_txn_uuid,
    v_txn_serial_id;
-- Insert entries
FOR v_line IN
SELECT *
FROM jsonb_to_recordset(p_lines) AS t(
        account_ref JSONB,
        debit NUMERIC,
        credit NUMERIC,
        memo TEXT
    ) LOOP v_line_no := v_line_no + 1;
-- Resolve account_ref: can be code (text) or serial_id (number)
IF jsonb_typeof(v_line.account_ref) = 'string' THEN -- Look up by code
SELECT uuid INTO v_account_uuid
FROM accounting.accounts
WHERE code = (v_line.account_ref)::TEXT;
IF v_account_uuid IS NULL THEN RAISE EXCEPTION 'Account with code % not found',
v_line.account_ref;
END IF;
ELSIF jsonb_typeof(v_line.account_ref) = 'number' THEN -- Look up by serial_id
SELECT uuid INTO v_account_uuid
FROM accounting.accounts
WHERE serial_id = (v_line.account_ref)::BIGINT;
IF v_account_uuid IS NULL THEN RAISE EXCEPTION 'Account with serial_id % not found',
v_line.account_ref;
END IF;
ELSE RAISE EXCEPTION 'Invalid account_ref type: must be string (code) or number (serial_id)';
END IF;
-- Insert line
INSERT INTO accounting.transaction_entries(
        transaction_uuid,
        account_uuid,
        line_no,
        amount,
        debit,
        credit,
        memo
    )
VALUES (
        v_txn_uuid,
        v_account_uuid,
        v_line_no,
        COALESCE(v_line.debit, 0) + COALESCE(v_line.credit, 0),
        COALESCE(v_line.debit, 0),
        COALESCE(v_line.credit, 0),
        v_line.memo
    );
END LOOP;
-- Return the front-end friendly serial_id
RETURN v_txn_serial_id;
END;
$$;
ALTER FUNCTION accounting.post_transaction(
    p_txn_date date,
    p_reference text,
    p_description text,
    p_created_by bigint,
    p_module text,
    p_lines jsonb
) OWNER TO chiefalry_user;
SET default_tablespace = '';
SET default_table_access_method = heap;
--
-- Name: accounts; Type: TABLE; Schema: accounting; Owner: chiefalry_user
--

CREATE TABLE accounting.accounts (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    type text NOT NULL,
    parent_uuid uuid,
    normal_balance text NOT NULL,
    is_contra boolean DEFAULT false,
    is_active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    subtype text
);
ALTER TABLE accounting.accounts OWNER TO chiefalry_user;
--
-- Name: accounts_serial_id_seq; Type: SEQUENCE; Schema: accounting; Owner: chiefalry_user
--

CREATE SEQUENCE accounting.accounts_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE accounting.accounts_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: accounts_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: accounting; Owner: chiefalry_user
--

ALTER SEQUENCE accounting.accounts_serial_id_seq OWNED BY accounting.accounts.serial_id;
--
-- Name: transaction_entries; Type: TABLE; Schema: accounting; Owner: chiefalry_user
--

CREATE TABLE accounting.transaction_entries (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    transaction_uuid uuid NOT NULL,
    account_uuid uuid NOT NULL,
    line_no integer NOT NULL,
    amount numeric(18, 2) NOT NULL,
    debit numeric(18, 2) DEFAULT 0 NOT NULL,
    credit numeric(18, 2) DEFAULT 0 NOT NULL,
    memo text,
    created_at timestamp with time zone DEFAULT now(),
    CONSTRAINT chk_amount_eq_debit_plus_credit CHECK ((amount = (debit + credit))),
    CONSTRAINT transaction_entries_amount_check CHECK ((amount >= (0)::numeric)),
    CONSTRAINT transaction_entries_credit_check CHECK ((credit >= (0)::numeric)),
    CONSTRAINT transaction_entries_debit_check CHECK ((debit >= (0)::numeric))
);
ALTER TABLE accounting.transaction_entries OWNER TO chiefalry_user;
--
-- Name: transaction_entries_serial_id_seq; Type: SEQUENCE; Schema: accounting; Owner: chiefalry_user
--

CREATE SEQUENCE accounting.transaction_entries_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE accounting.transaction_entries_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: transaction_entries_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: accounting; Owner: chiefalry_user
--

ALTER SEQUENCE accounting.transaction_entries_serial_id_seq OWNED BY accounting.transaction_entries.serial_id;
--
-- Name: transactions; Type: TABLE; Schema: accounting; Owner: chiefalry_user
--

CREATE TABLE accounting.transactions (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    txn_date date NOT NULL,
    reference text,
    description text,
    created_by bigint,
    created_at timestamp with time zone DEFAULT now(),
    posted boolean DEFAULT true,
    module text
);
ALTER TABLE accounting.transactions OWNER TO chiefalry_user;
--
-- Name: transactions_serial_id_seq; Type: SEQUENCE; Schema: accounting; Owner: chiefalry_user
--

CREATE SEQUENCE accounting.transactions_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE accounting.transactions_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: transactions_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: accounting; Owner: chiefalry_user
--

ALTER SEQUENCE accounting.transactions_serial_id_seq OWNED BY accounting.transactions.serial_id;
--
-- Name: accounts serial_id; Type: DEFAULT; Schema: accounting; Owner: chiefalry_user
--

ALTER TABLE ONLY accounting.accounts
ALTER COLUMN serial_id
SET DEFAULT nextval('accounting.accounts_serial_id_seq'::regclass);
--
-- Name: transaction_entries serial_id; Type: DEFAULT; Schema: accounting; Owner: chiefalry_user
--

ALTER TABLE ONLY accounting.transaction_entries
ALTER COLUMN serial_id
SET DEFAULT nextval(
        'accounting.transaction_entries_serial_id_seq'::regclass
    );
--
-- Name: transactions serial_id; Type: DEFAULT; Schema: accounting; Owner: chiefalry_user
--

ALTER TABLE ONLY accounting.transactions
ALTER COLUMN serial_id
SET DEFAULT nextval(
        'accounting.transactions_serial_id_seq'::regclass
    );
--
-- Name: accounts accounts_code_key; Type: CONSTRAINT; Schema: accounting; Owner: chiefalry_user
--

ALTER TABLE ONLY accounting.accounts
ADD CONSTRAINT accounts_code_key UNIQUE (code);
--
-- Name: accounts accounts_pkey; Type: CONSTRAINT; Schema: accounting; Owner: chiefalry_user
--

ALTER TABLE ONLY accounting.accounts
ADD CONSTRAINT accounts_pkey PRIMARY KEY (uuid);
--
-- Name: accounts accounts_serial_id_key; Type: CONSTRAINT; Schema: accounting; Owner: chiefalry_user
--

ALTER TABLE ONLY accounting.accounts
ADD CONSTRAINT accounts_serial_id_key UNIQUE (serial_id);
--
-- Name: transaction_entries transaction_entries_pkey; Type: CONSTRAINT; Schema: accounting; Owner: chiefalry_user
--

ALTER TABLE ONLY accounting.transaction_entries
ADD CONSTRAINT transaction_entries_pkey PRIMARY KEY (uuid);
--
-- Name: transaction_entries transaction_entries_serial_id_key; Type: CONSTRAINT; Schema: accounting; Owner: chiefalry_user
--

ALTER TABLE ONLY accounting.transaction_entries
ADD CONSTRAINT transaction_entries_serial_id_key UNIQUE (serial_id);
--
-- Name: transactions transactions_pkey; Type: CONSTRAINT; Schema: accounting; Owner: chiefalry_user
--

ALTER TABLE ONLY accounting.transactions
ADD CONSTRAINT transactions_pkey PRIMARY KEY (uuid);
--
-- Name: transactions transactions_serial_id_key; Type: CONSTRAINT; Schema: accounting; Owner: chiefalry_user
--

ALTER TABLE ONLY accounting.transactions
ADD CONSTRAINT transactions_serial_id_key UNIQUE (serial_id);
--
-- Name: accounts_code_idx; Type: INDEX; Schema: accounting; Owner: chiefalry_user
--

CREATE INDEX accounts_code_idx ON accounting.accounts USING btree (code);
--
-- Name: accounts_serial_id_idx; Type: INDEX; Schema: accounting; Owner: chiefalry_user
--

CREATE INDEX accounts_serial_id_idx ON accounting.accounts USING btree (serial_id);
--
-- Name: accounts_type_idx; Type: INDEX; Schema: accounting; Owner: chiefalry_user
--

CREATE INDEX accounts_type_idx ON accounting.accounts USING btree (type);
--
-- Name: transaction_entries_account_uuid_idx; Type: INDEX; Schema: accounting; Owner: chiefalry_user
--

CREATE INDEX transaction_entries_account_uuid_idx ON accounting.transaction_entries USING btree (account_uuid);
--
-- Name: transaction_entries_created_at_idx; Type: INDEX; Schema: accounting; Owner: chiefalry_user
--

CREATE INDEX transaction_entries_created_at_idx ON accounting.transaction_entries USING btree (created_at);
--
-- Name: transaction_entries_serial_id_idx; Type: INDEX; Schema: accounting; Owner: chiefalry_user
--

CREATE INDEX transaction_entries_serial_id_idx ON accounting.transaction_entries USING btree (serial_id);
--
-- Name: transaction_entries_transaction_uuid_idx; Type: INDEX; Schema: accounting; Owner: chiefalry_user
--

CREATE INDEX transaction_entries_transaction_uuid_idx ON accounting.transaction_entries USING btree (transaction_uuid);
--
-- Name: transactions_module_idx; Type: INDEX; Schema: accounting; Owner: chiefalry_user
--

CREATE INDEX transactions_module_idx ON accounting.transactions USING btree (module);
--
-- Name: transactions_serial_id_idx; Type: INDEX; Schema: accounting; Owner: chiefalry_user
--

CREATE INDEX transactions_serial_id_idx ON accounting.transactions USING btree (serial_id);
--
-- Name: transactions_txn_date_idx; Type: INDEX; Schema: accounting; Owner: chiefalry_user
--

CREATE INDEX transactions_txn_date_idx ON accounting.transactions USING btree (txn_date);
--
-- Name: transactions trg_audit_transactions; Type: TRIGGER; Schema: accounting; Owner: chiefalry_user
--

CREATE TRIGGER trg_audit_transactions
AFTER
INSERT
    OR DELETE
    OR
UPDATE ON accounting.transactions FOR EACH ROW EXECUTE FUNCTION system.audit_trigger_fn();
--
-- Name: transaction_entries trg_audit_tx_entries; Type: TRIGGER; Schema: accounting; Owner: chiefalry_user
--

CREATE TRIGGER trg_audit_tx_entries
AFTER
INSERT
    OR DELETE
    OR
UPDATE ON accounting.transaction_entries FOR EACH ROW EXECUTE FUNCTION system.audit_trigger_fn();
--
-- Name: accounts accounts_parent_uuid_fkey; Type: FK CONSTRAINT; Schema: accounting; Owner: chiefalry_user
--

ALTER TABLE ONLY accounting.accounts
ADD CONSTRAINT accounts_parent_uuid_fkey FOREIGN KEY (parent_uuid) REFERENCES accounting.accounts(uuid) ON DELETE
SET NULL;
--
-- Name: transaction_entries transaction_entries_account_uuid_fkey; Type: FK CONSTRAINT; Schema: accounting; Owner: chiefalry_user
--

ALTER TABLE ONLY accounting.transaction_entries
ADD CONSTRAINT transaction_entries_account_uuid_fkey FOREIGN KEY (account_uuid) REFERENCES accounting.accounts(uuid);
--
-- Name: transaction_entries transaction_entries_transaction_uuid_fkey; Type: FK CONSTRAINT; Schema: accounting; Owner: chiefalry_user
--

ALTER TABLE ONLY accounting.transaction_entries
ADD CONSTRAINT transaction_entries_transaction_uuid_fkey FOREIGN KEY (transaction_uuid) REFERENCES accounting.transactions(uuid) ON DELETE CASCADE;
--
-- PostgreSQL database dump complete
--