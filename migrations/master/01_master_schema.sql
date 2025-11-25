--
-- master_schema.db
--
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
    uuid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    email text UNIQUE NOT NULL,
    password_hash text NOT NULL,
    created_at timestamptz DEFAULT now(),
    is_active boolean DEFAULT true
);
CREATE TABLE IF NOT EXISTS tenants (
    uuid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    name TEXT NOT NULL,
    db_name TEXT NOT NULL UNIQUE,
    created_at timestamptz DEFAULT now()
);
-- Optional: tenant_db_uri should be encrypted at rest (VAULT/KMS)
CREATE TABLE IF NOT EXISTS companies (
    uuid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    name text NOT NULL,
    slug text UNIQUE NOT NULL,
    tenant_db_name text NOT NULL,
    tenant_db_uri text NOT NULL,
    industry text NOT NULL,
    business_type text NOT NULL,
    status text NOT NULL DEFAULT 'provisioning',
    created_by uuid REFERENCES users(uuid),
    created_at timestamptz DEFAULT now()
);
CREATE TABLE IF NOT EXISTS industry_coa_templates (
    industry text PRIMARY KEY,
    template jsonb NOT NULL,
    created_at timestamptz DEFAULT now()
);
CREATE TABLE IF NOT EXISTS user_companies(
    user_id uuid REFERENCES users(uuid) ON DELETE CASCADE,
    company_id uuid REFERENCES companies(uuid) ON DELETE CASCADE,
    role text NOT NULL DEFAULT 'member',
    primary key (user_id, company_id)
);
CREATE TABLE IF NOT EXISTS provision_events(
    uuid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    company_id uuid REFERENCES companies(uuid),
    event_type text NOT NULL,
    payload jsonb,
    created_at timestamptz DEFAULT now()
);
-- Optional: Store ephemeral cache metadata
CREATE TABLE IF NOT EXISTS tenant_secrets(
    company_id uuid PRIMARY KEY REFERENCES companies(uuid),
    secret jsonb,
    created_at timestamptz DEFAULT now()
);
