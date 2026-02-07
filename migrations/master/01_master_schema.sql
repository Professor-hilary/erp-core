--
-- master_schema.db
--
-- CREATE EXTENSION IF NOT EXISTSS pg_uuidv7;
CREATE TABLE IF NOT EXISTS users (
    uuid uuid PRIMARY KEY DEFAULT uuidv7() NOT NULL,
    email text UNIQUE NOT NULL,
    password_hash text NOT NULL,
    created_at timestamptz DEFAULT now(),
    is_active boolean DEFAULT true,
    is_deleted boolean DEFAULT false
);
CREATE TABLE IF NOT EXISTS companies (
    uuid uuid PRIMARY KEY DEFAULT uuidv7() NOT NULL,
    name text NOT NULL,
    slug text UNIQUE NOT NULL,
    tenant_db_name text NOT NULL,
    tenant_db_uri text NOT NULL,
    industry text NOT NULL,
    business_type text NOT NULL,
    country text NOT NULL,
    company_email text,
    legal_name text,
    telephone text,
    website text,
    co_address text,
    city text,
    currency text,
    co_state text,
    zip_code text,
    tax_id text,
    status text NOT NULL DEFAULT 'provisioning',
    created_by uuid REFERENCES users(uuid),
    is_deleted boolean DEFAULT false,
    period_start DATE,
    period_end DATE,
    created_at timestamptz DEFAULT now(),
    UNIQUE(created_by, name),
    CHECK (
        status IN (
            'provisioning',
            'active',
            'error',
            'suspended',
            'deleted'
        )
    )
);
CREATE INDEX companies_created_by_idx ON companies(created_by);
CREATE INDEX companies_tenant_db_name_idx ON companies(tenant_db_name);
CREATE INDEX companies_status_idx ON companies(status);

CREATE TABLE IF NOT EXISTS user_companies(
    user_id uuid REFERENCES users(uuid) ON DELETE CASCADE,
    company_id uuid REFERENCES companies(uuid) ON DELETE CASCADE,
    role text NOT NULL DEFAULT 'member',
    primary key (user_id, company_id)
);
-- Optional: Store ephemeral cache metadata
CREATE TABLE IF NOT EXISTS tenant_secrets(
    company_id uuid PRIMARY KEY REFERENCES companies(uuid),
    secret jsonb NOT NULL,
    db_name TEXT NOT NULL,
    db_host TEXT NOT NULL DEFAULT 'localhost',
    db_port INT NOT NULL DEFAULT 5433,
    db_user TEXT NOT NULL,
    db_password TEXT NOT NULL,
    created_by uuid REFERENCES users(uuid),
    created_at timestamptz DEFAULT now()
);