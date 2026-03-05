-- Create schema
CREATE SCHEMA IF NOT EXISTS accounting;

--Create extension for ltree
CREATE EXTENSION IF NOT EXISTS ltree schema public;

-- UUID extension Enabled automatically (required for uuidv7())
-- Sequences
CREATE SEQUENCE accounting.accounts_serial_id_seq;

CREATE SEQUENCE accounting.transactions_serial_id_seq;

CREATE SEQUENCE accounting.transaction_entries_serial_id_seq;

-----------------------------------------------------------------
-- accounts: chart of accounts
-----------------------------------------------------------------
CREATE TABLE accounting.accounts (
    uuid UUID DEFAULT uuidv7 () PRIMARY KEY,
    serial_id BIGSERIAL NOT NULL UNIQUE, -- front-end id
    current_balance NUMERIC(18, 2) DEFAULT 0.00 NOT NULL,
    code TEXT NOT NULL UNIQUE, -- e.g. "110100"
    name TEXT NOT NULL,
    category TEXT NOT NULL CHECK (
        category IN (
            'asset',
            'liability',
            'equity',
            'expense',
            'income'
        )
    ),
    parent_code TEXT REFERENCES accounting.accounts (code) ON DELETE SET NULL, -- FK uses UUID
    normal_balance TEXT NOT NULL CHECK (
        normal_balance IN ('cr', 'dr')
    ), -- 'DR' or 'CR'
    path public.ltree,
    hierarchy_depth smallint GENERATED ALWAYS AS (nlevel (path)) STORED,
    is_contra BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT now()
);

COMMENT ON COLUMN accounting.accounts.path IS 'Materialized hierarchical path using ltree (e.g. 1.100.1100 for asset -> current asset -> cash';

COMMENT ON COLUMN accounting.accounts.hierarchy_depth IS 'Is computed depth level (root = 1)';

-- Indexes (keep the ones you need)
CREATE INDEX ON accounting.accounts (category);

CREATE INDEX ON accounting.accounts (code);

CREATE INDEX ON accounting.accounts (serial_id);

-- Gist index: Optimized for @> (is_ancestor), <@ (is_descendant), ~(pattern), subpath, etc
CREATE INDEX idx_accounts_path_gist ON accounting.accounts USING gist (path);

-- B-tree index; fast exact matches, ordering, and prefix sorts (ORDER BY path)
CREATE INDEX idx_accounts_path_btree ON accounting.accounts USING btree (path);

-- Componsite for common reports filters
CREATE INDEX idx_accounts_category_path ON accounting.accounts USING btree (category, path)
WHERE
    is_active;

-----------------------------------------------------------------
-- Finance cycle
-----------------------------------------------------------------
CREATE TABLE accounting.financial_periods (
    uuid UUID DEFAULT uuidv7 () PRIMARY KEY,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    is_open BOOLEAN DEFAULT true,
    is_locked BOOLEAN DEFAULT false,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    name text,
    CONSTRAINT period_date_check CHECK (start_date < end_date),
    CONSTRAINT period_date_unique UNIQUE (start_date, end_date)
);

-----------------------------------------------------------------
-- transactions: header/journal
-----------------------------------------------------------------
CREATE TABLE accounting.transactions (
    uuid UUID DEFAULT uuidv7 () PRIMARY KEY,
    serial_id BIGSERIAL NOT NULL UNIQUE, -- front-end friendly id
    txn_date DATE NOT NULL,
    reference TEXT,
    description TEXT,
    posted BOOLEAN DEFAULT TRUE, -- drafts/approval
    module TEXT, -- e.g., 'invoice', 'payment', 'journal'
    created_by UUID, -- fk to system.users
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now()
);

CREATE INDEX ON accounting.transactions (txn_date);

CREATE INDEX ON accounting.transactions (module);

CREATE INDEX ON accounting.transactions (serial_id);

-----------------------------------------------------------------
-- transaction entries (double-entry lines)
-----------------------------------------------------------------
CREATE TABLE accounting.transaction_entries (
    uuid UUID DEFAULT uuidv7 () PRIMARY KEY,
    serial_id BIGSERIAL NOT NULL UNIQUE, -- front-end id
    transaction_uuid UUID NOT NULL REFERENCES accounting.transactions (uuid) ON DELETE CASCADE, -- FK uses UUID
    account_uuid UUID NOT NULL REFERENCES accounting.accounts (uuid), -- FK uses UUID
    line_no INT NOT NULL,
    amount NUMERIC(18, 2) NOT NULL, -- CHECK (amount >= 0),
    debit NUMERIC(18, 2) NOT NULL DEFAULT 0 CHECK (debit >= 0),
    credit NUMERIC(18, 2) NOT NULL DEFAULT 0 CHECK (credit >= 0),
    memo TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now()
);

