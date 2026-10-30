
-- ============================================================================
-- 11_account_role_resolution.sql
--
-- Purpose:
--   1. Define whether a chart-of-accounts entry can receive postings.
--   2. Store each tenant's mappings from stable accounting roles to accounts.
--   3. Enforce active/postable-account rules at the database boundary.
--
-- Assumptions:
--   - 01_accounting.sql has already run.
--   - This is a fresh tenant database with no production transactions.
--   - Canonical role definitions and accounting policies live in Rust.
-- ============================================================================

-- ============================================================================
-- 2. Tenant-specific accounting-role mappings
-- ============================================================================
-- role_key is a stable domain identifier, e.g. "work_in_progress".
-- context_key supports operational choices where a role can have multiple
-- mappings, e.g. different cashbooks or bank accounts.
--
-- The role catalogue itself is defined centrally in the Rust domain layer.
-- Account UUIDs always refer to accounts in this tenant database.

CREATE TABLE accounting.account_role_mappings (
    uuid UUID PRIMARY KEY DEFAULT uuidv7(),

    role_key TEXT NOT NULL,
    context_key TEXT NOT NULL DEFAULT 'default',

    account_uuid UUID NOT NULL
        REFERENCES accounting.accounts(uuid)
        ON DELETE RESTRICT,

    mapping_source TEXT NOT NULL DEFAULT 'manual'
        CHECK (mapping_source IN ('manual', 'discovery', 'seed')),

    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT account_role_mappings_role_context_unique
        UNIQUE (role_key, context_key)
);

COMMENT ON TABLE accounting.account_role_mappings IS
    'Tenant-specific mapping from stable accounting role keys to chart-of-accounts entries.';

COMMENT ON COLUMN accounting.account_role_mappings.context_key IS
    'Distinguishes operational mappings for a role; use default for a single general mapping.';

-- ============================================================================
-- 3. Enforce hierarchy and account posting eligibility
-- ============================================================================

CREATE OR REPLACE FUNCTION accounting.enforce_account_postability()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    -- A parent account cannot receive direct postings.
    IF NEW.is_postable AND EXISTS (
        SELECT 1
        FROM accounting.accounts child
        WHERE child.parent_code = NEW.code
          AND child.uuid IS DISTINCT FROM NEW.uuid
    ) THEN
        RAISE EXCEPTION
            'Account % cannot be postable because it has child accounts',
            NEW.code
            USING ERRCODE = '23514';
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_enforce_account_postability
    BEFORE INSERT OR UPDATE OF code, is_postable
    ON accounting.accounts
    FOR EACH ROW
    EXECUTE FUNCTION accounting.enforce_account_postability();

-- ============================================================================
-- 4. Validate accounts referenced by journal lines
-- ============================================================================
-- This protects journal insertion even if a caller bypasses the normal
-- Rust posting service. The FK in 01_accounting.sql already checks existence.


CREATE OR REPLACE FUNCTION accounting.validate_posting_account()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_is_active BOOLEAN;
    v_is_postable BOOLEAN;
    v_code TEXT;
BEGIN
    SELECT a.is_active, a.is_postable, a.code
    INTO v_is_active, v_is_postable, v_code
    FROM accounting.accounts a
    WHERE a.uuid = NEW.account_uuid
    FOR SHARE;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Posting account % does not exist', NEW.account_uuid
            USING ERRCODE = '23503';
    END IF;

    IF NOT v_is_active THEN
        RAISE EXCEPTION 'Account % is inactive', v_code
            USING ERRCODE = '23514';
    END IF;

    IF NOT v_is_postable THEN
        RAISE EXCEPTION 'Account % is not enabled for direct posting', v_code
            USING ERRCODE = '23514';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM accounting.accounts child
        WHERE child.parent_code = v_code
    ) THEN
        RAISE EXCEPTION 'Account % is a parent account and cannot receive direct postings', v_code
            USING ERRCODE = '23514';
    END IF;

    RETURN NEW;
END;
$$;


CREATE TRIGGER trg_validate_posting_account
    BEFORE INSERT OR UPDATE OF account_uuid
    ON accounting.transaction_entries
    FOR EACH ROW
    EXECUTE FUNCTION accounting.validate_posting_account();

-- ============================================================================
-- 5. Mapping integrity
-- ============================================================================
-- A mapping must point to an active, postable account.
-- Check on insert/update, and also prevent later account deactivation or
-- making a mapped account non-postable while an active mapping still uses it.

CREATE OR REPLACE FUNCTION accounting.validate_account_role_mapping()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_is_active BOOLEAN;
    v_is_postable BOOLEAN;
BEGIN
    SELECT a.is_active, a.is_postable
    INTO v_is_active, v_is_postable
    FROM accounting.accounts a
    WHERE a.uuid = NEW.account_uuid;

    IF NOT FOUND OR NOT v_is_active OR NOT v_is_postable THEN
        RAISE EXCEPTION
            'Role mapping requires an existing, active, postable account'
            USING ERRCODE = '23514';
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_validate_account_role_mapping
    BEFORE INSERT OR UPDATE OF account_uuid, is_active
    ON accounting.account_role_mappings
    FOR EACH ROW
    WHEN (NEW.is_active = TRUE)
    EXECUTE FUNCTION accounting.validate_account_role_mapping();

CREATE OR REPLACE FUNCTION accounting.prevent_invalid_mapped_account_change()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    IF (NOT NEW.is_active OR NOT NEW.is_postable)
       AND EXISTS (
           SELECT 1
           FROM accounting.account_role_mappings m
           WHERE m.account_uuid = NEW.uuid
             AND m.is_active = TRUE
       ) THEN
        RAISE EXCEPTION
            'Cannot deactivate or make account % non-postable while active role mappings reference it',
            NEW.code
            USING ERRCODE = '23514';
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_prevent_invalid_mapped_account_change
    BEFORE UPDATE OF is_active, is_postable
    ON accounting.accounts
    FOR EACH ROW
    EXECUTE FUNCTION accounting.prevent_invalid_mapped_account_change();

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

DROP TRIGGER IF EXISTS trg_validate_posting_account
    ON accounting.transaction_entries;
