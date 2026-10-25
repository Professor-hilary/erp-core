-- =============================================================================
-- HR MODULE — process-aligned (organisation, employment lifecycle, leave)
-- Payroll lives in 03_payroll.sql and READS hr views/functions only
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS hr;

-- ---------------------------------------------------------------------------
-- 1. ORGANISATION
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS hr.departments (
    uuid          uuid PRIMARY KEY DEFAULT uuidv7(),
	serial_id     bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    name          varchar(100) NOT NULL,
    description   text,
    parent_uuid   uuid REFERENCES hr.departments(uuid) ON DELETE SET NULL,
    is_active     boolean NOT NULL DEFAULT true,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS hr.job_titles (
    uuid          uuid PRIMARY KEY DEFAULT uuidv7(),
	serial_id     bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    title         varchar(100) NOT NULL,
    description   text,
    is_active     boolean NOT NULL DEFAULT true,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- 2. PERSON (lean) — not a dumping ground
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS hr.employees (
    uuid              uuid PRIMARY KEY DEFAULT uuidv7(),
    serial_id     	  bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    first_name        varchar(50) NOT NULL,
    last_name         varchar(50) NOT NULL,
    middle_name       varchar(50),
    email             varchar(100) UNIQUE,
    work_email        varchar(100),
    phone_number      varchar(20),
    date_of_birth     date,
    gender            varchar(20),
    hire_date         date NOT NULL,
    department_uuid   uuid REFERENCES hr.departments(uuid) ON DELETE SET NULL,
    job_title_uuid    uuid REFERENCES hr.job_titles(uuid) ON DELETE SET NULL,
    supervisor_uuid   uuid REFERENCES hr.employees(uuid) ON DELETE SET NULL,
    -- cache from active contract (updated by contract functions); optional
    employment_type   varchar(20)
                      CHECK (employment_type IS NULL OR employment_type IN
                        ('Full-time','Part-time','Contract','Intern')),
    pay_frequency     varchar(20)
                      CHECK (pay_frequency IS NULL OR pay_frequency IN
                        ('Monthly','Weekly','Bi-weekly')),
    status            varchar(20) NOT NULL DEFAULT 'Onboarding'
                      CHECK (status IN (
                        'Onboarding','Active','Suspended','Notice','Terminated','Alumni'
                      )),
    party_uuid        uuid,  -- optional link to capital.parties
    created_at        timestamptz NOT NULL DEFAULT now(),
    updated_at        timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_employees_status ON hr.employees (status);
CREATE INDEX IF NOT EXISTS idx_employees_dept ON hr.employees (department_uuid);
CREATE INDEX IF NOT EXISTS idx_employees_name ON hr.employees (last_name, first_name);

-- ---------------------------------------------------------------------------
-- 3. PROFILES (1:1 satellites)
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS hr.employee_statutory_profiles (
    employee_uuid           uuid PRIMARY KEY
                            REFERENCES hr.employees(uuid) ON DELETE CASCADE,
    national_id             varchar(50),
    tax_id                  varchar(50),
    social_security_number  varchar(50),
    other_ids               jsonb NOT NULL DEFAULT '{}',
    updated_at              timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS hr.employee_payment_profiles (
    employee_uuid           uuid PRIMARY KEY
                            REFERENCES hr.employees(uuid) ON DELETE CASCADE,
    bank_name               varchar(100),
    bank_account_number     varchar(50),
    bank_branch             varchar(100),
    payment_method          varchar(30) NOT NULL DEFAULT 'Bank Transfer',
    currency_code           varchar(3) NOT NULL DEFAULT 'UGX',
    updated_at              timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS hr.employee_lifecycle (
    employee_uuid           uuid PRIMARY KEY
                            REFERENCES hr.employees(uuid) ON DELETE CASCADE,
    notice_start_date       date,
    last_working_date       date,
    termination_date        date,
    exit_reason_code        varchar(40),
    -- Resignation | Dismissal | End_of_Contract | Retirement | Other
    exit_notes              text,
    eligible_for_rehire     boolean NOT NULL DEFAULT true,
    updated_at              timestamptz NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- 4. EMPLOYMENT (contracts = terms of work & pay)
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS hr.employment_contracts (
    uuid                uuid PRIMARY KEY DEFAULT uuidv7(),
	serial_id     		bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    employee_uuid       uuid NOT NULL REFERENCES hr.employees(uuid) ON DELETE CASCADE,
    contract_code       varchar(40),
    contract_type       varchar(30) NOT NULL DEFAULT 'Permanent' CHECK (contract_type IN (
                          'Permanent','Fixed-term','Probation','Contract','Internship'
                        )),
    start_date          date NOT NULL,
    end_date            date,
    job_title_uuid      uuid REFERENCES hr.job_titles(uuid) ON DELETE SET NULL,
    department_uuid     uuid REFERENCES hr.departments(uuid) ON DELETE SET NULL,
    basic_salary        numeric(14,2),
    pay_frequency       varchar(20),
    employment_type     varchar(20),
    probation_end_date  date,
    notice_days         integer,
    status              varchar(20) NOT NULL DEFAULT 'Active'
                        CHECK (status IN ('Draft','Active','Superseded','Terminated','Expired')),
    notes               text,
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_contracts_employee ON hr.employment_contracts (employee_uuid);

-- At most one Active contract per employee
CREATE UNIQUE INDEX IF NOT EXISTS uq_contracts_one_active
    ON hr.employment_contracts (employee_uuid)
    WHERE status = 'Active';

CREATE TABLE IF NOT EXISTS hr.contract_amendments (
    uuid                uuid PRIMARY KEY DEFAULT uuidv7(),
	serial_id     		bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    contract_uuid       uuid NOT NULL REFERENCES hr.employment_contracts(uuid) ON DELETE CASCADE,
    effective_date      date NOT NULL,
    amendment_type      varchar(40) NOT NULL, -- SALARY | TITLE | DEPARTMENT | OTHER
    old_value           text,
    new_value           text,
    basic_salary        numeric(14,2),
    job_title_uuid      uuid,
    notes               text,
    created_at          timestamptz NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- 5. EMPLOYMENT EVENTS (audit spine — never update/delete in app)
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS hr.employment_events (
    uuid            uuid PRIMARY KEY DEFAULT uuidv7(),
	serial_id     	bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    employee_uuid   uuid NOT NULL REFERENCES hr.employees(uuid) ON DELETE CASCADE,
    event_type      varchar(40) NOT NULL,
    -- HIRED, ONBOARDING_STARTED, ONBOARDING_COMPLETED, ACTIVATED, CONTRACT_ACTIVATED,
    -- SUSPENDED, REINSTATED, NOTICE_STARTED, TERMINATED, REHIRED, CONTRACT_AMENDED
    event_date      date NOT NULL DEFAULT CURRENT_DATE,
    from_status     varchar(20),
    to_status       varchar(20),
    reason_code     varchar(40),
    notes           text,
    recorded_by     uuid,
    created_at      timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_employment_events_emp
    ON hr.employment_events (employee_uuid, created_at);

-- ---------------------------------------------------------------------------
-- 6. LEAVE
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS hr.leave_types (
    uuid                    uuid PRIMARY KEY DEFAULT uuidv7(),
    serial_id       		bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    code                    varchar(20) NOT NULL UNIQUE,
    name                    varchar(100) NOT NULL,
    is_paid                 boolean NOT NULL DEFAULT true,
	grant_mode varchar(20) NOT NULL DEFAULT 'REQUEST'
    	CHECK (grant_mode IN ('REQUEST','SCHEDULED','BOTH')),
  	unused_treatment varchar(20) NOT NULL DEFAULT 'ACCRUE'
	    CHECK (unused_treatment IN ('ACCRUE','FORFEIT','ENCASH')),
    default_days_per_year   numeric(6,2) DEFAULT 0,
    requires_approval       boolean NOT NULL DEFAULT true,
    is_active               boolean NOT NULL DEFAULT true,
    created_at              timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS hr.leave_balances (
    uuid                uuid PRIMARY KEY DEFAULT uuidv7(),
    serial_id       	bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    employee_uuid       uuid NOT NULL REFERENCES hr.employees(uuid) ON DELETE CASCADE,
    leave_type_uuid     uuid NOT NULL REFERENCES hr.leave_types(uuid) ON DELETE CASCADE,
    year                integer NOT NULL,
    entitled            numeric(8,2) NOT NULL DEFAULT 0,
    taken               numeric(8,2) NOT NULL DEFAULT 0,
    balance             numeric(8,2) GENERATED ALWAYS AS (entitled - taken) STORED,
    updated_at          timestamptz NOT NULL DEFAULT now(),
    UNIQUE (employee_uuid, leave_type_uuid, year)
);

CREATE TABLE IF NOT EXISTS hr.leave_requests (
    uuid                uuid PRIMARY KEY DEFAULT uuidv7(),
    serial_id       	bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    employee_uuid       uuid NOT NULL REFERENCES hr.employees(uuid) ON DELETE CASCADE,
    leave_type_uuid     uuid NOT NULL REFERENCES hr.leave_types(uuid),
    start_date          date NOT NULL,
    end_date            date NOT NULL,
    days                numeric(6,2) NOT NULL,
    status              varchar(20) NOT NULL DEFAULT 'Pending'
                        CHECK (status IN ('Pending','Approved','Rejected','Cancelled')),
    reason              text,
    approver_uuid       uuid REFERENCES hr.employees(uuid),
    decided_at          timestamptz,
    created_at          timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS hr.leave_schedules (
  uuid 				uuid PRIMARY KEY DEFAULT uuidv7(),
  employee_uuid 	uuid NOT NULL REFERENCES hr.employees(uuid) ON DELETE CASCADE,
  leave_type_uuid 	uuid NOT NULL REFERENCES hr.leave_types(uuid),
  year 				int NOT NULL,
  planned_start 	date NOT NULL,
  planned_end 		date NOT NULL,
  days_planned 		numeric(6,2) NOT NULL,
  status 			varchar(20) NOT NULL DEFAULT 'Planned' CHECK (status IN (
						'Planned','Taken','Skipped','Encash_Pending','Encashed','Cancelled'
					)),
  source 			varchar(20) DEFAULT 'HR',
  notes 			text,
  created_at 		timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS hr.leave_encashments (
  uuid 				uuid PRIMARY KEY DEFAULT uuidv7(),
  employee_uuid 	uuid NOT NULL REFERENCES hr.employees(uuid) ON DELETE CASCADE,
  leave_type_uuid 	uuid NOT NULL REFERENCES hr.leave_types(uuid),
  year 				int NOT NULL,
  days 				numeric(6,2) NOT NULL,
  rate_per_day 		numeric(14,4) NOT NULL,
  amount 			numeric(14,2) NOT NULL,
  schedule_uuid 	uuid REFERENCES hr.leave_schedules(uuid),
  payrun_uuid 		uuid,
  status 			varchar(20) NOT NULL DEFAULT 'Pending' CHECK (status IN (
						'Pending','Included_In_Payrun','Paid','Cancelled'
					)),
  created_at timestamptz NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- 7. ONBOARDING / OFFBOARDING TASKS
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS hr.onboarding_tasks (
    uuid            uuid PRIMARY KEY DEFAULT uuidv7(),
    serial_id       bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    employee_uuid   uuid NOT NULL REFERENCES hr.employees(uuid) ON DELETE CASCADE,
    code            varchar(40) NOT NULL,
    title           varchar(150) NOT NULL,
    description     text,
    sort_order      integer NOT NULL DEFAULT 0,
    is_required     boolean NOT NULL DEFAULT true,
    is_done         boolean NOT NULL DEFAULT false,
    due_date        date,
    done_at         timestamptz,
    done_by         uuid,
    notes           text,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    UNIQUE (employee_uuid, code)
);

CREATE TABLE IF NOT EXISTS hr.offboarding_tasks (
    uuid            uuid PRIMARY KEY DEFAULT uuidv7(),
    serial_id       bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    employee_uuid   uuid NOT NULL REFERENCES hr.employees(uuid) ON DELETE CASCADE,
    code            varchar(40) NOT NULL,
    title           varchar(150) NOT NULL,
    description     text,
    sort_order      integer NOT NULL DEFAULT 0,
    is_required     boolean NOT NULL DEFAULT true,
    is_done         boolean NOT NULL DEFAULT false,
    due_date        date,
    done_at         timestamptz,
    done_by         uuid,
    notes           text,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    UNIQUE (employee_uuid, code)
);

CREATE INDEX IF NOT EXISTS idx_onboarding_pending
    ON hr.onboarding_tasks (employee_uuid) WHERE is_done = false;
CREATE INDEX IF NOT EXISTS idx_offboarding_pending
    ON hr.offboarding_tasks (employee_uuid) WHERE is_done = false;

-- Log event helper
CREATE OR REPLACE FUNCTION hr.log_event(
    p_employee uuid,
    p_type text,
    p_from text,
    p_to text,
    p_reason text DEFAULT NULL,
    p_notes text DEFAULT NULL,
    p_by uuid DEFAULT NULL,
    p_date date DEFAULT CURRENT_DATE
) RETURNS void
LANGUAGE sql AS $$
    INSERT INTO hr.employment_events (
        employee_uuid, event_type, event_date, from_status, to_status,
        reason_code, notes, recorded_by
    ) VALUES (
        p_employee, p_type, p_date, p_from, p_to, p_reason, p_notes, p_by
    );
$$;

-- Seed onboarding
CREATE OR REPLACE FUNCTION hr.seed_onboarding_tasks(p_employee uuid)
RETURNS void
LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO hr.onboarding_tasks (employee_uuid, code, title, sort_order, is_required)
    VALUES
        (p_employee, 'CONTRACT_SIGNED',       'Employment contract signed', 10, true),
        (p_employee, 'BANK_CAPTURED',         'Bank details captured', 20, true),
        (p_employee, 'TAX_ID_CAPTURED',       'Tax ID captured', 30, true),
        (p_employee, 'SS_ID_CAPTURED',        'Social security number captured', 40, true),
        (p_employee, 'LEAVE_BALANCES_SEEDED', 'Leave balances opened', 50, true),
        (p_employee, 'SYSTEM_ACCESS',         'System access granted', 60, true),
        (p_employee, 'EQUIPMENT_ISSUED',      'Equipment issued', 70, false)
    ON CONFLICT (employee_uuid, code) DO NOTHING;
END;
$$;

-- Seed offboarding
CREATE OR REPLACE FUNCTION hr.seed_offboarding_tasks(p_employee uuid)
RETURNS void
LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO hr.offboarding_tasks (employee_uuid, code, title, sort_order, is_required)
    VALUES
        (p_employee, 'EXIT_INTERVIEW',        'Exit interview completed', 10, false),
        (p_employee, 'ASSETS_RETURNED',       'Company assets returned', 20, true),
        (p_employee, 'ACCESS_REVOKED',        'Access revoked (badge/email/systems)', 30, true),
        (p_employee, 'HANDOVER_DONE',         'Work handover done', 40, true),
        (p_employee, 'FINAL_PAY_CALCULATED',  'Final pay calculated', 50, true),
        (p_employee, 'FINAL_PAY_POSTED',      'Final pay posted in payroll', 60, true),
        (p_employee, 'STATUTORY_CLEARANCE',   'Statutory exit notes recorded', 70, false)
    ON CONFLICT (employee_uuid, code) DO NOTHING;
END;
$$;

-- Hire: create empty profile shells + onboarding + event
CREATE OR REPLACE FUNCTION hr.after_employee_hired(p_employee uuid, p_by uuid DEFAULT NULL)
RETURNS void
LANGUAGE plpgsql AS $$
DECLARE v_status text;
BEGIN
    SELECT status INTO v_status FROM hr.employees WHERE uuid = p_employee;
    INSERT INTO hr.employee_statutory_profiles (employee_uuid) VALUES (p_employee)
        ON CONFLICT DO NOTHING;
    INSERT INTO hr.employee_payment_profiles (employee_uuid) VALUES (p_employee)
        ON CONFLICT DO NOTHING;
    INSERT INTO hr.employee_lifecycle (employee_uuid) VALUES (p_employee)
        ON CONFLICT DO NOTHING;
    PERFORM hr.seed_onboarding_tasks(p_employee);
    PERFORM hr.log_event(p_employee, 'HIRED', NULL, v_status, NULL, NULL, p_by);
    PERFORM hr.log_event(p_employee, 'ONBOARDING_STARTED', NULL, v_status, NULL, NULL, p_by);
END;
$$;

-- Complete onboarding → Active
CREATE OR REPLACE FUNCTION hr.complete_onboarding(p_employee uuid, p_by uuid DEFAULT NULL)
RETURNS hr.employees
LANGUAGE plpgsql AS $$
DECLARE
    v_open int;
    v_row hr.employees%ROWTYPE;
BEGIN
    SELECT count(*) INTO v_open
    FROM hr.onboarding_tasks
    WHERE employee_uuid = p_employee AND is_required AND NOT is_done;

    IF v_open > 0 THEN
        RAISE EXCEPTION 'Onboarding incomplete: % required task(s) open', v_open;
    END IF;

    UPDATE hr.employees
    SET status = 'Active', updated_at = now()
    WHERE uuid = p_employee AND status = 'Onboarding'
    RETURNING * INTO v_row;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Employee not in Onboarding status';
    END IF;

    PERFORM hr.log_event(p_employee, 'ONBOARDING_COMPLETED', 'Onboarding', 'Active', NULL, NULL, p_by);
    PERFORM hr.log_event(p_employee, 'ACTIVATED', 'Onboarding', 'Active', NULL, NULL, p_by);
    RETURN v_row;
END;
$$;

-- Start exit → Notice
CREATE OR REPLACE FUNCTION hr.start_exit(
    p_employee uuid,
    p_last_working date,
    p_reason text,
    p_notice_start date DEFAULT CURRENT_DATE,
    p_notes text DEFAULT NULL,
    p_by uuid DEFAULT NULL
) RETURNS hr.employees
LANGUAGE plpgsql AS $$
DECLARE
    v_from text;
    v_row hr.employees%ROWTYPE;
BEGIN
    SELECT status INTO v_from FROM hr.employees WHERE uuid = p_employee FOR UPDATE;
    IF v_from NOT IN ('Active', 'Suspended') THEN
        RAISE EXCEPTION 'Cannot start exit from status %', v_from;
    END IF;

    UPDATE hr.employees SET status = 'Notice', updated_at = now()
    WHERE uuid = p_employee RETURNING * INTO v_row;

    INSERT INTO hr.employee_lifecycle AS lc (employee_uuid, notice_start_date, last_working_date, exit_reason_code, exit_notes)
    VALUES (p_employee, p_notice_start, p_last_working, p_reason, p_notes)
    ON CONFLICT (employee_uuid) DO UPDATE SET
        notice_start_date = EXCLUDED.notice_start_date,
        last_working_date = EXCLUDED.last_working_date,
        exit_reason_code = EXCLUDED.exit_reason_code,
        exit_notes = EXCLUDED.exit_notes,
        updated_at = now();

    PERFORM hr.seed_offboarding_tasks(p_employee);
    PERFORM hr.log_event(p_employee, 'NOTICE_STARTED', v_from, 'Notice', p_reason, p_notes, p_by, p_notice_start);
    RETURN v_row;
END;
$$;

-- Complete exit → Terminated
CREATE OR REPLACE FUNCTION hr.complete_exit(p_employee uuid, p_by uuid DEFAULT NULL)
RETURNS hr.employees
LANGUAGE plpgsql AS $$
DECLARE
    v_open int;
    v_row hr.employees%ROWTYPE;
    v_term date;
BEGIN
    SELECT count(*) INTO v_open
    FROM hr.offboarding_tasks
    WHERE employee_uuid = p_employee AND is_required AND NOT is_done;

    IF v_open > 0 THEN
        RAISE EXCEPTION 'Offboarding incomplete: % required task(s) open', v_open;
    END IF;

    SELECT COALESCE(last_working_date, CURRENT_DATE) INTO v_term
    FROM hr.employee_lifecycle WHERE employee_uuid = p_employee;

    UPDATE hr.employees
    SET status = 'Terminated', updated_at = now()
    WHERE uuid = p_employee AND status = 'Notice'
    RETURNING * INTO v_row;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Employee not in Notice status';
    END IF;

    UPDATE hr.employee_lifecycle
    SET termination_date = COALESCE(termination_date, v_term), updated_at = now()
    WHERE employee_uuid = p_employee;

    UPDATE hr.employment_contracts
    SET status = 'Terminated', updated_at = now()
    WHERE employee_uuid = p_employee AND status = 'Active';

    PERFORM hr.log_event(p_employee, 'TERMINATED', 'Notice', 'Terminated', NULL, NULL, p_by, v_term);
    RETURN v_row;
END;
$$;

-- Payroll eligibility (HR rule, payroll calls this)
CREATE OR REPLACE FUNCTION hr.is_payroll_eligible(
    p_employee uuid,
    p_period_start date,
    p_period_end date
) RETURNS boolean
LANGUAGE sql STABLE AS $$
    SELECT EXISTS (
        SELECT 1
        FROM hr.employees e
        LEFT JOIN hr.employee_lifecycle lc ON lc.employee_uuid = e.uuid
        WHERE e.uuid = p_employee
          AND e.status IN ('Active', 'Notice')
          AND e.hire_date <= p_period_end
          AND (lc.termination_date IS NULL OR lc.termination_date >= p_period_start)
    );
$$;

CREATE OR REPLACE VIEW hr.v_employee_directory AS
SELECT
    e.uuid, e.serial_id,
    e.first_name, e.last_name, e.middle_name,
    e.email, e.work_email, e.phone_number,
    e.hire_date, e.status, e.employment_type, e.pay_frequency,
    d.name AS department_name,
    jt.title AS job_title,
    s.first_name || ' ' || s.last_name AS supervisor_name,
    e.department_uuid, e.job_title_uuid, e.supervisor_uuid,
    e.created_at, e.updated_at
FROM hr.employees e
LEFT JOIN hr.departments d ON d.uuid = e.department_uuid
LEFT JOIN hr.job_titles jt ON jt.uuid = e.job_title_uuid
LEFT JOIN hr.employees s ON s.uuid = e.supervisor_uuid;

CREATE OR REPLACE VIEW hr.v_employee_payroll_snapshot AS
SELECT
    e.uuid AS employee_uuid,
    e.status,
    e.hire_date,
    lc.termination_date,
    lc.last_working_date,
    c.uuid AS contract_uuid,
    c.basic_salary,
    COALESCE(c.pay_frequency, e.pay_frequency) AS pay_frequency,
    COALESCE(c.employment_type, e.employment_type) AS employment_type,
    pp.bank_name,
    pp.bank_account_number,
    pp.payment_method,
    pp.currency_code,
    sp.tax_id,
    sp.social_security_number
FROM hr.employees e
LEFT JOIN hr.employee_lifecycle lc ON lc.employee_uuid = e.uuid
LEFT JOIN hr.employment_contracts c
    ON c.employee_uuid = e.uuid AND c.status = 'Active'
LEFT JOIN hr.employee_payment_profiles pp ON pp.employee_uuid = e.uuid
LEFT JOIN hr.employee_statutory_profiles sp ON sp.employee_uuid = e.uuid;

CREATE OR REPLACE VIEW hr.v_onboarding_queue AS
SELECT e.uuid, e.first_name, e.last_name, e.hire_date,
       count(*) FILTER (WHERE t.is_required AND NOT t.is_done) AS open_required_tasks
FROM hr.employees e
JOIN hr.onboarding_tasks t ON t.employee_uuid = e.uuid
WHERE e.status = 'Onboarding'
GROUP BY 1,2,3,4;

CREATE OR REPLACE VIEW hr.v_offboarding_queue AS
SELECT e.uuid, e.first_name, e.last_name, lc.last_working_date, e.status,
       count(*) FILTER (WHERE t.is_required AND NOT t.is_done) AS open_required_tasks
FROM hr.employees e
LEFT JOIN hr.employee_lifecycle lc ON lc.employee_uuid = e.uuid
JOIN hr.offboarding_tasks t ON t.employee_uuid = e.uuid
WHERE e.status IN ('Notice', 'Terminated')
GROUP BY 1,2,3,4,5;

-- Enable required extension
-- CREATE EXTENSION IF NOT EXISTS pg_uuidv7;
-- Create schema
/*CREATE SCHEMA IF NOT EXISTS hr;
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
    job_title uuid,
    department_uuid uuid,
    supervisor_uuid uuid,
    employment_type varchar(20) CHECK (
        employment_type IN ('Full-time', 'Part-time', 'Contract')
    ),
    salary numeric(14, 2),
    pay_frequency varchar(20) CHECK (
        pay_frequency IN ('Monthly', 'Weekly', 'Bi-weekly')
    ),
    status varchar(20) DEFAULT 'Active',
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT employees_pkey PRIMARY KEY (uuid),
    CONSTRAINT employees_serial_id_key UNIQUE (serial_id),
    CONSTRAINT employees_department_uuid_fkey FOREIGN KEY (department_uuid) REFERENCES hr.departments(uuid) ON DELETE
    SET NULL,
        CONSTRAINT employees_supervisor_uuid_fkey FOREIGN KEY (supervisor_uuid) REFERENCES hr.employees(uuid) ON DELETE
    SET NULL
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
*/