CREATE INDEX ON accounting.transaction_entries (account_uuid);

CREATE INDEX ON accounting.transaction_entries (transaction_uuid);

CREATE INDEX ON accounting.transaction_entries (created_at);

CREATE INDEX ON accounting.transaction_entries (serial_id);

-- Fixed accounting.post_transaction function:
CREATE OR REPLACE FUNCTION accounting.post_transaction(
    p_reference TEXT,
    p_description TEXT,
    p_created_by UUID,
    p_module TEXT,
    p_txn_date DATE DEFAULT NULL,
    p_lines JSONB DEFAULT NULL
) RETURNS BIGINT
LANGUAGE plpgsql
AS $$
DECLARE
    v_txn_uuid UUID;
    v_txn_date DATE;
    v_txn_serial_id BIGINT;
    v_total_debits NUMERIC(18,2) := 0;
    v_total_credits NUMERIC(18,2) := 0;
    v_line RECORD;
    v_account_uuid UUID;
    v_line_no INT := 0;
    v_ref_text TEXT;
BEGIN
    IF p_txn_date IS NULL THEN
        -- Opening balance transactions - default transaction date to period start
        SELECT start_date INTO v_txn_date FROM accounting.financial_period
            ORDER BY start_date DESC LIMIT 1;
    ELSE
        -- Other transactions, adjustments, journals, etc
        v_txn_date := p_txn_date;
    END IF;

    -- Validate balance from JSON - they should equal
    FOR v_line IN SELECT * FROM jsonb_to_recordset(p_lines) AS t(account_ref JSONB, debit NUMERIC, credit NUMERIC, memo TEXT)
    LOOP
        v_total_debits := v_total_debits + COALESCE(v_line.debit, 0);
        v_total_credits := v_total_credits + COALESCE(v_line.credit, 0);
    END LOOP;
    IF v_total_debits <> v_total_credits THEN
        RAISE EXCEPTION 'Unbalanced transaction: debits (%) != credits (%)', v_total_debits, v_total_credits;
    END IF;

    -- Insert transaction header
    INSERT INTO accounting.transactions(txn_date, reference, description, created_by, module)
    VALUES (p_txn_date, p_reference, p_description, p_created_by, p_module)
    RETURNING uuid, serial_id INTO v_txn_uuid, v_txn_serial_id;

    -- Insert entries
    FOR v_line IN SELECT * FROM jsonb_to_recordset(p_lines) AS t(account_ref JSONB, debit NUMERIC, credit NUMERIC, memo TEXT)
    LOOP
        v_line_no := v_line_no + 1;

        -- Resolve account_ref: can be code (text), serial_id (number), or uuid
        IF jsonb_typeof(v_line.account_ref) = 'string' THEN
            v_ref_text := v_line.account_ref #>> '{}';
            BEGIN
                -- Try as UUID
                v_account_uuid := v_ref_text::UUID;
                -- Make sure it exists
                IF NOT EXISTS (SELECT 1 FROM accounting.accounts WHERE uuid = v_account_uuid) THEN
                    RAISE EXCEPTION 'Account not found for reference: %', v_ref_text;
                END IF;
            EXCEPTION WHEN others THEN
                -- Not a UUID, treat as code
                SELECT uuid INTO v_account_uuid
                FROM accounting.accounts
                WHERE code = v_ref_text;
                IF v_account_uuid IS NULL THEN
                    RAISE EXCEPTION 'Account not found for reference: %', v_ref_text;
                END IF;
            END;
        ELSIF jsonb_typeof(v_line.account_ref) = 'number' THEN
            -- Look up by serial_id
            SELECT uuid INTO v_account_uuid
            FROM accounting.accounts
            WHERE serial_id = (v_line.account_ref)::BIGINT;
            IF v_account_uuid IS NULL THEN
                RAISE EXCEPTION 'Account not found for reference: %', v_line.account_ref;
            END IF;
        ELSE
            RAISE EXCEPTION 'Invalid account_ref type: must be string (code/uuid) or number (serial_id)';
        END IF;

        -- Insert line
        INSERT INTO accounting.transaction_entries(
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

    RETURN v_txn_serial_id;
END;
$$;

-- Update account path on create new account entry
CREATE OR REPLACE FUNCTION accounting.maintain_account_path()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
DECLARE
    v_parent_path public.ltree;
    v_dummy int;
BEGIN
    -- Only process on INSERT or when code/parent_code changes
    IF TG_OP = 'INSERT'
       OR (TG_OP = 'UPDATE' AND (
            NEW.parent_code IS DISTINCT FROM OLD.parent_code
            OR NEW.code IS DISTINCT FROM OLD.code
        )) THEN

        -- Validate code format: only digits
        IF NEW.code !~ '^[0-9]{6}$' THEN
            RAISE EXCEPTION 'Invalid code %: must be exactly six digits', NEW.code;
        END IF;

        -- Root account (no parent)
        IF NEW.parent_code IS NULL THEN
            NEW.path := NEW.code::public.ltree;
        ELSE
            -- Get parent's path
            SELECT path INTO STRICT v_parent_path
            FROM accounting.accounts
            WHERE code = NEW.parent_code AND is_active;

            IF v_parent_path IS NULL THEN
                RAISE EXCEPTION 'Parent % not found or inactive', NEW.parent_code;
            END IF;

            -- Build new path
            NEW.path := (v_parent_path::text || '.' || NEW.code)::ltree;

            -- Check for cycles: parent cannot be a descendant of this account
            IF TG_OP = 'UPDATE' THEN
                PERFORM 1
                FROM accounting.accounts
                WHERE code = NEW.parent_code
                    AND path <@ OLD.path;

                -- RHS must be text for ltree operators
                IF FOUND THEN
                    RAISE EXCEPTION 'Cycle detected: cannot move % under its own subtree', NEW.code;
                END IF;
            END IF;
        END IF;
    END IF;

    RETURN NEW;
END;
$function$;

-- Move account to different parent procedure

CREATE OR REPLACE PROCEDURE accounting.move_account_subtree(
    IN p_account_code text, IN p_new_parent_code text
) LANGUAGE plpgsql
AS $procedure$
DECLARE
	v_old_path public.ltree;
	v_new_parent_path public.ltree;
BEGIN
	-- Lock the subtree to prevent concurrent changes
	perform * from accounting.accounts where path <@ (select path from accounting.accounts where code = p_account_code) for update;

	-- Get current root path
	select path into v_old_path from accounting.accounts where code = p_account_code;

	if v_old_path is null then
		raise exception 'Accounting % not found or no path', p_account_code;
	end if;

	-- Get new parent's path
	if p_new_parent_code is null then
		v_new_parent_path := p_account_code::ltree;
	else
		select path into v_new_parent_path
		from accounting.accounts
		where code = p_new_parent_code;

		if v_new_parent_path is null then
			raise exception 'New parent % not found', p_new_parent_code;
		end if;
		v_new_parent_path := v_new_parent_path || p_account_code::ltree;
	end if;

	-- Cascade update all descendants (including self)
	update accounting.accounts
	set path = v_new_parent_path || subpath(path, nlevel(v_old_path) + 1)
	where path <@ v_old_path;

	-- The before trigger will handle the moved root's path correctly
	update accounting.accounts
	set parent_code = p_new_parent_code
	where code = p_account_code;

	commit;
END;
$procedure$;

-- Create trigger function

CREATE OR REPLACE FUNCTION accounting.update_account_balance()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    -- Only update if the transaction is posted (skip drafts)
    IF EXISTS (SELECT 1 FROM accounting.transactions WHERE uuid = NEW.transaction_uuid AND posted = TRUE) THEN
        UPDATE accounting.accounts
        SET current_balance = current_balance + NEW.amount -- amount is already signed (debit - credit)
        WHERE uuid = NEW.account_uuid;
    END IF;
    RETURN NEW;
END;
$$;

-- Attach trigger to transaction_entries (after insert, for each row)
CREATE TRIGGER trig_update_balance
AFTER INSERT ON accounting.transaction_entries
FOR EACH ROW EXECUTE FUNCTION accounting.update_account_balance();

-- Handle deletes (e.g. for unposted drafts or rare voids - subtract the amount)
CREATE OR REPLACE FUNCTION accounting.revert_account_balance()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF EXISTS (SELECT 1 FROM accounting.transactions WHERE uuid = OLD.transaction_uuid AND posted = TRUE) THEN
        UPDATE accounting.accounts
        SET current_balance = current_balance - OLD.amount
        WHERE uuid = OLD.account_uuid;
    END IF;
    RETURN OLD;
END;
$$;

CREATE TRIGGER trig_revert_balance
AFTER DELETE ON accounting.transaction_entries
FOR EACH ROW EXECUTE FUNCTION accounting.revert_account_balance();

CREATE OR REPLACE TRIGGER trg_account_path_maintain BEFORE INSERT OR UPDATE OF
    parent_code, code ON accounting.accounts FOR EACH ROW
    EXECUTE FUNCTION accounting.maintain_account_path();
-- USE
--SELECT accounting.post_transaction(
--    'INV-001',
--    'Customer invoice',
--    1,
--    'invoice',
--    '2025-04-01',
--    '[
--        {"account_ref": "110100", "debit": 1000.00, "credit": 0, "memo": "Sales"},
--        {"account_ref": 42,       "debit": 0,      "credit": 1000.00, "memo": "AR"}
--    ]'::jsonb
--);