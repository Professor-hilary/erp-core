-- ============================================================================
-- 14_rbac.sql — Tenant RBAC (roles, permissions, modules, membership)
-- Users are identified by the same UUID as master.users.
-- Authz is resolved once per request into an in-memory Actor (cached).
-- ============================================================================

CREATE SCHEMA IF NOT EXISTS rbac;

-- --------------------------------------------------------------------------
-- Catalogue
-- --------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS rbac.permissions (
    code        text PRIMARY KEY,           -- e.g. manufacturing.issue_materials
    module      text NOT NULL,              -- manufacturing | payroll | ...
    description text NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS rbac.roles (
    code        text PRIMARY KEY,           -- TenantAdmin | ProductionSupervisor | ...
    name        text NOT NULL,
    description text NOT NULL DEFAULT '',
    is_system   boolean NOT NULL DEFAULT true
);

CREATE TABLE IF NOT EXISTS rbac.role_permissions (
    role_code       text NOT NULL REFERENCES rbac.roles(code) ON DELETE CASCADE,
    permission_code text NOT NULL REFERENCES rbac.permissions(code) ON DELETE CASCADE,
    PRIMARY KEY (role_code, permission_code)
);

-- --------------------------------------------------------------------------
-- Membership (per tenant DB)
-- --------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS rbac.user_memberships (
    user_uuid        uuid PRIMARY KEY,      -- master.users.uuid
    department_uuid  uuid,                  -- hr.departments.uuid when present
    is_tenant_admin  boolean NOT NULL DEFAULT false,
    authz_version    bigint NOT NULL DEFAULT 1,
    status           text NOT NULL DEFAULT 'active'
                     CHECK (status IN ('active', 'disabled', 'invited')),
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS rbac.user_roles (
    user_uuid uuid NOT NULL REFERENCES rbac.user_memberships(user_uuid) ON DELETE CASCADE,
    role_code text NOT NULL REFERENCES rbac.roles(code) ON DELETE CASCADE,
    PRIMARY KEY (user_uuid, role_code)
);

CREATE TABLE IF NOT EXISTS rbac.user_module_grants (
    user_uuid uuid NOT NULL REFERENCES rbac.user_memberships(user_uuid) ON DELETE CASCADE,
    module    text NOT NULL,
    PRIMARY KEY (user_uuid, module)
);

CREATE INDEX IF NOT EXISTS idx_user_roles_user ON rbac.user_roles(user_uuid);
CREATE INDEX IF NOT EXISTS idx_user_modules_user ON rbac.user_module_grants(user_uuid);

-- --------------------------------------------------------------------------
-- Seed catalogue
-- --------------------------------------------------------------------------
INSERT INTO rbac.permissions (code, module, description) VALUES
  ('settings.manage_users', 'settings', 'Invite users, assign roles and modules'),
  ('settings.manage_roles', 'settings', 'Edit role permission packs'),
  ('settings.company', 'settings', 'Company profile and tenant flags'),
  ('settings.coa_roles', 'settings', 'Account-role mappings'),
  ('accounting.post', 'accounting', 'Post journals'),
  ('accounting.period_close', 'accounting', 'Open/close periods'),
  ('reports.view', 'reports', 'Financial statements and TB'),
  ('workforce.manage', 'workforce', 'Employees, contracts, leave'),
  ('payroll.prepare', 'payroll', 'Prepare payruns'),
  ('payroll.process', 'payroll', 'Process and lock payruns'),
  ('payroll.settle', 'payroll', 'Settle employees and remit statutory'),
  ('inventory.manage', 'inventory', 'Items, warehouses, stock'),
  ('procurement.manage', 'procurement', 'Vendors and purchases'),
  ('sales.manage', 'sales', 'Customers and invoices'),
  ('manufacturing.manage', 'manufacturing', 'BOM, routing, production orders'),
  ('manufacturing.issue', 'manufacturing', 'Issue materials to production'),
  ('manufacturing.timesheet', 'manufacturing', 'Labour timesheets'),
  ('capital.manage', 'capital', 'Equity and debt facilities'),
  ('fixed_assets.manage', 'fixed_assets', 'Asset lifecycle'),
  ('investment.manage', 'investment', 'Investment cases')
ON CONFLICT (code) DO NOTHING;

INSERT INTO rbac.roles (code, name, description, is_system) VALUES
  ('TenantAdmin', 'Tenant administrator', 'Full policy control; ops allowed unless SoD enforced', true),
  ('SystemsManager', 'Systems manager', 'Configuration, CoA roles, periods', true),
  ('Accountant', 'Accountant', 'Journals, payroll post, reports', true),
  ('HROfficer', 'HR officer', 'Workforce and payroll prepare', true),
  ('Storekeeper', 'Storekeeper', 'Inventory and receiving', true),
  ('ProductionSupervisor', 'Production supervisor', 'Manufacturing floor', true),
  ('SalesOfficer', 'Sales officer', 'Customers and invoicing', true),
  ('Viewer', 'Viewer', 'Read-only reports', true)
ON CONFLICT (code) DO NOTHING;

-- TenantAdmin: everything
INSERT INTO rbac.role_permissions (role_code, permission_code)
SELECT 'TenantAdmin', code FROM rbac.permissions
ON CONFLICT DO NOTHING;

INSERT INTO rbac.role_permissions (role_code, permission_code) VALUES
  ('SystemsManager', 'settings.company'),
  ('SystemsManager', 'settings.coa_roles'),
  ('SystemsManager', 'settings.manage_users'),
  ('SystemsManager', 'accounting.period_close'),
  ('SystemsManager', 'reports.view'),
  ('Accountant', 'accounting.post'),
  ('Accountant', 'accounting.period_close'),
  ('Accountant', 'reports.view'),
  ('Accountant', 'payroll.prepare'),
  ('Accountant', 'payroll.process'),
  ('Accountant', 'payroll.settle'),
  ('HROfficer', 'workforce.manage'),
  ('HROfficer', 'payroll.prepare'),
  ('HROfficer', 'reports.view'),
  ('Storekeeper', 'inventory.manage'),
  ('Storekeeper', 'procurement.manage'),
  ('ProductionSupervisor', 'manufacturing.manage'),
  ('ProductionSupervisor', 'manufacturing.issue'),
  ('ProductionSupervisor', 'manufacturing.timesheet'),
  ('ProductionSupervisor', 'inventory.manage'),
  ('SalesOfficer', 'sales.manage'),
  ('SalesOfficer', 'reports.view'),
  ('Viewer', 'reports.view')
ON CONFLICT DO NOTHING;

-- --------------------------------------------------------------------------
-- Bootstrap helpers
-- --------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION rbac.bootstrap_tenant_admin(p_user uuid)
RETURNS void
LANGUAGE plpgsql
AS $$
DECLARE
    m text;
BEGIN
    INSERT INTO rbac.user_memberships (user_uuid, is_tenant_admin, status)
    VALUES (p_user, true, 'active')
    ON CONFLICT (user_uuid) DO UPDATE
      SET is_tenant_admin = true,
          status = 'active',
          authz_version = rbac.user_memberships.authz_version + 1,
          updated_at = now();

    INSERT INTO rbac.user_roles (user_uuid, role_code)
    VALUES (p_user, 'TenantAdmin')
    ON CONFLICT DO NOTHING;

    FOREACH m IN ARRAY ARRAY[
        'settings','accounting','reports','workforce','payroll',
        'inventory','procurement','sales','manufacturing','capital',
        'fixed_assets','investment'
    ]
    LOOP
        INSERT INTO rbac.user_module_grants (user_uuid, module)
        VALUES (p_user, m)
        ON CONFLICT DO NOTHING;
    END LOOP;
END;
$$;

CREATE OR REPLACE FUNCTION rbac.effective_permissions(p_user uuid)
RETURNS TABLE (permission_code text)
LANGUAGE sql
STABLE
AS $$
    SELECT DISTINCT rp.permission_code
    FROM rbac.user_roles ur
    JOIN rbac.role_permissions rp ON rp.role_code = ur.role_code
    WHERE ur.user_uuid = p_user
    ORDER BY 1;
$$;

CREATE OR REPLACE FUNCTION rbac.bump_authz_version(p_user uuid)
RETURNS bigint
LANGUAGE plpgsql
AS $$
DECLARE
    v bigint;
BEGIN
    UPDATE rbac.user_memberships
    SET authz_version = authz_version + 1,
        updated_at = now()
    WHERE user_uuid = p_user
    RETURNING authz_version INTO v;
    RETURN COALESCE(v, 0);
END;
$$;
