-- =============================================================================
-- Chiefalry: Account Role Resolution
--
-- Each tenant database maintains its own mappings.
-- Account codes and UUIDs are configuration, not business logic.
-- =============================================================================

-- 1. Explicit posting eligibility.
-- Grouping accounts cannot receive new postings by default.
UPDATE accounting.accounts AS parent
SET is_postable = FALSE
WHERE EXISTS (
    SELECT 1
    FROM accounting.accounts AS child
    WHERE child.parent_code = parent.code
);

COMMENT ON COLUMN accounting.accounts.is_postable IS
    'Whether the account is eligible to receive new general-ledger journal entries.';

-- 2. Persistent account-role configuration.
CREATE TABLE IF NOT EXISTS accounting.account_role_mappings (
    uuid UUID PRIMARY KEY DEFAULT uuidv7(),

    role_key TEXT NOT NULL UNIQUE,

    account_uuid UUID NOT NULL
        REFERENCES accounting.accounts(uuid)
        ON DELETE RESTRICT,

    mapping_source TEXT NOT NULL DEFAULT 'manual'
        CHECK (
            mapping_source IN ('auto_discovered', 'seed', 'manual')
        ),

    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT account_role_mappings_role_key_not_empty
        CHECK (length(trim(role_key)) > 0)
);

CREATE INDEX IF NOT EXISTS idx_account_role_mappings_account
    ON accounting.account_role_mappings(account_uuid);

CREATE INDEX IF NOT EXISTS idx_account_role_mappings_active
    ON accounting.account_role_mappings(role_key)
    WHERE is_active;

COMMENT ON TABLE accounting.account_role_mappings IS
    'Tenant-local mappings between stable application accounting roles and CoA accounts.';

-- 3. Adding a child makes its parent a grouping account.
CREATE OR REPLACE FUNCTION accounting.mark_parent_as_non_postable()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    IF NEW.parent_code IS NOT NULL THEN
        UPDATE accounting.accounts
        SET is_postable = FALSE,
            updated_at = NOW()
        WHERE code = NEW.parent_code;
    END IF;

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_mark_parent_non_postable
    ON accounting.accounts;

CREATE TRIGGER trg_mark_parent_non_postable
AFTER INSERT OR UPDATE OF parent_code
ON accounting.accounts
FOR EACH ROW
EXECUTE FUNCTION accounting.mark_parent_as_non_postable();

-- 4. Prevent journal lines from targeting inactive or grouping accounts.
CREATE OR REPLACE FUNCTION accounting.validate_posting_account()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_is_active BOOLEAN;
    v_is_postable BOOLEAN;
BEGIN
    SELECT is_active, is_postable
    INTO v_is_active, v_is_postable
    FROM accounting.accounts
    WHERE uuid = NEW.account_uuid
    FOR KEY SHARE;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Account % does not exist', NEW.account_uuid
            USING ERRCODE = '23503';
    END IF;

    IF NOT v_is_active THEN
        RAISE EXCEPTION 'Cannot post to inactive account %', NEW.account_uuid
            USING ERRCODE = '23514';
    END IF;

    IF NOT v_is_postable THEN
        RAISE EXCEPTION 'Cannot post directly to grouping account %',
            NEW.account_uuid
            USING ERRCODE = '23514';
    END IF;

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_validate_posting_account
    ON accounting.transaction_entries;

CREATE TRIGGER trg_validate_posting_account
BEFORE INSERT OR UPDATE OF account_uuid
ON accounting.transaction_entries
FOR EACH ROW
EXECUTE FUNCTION accounting.validate_posting_account();
