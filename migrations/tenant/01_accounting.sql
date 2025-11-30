-- Create schema
CREATE SCHEMA accounting;
-- Enable UUID extension (required for uuid_generate_v4)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- Sequences
CREATE SEQUENCE accounting.accounts_serial_id_seq;
CREATE SEQUENCE accounting.transactions_serial_id_seq;
CREATE SEQUENCE accounting.transaction_entries_serial_id_seq;

-- Enable the extension once (if not already enabled)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-----------------------------------------------------------------
-- accounts: chart of accounts
-----------------------------------------------------------------
CREATE TABLE accounting.accounts (
    uuid       UUID        DEFAULT uuid_generate_v4() PRIMARY KEY,
    serial_id  BIGSERIAL   NOT NULL UNIQUE,               -- front-end id

    code       TEXT        NOT NULL UNIQUE,               -- e.g. "110100"
    name       TEXT        NOT NULL,
    type       TEXT        NOT NULL,                     -- Asset, Liability, Equity, Revenue, Expense
    parent_code TEXT        REFERENCES accounting.accounts(code)
                           ON DELETE SET NULL,           -- FK uses UUID
    normal_balance TEXT    NOT NULL,                     -- 'DR' or 'CR'
    is_contra  BOOLEAN     DEFAULT FALSE,
    is_active  BOOLEAN     DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT now()
);

-- Indexes (keep the ones you need)
CREATE INDEX ON accounting.accounts (type);
CREATE INDEX ON accounting.accounts (code);
CREATE INDEX ON accounting.accounts (serial_id);   -- handy for front-end look-ups

-----------------------------------------------------------------
-- transactions: header/journal
-----------------------------------------------------------------
CREATE TABLE accounting.transactions (
    uuid       UUID        DEFAULT uuid_generate_v4() PRIMARY KEY,
    serial_id  BIGSERIAL   NOT NULL UNIQUE,               -- front-end id

    txn_date   DATE        NOT NULL,
    reference  TEXT,
    description TEXT,
    created_by BIGINT,                                 -- fk to system.users (keep your own PK type)
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now(),
    posted     BOOLEAN     DEFAULT TRUE,                -- drafts/approval
    module     TEXT                                    -- e.g., 'invoice', 'payment', 'journal'
);

CREATE INDEX ON accounting.transactions (txn_date);
CREATE INDEX ON accounting.transactions (module);
CREATE INDEX ON accounting.transactions (serial_id);

-----------------------------------------------------------------
-- transaction entries (double-entry lines)
-----------------------------------------------------------------
CREATE TABLE accounting.transaction_entries (
    uuid            UUID        DEFAULT uuid_generate_v4() PRIMARY KEY,
    serial_id       BIGSERIAL   NOT NULL UNIQUE,           -- front-end id

    transaction_uuid UUID       NOT NULL
                        REFERENCES accounting.transactions(uuid)
                        ON DELETE CASCADE,                 -- FK uses UUID
    account_uuid    UUID        NOT NULL
                        REFERENCES accounting.accounts(uuid), -- FK uses UUID
    line_no         INT         NOT NULL,
    amount          NUMERIC(18,2) NOT NULL CHECK (amount >= 0),
    debit           NUMERIC(18,2) NOT NULL DEFAULT 0 CHECK (debit >= 0),
    credit          NUMERIC(18,2) NOT NULL DEFAULT 0 CHECK (credit >= 0),
    memo            TEXT,
    created_at      TIMESTAMP WITH TIME ZONE DEFAULT now()
);

CREATE INDEX ON accounting.transaction_entries (account_uuid);
CREATE INDEX ON accounting.transaction_entries (transaction_uuid);
CREATE INDEX ON accounting.transaction_entries (created_at);
CREATE INDEX ON accounting.transaction_entries (serial_id);

CREATE OR REPLACE FUNCTION accounting.post_transaction(
    p_txn_date DATE,
    p_reference TEXT,
    p_description TEXT,
    p_created_by BIGINT,
    p_module TEXT,
    p_lines JSONB -- array of {account_ref: "110100" or 42, debit: 100.00, credit: 0, memo: "xyz"}
) RETURNS BIGINT  -- returns serial_id of the transaction
LANGUAGE plpgsql
AS $$
DECLARE
    v_txn_uuid UUID;
    v_txn_serial_id BIGINT;
    v_total_debits NUMERIC(18,2) := 0;
    v_total_credits NUMERIC(18,2) := 0;
    v_line RECORD;
    v_account_uuid UUID;
    v_account_serial_id BIGINT;
    v_line_no INT := 0;
BEGIN
    -- Validate balance from JSON
    FOR v_line IN SELECT * FROM jsonb_to_recordset(p_lines)
        AS t(account_ref JSONB, debit NUMERIC, credit NUMERIC, memo TEXT)
    LOOP
        v_total_debits := v_total_debits + COALESCE(v_line.debit, 0);
        v_total_credits := v_total_credits + COALESCE(v_line.credit, 0);
    END LOOP;

    IF v_total_debits <> v_total_credits THEN
        RAISE EXCEPTION 'Unbalanced transaction: debits (%) != credits (%)', v_total_debits, v_total_credits;
    END IF;

    -- Insert transaction header
    INSERT INTO accounting.transactions(
        txn_date, reference, description, created_by, module
    )
    VALUES (
        p_txn_date, p_reference, p_description, p_created_by, p_module
    )
    RETURNING uuid, serial_id INTO v_txn_uuid, v_txn_serial_id;

    -- Insert entries
    FOR v_line IN SELECT * FROM jsonb_to_recordset(p_lines)
        AS t(account_ref JSONB, debit NUMERIC, credit NUMERIC, memo TEXT)
    LOOP
        v_line_no := v_line_no + 1;

        -- Resolve account_ref: can be code (text) or serial_id (number)
        IF jsonb_typeof(v_line.account_ref) = 'string' THEN
            -- Look up by code
            SELECT uuid INTO v_account_uuid
            FROM accounting.accounts
            WHERE code = (v_line.account_ref)::TEXT;

            IF v_account_uuid IS NULL THEN
                RAISE EXCEPTION 'Account with code % not found', v_line.account_ref;
            END IF;

        ELSIF jsonb_typeof(v_line.account_ref) = 'number' THEN
            -- Look up by serial_id
            SELECT uuid INTO v_account_uuid
            FROM accounting.accounts
            WHERE serial_id = (v_line.account_ref)::BIGINT;

            IF v_account_uuid IS NULL THEN
                RAISE EXCEPTION 'Account with serial_id % not found', v_line.account_ref;
            END IF;

        ELSE
            RAISE EXCEPTION 'Invalid account_ref type: must be string (code) or number (serial_id)';
        END IF;

        -- Insert line
        INSERT INTO accounting.transaction_entries(
            transaction_uuid, account_uuid, line_no,
            amount, debit, credit, memo
        ) VALUES (
            v_txn_uuid, v_account_uuid, v_line_no,
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

--
--SELECT accounting.post_transaction(
--    '2025-04-01',
--    'INV-001',
--    'Customer invoice',
--    1,
--    'invoice',
--    '[
--        {"account_ref": "1.1.01", "debit": 1000.00, "credit": 0, "memo": "Sales"},
--        {"account_ref": 42,       "debit": 0,      "credit": 1000.00, "memo": "AR"}
--    ]'::jsonb
--);
