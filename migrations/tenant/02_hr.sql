-- Enable required extension
-- CREATE EXTENSION IF NOT EXISTS pg_uuidv7;

-- Create schema
CREATE SCHEMA IF NOT EXISTS hr;

-- ========================================
-- SEQUENCES
-- ========================================
CREATE SEQUENCE IF NOT EXISTS hr.departments_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS hr.job_titles_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS hr.employees_serial_id_seq;

-- ========================================
-- TABLE: departments
-- ========================================
CREATE TABLE IF NOT EXISTS hr.departments (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('hr.departments_serial_id_seq') NOT NULL,
    name varchar(100) NOT NULL,
    description text,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT departments_pkey PRIMARY KEY (uuid),
    CONSTRAINT departments_serial_id_key UNIQUE (serial_id)
);

-- ========================================
-- TABLE: job_titles
-- ========================================
CREATE TABLE IF NOT EXISTS hr.job_titles (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('hr.job_titles_serial_id_seq') NOT NULL,
    title varchar(100) NOT NULL,
    description text,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT job_titles_pkey PRIMARY KEY (uuid),
    CONSTRAINT job_titles_serial_id_key UNIQUE (serial_id)
);

-- ========================================
-- TABLE: employees
-- ========================================
CREATE TABLE IF NOT EXISTS hr.employees (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('hr.employees_serial_id_seq') NOT NULL,
    first_name varchar(50) NOT NULL,
    last_name varchar(50) NOT NULL,
    email varchar(100) UNIQUE,
    phone_number varchar(20),
    hire_date date NOT NULL,
    termination_date date,
    job_title varchar(100),
    department_uuid uuid,
    supervisor_uuid uuid,
    employment_type varchar(20) CHECK (employment_type IN ('Full-time', 'Part-time', 'Contract')),
    salary numeric(14, 2),
    pay_frequency varchar(20) CHECK (pay_frequency IN ('Monthly', 'Weekly', 'Bi-weekly')),
    status varchar(20) DEFAULT 'Active',
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT employees_pkey PRIMARY KEY (uuid),
    CONSTRAINT employees_serial_id_key UNIQUE (serial_id),
    CONSTRAINT employees_department_uuid_fkey
        FOREIGN KEY (department_uuid) REFERENCES hr.departments(uuid) ON DELETE SET NULL,
    CONSTRAINT employees_supervisor_uuid_fkey
        FOREIGN KEY (supervisor_uuid) REFERENCES hr.employees(uuid) ON DELETE SET NULL
);

-- ========================================
-- INDEXES (Performance + UX)
-- ========================================
CREATE INDEX IF NOT EXISTS idx_employees_last_first ON hr.employees (last_name, first_name);
CREATE INDEX IF NOT EXISTS idx_employees_email ON hr.employees (email);
CREATE INDEX IF NOT EXISTS idx_employees_status ON hr.employees (status);
CREATE INDEX IF NOT EXISTS idx_employees_dept ON hr.employees (department_uuid);
CREATE INDEX IF NOT EXISTS idx_departments_name ON hr.departments (name);
CREATE INDEX IF NOT EXISTS idx_job_titles_title ON hr.job_titles (title);
