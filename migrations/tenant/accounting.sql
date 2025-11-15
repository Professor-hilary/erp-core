-- Create schema
CREATE SCHEMA accounting;
-- Enable UUID extension (required for uuid_generate_v4)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- Sequences
CREATE SEQUENCE accounting.accounts_serial_id_seq;
CREATE SEQUENCE accounting.transactions_serial_id_seq;
CREATE SEQUENCE accounting.transaction_entries_serial_id_seq;
-- Table: accounts
CREATE TABLE accounting.accounts (
    uuid uuid DEFAULT uuid_generate_v4() NOT NULL,
    serial_id bigint DEFAULT nextval('accounting.accounts_serial_id_seq') NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    category text NOT NULL,
    parent_serial INT REFERENCES accounts(serial) ON DELETE SET NULL,
    normal_balance text NOT NULL,
    is_contra boolean DEFAULT false,
    is_active boolean DEFAULT false,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    CONSTRAINT accounts_pkey PRIMARY KEY (uuid),
    CONSTRAINT accounts_code_key UNIQUE (code),
    CONSTRAINT accounts_serial_id_key UNIQUE (serial_id),
    CONSTRAINT accounts_parent_serial_fkey FOREIGN KEY (parent_serial) REFERENCES accounting.accounts(serial_id) ON DELETE
    SET NULL
);
-- Indexes for accounts
CREATE INDEX accounts_code_idx ON accounting.accounts USING btree (code);
CREATE INDEX accounts_type_idx ON accounting.accounts USING btree (type);
-- Table: transactions
CREATE TABLE accounting.transactions (
    uuid uuid DEFAULT uuid_generate_v4() NOT NULL,
    serial_id bigint DEFAULT nextval('accounting.transactions_serial_id_seq') NOT NULL,
    txn_date date NOT NULL,
    reference text,
    description text,
    created_by uuid REFERENCES master_schema.users(uuid),
    created_at timestamp with time zone DEFAULT now(),
    posted boolean DEFAULT true,
    module text,
    posted_by uuid REFERENCES master_schema.users(uuid),
    posted_at timestamptz CONSTRAINT transactions_pkey PRIMARY KEY (uuid),
    CONSTRAINT transactions_serial_id_key UNIQUE (serial_id)
);
-- Indexes for transactions
CREATE INDEX transactions_txn_date_idx ON accounting.transactions USING btree (txn_date);
CREATE INDEX transactions_module_idx ON accounting.transactions USING btree (module);
-- Table: transaction_entries
CREATE TABLE accounting.transaction_entries (
    uuid uuid DEFAULT uuid_generate_v4() NOT NULL,
    serial_id bigint DEFAULT nextval('accounting.transaction_entries_serial_id_seq') NOT NULL,
    transaction_uuid uuid NOT NULL,
    account_uuid uuid NOT NULL,
    line_no integer NOT NULL,
    amount numeric(18, 2) NOT NULL,
    debit numeric(18, 2) DEFAULT 0 NOT NULL,
    credit numeric(18, 2) DEFAULT 0 NOT NULL,
    memo text,
    created_at timestamp with time zone DEFAULT now(),
    CONSTRAINT transaction_entries_pkey PRIMARY KEY (uuid),
    CONSTRAINT transaction_entries_serial_id_key UNIQUE (serial_id),
    CONSTRAINT chk_amount_eq_debit_plus_credit CHECK (amount = debit + credit),
    CONSTRAINT transaction_entries_amount_check CHECK (amount >= 0),
    CONSTRAINT transaction_entries_debit_check CHECK (debit >= 0),
    CONSTRAINT transaction_entries_credit_check CHECK (credit >= 0),
    CONSTRAINT transaction_entries_account_uuid_fkey FOREIGN KEY (account_uuid) REFERENCES accounting.accounts(uuid),
    CONSTRAINT transaction_entries_transaction_uuid_fkey FOREIGN KEY (transaction_uuid) REFERENCES accounting.transactions(uuid) ON DELETE CASCADE
);
-- Indexes for transaction_entries
CREATE INDEX transaction_entries_transaction_uuid_idx ON accounting.transaction_entries USING btree (transaction_uuid);
CREATE INDEX transaction_entries_account_uuid_idx ON accounting.transaction_entries USING btree (account_uuid);
CREATE INDEX transaction_entries_created_at_idx ON accounting.transaction_entries USING btree (created_at);
-- Function: post_transaction
CREATE OR REPLACE FUNCTION accounting.post_transaction(
        p_txn_date date,
        p_reference text,
        p_description text,
        p_created_by uuid,
        p_module text,
        p_lines jsonb
    ) RETURNS bigint LANGUAGE plpgsql AS $$
DECLARE v_txn_uuid UUID;
v_txn_serial_id BIGINT;
v_total_debits NUMERIC(18, 2) := 0;
v_total_credits NUMERIC(18, 2) := 0;
v_line RECORD;
v_account_uuid UUID;
v_line_no INT := 0;
BEGIN -- Validate balance
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
-- Resolve account_ref
IF jsonb_typeof(v_line.account_ref) = 'string' THEN
SELECT uuid INTO v_account_uuid
FROM accounting.accounts
WHERE code = (v_line.account_ref)::TEXT;
IF v_account_uuid IS NULL THEN RAISE EXCEPTION 'Account with code % not found',
v_line.account_ref;
END IF;
ELSIF jsonb_typeof(v_line.account_ref) = 'number' THEN
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
RETURN v_txn_serial_id;
END;
$$;