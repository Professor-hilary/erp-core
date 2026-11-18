-- ==============================================================================
-- MANUFACTURING MODULE - FULL SCHEMA (GLOBAL-READY)
-- Run this ONCE after all core modules exist:
--   accounting, procurement, sales, inventory
-- ==============================================================================

-- Create schema
CREATE SCHEMA IF NOT EXISTS manufacturing;

-- ============================================================================
-- TABLES
-- ============================================================================

-- Production Orders / Jobs (header)
CREATE table if not exists manufacturing.production_orders (
    uuid uuid DEFAULT uuidv7 () PRIMARY KEY,
    serial_id bigint GENERATED ALWAYS AS IDENTITY UNIQUE,
    order_number text NOT NULL UNIQUE,
    product_item_uuid uuid NOT NULL, -- references inventory.items
    quantity_ordered numeric(12, 4) NOT NULL,
    quantity_completed numeric(12, 4) DEFAULT 0,
    start_date date,
    expected_completion_date date,
    material_usage_variance numeric(18, 2) DEFAULT 0,
    labor_efficiency_variance numeric(18, 2) DEFAULT 0,
    overhead_variance numeric(18, 2) DEFAULT 0,
    total_variance numeric(18, 2) GENERATED ALWAYS AS (
        material_usage_variance + labor_efficiency_variance + overhead_variance
    ) STORED,
    actual_completion_date date,
    status text DEFAULT 'Planned' CHECK (
        status IN (
            'Planned',
            'In Progress',
            'Completed',
            'Cancelled'
        )
    ),
    materials_status text DEFAULT 'Pending' CHECK (
        materials_status IN (
            'Pending',
            'Partial',
            'Complete'
        )
    ),
    gl_transaction_uuid uuid, -- link to posted entries
    -- Spec pinned at release (FKs added after bom/routing tables exist)
    bom_header_uuid uuid,
    routing_uuid uuid,
    standard_unit_cost numeric(18, 4),
    fg_warehouse_serial bigint, -- finished-goods destination; null → product.warehouse_serial
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now()
);

-- helper log table (recommended for audit trail)
CREATE TABLE IF NOT EXISTS accounting.variance_proration_logs (
    uuid uuid DEFAULT uuidv7 () PRIMARY KEY,
    proration_date date NOT NULL,
    variance_amount numeric(18, 2) NOT NULL,
    wip_alloc numeric(18, 2),
    fg_alloc numeric(18, 2),
    cogs_alloc numeric(18, 2),
    gl_transaction_uuid uuid REFERENCES accounting.transactions (uuid),
    memo text,
    created_by uuid,
    created_at timestamptz DEFAULT now()
);

-- ============================================================================
-- Bill of Materials (BOM) – Header & Lines
-- ============================================================================
CREATE TABLE IF NOT EXISTS manufacturing.bom_headers (
    uuid uuid DEFAULT uuidv7 () PRIMARY KEY,
    serial_id bigint GENERATED ALWAYS AS IDENTITY UNIQUE,
    bom_code text NOT NULL UNIQUE, -- e.g. BOM-FG-001
    product_item_uuid uuid NOT NULL REFERENCES inventory.items (uuid) ON DELETE RESTRICT,
    description text,
    revision text DEFAULT 'A' NOT NULL,
    is_active boolean DEFAULT true,
    is_default boolean DEFAULT false, -- can have multiple BOMs per product
    created_by uuid,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT bom_headers_product_revision_unique UNIQUE (product_item_uuid, revision)
);
-- At most one default active BOM per product
CREATE UNIQUE INDEX IF NOT EXISTS bom_one_default_active_per_product
    ON manufacturing.bom_headers (product_item_uuid)
    WHERE is_default = true AND is_active = true;


CREATE TABLE IF NOT EXISTS manufacturing.bom_lines (
    uuid uuid DEFAULT uuidv7 () PRIMARY KEY,
    bom_header_uuid uuid NOT NULL REFERENCES manufacturing.bom_headers (uuid) ON DELETE CASCADE,
    line_number smallint NOT NULL,
    component_item_uuid uuid NOT NULL REFERENCES inventory.items (uuid) ON DELETE RESTRICT,
    child_bom_uuid uuid REFERENCES manufacturing.bom_headers (uuid) ON DELETE RESTRICT,
    is_phantom boolean NOT NULL DEFAULT false,
    quantity_per numeric(18, 6) NOT NULL CHECK (quantity_per > 0),
    uom text DEFAULT 'pcs',
    scrap_factor numeric(5, 4) DEFAULT 1.0000, -- 1.05 = 5% expected scrap
    notes text,
    created_at timestamptz DEFAULT now(),
    UNIQUE (bom_header_uuid, line_number),
    UNIQUE (
        bom_header_uuid,
        component_item_uuid
    ) -- no duplicate components
);

-- For tracking material availability for production
CREATE TABLE manufacturing.production_order_materials (
    uuid uuid DEFAULT uuidv7 () PRIMARY KEY,
    production_order_uuid uuid NOT NULL REFERENCES manufacturing.production_orders (uuid) ON DELETE CASCADE,
    bom_line_uuid uuid REFERENCES manufacturing.bom_lines (uuid),
    component_item_uuid uuid NOT NULL REFERENCES inventory.items (uuid),
    required_qty numeric(18, 6) NOT NULL,
    issued_qty numeric(18, 6) DEFAULT 0,
    preferred_warehouse_serial bigint, -- optional per-line source WH; null → item default then bulk default
    status text DEFAULT 'Pending' CHECK (
        status IN (
            'Pending',
            'PartiallyIssued',
            'FullyIssued'
        )
    ),
    created_at timestamptz DEFAULT now(),
    UNIQUE (
        production_order_uuid,
        component_item_uuid
    )
);

-- Optional: index for fast BOM explosion
CREATE INDEX idx_bom_lines_header ON manufacturing.bom_lines (bom_header_uuid);

CREATE INDEX idx_bom_lines_component ON manufacturing.bom_lines (component_item_uuid);

-- Material Issues / Consumption (to WIP)
CREATE table if not exists manufacturing.material_issues (
    uuid uuid DEFAULT uuidv7 () PRIMARY KEY,
    production_order_uuid uuid NOT NULL,
    stock_item_id bigint NOT NULL,
    stock_item_name text NOT NULL,
    quantity numeric(12, 4) NOT NULL,
    unit_cost numeric(18, 4) NOT NULL, -- from costing method
    total_cost numeric(18, 2) GENERATED ALWAYS AS (quantity * unit_cost) STORED,
    warehouse_uuid uuid REFERENCES inventory.warehouses (uuid),
    issued_at timestamptz DEFAULT now(),
    CONSTRAINT fk_material_issues_order FOREIGN KEY (production_order_uuid) REFERENCES manufacturing.production_orders (uuid)
);

-- Labor & Overhead Application (to WIP)
CREATE table if not exists manufacturing.cost_applications (
    uuid uuid DEFAULT uuidv7 () PRIMARY KEY,
    production_order_uuid uuid NOT NULL,
    type text CHECK (
        type IN (
            'DirectLabor',      -- routing-applied (standard) labour
            'ActualLabor',      -- time/payroll-approved actual direct labour
            'IndirectLabor',    -- indirect labour charged to production/OH
            'DirectMaterial',
            'AppliedOverhead',  -- predetermined OH applied to WIP
            'ActualOverhead'    -- actual OH expenditure recognised
        )
    ),
    amount numeric(18, 2) NOT NULL,
    source_account uuid,
    destination_account uuid,
    applied_at timestamptz DEFAULT now(),
    reference text,
    
CONSTRAINT fk_applications_order FOREIGN KEY (production_order_uuid) REFERENCES manufacturing.production_orders (uuid)
);

-- Approved labour time / cost allocations (actual labour, distinct from routing-applied)
CREATE TABLE IF NOT EXISTS manufacturing.labour_allocations (
    uuid uuid DEFAULT uuidv7() PRIMARY KEY,
    production_order_uuid uuid NOT NULL REFERENCES manufacturing.production_orders (uuid) ON DELETE CASCADE,
    labour_class text NOT NULL CHECK (labour_class IN ('Direct', 'Indirect')),
    hours numeric(18, 4) NOT NULL CHECK (hours > 0),
    rate numeric(18, 4) NOT NULL CHECK (rate >= 0),
    amount numeric(18, 2) GENERATED ALWAYS AS (ROUND(hours * rate, 2)) STORED,
    work_center_code text,
    employee_uuid uuid,              -- optional link to hr.employees
    source text NOT NULL DEFAULT 'Manual'
        CHECK (source IN ('Manual', 'Payroll', 'TimeEntry')),
    source_ref text,                 -- payslip uuid, timesheet id, etc.
    memo text,
    status text NOT NULL DEFAULT 'Draft'
        CHECK (status IN ('Draft', 'Posted', 'Reversed')),
    cost_application_uuid uuid,
    posted_at timestamptz,
    created_by uuid,
    created_at timestamptz DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_labour_alloc_order
    ON manufacturing.labour_allocations (production_order_uuid);


-- Completions (WIP → Finished Goods)
CREATE TABLE if not exists manufacturing.completions (
    uuid uuid DEFAULT uuidv7 () PRIMARY KEY,
    production_order_uuid uuid NOT NULL,
    quantity_completed numeric(12, 4) NOT NULL,
    unit_cost numeric(18, 4) NOT NULL, -- calculated total cost / qty
    completed_at timestamptz DEFAULT now(),
    CONSTRAINT fk_completions_order FOREIGN KEY (production_order_uuid) REFERENCES manufacturing.production_orders (uuid)
);

-- Predetermined Overhead Rate Table (new table)
-- Most manufacturing systems store the rate per period/division (e.g. per direct labor hour, machine hour, or direct labor cost).
CREATE TABLE IF NOT EXISTS manufacturing.overhead_rates (
    uuid uuid DEFAULT uuidv7 () PRIMARY KEY,
    period_start date NOT NULL,
    period_end date NOT NULL,
    allocation_base text NOT NULL CHECK (
        allocation_base IN (
            'DirectLaborHours',
            'DirectLaborCost',
            'MachineHours',
            'MaterialCost'
        )
    ),
    estimated_overhead numeric(18, 2) NOT NULL, -- budgeted total overhead for period
    estimated_base numeric(18, 4) NOT NULL, -- budgeted activity level (hours, cost, etc.)
    rate numeric(18, 6) GENERATED ALWAYS AS (
        CASE
            WHEN estimated_base = 0 THEN 0
            ELSE estimated_overhead / estimated_base
        END
    ) STORED,
    department_code text, -- optional: if multiple production departments
    is_active boolean DEFAULT true,
    status text CHECK (
        status IN ('Draft', 'Active', 'Archived')
    ) DEFAULT 'Active',
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT unique_period_base UNIQUE (
        period_start,
        allocation_base,
        department_code
    )
);

CREATE TABLE IF NOT EXISTS manufacturing.overhead_actuals (
    uuid uuid DEFAULT uuidv7 () PRIMARY KEY,
    account_uuid uuid NOT NULL,
    amount numeric(18, 2) NOT NULL,
    incurred_at timestamptz DEFAULT now(),
    reference text
);

CREATE TABLE IF NOT EXISTS manufacturing.work_centers (
    uuid UUID PRIMARY KEY DEFAULT uuidv7 (),
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    labor_rate NUMERIC NOT NULL,
    allocation_base TEXT NOT NULL CHECK (
        allocation_base IN (
            'DirectLaborHours',
            'MachineHours'
        )
    ),
    department_code TEXT,
    created_at timestamptz DEFAULT now()
);

-- Routings / Operations (for labor & machine standards)
CREATE TABLE IF NOT EXISTS manufacturing.routings (
    uuid uuid DEFAULT uuidv7 () PRIMARY KEY,
    product_item_uuid uuid REFERENCES inventory.items (uuid),
    routing_code text UNIQUE,
    notes text,
    version TEXT NOT NULL,
    base_quantity NUMERIC NOT NULL DEFAULT 1,
    effective_date DATE NOT NULL DEFAULT CURRENT_DATE,
    status text DEFAULT 'active',
    is_default boolean DEFAULT false,
    created_at timestamptz DEFAULT now()
);

-- Production order pins (after BOM / routing tables exist)
DO $$ BEGIN
    ALTER TABLE manufacturing.production_orders
        ADD CONSTRAINT fk_po_bom_header
        FOREIGN KEY (bom_header_uuid) REFERENCES manufacturing.bom_headers (uuid);
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;
DO $$ BEGIN
    ALTER TABLE manufacturing.production_orders
        ADD CONSTRAINT fk_po_routing
        FOREIGN KEY (routing_uuid) REFERENCES manufacturing.routings (uuid);
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS manufacturing.routing_operations (
    uuid uuid DEFAULT uuidv7 () PRIMARY KEY,
    routing_uuid uuid REFERENCES manufacturing.routings (uuid) ON DELETE CASCADE,
    sequence smallint NOT NULL,
    operation_name TEXT NOT NULL,
    work_center text REFERENCES manufacturing.work_centers (code),
    description text,
    setup_time_minutes numeric(12, 2) NOT NULL,
    run_time_minutes numeric(18, 2) NOT NULL,
    created_at timestamptz DEFAULT now(),
    UNIQUE (routing_uuid, sequence)
);

-- ============================================================================
-- FUNCTIONS
-- ============================================================================

CREATE OR REPLACE FUNCTION manufacturing.calculate_standard_cost(
    p_item_uuid uuid,
    p_costing_date date DEFAULT CURRENT_DATE
)
RETURNS numeric(18,4)
LANGUAGE plpgsql
AS $$
DECLARE
    v_material_cost numeric := 0;
    v_labor_cost    numeric := 0;
    v_overhead_cost numeric := 0;
    v_bom_uuid uuid;
    v_routing_uuid uuid;
    v_base_quantity numeric := 1;
BEGIN
    SELECT bh.uuid
    INTO v_bom_uuid
    FROM manufacturing.bom_headers bh
    WHERE bh.product_item_uuid = p_item_uuid
      AND bh.is_active
    ORDER BY bh.is_default DESC, bh.created_at DESC
    LIMIT 1;

    IF v_bom_uuid IS NOT NULL THEN
        SELECT COALESCE(SUM(e.quantity_required * COALESCE(i.standard_cost, 0)), 0)
        INTO v_material_cost
        FROM manufacturing.explode_bom(v_bom_uuid, 1) e
        JOIN inventory.items i ON i.uuid = e.component_item_uuid
        WHERE e.is_leaf = true
          AND COALESCE(e.is_phantom, false) = false;
    END IF;

    SELECT r.uuid, COALESCE(NULLIF(r.base_quantity, 0), 1)
    INTO v_routing_uuid, v_base_quantity
    FROM manufacturing.routings r
    WHERE r.product_item_uuid = p_item_uuid
      AND r.status = 'active'
    ORDER BY r.is_default DESC, r.created_at DESC
    LIMIT 1;

    IF v_routing_uuid IS NOT NULL THEN
        SELECT COALESCE(SUM(
            (
                COALESCE(ro.setup_time_minutes, 0) / v_base_quantity
              + COALESCE(ro.run_time_minutes, 0) / v_base_quantity
            ) / 60.0 * wc.labor_rate
        ), 0)
        INTO v_labor_cost
        FROM manufacturing.routing_operations ro
        JOIN manufacturing.work_centers wc ON wc.code = ro.work_center
        WHERE ro.routing_uuid = v_routing_uuid;

        SELECT COALESCE(SUM(
            (
                COALESCE(ro.setup_time_minutes, 0) / v_base_quantity
              + COALESCE(ro.run_time_minutes, 0) / v_base_quantity
            ) / 60.0 * ohr.rate
        ), 0)
        INTO v_overhead_cost
        FROM manufacturing.routing_operations ro
        JOIN manufacturing.work_centers wc ON wc.code = ro.work_center
        JOIN LATERAL (
            SELECT rate
            FROM manufacturing.overhead_rates ohr
            WHERE ohr.is_active
              AND ohr.allocation_base = wc.allocation_base
              AND (ohr.department_code IS NULL OR ohr.department_code = wc.department_code)
              AND p_costing_date BETWEEN ohr.period_start AND ohr.period_end
            ORDER BY ohr.period_start DESC, ohr.created_at DESC
            LIMIT 1
        ) ohr ON true
        WHERE ro.routing_uuid = v_routing_uuid;
    END IF;

    RETURN ROUND(v_material_cost + v_labor_cost + v_overhead_cost, 4);
END;
$$;


-- Helper Function: Apply Overhead to a Production Order
CREATE OR REPLACE FUNCTION manufacturing.apply_overhead_to_order(
    p_production_order      text,
    p_activity_amount       numeric, -- e.g. labor hours used on this order
    p_wip_account           text,
    p_overhead_control_code text,
    p_allocation_rate       text,
    p_user                  uuid
) RETURNS manufacturing.cost_applications -- return full row
LANGUAGE plpgsql
AS $$
DECLARE
    v_rate                  numeric(18,6);
    v_applied_amount        numeric(18,2);
    v_wip_account_uuid      uuid;
    v_overhead_control_uuid uuid;
    v_txn_serial_id         bigint;
    v_order                 manufacturing.production_orders%ROWTYPE;
    v_application           manufacturing.cost_applications%ROWTYPE;
BEGIN
    -- Get current production order
    SELECT * INTO STRICT v_order FROM manufacturing.production_orders WHERE order_number = p_production_order;

    -- Rate must cover order start_date (else today)
    SELECT rate INTO v_rate
    FROM manufacturing.overhead_rates
    WHERE is_active
      AND allocation_base = p_allocation_rate
      AND COALESCE(v_order.start_date, CURRENT_DATE)
          BETWEEN period_start AND period_end
    ORDER BY period_start DESC, created_at DESC
    LIMIT 1;

    IF v_rate IS NULL THEN
        RAISE EXCEPTION
            'No active predetermined overhead rate for base % covering date %',
            p_allocation_rate,
            COALESCE(v_order.start_date, CURRENT_DATE);
    END IF;

    v_applied_amount := ROUND(p_activity_amount * v_rate, 2);

    -- Get account UUIDs (adjust codes if needed)
    v_wip_account_uuid := accounting.resolve_account_ref(p_wip_account);

    v_overhead_control_uuid := accounting.resolve_account_ref(p_overhead_control_code);

    -- Journal entry: Dr WIP, Cr Applied Overhead
    v_txn_serial_id := accounting.post_transaction(
        'OVERHEAD-APPLY-' || v_order.order_number,
        'Applied Manufacturing Overhead',
        p_user,
        'manufacturing',
        CURRENT_DATE,
        jsonb_build_array(
            jsonb_build_object(
                'account_ref', v_wip_account_uuid, 'debit', v_applied_amount, 'credit', 0,
                'memo', format(
                    'Applied overhead to production order %s - base %s x rate %s',
                    p_production_order, p_activity_amount, v_rate
                )
            ),
            jsonb_build_object(
                'account_ref', v_overhead_control_uuid, 'debit', 0, 'credit', v_applied_amount,
                'memo', format(
                    'Applied overhead to production order %s - base %s x rate %s',
                    p_production_order, p_activity_amount, v_rate
				)
            )
        )
    );

    -- Record the application (for later reporting)
    INSERT INTO manufacturing.cost_applications (
        production_order_uuid, type, amount, reference, source_account,
        destination_account, applied_at
    ) VALUES (
        v_order.uuid,
        'AppliedOverhead',
        v_applied_amount,
        format(
            'Applied at rate %s x base %s (txn %s)',
            v_rate, p_activity_amount,
            (SELECT uuid FROM accounting.transactions WHERE serial_id = v_txn_serial_id)
        ),
        v_overhead_control_uuid,
        v_wip_account_uuid,
        now()
    )
    RETURNING * INTO v_application;

    -- Update order if needed (optional)
    UPDATE manufacturing.production_orders
    SET updated_at = now()
    WHERE uuid = v_order.uuid;

    RETURN v_application;
END;
$$;

-- EXAMPLE
-- SELECT manufacturing.apply_overhead_to_order(
--     order_number, 10 (labor hours), 'WIP CODE', 'MOH_CTRL', user_uuid
-- );
-- DR Work In Progress  80
--     CR Overhead Ctrl      80

CREATE OR REPLACE FUNCTION manufacturing.record_actual_overhead(
    p_production_order_uuid uuid,
    p_amount                numeric, -- actual cost (invoice, cash, etc.)
    p_credit_account_code   text, -- payable or cash
    p_overhead_control_code text,
    p_overhead_type         text,
    p_reference             text,
    p_user                  uuid
) RETURNS manufacturing.cost_applications
LANGUAGE plpgsql
AS $$
DECLARE
    v_credit_account_uuid   uuid;
    v_control_account_uuid  uuid;
    v_txn_serial            bigint;
    v_application           manufacturing.cost_applications%ROWTYPE;
    v_order                 manufacturing.production_orders%ROWTYPE;
BEGIN
    -- GET account UUIDs
    v_credit_account_uuid := accounting.resolve_account_ref(p_credit_account_code);

    v_control_account_uuid := accounting.resolve_account_ref(p_overhead_control_code);

    -- Get current production order
    SELECT * INTO STRICT v_order FROM manufacturing.production_orders WHERE uuid = p_production_order_uuid;

    -- Post ACTUAL Overhead
    -- Dr MOH Control
    -- Cr Cash / Payables
    v_txn_serial := accounting.post_transaction(
        'OVERHEAD-ACTUAL-' || v_order.order_number,
        'Actual Manufacturing Overhead',
        p_user,
        'manufacturing',
        CURRENT_DATE,
        jsonb_build_array(
            jsonb_build_object(
                'account_ref', v_control_account_uuid,
                'debit', p_amount,
                'credit', 0,
                'memo', format(
                    'Actual overhead (%s) for order %s: %s',
                    p_overhead_type, v_order.order_number, p_reference
                )
            ),
            jsonb_build_object(
                'account_ref', v_credit_account_uuid,
                'debit', 0,
                'credit', p_amount,
                'memo', format(
                    'Off-setting account for order %s: %s',
                    v_order.order_number, p_reference
                )
            )
        )
    );

    -- Log
    INSERT INTO manufacturing.cost_applications(
        production_order_uuid, type, amount, reference, applied_at,
        source_account, destination_account
    ) VALUES (
        p_production_order_uuid,
        'ActualOverhead',
        p_amount,
        format('Actual overhead: %s', p_reference),
        now(),
        v_credit_account_uuid,
        v_control_account_uuid
    ) RETURNING * INTO v_application;

    RETURN v_application;
END;
$$;

-- EXAMPLE
-- SELECT manufacturing.record_actual_overhead(
--     order_uuid, 100, 'AP/CASH CODE', 'MOH-CTRL', 'electricity', user_uuid
-- );
-- DR MOH CONTROL      100
--     CR ACC PAYABLE      100

-- Function for both direct (tied to routing and peoduction order)
CREATE OR REPLACE FUNCTION manufacturing.routing_labour_for_qty(
    p_product_item_uuid uuid,
    p_qty numeric
)
RETURNS TABLE (
    total_hours numeric,
    total_cost  numeric
)
LANGUAGE plpgsql
STABLE
AS $$
DECLARE
    v_routing_uuid uuid;
    v_base_quantity numeric := 1;
BEGIN
    SELECT r.uuid, COALESCE(NULLIF(r.base_quantity, 0), 1)
    INTO v_routing_uuid, v_base_quantity
    FROM manufacturing.routings r
    WHERE r.product_item_uuid = p_product_item_uuid
      AND r.status = 'active'
    ORDER BY r.is_default DESC, r.created_at DESC
    LIMIT 1;

    IF v_routing_uuid IS NULL OR p_qty IS NULL OR p_qty <= 0 THEN
        total_hours := 0;
        total_cost := 0;
        RETURN NEXT;
        RETURN;
    END IF;

    SELECT
        COALESCE(SUM(
            (
                COALESCE(ro.setup_time_minutes, 0)
              + COALESCE(ro.run_time_minutes, 0) * (p_qty / v_base_quantity)
            ) / 60.0
        ), 0),
        COALESCE(SUM(
            (
                COALESCE(ro.setup_time_minutes, 0)
              + COALESCE(ro.run_time_minutes, 0) * (p_qty / v_base_quantity)
            ) / 60.0 * wc.labor_rate
        ), 0)
    INTO total_hours, total_cost
    FROM manufacturing.routing_operations ro
    JOIN manufacturing.work_centers wc ON wc.code = ro.work_center
    WHERE ro.routing_uuid = v_routing_uuid;

    RETURN NEXT;
END;
$$;


-- Post approved actual labour (Direct → WIP; Indirect → OH control or WIP per flag)
CREATE OR REPLACE FUNCTION manufacturing.post_labour_allocation(
    p_allocation_uuid   uuid,
    p_user              uuid,
    p_wip_account       text,
    p_wages_account     text,
    p_oh_control_account text DEFAULT NULL  -- required for Indirect if not posting to WIP
) RETURNS manufacturing.cost_applications
LANGUAGE plpgsql
AS $$
DECLARE
    v_alloc     manufacturing.labour_allocations%ROWTYPE;
    v_order     manufacturing.production_orders%ROWTYPE;
    v_wip       uuid;
    v_wages     uuid;
    v_oh        uuid;
    v_dest      uuid;
    v_type      text;
    v_txn       bigint;
    v_app       manufacturing.cost_applications%ROWTYPE;
BEGIN
    SELECT * INTO STRICT v_alloc
    FROM manufacturing.labour_allocations
    WHERE uuid = p_allocation_uuid
    FOR UPDATE;

    IF v_alloc.status <> 'Draft' THEN
        RAISE EXCEPTION 'Labour allocation % is not Draft (status=%)', p_allocation_uuid, v_alloc.status;
    END IF;

    SELECT * INTO STRICT v_order
    FROM manufacturing.production_orders
    WHERE uuid = v_alloc.production_order_uuid;

    IF v_order.status NOT IN ('Planned', 'In Progress') THEN
        RAISE EXCEPTION 'Order must be Planned or In Progress to post labour';
    END IF;

    v_wip := accounting.resolve_account_ref(p_wip_account);
    v_wages := accounting.resolve_account_ref(p_wages_account);

    IF v_alloc.labour_class = 'Direct' THEN
        v_type := 'ActualLabor';
        v_dest := v_wip;
    ELSE
        v_type := 'IndirectLabor';
        IF p_oh_control_account IS NULL OR btrim(p_oh_control_account) = '' THEN
            -- Indirect absorbed into WIP when no OH control supplied
            v_dest := v_wip;
        ELSE
            v_oh := accounting.resolve_account_ref(p_oh_control_account);
            v_dest := v_oh;
        END IF;
    END IF;

    v_txn := accounting.post_transaction(
        v_order.order_number || '-LAB-' || v_alloc.uuid::text,
        format(
            '%s labour %s hrs @ %s (%s)',
            v_alloc.labour_class, v_alloc.hours, v_alloc.rate, COALESCE(v_alloc.source, 'Manual')
        ),
        p_user,
        'manufacturing',
        CURRENT_DATE,
        jsonb_build_array(
            jsonb_build_object(
                'account_ref', v_dest, 'debit', v_alloc.amount, 'credit', 0,
                'memo', COALESCE(v_alloc.memo, 'Labour allocation')
            ),
            jsonb_build_object(
                'account_ref', v_wages, 'debit', 0, 'credit', v_alloc.amount,
                'memo', format('Wages payable - %s labour', v_alloc.labour_class)
            )
        )
    );

    INSERT INTO manufacturing.cost_applications (
        production_order_uuid, type, amount, reference, source_account,
        destination_account, applied_at
    ) VALUES (
        v_alloc.production_order_uuid,
        v_type,
        v_alloc.amount,
        format('Labour alloc %s source=%s ref=%s', v_alloc.uuid, v_alloc.source, COALESCE(v_alloc.source_ref, '')),
        v_wages,
        v_dest,
        now()
    )
    RETURNING * INTO v_app;

    UPDATE manufacturing.labour_allocations
    SET status = 'Posted',
        cost_application_uuid = v_app.uuid,
        posted_at = now()
    WHERE uuid = p_allocation_uuid;

    UPDATE manufacturing.production_orders
    SET status = CASE WHEN status = 'Planned' THEN 'In Progress' ELSE status END,
        updated_at = now()
    WHERE uuid = v_alloc.production_order_uuid;

    RETURN v_app;
END;
$$;

-- Create draft labour allocation (actual hours/rate)
CREATE OR REPLACE FUNCTION manufacturing.create_labour_allocation(
    p_production_order_uuid uuid,
    p_labour_class          text,
    p_hours                 numeric,
    p_rate                  numeric,
    p_work_center_code      text DEFAULT NULL,
    p_employee_uuid         uuid DEFAULT NULL,
    p_source                text DEFAULT 'Manual',
    p_source_ref            text DEFAULT NULL,
    p_memo                  text DEFAULT NULL,
    p_created_by            uuid DEFAULT NULL
) RETURNS manufacturing.labour_allocations
LANGUAGE plpgsql
AS $$
DECLARE
    v_row manufacturing.labour_allocations%ROWTYPE;
BEGIN
    IF p_labour_class NOT IN ('Direct', 'Indirect') THEN
        RAISE EXCEPTION 'labour_class must be Direct or Indirect';
    END IF;
    IF p_hours IS NULL OR p_hours <= 0 THEN
        RAISE EXCEPTION 'hours must be positive';
    END IF;
    IF p_rate IS NULL OR p_rate < 0 THEN
        RAISE EXCEPTION 'rate must be non-negative';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM manufacturing.production_orders
        WHERE uuid = p_production_order_uuid
          AND status IN ('Planned', 'In Progress')
    ) THEN
        RAISE EXCEPTION 'Production order not found or not open for labour';
    END IF;

    INSERT INTO manufacturing.labour_allocations (
        production_order_uuid, labour_class, hours, rate, work_center_code,
        employee_uuid, source, source_ref, memo, created_by
    ) VALUES (
        p_production_order_uuid, p_labour_class, p_hours, p_rate, p_work_center_code,
        p_employee_uuid, COALESCE(p_source, 'Manual'), p_source_ref, p_memo, p_created_by
    )
    RETURNING * INTO v_row;

    RETURN v_row;
END;
$$;

-- Routing-based APPLIED labour (standard hours × WC rate). For actual hours use labour_allocations.
CREATE OR REPLACE FUNCTION manufacturing.apply_direct_labor_from_routing(
    p_production_order_uuid uuid,
    p_user                  uuid,
    p_wip_account_code      text,
    p_labor_account_code    text,
    p_reference             text DEFAULT NULL
)
RETURNS manufacturing.cost_applications
LANGUAGE plpgsql
AS $$
DECLARE
    v_order              manufacturing.production_orders%ROWTYPE;
    v_routing_uuid       uuid;

    v_total_labor_cost   numeric(18,2) := 0;
    v_total_hours        numeric(18,4) := 0;

    v_wip_account_uuid   uuid;
    v_labor_account_uuid uuid;
    v_base_quantity          numeric;

    v_txn_serial         bigint;
    v_application        manufacturing.cost_applications%ROWTYPE;

    rec                  RECORD;
    v_time_hours         numeric;
BEGIN
    -- Get production order
    SELECT * INTO STRICT v_order
    FROM manufacturing.production_orders
    WHERE uuid = p_production_order_uuid;

    -- Prefer routing pinned on the order at release
    v_routing_uuid := v_order.routing_uuid;
    IF v_routing_uuid IS NULL THEN
        SELECT r.uuid INTO v_routing_uuid
        FROM manufacturing.routings r
        WHERE r.product_item_uuid = v_order.product_item_uuid
          AND r.status = 'active'
        ORDER BY r.is_default DESC, r.created_at DESC
        LIMIT 1;
    END IF;

    -- Get base quantity for time computation (latest active)
    SELECT r.base_quantity INTO v_base_quantity
    FROM manufacturing.routings r
    WHERE r.uuid = v_routing_uuid
      AND r.status = 'active'
    LIMIT 1;

    IF v_routing_uuid IS NULL THEN
        RAISE EXCEPTION 'No active routing found for product %',
            v_order.product_item_uuid;
    END IF;

    -- Prevent division by zero
    IF v_base_quantity IS NULL OR v_base_quantity = 0 THEN
        v_base_quantity := 1;
    END IF;

    -- Loop operations
    FOR rec IN
        SELECT ro.*, wc.labor_rate
        FROM manufacturing.routing_operations ro
        JOIN manufacturing.work_centers wc
          ON wc.code = ro.work_center
        WHERE ro.routing_uuid = v_routing_uuid
        ORDER BY ro.sequence
    LOOP
        -- Time in hours
        /*v_time_hours := (rec.run_time_minutes * (v_order.quantity_ordered / v_base_quantity))) / 60.0;*/

        v_time_hours := (
            COALESCE(rec.setup_time_minutes, 0) + (
                COALESCE(rec.run_time_minutes, 0) * v_order.quantity_ordered / v_base_quantity
            )
        ) / 60.0;

        v_total_hours := v_total_hours + v_time_hours;

        -- Labor cost
        v_total_labor_cost :=
            v_total_labor_cost + (v_time_hours * rec.labor_rate);
    END LOOP;

    v_total_labor_cost := ROUND(v_total_labor_cost, 2);

    -- Accounts
    v_wip_account_uuid := accounting.resolve_account_ref(p_wip_account_code);

    v_labor_account_uuid := accounting.resolve_account_ref(p_labor_account_code);

    -- Post GL
    v_txn_serial := accounting.post_transaction(
        format('DL-%s', v_order.order_number),
        format(
            'Direct labor for order %s: %s hrs%s',
            v_order.order_number,
            to_char(v_total_hours, 'FM999999990.00'),
            COALESCE(' - ' || p_reference, '')
        ),
        p_user,
        'manufacturing',
        CURRENT_DATE,
        jsonb_build_array(
            jsonb_build_object(
                'account_ref', v_wip_account_uuid,
                'debit', v_total_labor_cost,
                'credit', 0
            ),
            jsonb_build_object(
                'account_ref', v_labor_account_uuid,
                'debit', 0,
                'credit', v_total_labor_cost
            )
        )
    );

    -- Record cost application
    INSERT INTO manufacturing.cost_applications (
        production_order_uuid,
        type,
        amount,
        reference,
        applied_at,
        source_account,
        destination_account
    )
    VALUES (
        p_production_order_uuid,
        'DirectLabor',
        v_total_labor_cost,
        format(
            'Routing labor: %s hrs%s',
            to_char(v_total_hours, 'FM999999990.00'),
            COALESCE(' - ' || p_reference, '')
        ),
        now(),
        v_labor_account_uuid,
        v_wip_account_uuid
    )
    RETURNING * INTO v_application;

    -- Touch order
    UPDATE manufacturing.production_orders
    SET updated_at = now()
    WHERE uuid = p_production_order_uuid;

    RETURN v_application;
END;
$$;

-- ====================================================================================
-- Complete Production Order (Standard Costing)
-- ====================================================================================
CREATE OR REPLACE FUNCTION manufacturing.complete_production_order(
    p_order_uuid            uuid,
    p_completed_quantity    numeric(18,4),
    p_user                  uuid,
    p_control_account       text,
    p_wip_account           text,
    p_fg_account            text,
    p_mfg_mat_var_account   text,
    p_mfg_lab_var_account   text,
    p_mfg_moh_var_account   text,
    p_completion_date       date  DEFAULT CURRENT_DATE,
    p_fg_warehouse_serial   bigint DEFAULT NULL
) RETURNS uuid
LANGUAGE plpgsql
AS $$
DECLARE
    v_order                 manufacturing.production_orders%ROWTYPE;
    v_product               inventory.items%ROWTYPE;
    v_std_unit_cost         numeric(18,4);
    v_std_total             numeric(18,4);
    v_wip_uuid              uuid;
    v_woh_uuid              uuid;
    v_fg_uuid               uuid;
    v_txn_serial            bigint;
    v_new_completed         numeric(18,4);
    v_movement_uuid         uuid;
BEGIN
    -- 1. Fetch order
    SELECT * INTO v_order FROM manufacturing.production_orders WHERE uuid = p_order_uuid;
    IF v_order.status != 'In Progress' THEN
        RAISE EXCEPTION 'Order must be In Progress to complete';
    END IF;

    IF p_completed_quantity IS NULL OR p_completed_quantity <= 0 THEN
        RAISE EXCEPTION 'Completed quantity must be positive';
    END IF;
    IF p_completed_quantity > (v_order.quantity_ordered - v_order.quantity_completed) THEN
        RAISE EXCEPTION 'Completed quantity % exceeds remaining order quantity %',
            p_completed_quantity,
            v_order.quantity_ordered - v_order.quantity_completed;
    END IF;

    SELECT * INTO v_product FROM inventory.items WHERE uuid = v_order.product_item_uuid;

    -- FG destination: explicit param → order pin → product default
    -- (declared via nested block by using a simple subselect in movements)

    -- 2. Standard unit cost => must come from your calculate_standard_cost or enhanced version
    v_std_unit_cost := COALESCE(
        v_order.standard_unit_cost,
        manufacturing.calculate_standard_cost(v_product.uuid, COALESCE(v_order.start_date, CURRENT_DATE))
    );
    v_std_total := ROUND(p_completed_quantity * v_std_unit_cost, 2);

    -- 3. Get uuid for wip, fg and control accounts
    v_wip_uuid := accounting.resolve_account_ref(p_wip_account);
    v_fg_uuid := accounting.resolve_account_ref(p_fg_account);
    v_woh_uuid := accounting.resolve_account_ref(p_control_account);

    -- Applied overhead is already in WIP when apply_overhead_to_order runs
    -- (Dr WIP / Cr MOH control). Do NOT transfer again at completion.

    -- 5. Update order status to completed
    v_new_completed := v_order.quantity_completed + p_completed_quantity;

    UPDATE manufacturing.production_orders
    SET quantity_completed = v_new_completed,
        actual_completion_date = p_completion_date,
        status = CASE
            WHEN v_new_completed >= quantity_ordered THEN 'Completed'
            ELSE 'In Progress'
        END,
        updated_at = now()
    WHERE uuid = p_order_uuid;

    -- 6. Journal: Dr FG @ standard, Cr WIP @ standard
    v_txn_serial := accounting.post_transaction(
        v_order.order_number || '-COMPLETION',
        'Production Completion - Standard Cost',
        p_user, 'manufacturing', p_completion_date,
        jsonb_build_array(
            jsonb_build_object(
                'account_ref', v_fg_uuid, 'debit', v_std_total, 'credit', 0, 'memo', 'FG at standard cost'
            ),
            jsonb_build_object(
                'account_ref', v_wip_uuid, 'debit', 0, 'credit', v_std_total, 'memo', 'Relieve WIP at standard'
            )
        )
    );

    -- 7. Record inventory movement IN to FG, update product lots
    INSERT INTO inventory.movements (
        item_uuid, warehouse_uuid, movement_type, reference_type, reference_id,
        quantity, unit_cost, direction, gl_transaction_uuid, movement_date
    ) VALUES (
        v_product.uuid,
        (SELECT uuid FROM inventory.warehouses WHERE serial_id = COALESCE(
            p_fg_warehouse_serial, v_order.fg_warehouse_serial, v_product.warehouse_serial
        )),
        'PROD_COMPLETION',
        'production',
        v_order.serial_id,
        p_completed_quantity,
        v_std_unit_cost,
        'IN',
        (SELECT uuid FROM accounting.transactions WHERE serial_id = v_txn_serial),
        p_completion_date
    );

    -- 8. Record completion row – unit_cost MUST be standard
    INSERT INTO manufacturing.completions (
        production_order_uuid, quantity_completed, unit_cost, completed_at
    ) VALUES (
        p_order_uuid, p_completed_quantity, v_std_unit_cost, p_completion_date
    );

    -- 9. If FIFO/LIFO -> create new lot for FG
    IF v_product.valuation_method IN ('FIFO','LIFO') THEN
        INSERT INTO inventory.lots (
            item_uuid, warehouse_uuid, reference_type, reference_id,
            received_date, original_quantity, original_cost_per_unit, remaining_quantity
        ) VALUES (
            v_product.uuid,
            (SELECT uuid FROM inventory.warehouses WHERE serial_id = COALESCE(
            p_fg_warehouse_serial, v_order.fg_warehouse_serial, v_product.warehouse_serial
        )),
            'PROD_COMPLETION', v_order.serial_id,
            p_completion_date, p_completed_quantity, v_std_unit_cost, p_completed_quantity
        );
    ELSIF v_product.valuation_method = 'WAVG' THEN
        INSERT INTO inventory.wavg_warehouse (item_uuid, warehouse_uuid, total_quantity, total_cost)
        VALUES (
            v_product.uuid,
            (SELECT uuid FROM inventory.warehouses WHERE serial_id = COALESCE(
            p_fg_warehouse_serial, v_order.fg_warehouse_serial, v_product.warehouse_serial
        )),
            p_completed_quantity,
            v_std_total
        )
        ON CONFLICT (item_uuid, warehouse_uuid) DO UPDATE SET
            total_quantity = inventory.wavg_warehouse.total_quantity + p_completed_quantity,
            total_cost     = inventory.wavg_warehouse.total_cost     + (p_completed_quantity * v_std_unit_cost),
            last_updated_at = now();
    END IF;

    -- 10. Run variances ONLY when fully complete
    IF v_new_completed >= v_order.quantity_ordered THEN
        PERFORM manufacturing.calculate_and_post_variances(
            p_order_uuid, p_user, v_wip_uuid, p_mfg_mat_var_account,
            p_mfg_lab_var_account, p_mfg_moh_var_account, p_completion_date
        );
    END IF;

    RETURN (SELECT uuid FROM accounting.transactions WHERE serial_id = v_txn_serial);
END;
$$;

-- Move overhead to Work In Progress
CREATE OR REPLACE FUNCTION manufacturing.transfer_applied_overhead_to_wip(
    p_order_uuid        uuid,
    p_wip_account_uuid  uuid,
    p_overhead_control  uuid,
    p_user              uuid,
    p_completion_date   date DEFAULT CURRENT_DATE
)
RETURNS void
LANGUAGE plpgsql
AS $$
BEGIN
    -- apply_overhead_to_order already posts Dr WIP / Cr MOH control (AppliedOverhead).
    -- Re-posting at completion would double WIP and OH control. Kept as no-op for callers.
    RAISE NOTICE
        'transfer_applied_overhead_to_wip skipped for order % — overhead already applied to WIP',
        p_order_uuid;
    RETURN;
END;
$$;


-- ====================================================================================
-- Record Standard vs Actual Variance
-- ====================================================================================
CREATE OR REPLACE FUNCTION manufacturing.calculate_and_post_variances(
    p_order_uuid    uuid,
    p_user          uuid,
    v_wip_uuid      uuid,
    v_mat_var_code  text,
    v_lab_var_code  text,
    v_moh_var_code  text,
    p_date          date
) RETURNS void
LANGUAGE plpgsql
AS $$
DECLARE
    v_order         manufacturing.production_orders%ROWTYPE;
    v_completed_qty numeric(18,4);

    v_act_mat       numeric(18,2) := 0;
    v_act_lab       numeric(18,2) := 0;
    v_act_oh        numeric(18,2) := 0;
    v_app_oh        numeric(18,2) := 0;

    v_std_mat       numeric(18,2) := 0;
    v_std_lab       numeric(18,2) := 0;

    v_mat_var       numeric(18,2);
    v_lab_var       numeric(18,2);
    v_oh_var        numeric(18,2);

    -- v_wip_uuid      uuid;
    v_mat_var_uuid  uuid;
    v_lab_var_uuid  uuid;
    v_oh_var_uuid   uuid;
BEGIN
    SELECT * INTO STRICT v_order FROM manufacturing.production_orders WHERE uuid = p_order_uuid;

    IF v_order.status <> 'Completed' THEN
        RAISE NOTICE 'Order not completed → skipping variance posting';
        RETURN;
    END IF;

    v_completed_qty := v_order.quantity_completed;

    -- Actuals from materials issue table for requested production qty
    SELECT COALESCE(SUM(total_cost), 0) INTO v_act_mat
    FROM manufacturing.material_issues
    WHERE production_order_uuid = p_order_uuid;

    -- Actual labour only from ActualLabor (never invent from applied DirectLabor)
    SELECT COALESCE(SUM(amount), 0) INTO v_act_lab
    FROM manufacturing.cost_applications
    WHERE production_order_uuid = p_order_uuid AND type = 'ActualLabor';

    -- Overhead costs expensed e.g. bills and renumerations from cost applications
    SELECT COALESCE(SUM(amount), 0) INTO v_act_oh
    FROM manufacturing.cost_applications
    WHERE production_order_uuid = p_order_uuid AND type = 'ActualOverhead';

    -- Overhead standards derived using labor, material or machine rates
    SELECT COALESCE(SUM(amount), 0) INTO v_app_oh
    FROM manufacturing.cost_applications
    WHERE production_order_uuid = p_order_uuid AND type = 'AppliedOverhead';

    -- Standard material: pinned BOM on order, else current default
    SELECT COALESCE(SUM(e.quantity_required * COALESCE(i.standard_cost, 0)), 0)
    INTO v_std_mat
    FROM manufacturing.explode_bom(
        COALESCE(
            v_order.bom_header_uuid,
            (
                SELECT bh.uuid FROM manufacturing.bom_headers bh
                WHERE bh.product_item_uuid = v_order.product_item_uuid
                  AND bh.is_active
                ORDER BY bh.is_default DESC, bh.created_at DESC
                LIMIT 1
            )
        ),
        v_completed_qty
    ) e
    JOIN inventory.items i ON i.uuid = e.component_item_uuid
    WHERE e.is_leaf = true
      AND COALESCE(e.is_phantom, false) = false;

    -- Standard labour: pinned routing when present
    IF v_order.routing_uuid IS NOT NULL THEN
        SELECT COALESCE(SUM(
            (
                COALESCE(ro.setup_time_minutes, 0)
              + COALESCE(ro.run_time_minutes, 0) * (
                    v_completed_qty / COALESCE(NULLIF(r.base_quantity, 0), 1)
                )
            ) / 60.0 * wc.labor_rate
        ), 0)
        INTO v_std_lab
        FROM manufacturing.routings r
        JOIN manufacturing.routing_operations ro ON ro.routing_uuid = r.uuid
        JOIN manufacturing.work_centers wc ON wc.code = ro.work_center
        WHERE r.uuid = v_order.routing_uuid;
    ELSE
        SELECT COALESCE(rl.total_cost, 0)
        INTO v_std_lab
        FROM manufacturing.routing_labour_for_qty(v_order.product_item_uuid, v_completed_qty) rl;
    END IF;

    -- Variances
    v_mat_var := v_act_mat - v_std_mat;
    v_lab_var := v_act_lab - v_std_lab;
    v_oh_var  := v_act_oh  - v_app_oh;

    -- Get variance accounts (add these codes to your chart of accounts)
    -- v_wip_uuid := accounting.resolve_account_ref(v_wip_code);
    v_mat_var_uuid := accounting.resolve_account_ref(v_mat_var_code);
    v_lab_var_uuid := accounting.resolve_account_ref(v_lab_var_code);
    v_oh_var_uuid := accounting.resolve_account_ref(v_moh_var_code);

    -- Post material variance (unfavorable → debit variance, credit WIP)
    IF ABS(v_mat_var) > 0.01 THEN
        PERFORM accounting.post_transaction(
            v_order.order_number || '-MAT-VAR',
            'Material Usage/Price Variance',
            p_user, 'manufacturing', p_date,
            jsonb_build_array(
                jsonb_build_object('account_ref', v_mat_var_uuid, 'debit',  GREATEST(v_mat_var,0), 'credit', GREATEST(-v_mat_var,0)),
                jsonb_build_object('account_ref', v_wip_uuid,     'debit',  GREATEST(-v_mat_var,0), 'credit', GREATEST(v_mat_var,0))
            )
        );
    END IF;

    -- Labor variance
    IF ABS(v_lab_var) > 0.01 THEN
        PERFORM accounting.post_transaction(
            v_order.order_number || '-LAB-VAR',
            'Labor Rate/Efficiency Variance',
            p_user, 'manufacturing', p_date,
            jsonb_build_array(
                jsonb_build_object('account_ref', v_lab_var_uuid, 'debit',  GREATEST(v_lab_var,0), 'credit', GREATEST(-v_lab_var,0)),
                jsonb_build_object('account_ref', v_wip_uuid,     'debit',  GREATEST(-v_lab_var,0), 'credit', GREATEST(v_lab_var,0))
            )
        );
    END IF;

    -- Overhead variance → usually closes MOH control (we defer to period-end)
    -- Here we can just record to variance if small order; otherwise period close handles it

    -- Optional: store on order for reporting
    UPDATE manufacturing.production_orders
    SET material_usage_variance   = v_mat_var,
        labor_efficiency_variance = v_lab_var,
        overhead_variance = v_oh_var,
        updated_at = now()
    WHERE uuid = p_order_uuid;
END;
$$;

-- ====================================================================================
-- Explode multilayer bom
-- ====================================================================================
CREATE OR REPLACE FUNCTION manufacturing.explode_bom(
    p_bom_uuid   uuid,
    p_qty        numeric DEFAULT 1,
    p_path       uuid[]  DEFAULT ARRAY[]::uuid[]
)
RETURNS TABLE (
    level               int,
    component_item_uuid uuid,
    quantity_required   numeric,
    is_leaf             boolean,
    is_phantom          boolean,
    source_bom_uuid     uuid,
    path                uuid[]
)
LANGUAGE plpgsql
AS $$
BEGIN
    IF p_bom_uuid = ANY(p_path) THEN
        RAISE EXCEPTION 'Circular BOM detected: %', p_path || p_bom_uuid;
    END IF;

    RETURN QUERY
    WITH RECURSIVE bom_tree AS (
        -- direct lines
        SELECT
            1 AS level,
            bl.component_item_uuid,
            (bl.quantity_per * bl.scrap_factor * p_qty)::numeric AS quantity_required,
            bl.is_phantom,
            COALESCE(bl.child_bom_uuid, child_bh.uuid) AS child_bom,
            bl.bom_header_uuid AS source_bom_uuid,
            p_path || p_bom_uuid AS path
        FROM manufacturing.bom_lines bl
        LEFT JOIN manufacturing.bom_headers child_bh
               ON child_bh.product_item_uuid = bl.component_item_uuid
              AND child_bh.is_active
              AND (child_bh.is_default OR bl.child_bom_uuid IS NOT NULL)
        WHERE bl.bom_header_uuid = p_bom_uuid

        UNION ALL

        -- recurse into manufactured components that have a BOM
        SELECT
            bt.level + 1,
            bl2.component_item_uuid,
            (bl2.quantity_per * bl2.scrap_factor * bt.quantity_required)::numeric,
            bl2.is_phantom,
            COALESCE(bl2.child_bom_uuid, child_bh2.uuid),
            bl2.bom_header_uuid,
            bt.path || bt.child_bom
        FROM bom_tree bt
        JOIN manufacturing.bom_lines bl2
          ON bl2.bom_header_uuid = bt.child_bom
        LEFT JOIN manufacturing.bom_headers child_bh2
               ON child_bh2.product_item_uuid = bl2.component_item_uuid
              AND child_bh2.is_active
              AND (child_bh2.is_default OR bl2.child_bom_uuid IS NOT NULL)
        WHERE bt.child_bom IS NOT NULL
          AND NOT (bt.child_bom = ANY(bt.path))   -- cycle guard
    )
    SELECT
        t.level,
        t.component_item_uuid,
        t.quantity_required,
        (t.child_bom IS NULL) AS is_leaf,
        t.is_phantom,
        t.source_bom_uuid,
        t.path
    FROM bom_tree t;
END;
$$;

-- Fix bulk material issue: surface failures, unique journal refs, account resolve
-- (CREATE OR REPLACE so existing tenants pick this up on next migrate)

CREATE OR REPLACE FUNCTION manufacturing.issue_material_from_bom(
    p_production_order_uuid uuid,
    p_warehouse_serial      bigint, -- fallback WH when line/item has none
    p_raw_materials_code    text,
    p_work_in_progress_code text,
    p_user_uuid             uuid
)
RETURNS TABLE (
    component_item_uuid uuid,
    required_qty        numeric,
    issued_qty          numeric,
    remaining_qty       numeric,
    material_status     text
)
LANGUAGE plpgsql
AS $$
DECLARE
    rec             RECORD;
    v_remaining_qty numeric(18, 6);
    v_wh_serial     bigint;
    v_raw_uuid      uuid;
    v_wip_uuid      uuid;
    v_any_issued    boolean := false;
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM manufacturing.production_order_materials
        WHERE production_order_uuid = p_production_order_uuid
    ) THEN
        RAISE EXCEPTION
            'No production_order_materials for order % — BOM explode produced no leaf components',
            p_production_order_uuid;
    END IF;

    v_raw_uuid := accounting.resolve_account_ref(p_raw_materials_code);
    v_wip_uuid := accounting.resolve_account_ref(p_work_in_progress_code);

    FOR rec IN
        SELECT pom.*, i.serial_id, i.warehouse_serial AS item_wh
        FROM manufacturing.production_order_materials pom
        JOIN inventory.items i ON i.uuid = pom.component_item_uuid
        WHERE pom.production_order_uuid = p_production_order_uuid
        ORDER BY i.serial_id
    LOOP
        v_remaining_qty := rec.required_qty - rec.issued_qty;
        IF v_remaining_qty <= 0 THEN
            CONTINUE;
        END IF;

        v_wh_serial := COALESCE(rec.preferred_warehouse_serial, rec.item_wh, p_warehouse_serial);
        IF v_wh_serial IS NULL THEN
            RAISE EXCEPTION 'No warehouse for component serial % on order %',
                rec.serial_id, p_production_order_uuid;
        END IF;

        PERFORM manufacturing.issue_material_to_order(
            rec.serial_id,
            v_wh_serial,
            v_remaining_qty,
            p_production_order_uuid,
            v_raw_uuid,
            v_wip_uuid,
            p_user_uuid
        );
        v_any_issued := true;
    END LOOP;

    IF NOT v_any_issued THEN
        RAISE EXCEPTION
            'No materials issued for production order % — check stock, warehouse and order materials',
            p_production_order_uuid;
    END IF;

    RETURN QUERY
    SELECT
        pom.component_item_uuid,
        pom.required_qty,
        pom.issued_qty,
        pom.required_qty - pom.issued_qty,
        pom.status
    FROM manufacturing.production_order_materials pom
    WHERE pom.production_order_uuid = p_production_order_uuid;
END;
$$;


CREATE OR REPLACE FUNCTION manufacturing.issue_material_to_order(
    p_item_serial_id        bigint,
    p_warehouse_serial      bigint,
    p_quantity              numeric(18,4),
    p_production_order_uuid uuid,
    p_raw_mat_uuid          uuid,
    p_wip_uuid              uuid,
    p_user_uuid             uuid
) RETURNS manufacturing.material_issues
LANGUAGE plpgsql
AS $$
DECLARE
    v_order         manufacturing.production_orders%ROWTYPE;
    v_item          inventory.items%ROWTYPE;
    v_warehouse     inventory.warehouses%ROWTYPE;
    v_total_cost    numeric(18,4);
    v_unit_cost     numeric(18,4);
    v_issue_row     manufacturing.material_issues%ROWTYPE;
    v_txn_serial    bigint;
    v_remaining     numeric(18,6);
    v_pom_uuid      uuid;
    v_new_issued    numeric(18,6);
BEGIN
    IF p_quantity IS NULL OR p_quantity <= 0 THEN
        RAISE EXCEPTION 'Issue quantity must be positive';
    END IF;

    SELECT * INTO STRICT v_order
    FROM manufacturing.production_orders
    WHERE uuid = p_production_order_uuid
    FOR UPDATE;

    IF v_order.status NOT IN ('Planned', 'In Progress') THEN
        RAISE EXCEPTION 'Production order must be Planned or In Progress to issue materials';
    END IF;

    SELECT * INTO STRICT v_item
    FROM inventory.items WHERE serial_id = p_item_serial_id;

    SELECT * INTO STRICT v_warehouse
    FROM inventory.warehouses WHERE serial_id = p_warehouse_serial;

    -- Cap to remaining requirement when line exists
    SELECT pom.uuid, pom.required_qty - pom.issued_qty
    INTO v_pom_uuid, v_remaining
    FROM manufacturing.production_order_materials pom
    WHERE pom.production_order_uuid = p_production_order_uuid
      AND pom.component_item_uuid = v_item.uuid;

    IF v_pom_uuid IS NOT NULL THEN
        IF v_remaining <= 0 THEN
            RAISE EXCEPTION 'No remaining material requirement for item % on order %',
                v_item.sku, v_order.order_number;
        END IF;
        IF p_quantity > v_remaining THEN
            RAISE EXCEPTION 'Issue qty % exceeds remaining requirement % for item %',
                p_quantity, v_remaining, v_item.sku;
        END IF;
    END IF;

    SELECT * INTO v_total_cost
    FROM inventory.deplete_inventory(
        p_item_serial_id, p_warehouse_serial, p_quantity, 'PRODUCTION', v_order.serial_id,
        p_user_uuid, NULL, NULL
    );

    v_unit_cost := CASE WHEN p_quantity > 0 THEN v_total_cost / p_quantity ELSE 0 END;

    INSERT INTO manufacturing.material_issues(
        production_order_uuid, stock_item_id, stock_item_name, warehouse_uuid, quantity, unit_cost, issued_at
    ) VALUES (
        p_production_order_uuid, v_item.serial_id, v_item.name, v_warehouse.uuid, p_quantity, v_unit_cost, now()
    ) RETURNING * INTO v_issue_row;

    INSERT INTO manufacturing.cost_applications(
        production_order_uuid, type, amount, applied_at, reference,
        source_account, destination_account
    ) VALUES (
        p_production_order_uuid, 'DirectMaterial', v_total_cost, now(),
        format('Material issue: %s units of item %s', p_quantity, v_item.name),
        p_raw_mat_uuid, p_wip_uuid
    );

    -- Keep requirement tracker in sync (single-item and bulk share this path)
    IF v_pom_uuid IS NOT NULL THEN
        UPDATE manufacturing.production_order_materials pom
        SET issued_qty = pom.issued_qty + p_quantity,
            status = CASE
                WHEN pom.issued_qty + p_quantity >= pom.required_qty THEN 'FullyIssued'
                WHEN pom.issued_qty + p_quantity > 0 THEN 'PartiallyIssued'
                ELSE 'Pending'
            END
        WHERE pom.uuid = v_pom_uuid;
    END IF;

    UPDATE manufacturing.production_orders
    SET status = CASE WHEN status = 'Planned' THEN 'In Progress' ELSE status END,
        materials_status = CASE
            WHEN NOT EXISTS (
                SELECT 1 FROM manufacturing.production_order_materials
                WHERE production_order_uuid = p_production_order_uuid
                  AND status <> 'FullyIssued'
            ) THEN 'Complete'
            WHEN EXISTS (
                SELECT 1 FROM manufacturing.production_order_materials
                WHERE production_order_uuid = p_production_order_uuid
                  AND issued_qty > 0
            ) THEN 'Partial'
            ELSE 'Pending'
        END,
        updated_at = now()
    WHERE uuid = p_production_order_uuid;

    -- Unique ref per issue event (repeat issues of same item allowed)
    v_txn_serial := accounting.post_transaction(
        v_order.order_number || '-MAT-' || v_issue_row.uuid::text,
        'Material To Work In Progress',
        p_user_uuid,
        'manufacturing',
        CURRENT_DATE,
        jsonb_build_array(
            jsonb_build_object(
                'account_ref', p_wip_uuid, 'debit', v_total_cost, 'credit', 0,
                'memo', format('Material issue on order %s', v_order.order_number)
            ),
            jsonb_build_object(
                'account_ref', p_raw_mat_uuid, 'debit', 0, 'credit', v_total_cost,
                'memo', format('Offset to WIP - order %s', v_order.order_number)
            )
        )
    );

    RETURN v_issue_row;
END;
$$;

CREATE OR REPLACE PROCEDURE manufacturing.close_period_overhead(
    p_period_start          date,
    p_period_end            date,
    p_user                  uuid,
    p_oh_control_code       text DEFAULT '2100-MOH-CONTROL',
    p_oh_variance_code      text DEFAULT '5120-OH-VAR',
    p_wip_code              text DEFAULT '1500-WIP',
    p_fg_code               text DEFAULT '1600-FG',
    p_cogs_code             text DEFAULT '5100-COGS',
    p_materiality_threshold numeric(18,2) DEFAULT 5000.00,   -- e.g., $5,000 — make this configurable via a settings table
    p_min_alloc_threshold   numeric(18,2) DEFAULT 100.00,    -- ignore tiny allocations
    p_cutoff_date           date DEFAULT CURRENT_DATE
)
LANGUAGE plpgsql
AS $$
DECLARE
    v_actual          numeric(18,2) := 0;
    v_applied         numeric(18,2) := 0;
    v_balance         numeric(18,2);  -- >0 = under-applied (debit variance), <0 = over-applied (credit variance)
    v_moh_uuid        uuid;
    v_var_uuid        uuid;
    v_txn_serial      bigint;
    v_is_material     boolean;
BEGIN
    -- 1. Compute net balance for the period
    SELECT COALESCE(SUM(amount), 0) INTO v_actual
    FROM manufacturing.cost_applications
    WHERE type = 'ActualOverhead'
      AND applied_at::date BETWEEN p_period_start AND p_period_end;

    SELECT COALESCE(SUM(amount), 0) INTO v_applied
    FROM manufacturing.cost_applications
    WHERE type = 'AppliedOverhead'
      AND applied_at::date BETWEEN p_period_start AND p_period_end;

    v_balance := v_actual - v_applied;

    IF ABS(v_balance) < 1.00 THEN
        RAISE NOTICE 'Negligible overhead balance (%), no closure needed', v_balance;
        RETURN;
    END IF;

    v_moh_uuid := accounting.resolve_account_ref(p_oh_control_code);
    v_var_uuid := accounting.resolve_account_ref(p_oh_variance_code);

    -- 2. Materiality check
    v_is_material := ABS(v_balance) >= p_materiality_threshold;

    IF NOT v_is_material THEN
        -- Simple write-off to variance account
        v_txn_serial := accounting.post_transaction(
            'MOH-WRITEOFF-' || to_char(p_cutoff_date, 'YYYYMM'),
            'Immaterial Overhead Variance Write-off',
            p_user, 'manufacturing', p_cutoff_date,
            jsonb_build_array(
                jsonb_build_object('account_ref', v_var_uuid,  'debit',  GREATEST(v_balance,0), 'credit', GREATEST(-v_balance,0), 'memo', 'Immaterial under/over-applied OH write-off'),
                jsonb_build_object('account_ref', v_moh_uuid, 'debit',  GREATEST(-v_balance,0), 'credit', GREATEST(v_balance,0), 'memo', 'Clear MOH control')
            )
        );
        RAISE NOTICE 'Immaterial balance % written off to variance (txn %)', v_balance, v_txn_serial;
    ELSE
        -- Material → prorate (call your existing prorate_variance logic)
        CALL manufacturing.prorate_variance(
            v_balance,                  -- variance_amount
            p_user,
            p_wip_code,
            p_fg_code,
            p_cogs_code,
            p_cutoff_date,
            p_min_alloc_threshold,
            p_materiality_threshold,    -- reuse as log threshold
            'Material overhead variance proration for period ' || to_char(p_period_start, 'YYYY-MM') || ' to ' || to_char(p_period_end, 'YYYY-MM'),
            false                       -- not dry-run
        );

        -- After proration, clear MOH control fully (proration should have offset it indirectly via WIP/FG/COGS adjustments)
        -- But to be safe, post a final zeroing entry if needed (check balance post-proration)
        -- For simplicity, assume proration handled the offset; if not, add a final MOH → variance entry here
    END IF;
END;
$$;

-- CALL manufacturing.close_period_overhead(
--     '2026-02-01', '2026-02-28', your_user_uuid,
--     materiality_threshold => 5000.00  -- adjust per your policy (e.g., 2% of budgeted OH)
-- );

-- Manual Variance Proration (prorate variance to WIP/FG/COGS for better accuracy and materiality)
CREATE OR REPLACE PROCEDURE manufacturing.prorate_variance(
    p_variance_amount         numeric(18,2),     -- > 0 = under-applied, < 0 = over-applied
    p_user_uuid               uuid,
    p_wip_code                text,
    p_fg_code                 text,
    p_cogs_code               text,
    p_as_of_date              date            DEFAULT CURRENT_DATE,
    p_min_allocation_threshold numeric(18,2)  DEFAULT 1.00,   -- ignore allocations smaller than this
    p_materiality_threshold   numeric(18,2)   DEFAULT 100.00, -- if |variance| < this → skip or log only
    p_memo                    text            DEFAULT 'Proration of manufacturing overhead variance',
    p_dry_run                 boolean         DEFAULT false   -- if true, just return allocations without posting
)
LANGUAGE plpgsql
AS $$
DECLARE
    v_wip_account_uuid     uuid;
    v_fg_account_uuid      uuid;
    v_cogs_account_uuid    uuid;
    v_total_applied        numeric(18,2) := 0;
    v_wip_applied          numeric(18,2) := 0;
    v_fg_applied           numeric(18,2) := 0;
    v_cogs_applied         numeric(18,2) := 0;
    v_wip_percentage       numeric(18,6);
    v_fg_percentage        numeric(18,6);
    v_cogs_percentage      numeric(18,6);
    v_wip_alloc            numeric(18,2);
    v_fg_alloc             numeric(18,2);
    v_cogs_alloc           numeric(18,2);
    v_txn_uuid             uuid;
    v_log_uuid             uuid;
BEGIN
    -- Basic validation
    IF ABS(p_variance_amount) < p_materiality_threshold THEN
        RAISE NOTICE 'Variance % is below materiality threshold % — no adjustment posted',
            p_variance_amount, p_materiality_threshold;
        -- Still log if desired
        INSERT INTO accounting.variance_proration_logs (
            proration_date, variance_amount, memo, created_by
        ) VALUES (p_as_of_date, p_variance_amount, p_memo || ' (below materiality)', p_user_uuid)
        RETURNING uuid INTO v_log_uuid;
        RETURN;
    END IF;

    -- Get account UUIDs (fail fast if missing)
    v_wip_account_uuid := accounting.resolve_account_ref(p_wip_code);
    v_fg_account_uuid := accounting.resolve_account_ref(p_fg_code);
    v_cogs_account_uuid := accounting.resolve_account_ref(p_cogs_code);

    -- Lock relevant GL accounts to prevent concurrent changes (conservative but safe)
    PERFORM 1 FROM accounting.transaction_entries
    WHERE account_uuid IN (v_wip_account_uuid, v_fg_account_uuid, v_cogs_account_uuid)
    FOR UPDATE;

    -- Calculate cumulative applied overhead remaining in each bucket as of date
    -- We look at net applied credits in inventory accounts and net debits in COGS
    -- (assumes applied overhead was credited to xxxx and debited to WIP/FG/COGS indirectly)
    WITH applied_flow AS (
        SELECT
            a.code,
            COALESCE(SUM(
                CASE
                    WHEN a.code = p_wip_code THEN te.credit - te.debit   -- WIP: applied = credit
                    WHEN a.code = p_fg_code THEN te.credit - te.debit   -- FG: applied = credit
                    WHEN a.code = p_cogs_code  THEN te.debit  - te.credit -- COGS: applied = debit (cost increase)
                    ELSE 0
                END
            ), 0) AS applied_remaining
        FROM accounting.accounts a
        LEFT JOIN accounting.transaction_entries te ON te.account_uuid = a.uuid
        LEFT JOIN accounting.transactions tx ON tx.uuid = te.transaction_uuid
        WHERE a.code IN (p_wip_code, p_fg_code, p_cogs_code)
          AND tx.transaction_date <= p_as_of_date
        GROUP BY a.code
    )
    SELECT
        COALESCE(SUM(CASE WHEN code = p_wip_code THEN applied_remaining ELSE 0 END), 0),
        COALESCE(SUM(CASE WHEN code = p_fg_code THEN applied_remaining ELSE 0 END), 0),
        COALESCE(SUM(CASE WHEN code = p_cogs_code  THEN applied_remaining ELSE 0 END), 0)
    INTO v_wip_applied, v_fg_applied, v_cogs_applied
    FROM applied_flow;

    v_total_applied := v_wip_applied + v_fg_applied + v_cogs_applied;

    IF v_total_applied <= 0 THEN
        RAISE EXCEPTION 'No applied overhead found in WIP/FG/COGS as of %. Cannot prorate.', p_as_of_date;
    END IF;

    -- Calculate allocation percentages
    v_wip_percentage  := v_wip_applied  / v_total_applied;
    v_fg_percentage   := v_fg_applied   / v_total_applied;
    v_cogs_percentage := v_cogs_applied / v_total_applied;

    -- Calculate allocations
    v_wip_alloc  := ROUND(p_variance_amount * v_wip_percentage, 2);
    v_fg_alloc   := ROUND(p_variance_amount * v_fg_percentage, 2);
    v_cogs_alloc := ROUND(p_variance_amount * v_cogs_percentage, 2);

    -- Enforce minimum threshold (avoid noise entries)
    IF ABS(v_wip_alloc) < p_min_allocation_threshold THEN v_wip_alloc := 0; END IF;
    IF ABS(v_fg_alloc) < p_min_allocation_threshold THEN v_fg_alloc := 0; END IF;
    IF ABS(v_cogs_alloc) < p_min_allocation_threshold THEN v_cogs_alloc := 0; END IF;

    -- Dry-run mode: just return values without posting
    IF p_dry_run THEN
        RAISE NOTICE 'Dry run - would allocate: WIP=%, FG=%, COGS=% (total=%)',
            v_wip_alloc, v_fg_alloc, v_cogs_alloc, v_wip_alloc + v_fg_alloc + v_cogs_alloc;
        RETURN;
    END IF;

    -- Create adjustment transaction
    v_txn_uuid := accounting.post_transaction(
        'VAR-PRON-' || to_char(p_as_of_date, 'YYYYMMDD'),
        'Prorated Overhead Variance Adjustment',
        p_user_uuid,
        'adjustment',
        p_as_of_date,
        '[]'::jsonb
    );

    -- Build the entry lines (only for non-zero allocations)
    WITH entries AS (
        SELECT unnest(ARRAY[p_wip_code,p_fg_code,p_cogs_code]) AS code,
               unnest(ARRAY[v_wip_alloc, v_fg_alloc, v_cogs_alloc]) AS alloc_amt,
               unnest(ARRAY[1,2,3]) AS line_no
        WHERE alloc_amt != 0
    )
    INSERT INTO accounting.transaction_entries (
        transaction_uuid, account_uuid, line_no, amount,
        debit, credit, memo
    )
    SELECT
        v_txn_uuid,
        a.uuid,
        e.line_no,
        ABS(e.alloc_amt),
        CASE WHEN p_variance_amount > 0 THEN ABS(e.alloc_amt) ELSE 0 END,          -- debit if under-applied
        CASE WHEN p_variance_amount < 0 THEN ABS(e.alloc_amt) ELSE 0 END,          -- credit if over-applied
        p_memo || format(' - %s proration (%% %.4f)', a.code,
            CASE a.code WHEN p_wip_code THEN v_wip_percentage
                        WHEN p_fg_code THEN v_fg_percentage
                        ELSE v_cogs_percentage END)
    FROM entries e
    JOIN accounting.accounts a ON a.code = e.code;

    -- Log the proration
    INSERT INTO accounting.variance_proration_logs (
        proration_date, variance_amount, wip_alloc, fg_alloc, cogs_alloc,
        gl_transaction_uuid, memo, created_by
    ) VALUES (
        p_as_of_date, p_variance_amount, v_wip_alloc, v_fg_alloc, v_cogs_alloc,
        v_txn_uuid, p_memo, p_user_uuid
    ) RETURNING uuid INTO v_log_uuid;

    RAISE NOTICE 'Variance proration completed. Transaction: %, Log: %', v_txn_uuid, v_log_uuid;
END;
$$;

-- ============================================================================
-- VIEWS
-- ============================================================================

CREATE OR REPLACE VIEW manufacturing.order_cost_summary AS
WITH mat AS (
    SELECT production_order_uuid, COALESCE(SUM(total_cost), 0) AS actual_material
    FROM manufacturing.material_issues
    GROUP BY production_order_uuid
),
ca AS (
    SELECT
        production_order_uuid,
        COALESCE(SUM(amount) FILTER (WHERE type = 'ActualLabor'), 0) AS actual_labor,
        COALESCE(SUM(amount) FILTER (WHERE type = 'DirectLabor'), 0) AS applied_labor,
        COALESCE(SUM(amount) FILTER (WHERE type = 'IndirectLabor'), 0) AS indirect_labor,
        COALESCE(SUM(amount) FILTER (WHERE type = 'AppliedOverhead'), 0) AS applied_overhead,
        COALESCE(SUM(amount) FILTER (WHERE type = 'ActualOverhead'), 0) AS actual_overhead
    FROM manufacturing.cost_applications
    GROUP BY production_order_uuid
)
SELECT
    po.order_number,
    i.name AS product_name,
    po.quantity_ordered,
    po.quantity_completed,
    po.status,
    COALESCE(mat.actual_material, 0) AS actual_material,
    COALESCE(ca.actual_labor, 0) AS actual_labor,
    COALESCE(ca.applied_labor, 0) AS applied_labor,
    COALESCE(ca.indirect_labor, 0) AS indirect_labor,
    COALESCE(ca.applied_overhead, 0) AS applied_overhead,
    COALESCE(ca.actual_overhead, 0) AS actual_overhead,
    po.material_usage_variance,
    po.labor_efficiency_variance,
    po.overhead_variance,
    (
        COALESCE(po.material_usage_variance, 0)
        + COALESCE(po.labor_efficiency_variance, 0)
        + COALESCE(po.overhead_variance, 0)
    ) AS total_variance,
    (
        COALESCE(mat.actual_material, 0)
        + COALESCE(NULLIF(ca.actual_labor, 0), ca.applied_labor, 0)
        + COALESCE(ca.applied_overhead, 0)
    ) / NULLIF(po.quantity_completed, 0) AS actual_unit_cost,
    i.standard_cost AS standard_unit_cost
FROM manufacturing.production_orders po
JOIN inventory.items i ON i.uuid = po.product_item_uuid
LEFT JOIN mat ON mat.production_order_uuid = po.uuid
LEFT JOIN ca ON ca.production_order_uuid = po.uuid;


-- NOTES:
-----------------------------------------------------------------------------------------------
-- Typical codes for manufacturing (adjust numbering to fit your CoA)
-- Raw Materials, WIP, Finished Goods → separate sub-accounts under Inventory (Asset)
-- Overhead control + applied → temporary / clearing accounts

-- Assets – Inventory sub-ledgers
-- ('1300', 'Raw Materials Inventory',     'Asset', '1200', 'DR', false),
-- ('1310', 'Work in Process Inventory',   'Asset', '1200', 'DR', false),
-- ('1320', 'Finished Goods Inventory',    'Asset', '1200', 'DR', false),

-- -- Manufacturing Overhead – actual costs go here (debit)
-- ('501000', 'Manufacturing Overhead Control','Expense','500000','DR', false),  -- parent = Cost of Goods Sold / Manufacturing Expenses

-- -- Applied Overhead – credit when applied to WIP
-- ('501010', 'Manufacturing Overhead Applied','Expense','500000','CR', true),   -- contra-like, will net against control at period end

-- -- Optional: separate variance if you want more detail
-- ('501020', 'Over/Under Applied Overhead Adj','Expense','500000','DR', false); -- usually temporary, cleared to COGS

-- Example insert (yearly rate)
-- INSERT INTO manufacturing.overhead_rates (period_start, period_end, allocation_base, estimated_overhead, estimated_base)
-- VALUES ('2026-01-01', '2026-12-31', 'DirectLaborHours', 1200000.00, 40000.00);  -- → rate = 30.00 per DLH

-----------------------------------------------------------------------------------------------
-- Three standard methods of overhead adjustments at the end of a financial period:
-----------------------------------------------------------------------------------------------
-- 1. Close Entire Variance to COGS
-- All over or under applied overhead goes to Cost of Goods Sold:
-- Fast, Simple, Common
-- 2. Prorate Across WIP, FG, and COGS
-- Spread the variance accross WIP, FG and COGS Used When:
-- inventory is material, desire for reasonable accuracy
-- 3. The Pure Method
-- Recomputation using actual overhead rate, involves bactracking to get
-- actual overhead and reapply corrected ones

-- SELECT conname FROM pg_constraint WHERE conrelid = 'cost_applications'::regclass;
-- alter Table cost_applications DROP constraint cost_applications_type_check;
-- ALTER Table cost_applications add constraint cost_applications_type_check check(type in (
--     'DirectLabor', 'Overhead', 'DirectMaterial', 'ActualOverview'
-- ));


-- Material reservation / shortage visibility (required vs on-hand at preferred/default WH)
CREATE OR REPLACE VIEW manufacturing.v_order_material_availability AS
SELECT
    po.uuid AS production_order_uuid,
    po.order_number,
    po.status AS order_status,
    pom.component_item_uuid,
    i.sku,
    i.name AS component_name,
    pom.required_qty,
    pom.issued_qty,
    pom.required_qty - pom.issued_qty AS remaining_qty,
    pom.status AS material_status,
    COALESCE(pom.preferred_warehouse_serial, i.warehouse_serial) AS warehouse_serial,
    w.uuid AS warehouse_uuid,
    COALESCE(inventory.get_quantity_on_hand(i.uuid, w.uuid), 0) AS qty_on_hand,
    GREATEST(
        0,
        (pom.required_qty - pom.issued_qty)
        - COALESCE(inventory.get_quantity_on_hand(i.uuid, w.uuid), 0)
    ) AS short_qty
FROM manufacturing.production_orders po
JOIN manufacturing.production_order_materials pom ON pom.production_order_uuid = po.uuid
JOIN inventory.items i ON i.uuid = pom.component_item_uuid
LEFT JOIN inventory.warehouses w
  ON w.serial_id = COALESCE(pom.preferred_warehouse_serial, i.warehouse_serial)
WHERE po.status IN ('Planned', 'In Progress');


-- ============================================================================
-- STAGE 6: Operation execution, scrap/rework, QC, light capacity
-- ============================================================================

-- Per-routing operation sequence (for existing DBs that still have global UNIQUE on sequence)
DO $$ BEGIN
    ALTER TABLE manufacturing.routing_operations DROP CONSTRAINT IF EXISTS routing_operations_sequence_key;
EXCEPTION WHEN undefined_object THEN NULL;
END $$;
DO $$ BEGIN
    ALTER TABLE manufacturing.routing_operations
        ADD CONSTRAINT routing_operations_routing_seq_unique UNIQUE (routing_uuid, sequence);
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    ALTER TABLE manufacturing.work_centers ADD COLUMN IF NOT EXISTS capacity_hours_per_day numeric(12,2);
    ALTER TABLE manufacturing.work_centers ADD COLUMN IF NOT EXISTS capacity_hours_per_week numeric(12,2);
EXCEPTION WHEN others THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS manufacturing.operation_executions (
    uuid uuid DEFAULT uuidv7() PRIMARY KEY,
    production_order_uuid uuid NOT NULL REFERENCES manufacturing.production_orders (uuid) ON DELETE CASCADE,
    routing_operation_uuid uuid REFERENCES manufacturing.routing_operations (uuid),
    sequence smallint,
    operation_name text NOT NULL,
    work_center_code text,
    status text NOT NULL DEFAULT 'Pending'
        CHECK (status IN ('Pending', 'InProgress', 'Completed', 'Skipped')),
    qty_started numeric(18,4) DEFAULT 0,
    qty_completed numeric(18,4) DEFAULT 0,
    qty_scrapped numeric(18,4) DEFAULT 0,
    qty_reworked numeric(18,4) DEFAULT 0,
    setup_minutes_actual numeric(18,4) DEFAULT 0,
    run_minutes_actual numeric(18,4) DEFAULT 0,
    machine_minutes_actual numeric(18,4) DEFAULT 0,
    started_at timestamptz,
    completed_at timestamptz,
    operator_uuid uuid,
    notes text,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_op_exec_order
    ON manufacturing.operation_executions (production_order_uuid);

CREATE TABLE IF NOT EXISTS manufacturing.scrap_rework_events (
    uuid uuid DEFAULT uuidv7() PRIMARY KEY,
    production_order_uuid uuid NOT NULL REFERENCES manufacturing.production_orders (uuid) ON DELETE CASCADE,
    operation_execution_uuid uuid REFERENCES manufacturing.operation_executions (uuid),
    event_type text NOT NULL CHECK (event_type IN ('Scrap', 'Rework', 'Reject')),
    quantity numeric(18,4) NOT NULL CHECK (quantity > 0),
    reason_code text,
    reason_text text,
    disposition text CHECK (disposition IN ('Scrap', 'Rework', 'ReturnToStock', 'Hold')),
    recorded_by uuid,
    recorded_at timestamptz DEFAULT now()
);

CREATE TABLE IF NOT EXISTS manufacturing.quality_inspections (
    uuid uuid DEFAULT uuidv7() PRIMARY KEY,
    production_order_uuid uuid NOT NULL REFERENCES manufacturing.production_orders (uuid) ON DELETE CASCADE,
    operation_execution_uuid uuid REFERENCES manufacturing.operation_executions (uuid),
    inspection_type text NOT NULL DEFAULT 'InProcess'
        CHECK (inspection_type IN ('Incoming', 'InProcess', 'Final')),
    qty_inspected numeric(18,4) NOT NULL CHECK (qty_inspected > 0),
    qty_passed numeric(18,4) NOT NULL DEFAULT 0,
    qty_failed numeric(18,4) NOT NULL DEFAULT 0,
    result text NOT NULL CHECK (result IN ('Pass', 'Fail', 'Conditional')),
    inspector_uuid uuid,
    notes text,
    inspected_at timestamptz DEFAULT now(),
    CONSTRAINT qi_qty_check CHECK (qty_passed + qty_failed <= qty_inspected)
);

-- Seed operation executions from pinned (or default) routing when order starts / on demand
CREATE OR REPLACE FUNCTION manufacturing.ensure_operation_executions(
    p_order_uuid uuid
) RETURNS integer
LANGUAGE plpgsql
AS $$
DECLARE
    v_order manufacturing.production_orders%ROWTYPE;
    v_routing uuid;
    v_count int := 0;
BEGIN
    SELECT * INTO STRICT v_order FROM manufacturing.production_orders WHERE uuid = p_order_uuid;
    v_routing := v_order.routing_uuid;
    IF v_routing IS NULL THEN
        SELECT r.uuid INTO v_routing
        FROM manufacturing.routings r
        WHERE r.product_item_uuid = v_order.product_item_uuid AND r.status = 'active'
        ORDER BY r.is_default DESC, r.created_at DESC
        LIMIT 1;
    END IF;
    IF v_routing IS NULL THEN
        RETURN 0;
    END IF;

    INSERT INTO manufacturing.operation_executions (
        production_order_uuid, routing_operation_uuid, sequence, operation_name, work_center_code, status
    )
    SELECT
        p_order_uuid, ro.uuid, ro.sequence, ro.operation_name, ro.work_center, 'Pending'
    FROM manufacturing.routing_operations ro
    WHERE ro.routing_uuid = v_routing
      AND NOT EXISTS (
          SELECT 1 FROM manufacturing.operation_executions oe
          WHERE oe.production_order_uuid = p_order_uuid
            AND oe.routing_operation_uuid = ro.uuid
      )
    ORDER BY ro.sequence;

    GET DIAGNOSTICS v_count = ROW_COUNT;
    RETURN v_count;
END;
$$;

CREATE OR REPLACE FUNCTION manufacturing.start_operation(
    p_execution_uuid uuid,
    p_qty_started numeric DEFAULT NULL,
    p_operator_uuid uuid DEFAULT NULL
) RETURNS manufacturing.operation_executions
LANGUAGE plpgsql
AS $$
DECLARE
    v_row manufacturing.operation_executions%ROWTYPE;
    v_order manufacturing.production_orders%ROWTYPE;
BEGIN
    SELECT * INTO STRICT v_row FROM manufacturing.operation_executions
    WHERE uuid = p_execution_uuid FOR UPDATE;

    SELECT * INTO STRICT v_order FROM manufacturing.production_orders
    WHERE uuid = v_row.production_order_uuid;

    IF v_order.status NOT IN ('Planned', 'In Progress') THEN
        RAISE EXCEPTION 'Order must be open to start an operation';
    END IF;
    IF v_row.status NOT IN ('Pending', 'InProgress') THEN
        RAISE EXCEPTION 'Operation execution % cannot start from status %', p_execution_uuid, v_row.status;
    END IF;

    UPDATE manufacturing.operation_executions
    SET status = 'InProgress',
        started_at = COALESCE(started_at, now()),
        qty_started = COALESCE(p_qty_started, qty_started, v_order.quantity_ordered),
        operator_uuid = COALESCE(p_operator_uuid, operator_uuid),
        updated_at = now()
    WHERE uuid = p_execution_uuid
    RETURNING * INTO v_row;

    UPDATE manufacturing.production_orders
    SET status = CASE WHEN status = 'Planned' THEN 'In Progress' ELSE status END,
        updated_at = now()
    WHERE uuid = v_row.production_order_uuid;

    RETURN v_row;
END;
$$;

CREATE OR REPLACE FUNCTION manufacturing.complete_operation(
    p_execution_uuid uuid,
    p_qty_completed numeric,
    p_qty_scrapped numeric DEFAULT 0,
    p_setup_minutes_actual numeric DEFAULT NULL,
    p_run_minutes_actual numeric DEFAULT NULL,
    p_machine_minutes_actual numeric DEFAULT NULL,
    p_operator_uuid uuid DEFAULT NULL,
    p_notes text DEFAULT NULL
) RETURNS manufacturing.operation_executions
LANGUAGE plpgsql
AS $$
DECLARE
    v_row manufacturing.operation_executions%ROWTYPE;
BEGIN
    SELECT * INTO STRICT v_row FROM manufacturing.operation_executions
    WHERE uuid = p_execution_uuid FOR UPDATE;

    IF v_row.status NOT IN ('Pending', 'InProgress') THEN
        RAISE EXCEPTION 'Operation % is not open (status=%)', p_execution_uuid, v_row.status;
    END IF;
    IF p_qty_completed IS NULL OR p_qty_completed < 0 THEN
        RAISE EXCEPTION 'qty_completed must be non-negative';
    END IF;

    UPDATE manufacturing.operation_executions
    SET status = 'Completed',
        qty_completed = p_qty_completed,
        qty_scrapped = COALESCE(p_qty_scrapped, 0),
        setup_minutes_actual = COALESCE(p_setup_minutes_actual, setup_minutes_actual),
        run_minutes_actual = COALESCE(p_run_minutes_actual, run_minutes_actual),
        machine_minutes_actual = COALESCE(p_machine_minutes_actual, machine_minutes_actual),
        operator_uuid = COALESCE(p_operator_uuid, operator_uuid),
        notes = COALESCE(p_notes, notes),
        started_at = COALESCE(started_at, now()),
        completed_at = now(),
        updated_at = now()
    WHERE uuid = p_execution_uuid
    RETURNING * INTO v_row;

    IF COALESCE(p_qty_scrapped, 0) > 0 THEN
        INSERT INTO manufacturing.scrap_rework_events (
            production_order_uuid, operation_execution_uuid, event_type, quantity, disposition
        ) VALUES (
            v_row.production_order_uuid, v_row.uuid, 'Scrap', p_qty_scrapped, 'Scrap'
        );
    END IF;

    RETURN v_row;
END;
$$;

CREATE OR REPLACE FUNCTION manufacturing.record_scrap_rework(
    p_production_order_uuid uuid,
    p_event_type text,
    p_quantity numeric,
    p_operation_execution_uuid uuid DEFAULT NULL,
    p_reason_code text DEFAULT NULL,
    p_reason_text text DEFAULT NULL,
    p_disposition text DEFAULT NULL,
    p_recorded_by uuid DEFAULT NULL
) RETURNS manufacturing.scrap_rework_events
LANGUAGE plpgsql
AS $$
DECLARE
    v_row manufacturing.scrap_rework_events%ROWTYPE;
    v_disp text;
BEGIN
    IF p_event_type NOT IN ('Scrap', 'Rework', 'Reject') THEN
        RAISE EXCEPTION 'event_type must be Scrap, Rework or Reject';
    END IF;
    IF p_quantity IS NULL OR p_quantity <= 0 THEN
        RAISE EXCEPTION 'quantity must be positive';
    END IF;

    v_disp := COALESCE(p_disposition, CASE p_event_type
        WHEN 'Rework' THEN 'Rework'
        WHEN 'Reject' THEN 'Hold'
        ELSE 'Scrap'
    END);

    INSERT INTO manufacturing.scrap_rework_events (
        production_order_uuid, operation_execution_uuid, event_type, quantity,
        reason_code, reason_text, disposition, recorded_by
    ) VALUES (
        p_production_order_uuid, p_operation_execution_uuid, p_event_type, p_quantity,
        p_reason_code, p_reason_text, v_disp, p_recorded_by
    )
    RETURNING * INTO v_row;

    IF p_operation_execution_uuid IS NOT NULL AND p_event_type = 'Scrap' THEN
        UPDATE manufacturing.operation_executions
        SET qty_scrapped = qty_scrapped + p_quantity, updated_at = now()
        WHERE uuid = p_operation_execution_uuid;
    ELSIF p_operation_execution_uuid IS NOT NULL AND p_event_type = 'Rework' THEN
        UPDATE manufacturing.operation_executions
        SET qty_reworked = qty_reworked + p_quantity, updated_at = now()
        WHERE uuid = p_operation_execution_uuid;
    END IF;

    RETURN v_row;
END;
$$;

CREATE OR REPLACE FUNCTION manufacturing.record_quality_inspection(
    p_production_order_uuid uuid,
    p_qty_inspected numeric,
    p_qty_passed numeric,
    p_qty_failed numeric,
    p_result text,
    p_inspection_type text DEFAULT 'InProcess',
    p_operation_execution_uuid uuid DEFAULT NULL,
    p_inspector_uuid uuid DEFAULT NULL,
    p_notes text DEFAULT NULL
) RETURNS manufacturing.quality_inspections
LANGUAGE plpgsql
AS $$
DECLARE
    v_row manufacturing.quality_inspections%ROWTYPE;
BEGIN
    IF p_result NOT IN ('Pass', 'Fail', 'Conditional') THEN
        RAISE EXCEPTION 'result must be Pass, Fail or Conditional';
    END IF;
    IF p_qty_inspected IS NULL OR p_qty_inspected <= 0 THEN
        RAISE EXCEPTION 'qty_inspected must be positive';
    END IF;
    IF COALESCE(p_qty_passed,0) + COALESCE(p_qty_failed,0) > p_qty_inspected THEN
        RAISE EXCEPTION 'passed + failed cannot exceed inspected';
    END IF;

    INSERT INTO manufacturing.quality_inspections (
        production_order_uuid, operation_execution_uuid, inspection_type,
        qty_inspected, qty_passed, qty_failed, result, inspector_uuid, notes
    ) VALUES (
        p_production_order_uuid, p_operation_execution_uuid, COALESCE(p_inspection_type, 'InProcess'),
        p_qty_inspected, COALESCE(p_qty_passed, 0), COALESCE(p_qty_failed, 0),
        p_result, p_inspector_uuid, p_notes
    )
    RETURNING * INTO v_row;

    RETURN v_row;
END;
$$;

-- Work-centre load vs capacity (open executions + actual minutes)
CREATE OR REPLACE VIEW manufacturing.v_work_center_load AS
SELECT
    wc.code AS work_center_code,
    wc.name,
    wc.capacity_hours_per_day,
    wc.capacity_hours_per_week,
    COUNT(oe.uuid) FILTER (WHERE oe.status = 'InProgress') AS ops_in_progress,
    COUNT(oe.uuid) FILTER (WHERE oe.status IN ('Pending', 'InProgress')) AS ops_open,
    COALESCE(SUM(oe.run_minutes_actual + oe.setup_minutes_actual) FILTER (WHERE oe.status = 'Completed'), 0) / 60.0
        AS completed_labour_hours,
    COALESCE(SUM(oe.machine_minutes_actual) FILTER (WHERE oe.status = 'Completed'), 0) / 60.0
        AS completed_machine_hours
FROM manufacturing.work_centers wc
LEFT JOIN manufacturing.operation_executions oe ON oe.work_center_code = wc.code
GROUP BY wc.code, wc.name, wc.capacity_hours_per_day, wc.capacity_hours_per_week;


-- ============================================================================
-- STAGE: LABOUR TRACKING (actual vs applied)
-- Attendance/payroll remains HR responsibility. Manufacturing owns productive time
-- booking, rates, approval and ActualLabor posting into WIP.
-- ============================================================================

CREATE TABLE IF NOT EXISTS manufacturing.labour_categories (
    uuid uuid DEFAULT uuidv7() PRIMARY KEY,
    code text NOT NULL UNIQUE,
    name text NOT NULL,
    description text,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz DEFAULT now()
);

-- Effective-dated rates: Standard (applied/planning) vs Actual (costing timesheets)
CREATE TABLE IF NOT EXISTS manufacturing.labour_rates (
    uuid uuid DEFAULT uuidv7() PRIMARY KEY,
    labour_category_uuid uuid REFERENCES manufacturing.labour_categories (uuid),
    employee_uuid uuid, -- optional override per employee
    work_center_code text,
    rate_kind text NOT NULL CHECK (rate_kind IN ('Standard', 'Actual')),
    hourly_rate numeric(18, 4) NOT NULL CHECK (hourly_rate >= 0),
    effective_from date NOT NULL,
    effective_to date,
    currency_code text DEFAULT 'UGX',
    notes text,
    created_at timestamptz DEFAULT now(),
    CONSTRAINT labour_rates_window CHECK (
        effective_to IS NULL OR effective_to >= effective_from
    )
);
CREATE INDEX IF NOT EXISTS idx_labour_rates_lookup
    ON manufacturing.labour_rates (rate_kind, effective_from, effective_to);

-- Manufacturing timesheets (productive / non-productive activity on the floor)
CREATE TABLE IF NOT EXISTS manufacturing.labour_time_entries (
    uuid uuid DEFAULT uuidv7() PRIMARY KEY,
    employee_uuid uuid NOT NULL,
    production_order_uuid uuid REFERENCES manufacturing.production_orders (uuid),
    operation_execution_uuid uuid REFERENCES manufacturing.operation_executions (uuid),
    work_center_code text,
    labour_category_uuid uuid REFERENCES manufacturing.labour_categories (uuid),
    work_date date NOT NULL,
    start_time timestamptz,
    end_time timestamptz,
    break_minutes numeric(10, 2) NOT NULL DEFAULT 0,
    hours numeric(18, 4), -- if null, derived from start/end - break
    activity_type text NOT NULL DEFAULT 'Direct'
        CHECK (activity_type IN (
            'Direct', 'Indirect', 'Setup', 'Rework', 'Idle', 'Training', 'Other'
        )),
    quantity_completed numeric(18, 4) DEFAULT 0,
    quantity_scrapped numeric(18, 4) DEFAULT 0,
    status text NOT NULL DEFAULT 'Draft'
        CHECK (status IN ('Draft', 'Submitted', 'Approved', 'Rejected', 'Posted')),
    hourly_rate numeric(18, 4), -- frozen at approve/post when resolved
    amount numeric(18, 2),
    memo text,
    submitted_at timestamptz,
    approved_at timestamptz,
    approved_by uuid,
    rejection_reason text,
    labour_allocation_uuid uuid REFERENCES manufacturing.labour_allocations (uuid),
    created_by uuid,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_lte_order
    ON manufacturing.labour_time_entries (production_order_uuid);
CREATE INDEX IF NOT EXISTS idx_lte_employee_date
    ON manufacturing.labour_time_entries (employee_uuid, work_date);
CREATE INDEX IF NOT EXISTS idx_lte_status
    ON manufacturing.labour_time_entries (status);

-- Resolve hourly rate: employee override → category → work centre (Actual preferred for costing)
CREATE OR REPLACE FUNCTION manufacturing.resolve_labour_rate(
    p_rate_kind text,
    p_as_of date,
    p_employee_uuid uuid DEFAULT NULL,
    p_category_uuid uuid DEFAULT NULL,
    p_work_center_code text DEFAULT NULL
) RETURNS numeric
LANGUAGE plpgsql
STABLE
AS $$
DECLARE
    v_rate numeric(18, 4);
BEGIN
    IF p_employee_uuid IS NOT NULL THEN
        SELECT hourly_rate INTO v_rate
        FROM manufacturing.labour_rates
        WHERE rate_kind = p_rate_kind
          AND employee_uuid = p_employee_uuid
          AND effective_from <= p_as_of
          AND (effective_to IS NULL OR effective_to >= p_as_of)
        ORDER BY effective_from DESC
        LIMIT 1;
        IF v_rate IS NOT NULL THEN RETURN v_rate; END IF;
    END IF;

    IF p_category_uuid IS NOT NULL THEN
        SELECT hourly_rate INTO v_rate
        FROM manufacturing.labour_rates
        WHERE rate_kind = p_rate_kind
          AND labour_category_uuid = p_category_uuid
          AND employee_uuid IS NULL
          AND effective_from <= p_as_of
          AND (effective_to IS NULL OR effective_to >= p_as_of)
        ORDER BY effective_from DESC
        LIMIT 1;
        IF v_rate IS NOT NULL THEN RETURN v_rate; END IF;
    END IF;

    IF p_work_center_code IS NOT NULL THEN
        SELECT hourly_rate INTO v_rate
        FROM manufacturing.labour_rates
        WHERE rate_kind = p_rate_kind
          AND work_center_code = p_work_center_code
          AND employee_uuid IS NULL
          AND labour_category_uuid IS NULL
          AND effective_from <= p_as_of
          AND (effective_to IS NULL OR effective_to >= p_as_of)
        ORDER BY effective_from DESC
        LIMIT 1;
        IF v_rate IS NOT NULL THEN RETURN v_rate; END IF;

        -- No WC labor_rate fallback for Actual (planning rate must not masquerade as actual)
    END IF;

    RETURN NULL;
END;
$$;

CREATE OR REPLACE FUNCTION manufacturing.create_labour_time_entry(
    p_employee_uuid uuid,
    p_work_date date,
    p_hours numeric DEFAULT NULL,
    p_activity_type text DEFAULT 'Direct',
    p_production_order_uuid uuid DEFAULT NULL,
    p_operation_execution_uuid uuid DEFAULT NULL,
    p_work_center_code text DEFAULT NULL,
    p_labour_category_uuid uuid DEFAULT NULL,
    p_start_time timestamptz DEFAULT NULL,
    p_end_time timestamptz DEFAULT NULL,
    p_break_minutes numeric DEFAULT 0,
    p_quantity_completed numeric DEFAULT 0,
    p_quantity_scrapped numeric DEFAULT 0,
    p_memo text DEFAULT NULL,
    p_created_by uuid DEFAULT NULL
) RETURNS manufacturing.labour_time_entries
LANGUAGE plpgsql
AS $$
DECLARE
    v_hours numeric(18, 4);
    v_row manufacturing.labour_time_entries%ROWTYPE;
BEGIN
    IF p_activity_type NOT IN ('Direct', 'Indirect', 'Setup', 'Rework', 'Idle', 'Training', 'Other') THEN
        RAISE EXCEPTION 'Invalid activity_type %', p_activity_type;
    END IF;

    v_hours := p_hours;
    IF v_hours IS NULL AND p_start_time IS NOT NULL AND p_end_time IS NOT NULL THEN
        v_hours := EXTRACT(EPOCH FROM (p_end_time - p_start_time)) / 3600.0
                   - COALESCE(p_break_minutes, 0) / 60.0;
    END IF;
    IF v_hours IS NULL OR v_hours <= 0 THEN
        RAISE EXCEPTION 'hours must be positive (provide hours or start/end)';
    END IF;

    IF p_activity_type IN ('Direct', 'Setup', 'Rework') AND p_production_order_uuid IS NULL THEN
        RAISE EXCEPTION 'production_order_uuid required for activity_type %', p_activity_type;
    END IF;

    INSERT INTO manufacturing.labour_time_entries (
        employee_uuid, production_order_uuid, operation_execution_uuid,
        work_center_code, labour_category_uuid, work_date,
        start_time, end_time, break_minutes, hours, activity_type,
        quantity_completed, quantity_scrapped, memo, created_by
    ) VALUES (
        p_employee_uuid, p_production_order_uuid, p_operation_execution_uuid,
        p_work_center_code, p_labour_category_uuid, p_work_date,
        p_start_time, p_end_time, COALESCE(p_break_minutes, 0), v_hours, p_activity_type,
        COALESCE(p_quantity_completed, 0), COALESCE(p_quantity_scrapped, 0),
        p_memo, p_created_by
    )
    RETURNING * INTO v_row;
    RETURN v_row;
END;
$$;

CREATE OR REPLACE FUNCTION manufacturing.submit_labour_time_entry(
    p_entry_uuid uuid
) RETURNS manufacturing.labour_time_entries
LANGUAGE plpgsql
AS $$
DECLARE
    v_row manufacturing.labour_time_entries%ROWTYPE;
BEGIN
    UPDATE manufacturing.labour_time_entries
    SET status = 'Submitted',
        submitted_at = now(),
        updated_at = now()
    WHERE uuid = p_entry_uuid
      AND status IN ('Draft', 'Rejected')
    RETURNING * INTO v_row;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Time entry % not found or not submittable', p_entry_uuid;
    END IF;
    RETURN v_row;
END;
$$;

CREATE OR REPLACE FUNCTION manufacturing.approve_labour_time_entry(
    p_entry_uuid uuid,
    p_approver uuid,
    p_hourly_rate numeric DEFAULT NULL
) RETURNS manufacturing.labour_time_entries
LANGUAGE plpgsql
AS $$
DECLARE
    v_row manufacturing.labour_time_entries%ROWTYPE;
    v_rate numeric(18, 4);
BEGIN
    SELECT * INTO STRICT v_row
    FROM manufacturing.labour_time_entries
    WHERE uuid = p_entry_uuid
    FOR UPDATE;

    IF v_row.status NOT IN ('Draft', 'Submitted') THEN
        RAISE EXCEPTION 'Time entry % cannot be approved from status %', p_entry_uuid, v_row.status;
    END IF;

    v_rate := COALESCE(
        p_hourly_rate,
        v_row.hourly_rate,
        manufacturing.resolve_labour_rate(
            'Actual', v_row.work_date, v_row.employee_uuid,
            v_row.labour_category_uuid, v_row.work_center_code
        )
    );
    IF v_rate IS NULL THEN
        RAISE EXCEPTION 'No Actual labour rate resolved for entry % — set rate, category or work centre rate',
            p_entry_uuid;
    END IF;

    UPDATE manufacturing.labour_time_entries
    SET status = 'Approved',
        hourly_rate = v_rate,
        amount = ROUND(hours * v_rate, 2),
        approved_at = now(),
        approved_by = p_approver,
        rejection_reason = NULL,
        updated_at = now()
    WHERE uuid = p_entry_uuid
    RETURNING * INTO v_row;

    RETURN v_row;
END;
$$;

CREATE OR REPLACE FUNCTION manufacturing.reject_labour_time_entry(
    p_entry_uuid uuid,
    p_reason text DEFAULT NULL
) RETURNS manufacturing.labour_time_entries
LANGUAGE plpgsql
AS $$
DECLARE
    v_row manufacturing.labour_time_entries%ROWTYPE;
BEGIN
    UPDATE manufacturing.labour_time_entries
    SET status = 'Rejected',
        rejection_reason = p_reason,
        updated_at = now()
    WHERE uuid = p_entry_uuid
      AND status IN ('Draft', 'Submitted')
    RETURNING * INTO v_row;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Time entry % not found or not rejectable', p_entry_uuid;
    END IF;
    RETURN v_row;
END;
$$;

-- Post approved timesheet → labour_allocations (Draft) → post_labour_allocation → ActualLabor/IndirectLabor
CREATE OR REPLACE FUNCTION manufacturing.post_labour_time_entry(
    p_entry_uuid uuid,
    p_user uuid,
    p_wip_account text,
    p_wages_account text,
    p_oh_control_account text DEFAULT NULL
) RETURNS manufacturing.cost_applications
LANGUAGE plpgsql
AS $$
DECLARE
    v_entry manufacturing.labour_time_entries%ROWTYPE;
    v_alloc manufacturing.labour_allocations%ROWTYPE;
    v_class text;
    v_app manufacturing.cost_applications%ROWTYPE;
BEGIN
    SELECT * INTO STRICT v_entry
    FROM manufacturing.labour_time_entries
    WHERE uuid = p_entry_uuid
    FOR UPDATE;

    IF v_entry.status <> 'Approved' THEN
        RAISE EXCEPTION 'Time entry % must be Approved to post (status=%)', p_entry_uuid, v_entry.status;
    END IF;
    IF v_entry.production_order_uuid IS NULL AND v_entry.activity_type IN ('Direct', 'Setup', 'Rework') THEN
        RAISE EXCEPTION 'Direct/Setup/Rework entry requires production_order_uuid';
    END IF;
    IF v_entry.hourly_rate IS NULL OR v_entry.hours IS NULL THEN
        RAISE EXCEPTION 'Approved entry must have hours and hourly_rate';
    END IF;

    v_class := CASE
        WHEN v_entry.activity_type IN ('Direct', 'Setup', 'Rework') THEN 'Direct'
        ELSE 'Indirect'
    END;

    -- For indirect without a production order, still need an order for cost_applications FK.
    -- Policy: indirect must name a production order OR we skip WIP and use OH only —
    -- current cost_applications requires production_order_uuid. Require order for all posts.
    IF v_entry.production_order_uuid IS NULL THEN
        RAISE EXCEPTION 'production_order_uuid required to post labour cost application';
    END IF;

    INSERT INTO manufacturing.labour_allocations (
        production_order_uuid, labour_class, hours, rate,
        work_center_code, employee_uuid, source, source_ref, memo, created_by, status
    ) VALUES (
        v_entry.production_order_uuid,
        v_class,
        v_entry.hours,
        v_entry.hourly_rate,
        v_entry.work_center_code,
        v_entry.employee_uuid,
        'TimeEntry',
        p_entry_uuid::text,
        COALESCE(v_entry.memo, format('Timesheet %s %s', v_entry.activity_type, v_entry.work_date)),
        p_user,
        'Draft'
    )
    RETURNING * INTO v_alloc;

    v_app := manufacturing.post_labour_allocation(
        v_alloc.uuid,
        p_user,
        p_wip_account,
        p_wages_account,
        CASE WHEN v_class = 'Indirect' THEN p_oh_control_account ELSE NULL END
    );

    UPDATE manufacturing.labour_time_entries
    SET status = 'Posted',
        labour_allocation_uuid = v_alloc.uuid,
        amount = v_app.amount,
        updated_at = now()
    WHERE uuid = p_entry_uuid;

    RETURN v_app;
END;
$$;

-- Auto-draft timesheet from completed operation execution (operator + actual minutes)
CREATE OR REPLACE FUNCTION manufacturing.draft_time_entry_from_operation(
    p_execution_uuid uuid,
    p_employee_uuid uuid DEFAULT NULL,
    p_created_by uuid DEFAULT NULL
) RETURNS manufacturing.labour_time_entries
LANGUAGE plpgsql
AS $$
DECLARE
    v_oe manufacturing.operation_executions%ROWTYPE;
    v_emp uuid;
    v_hours numeric(18, 4);
    v_row manufacturing.labour_time_entries%ROWTYPE;
BEGIN
    SELECT * INTO STRICT v_oe
    FROM manufacturing.operation_executions
    WHERE uuid = p_execution_uuid;

    IF v_oe.status <> 'Completed' THEN
        RAISE EXCEPTION 'Operation must be Completed to draft time entry';
    END IF;

    v_emp := COALESCE(p_employee_uuid, v_oe.operator_uuid);
    IF v_emp IS NULL THEN
        RAISE EXCEPTION 'employee_uuid or operation operator_uuid required';
    END IF;

    v_hours := (COALESCE(v_oe.setup_minutes_actual, 0) + COALESCE(v_oe.run_minutes_actual, 0)) / 60.0;
    IF v_hours <= 0 THEN
        RAISE EXCEPTION 'Operation has no actual setup/run minutes';
    END IF;

    RETURN manufacturing.create_labour_time_entry(
        v_emp,
        COALESCE(v_oe.completed_at::date, CURRENT_DATE),
        v_hours,
        'Direct',
        v_oe.production_order_uuid,
        v_oe.uuid,
        v_oe.work_center_code,
        NULL,
        v_oe.started_at,
        v_oe.completed_at,
        0,
        COALESCE(v_oe.qty_completed, 0),
        COALESCE(v_oe.qty_scrapped, 0),
        format('Auto from op %s', COALESCE(v_oe.operation_name, v_oe.uuid::text)),
        p_created_by
    );
END;
$$;

CREATE OR REPLACE VIEW manufacturing.v_production_order_labour AS
SELECT
    po.uuid AS production_order_uuid,
    po.order_number,
    po.status AS order_status,
    COALESCE(SUM(ca.amount) FILTER (WHERE ca.type = 'DirectLabor'), 0) AS applied_labour,
    COALESCE(SUM(ca.amount) FILTER (WHERE ca.type = 'ActualLabor'), 0) AS actual_labour,
    COALESCE(SUM(ca.amount) FILTER (WHERE ca.type = 'IndirectLabor'), 0) AS indirect_labour,
    COALESCE(SUM(lte.hours) FILTER (WHERE lte.status = 'Posted' AND lte.activity_type IN ('Direct','Setup','Rework')), 0)
        AS actual_direct_hours,
    COALESCE(SUM(lte.hours) FILTER (WHERE lte.status IN ('Draft','Submitted','Approved')), 0)
        AS open_timesheet_hours,
    COALESCE(SUM(ca.amount) FILTER (WHERE ca.type = 'ActualLabor'), 0)
        - COALESCE(SUM(ca.amount) FILTER (WHERE ca.type = 'DirectLabor'), 0)
        AS labour_cost_variance
FROM manufacturing.production_orders po
LEFT JOIN manufacturing.cost_applications ca ON ca.production_order_uuid = po.uuid
LEFT JOIN manufacturing.labour_time_entries lte ON lte.production_order_uuid = po.uuid
GROUP BY po.uuid, po.order_number, po.status;

-- Seed a few default categories (idempotent)
INSERT INTO manufacturing.labour_categories (code, name, description)
VALUES
    ('ASSY', 'Assembly technician', 'General assembly labour'),
    ('ELEC', 'Electrical technician', 'Wiring and electronics'),
    ('MACH', 'Machine operator', 'CNC / machine run'),
    ('QA', 'Quality inspector', 'Inspection and test')
ON CONFLICT (code) DO NOTHING;


-- ============================================================================
-- STAGE: LABOUR VARIANCE (rate × efficiency decomposition)
-- Actual hours/rates come from posted timesheets; standard from routing.
-- Variance rows are analytical; optional GL post uses labour efficiency account.
-- ============================================================================

CREATE TABLE IF NOT EXISTS manufacturing.labour_variances (
    uuid uuid DEFAULT uuidv7() PRIMARY KEY,
    production_order_uuid uuid NOT NULL REFERENCES manufacturing.production_orders (uuid) ON DELETE CASCADE,
    as_of_date date NOT NULL DEFAULT CURRENT_DATE,
    standard_hours numeric(18, 4) NOT NULL DEFAULT 0,
    actual_hours numeric(18, 4) NOT NULL DEFAULT 0,
    standard_rate numeric(18, 6) NOT NULL DEFAULT 0,
    actual_rate numeric(18, 6) NOT NULL DEFAULT 0,
    applied_labour_cost numeric(18, 2) NOT NULL DEFAULT 0,
    actual_labour_cost numeric(18, 2) NOT NULL DEFAULT 0,
    -- AH×(AR−SR)
    rate_variance numeric(18, 2) NOT NULL DEFAULT 0,
    -- SR×(AH−SH)
    efficiency_variance numeric(18, 2) NOT NULL DEFAULT 0,
    -- AR×AH − SR×SH  (should ≈ rate + efficiency)
    total_variance numeric(18, 2) NOT NULL DEFAULT 0,
    posted boolean NOT NULL DEFAULT false,
    cost_application_uuid uuid,
    notes text,
    created_at timestamptz DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_labour_var_order
    ON manufacturing.labour_variances (production_order_uuid);

CREATE OR REPLACE FUNCTION manufacturing.compute_labour_variances(
    p_order_uuid uuid,
    p_as_of date DEFAULT CURRENT_DATE
) RETURNS manufacturing.labour_variances
LANGUAGE plpgsql
AS $$
DECLARE
    v_order manufacturing.production_orders%ROWTYPE;
    v_std_hours numeric(18, 4) := 0;
    v_act_hours numeric(18, 4) := 0;
    v_app_cost numeric(18, 2) := 0;
    v_act_cost numeric(18, 2) := 0;
    v_std_rate numeric(18, 6) := 0;
    v_act_rate numeric(18, 6) := 0;
    v_rate_var numeric(18, 2);
    v_eff_var numeric(18, 2);
    v_tot numeric(18, 2);
    v_qty numeric(18, 4);
    v_row manufacturing.labour_variances%ROWTYPE;
BEGIN
    SELECT * INTO STRICT v_order
    FROM manufacturing.production_orders
    WHERE uuid = p_order_uuid;

    v_qty := COALESCE(NULLIF(v_order.quantity_completed, 0), v_order.quantity_ordered);

    -- Standard hours from routing for completed (or ordered) qty
    SELECT COALESCE(rl.total_hours, 0) INTO v_std_hours
    FROM manufacturing.routing_labour_for_qty(v_order.product_item_uuid, v_qty) rl
    LIMIT 1;

    -- If routing pin exists, prefer hours from pinned routing ops × qty
    IF v_order.routing_uuid IS NOT NULL THEN
        SELECT COALESCE(SUM(
            (ro.setup_time_minutes + ro.run_time_minutes * (v_qty / NULLIF(r.base_quantity, 0)))
            / 60.0
        ), 0)
        INTO v_std_hours
        FROM manufacturing.routing_operations ro
        JOIN manufacturing.routings r ON r.uuid = ro.routing_uuid
        WHERE ro.routing_uuid = v_order.routing_uuid;
    END IF;

    -- Actual hours from posted timesheets (direct activities)
    SELECT COALESCE(SUM(hours), 0) INTO v_act_hours
    FROM manufacturing.labour_time_entries
    WHERE production_order_uuid = p_order_uuid
      AND status = 'Posted'
      AND activity_type IN ('Direct', 'Setup', 'Rework');

    -- Costs from cost_applications
    SELECT COALESCE(SUM(amount), 0) INTO v_app_cost
    FROM manufacturing.cost_applications
    WHERE production_order_uuid = p_order_uuid AND type = 'DirectLabor';

    SELECT COALESCE(SUM(amount), 0) INTO v_act_cost
    FROM manufacturing.cost_applications
    WHERE production_order_uuid = p_order_uuid AND type = 'ActualLabor';

    -- Implied rates
    IF v_std_hours > 0 AND v_app_cost > 0 THEN
        v_std_rate := v_app_cost / v_std_hours;
    ELSIF v_std_hours > 0 THEN
        -- average standard rate from labour_rates / WC
        SELECT COALESCE(AVG(hourly_rate), 0) INTO v_std_rate
        FROM manufacturing.labour_rates
        WHERE rate_kind = 'Standard'
          AND effective_from <= p_as_of
          AND (effective_to IS NULL OR effective_to >= p_as_of);
    END IF;

    IF v_act_hours > 0 AND v_act_cost > 0 THEN
        v_act_rate := v_act_cost / v_act_hours;
    ELSIF v_act_hours > 0 THEN
        SELECT COALESCE(AVG(hourly_rate), 0) INTO v_act_rate
        FROM manufacturing.labour_time_entries
        WHERE production_order_uuid = p_order_uuid AND status = 'Posted'
          AND hourly_rate IS NOT NULL;
    END IF;

    -- Classic formulas (favourable negative if actual cheaper/faster)
    v_rate_var := ROUND(v_act_hours * (v_act_rate - v_std_rate), 2);
    v_eff_var := ROUND(v_std_rate * (v_act_hours - v_std_hours), 2);
    v_tot := ROUND(v_act_cost - v_app_cost, 2);
    -- If no applied cost yet, total = rate + efficiency at standard identity
    IF v_app_cost = 0 AND v_act_cost = 0 THEN
        v_tot := ROUND(v_rate_var + v_eff_var, 2);
    END IF;

    INSERT INTO manufacturing.labour_variances (
        production_order_uuid, as_of_date,
        standard_hours, actual_hours, standard_rate, actual_rate,
        applied_labour_cost, actual_labour_cost,
        rate_variance, efficiency_variance, total_variance,
        notes
    ) VALUES (
        p_order_uuid, p_as_of,
        v_std_hours, v_act_hours, v_std_rate, v_act_rate,
        v_app_cost, v_act_cost,
        v_rate_var, v_eff_var, v_tot,
        format('SH=%s AH=%s SR=%s AR=%s', v_std_hours, v_act_hours, v_std_rate, v_act_rate)
    )
    RETURNING * INTO v_row;

    RETURN v_row;
END;
$$;

-- Replace summary view with richer metrics
CREATE OR REPLACE VIEW manufacturing.v_production_order_labour AS
SELECT
    po.uuid AS production_order_uuid,
    po.order_number,
    po.status AS order_status,
    COALESCE(ca.applied_labour, 0) AS applied_labour,
    COALESCE(ca.actual_labour, 0) AS actual_labour,
    COALESCE(ca.indirect_labour, 0) AS indirect_labour,
    COALESCE(lte.actual_direct_hours, 0) AS actual_direct_hours,
    COALESCE(lte.open_timesheet_hours, 0) AS open_timesheet_hours,
    COALESCE(ca.actual_labour, 0) - COALESCE(ca.applied_labour, 0) AS labour_cost_variance,
    lv.standard_hours,
    lv.actual_hours AS variance_actual_hours,
    lv.standard_rate,
    lv.actual_rate,
    lv.rate_variance,
    lv.efficiency_variance,
    lv.total_variance AS last_computed_total_variance
FROM manufacturing.production_orders po
LEFT JOIN LATERAL (
    SELECT
        SUM(amount) FILTER (WHERE type = 'DirectLabor') AS applied_labour,
        SUM(amount) FILTER (WHERE type = 'ActualLabor') AS actual_labour,
        SUM(amount) FILTER (WHERE type = 'IndirectLabor') AS indirect_labour
    FROM manufacturing.cost_applications
    WHERE production_order_uuid = po.uuid
) ca ON true
LEFT JOIN LATERAL (
    SELECT
        SUM(hours) FILTER (
            WHERE status = 'Posted' AND activity_type IN ('Direct', 'Setup', 'Rework')
        ) AS actual_direct_hours,
        SUM(hours) FILTER (
            WHERE status IN ('Draft', 'Submitted', 'Approved')
        ) AS open_timesheet_hours
    FROM manufacturing.labour_time_entries
    WHERE production_order_uuid = po.uuid
) lte ON true
LEFT JOIN LATERAL (
    SELECT *
    FROM manufacturing.labour_variances
    WHERE production_order_uuid = po.uuid
    ORDER BY created_at DESC
    LIMIT 1
) lv ON true;


-- ============================================================================
-- STAGE: LABOUR REPORTING + payroll-aware rate (contract basic ÷ standard hours)
-- Manufacturing reports productivity; payroll remains source of employment cost.
-- ============================================================================

-- Derive indicative actual hourly rate from active employment contract.
-- Monthly → / 173.33; Weekly → / 40; Daily → / 8; Hourly → as-is.
CREATE OR REPLACE FUNCTION manufacturing.hourly_rate_from_contract(
    p_employee_uuid uuid,
    p_as_of date DEFAULT CURRENT_DATE
) RETURNS numeric
LANGUAGE plpgsql
STABLE
AS $$
DECLARE
    v_salary numeric(14, 2);
    v_freq text;
    v_hours numeric;
BEGIN
    SELECT c.basic_salary, COALESCE(c.pay_frequency, e.pay_frequency)
    INTO v_salary, v_freq
    FROM hr.employees e
    LEFT JOIN hr.employment_contracts c
      ON c.employee_uuid = e.uuid
     AND c.status = 'Active'
     AND (c.start_date IS NULL OR c.start_date <= p_as_of)
     AND (c.end_date IS NULL OR c.end_date >= p_as_of)
    WHERE e.uuid = p_employee_uuid
    ORDER BY c.start_date DESC NULLS LAST
    LIMIT 1;

    IF v_salary IS NULL OR v_salary <= 0 THEN
        RETURN NULL;
    END IF;

    v_freq := upper(COALESCE(v_freq, 'Monthly'));
    v_hours := CASE
        WHEN v_freq IN ('HOURLY', 'HOUR') THEN 1
        WHEN v_freq IN ('DAILY', 'DAY') THEN 8
        WHEN v_freq IN ('WEEKLY', 'WEEK') THEN 40
        WHEN v_freq IN ('BIWEEKLY', 'FORTNIGHTLY') THEN 80
        ELSE 173.33  -- Monthly default (52*40/12)
    END;

    RETURN ROUND(v_salary / v_hours, 4);
END;
$$;

-- Prefer: explicit rate → labour_rates Actual → contract-derived → WC rate
CREATE OR REPLACE FUNCTION manufacturing.resolve_labour_rate(
    p_rate_kind text,
    p_as_of date,
    p_employee_uuid uuid DEFAULT NULL,
    p_category_uuid uuid DEFAULT NULL,
    p_work_center_code text DEFAULT NULL
) RETURNS numeric
LANGUAGE plpgsql
STABLE
AS $$
DECLARE
    v_rate numeric(18, 4);
BEGIN
    IF p_employee_uuid IS NOT NULL THEN
        SELECT hourly_rate INTO v_rate
        FROM manufacturing.labour_rates
        WHERE rate_kind = p_rate_kind
          AND employee_uuid = p_employee_uuid
          AND effective_from <= p_as_of
          AND (effective_to IS NULL OR effective_to >= p_as_of)
        ORDER BY effective_from DESC
        LIMIT 1;
        IF v_rate IS NOT NULL THEN RETURN v_rate; END IF;
    END IF;

    IF p_category_uuid IS NOT NULL THEN
        SELECT hourly_rate INTO v_rate
        FROM manufacturing.labour_rates
        WHERE rate_kind = p_rate_kind
          AND labour_category_uuid = p_category_uuid
          AND employee_uuid IS NULL
          AND effective_from <= p_as_of
          AND (effective_to IS NULL OR effective_to >= p_as_of)
        ORDER BY effective_from DESC
        LIMIT 1;
        IF v_rate IS NOT NULL THEN RETURN v_rate; END IF;
    END IF;

    IF p_work_center_code IS NOT NULL THEN
        SELECT hourly_rate INTO v_rate
        FROM manufacturing.labour_rates
        WHERE rate_kind = p_rate_kind
          AND work_center_code = p_work_center_code
          AND employee_uuid IS NULL
          AND labour_category_uuid IS NULL
          AND effective_from <= p_as_of
          AND (effective_to IS NULL OR effective_to >= p_as_of)
        ORDER BY effective_from DESC
        LIMIT 1;
        IF v_rate IS NOT NULL THEN RETURN v_rate; END IF;
    END IF;

    -- Payroll-aware fallback for Actual rates only
    IF p_rate_kind = 'Actual' AND p_employee_uuid IS NOT NULL THEN
        v_rate := manufacturing.hourly_rate_from_contract(p_employee_uuid, p_as_of);
        IF v_rate IS NOT NULL THEN RETURN v_rate; END IF;
    END IF;

    -- No WC labor_rate fallback for Actual
    RETURN NULL;
END;
$$;

-- Productivity by production order
CREATE OR REPLACE VIEW manufacturing.v_labour_productivity_by_order AS
SELECT
    po.uuid AS production_order_uuid,
    po.order_number,
    po.status,
    po.quantity_ordered,
    po.quantity_completed,
    COALESCE(SUM(lte.hours) FILTER (
        WHERE lte.status = 'Posted' AND lte.activity_type IN ('Direct', 'Setup', 'Rework')
    ), 0) AS direct_hours,
    COALESCE(SUM(lte.hours) FILTER (
        WHERE lte.status = 'Posted' AND lte.activity_type IN ('Idle', 'Indirect', 'Training')
    ), 0) AS non_productive_hours,
    COALESCE(SUM(lte.amount) FILTER (WHERE lte.status = 'Posted'), 0) AS timesheet_cost,
    CASE
        WHEN COALESCE(SUM(lte.hours) FILTER (
            WHERE lte.status = 'Posted' AND lte.activity_type IN ('Direct', 'Setup', 'Rework')
        ), 0) > 0
        THEN ROUND(
            COALESCE(po.quantity_completed, 0)
            / SUM(lte.hours) FILTER (
                WHERE lte.status = 'Posted' AND lte.activity_type IN ('Direct', 'Setup', 'Rework')
            ),
            4
        )
        ELSE NULL
    END AS units_per_direct_hour,
    CASE
        WHEN COALESCE(po.quantity_completed, 0) > 0
        THEN ROUND(
            COALESCE(SUM(lte.amount) FILTER (
                WHERE lte.status = 'Posted' AND lte.activity_type IN ('Direct', 'Setup', 'Rework')
            ), 0) / po.quantity_completed,
            2
        )
        ELSE NULL
    END AS actual_labour_cost_per_unit
FROM manufacturing.production_orders po
LEFT JOIN manufacturing.labour_time_entries lte
  ON lte.production_order_uuid = po.uuid
GROUP BY po.uuid, po.order_number, po.status, po.quantity_ordered, po.quantity_completed;

-- By employee (booked vs productive)
CREATE OR REPLACE VIEW manufacturing.v_labour_by_employee AS
SELECT
    lte.employee_uuid,
    date_trunc('month', lte.work_date)::date AS month_start,
    COALESCE(SUM(lte.hours) FILTER (WHERE lte.status = 'Posted'), 0) AS posted_hours,
    COALESCE(SUM(lte.hours) FILTER (
        WHERE lte.status = 'Posted' AND lte.activity_type IN ('Direct', 'Setup', 'Rework')
    ), 0) AS productive_hours,
    COALESCE(SUM(lte.hours) FILTER (
        WHERE lte.status = 'Posted' AND lte.activity_type IN ('Idle', 'Indirect', 'Training', 'Other')
    ), 0) AS non_productive_hours,
    COALESCE(SUM(lte.amount) FILTER (WHERE lte.status = 'Posted'), 0) AS posted_cost,
    CASE
        WHEN COALESCE(SUM(lte.hours) FILTER (WHERE lte.status = 'Posted'), 0) > 0
        THEN ROUND(
            100.0 * SUM(lte.hours) FILTER (
                WHERE lte.status = 'Posted' AND lte.activity_type IN ('Direct', 'Setup', 'Rework')
            ) / SUM(lte.hours) FILTER (WHERE lte.status = 'Posted'),
            2
        )
        ELSE NULL
    END AS utilisation_pct
FROM manufacturing.labour_time_entries lte
GROUP BY lte.employee_uuid, date_trunc('month', lte.work_date);

-- By work centre
CREATE OR REPLACE VIEW manufacturing.v_labour_by_work_center AS
SELECT
    COALESCE(lte.work_center_code, '(none)') AS work_center_code,
    date_trunc('month', lte.work_date)::date AS month_start,
    COUNT(*) FILTER (WHERE lte.status = 'Posted') AS posted_entries,
    COALESCE(SUM(lte.hours) FILTER (WHERE lte.status = 'Posted'), 0) AS posted_hours,
    COALESCE(SUM(lte.amount) FILTER (WHERE lte.status = 'Posted'), 0) AS posted_cost,
    COALESCE(SUM(lte.quantity_completed) FILTER (WHERE lte.status = 'Posted'), 0) AS qty_completed
FROM manufacturing.labour_time_entries lte
GROUP BY COALESCE(lte.work_center_code, '(none)'), date_trunc('month', lte.work_date);

-- Operation efficiency: actual minutes vs routing standard for same sequence
CREATE OR REPLACE VIEW manufacturing.v_operation_efficiency AS
SELECT
    oe.uuid AS execution_uuid,
    oe.production_order_uuid,
    po.order_number,
    oe.sequence,
    oe.operation_name,
    oe.work_center_code,
    oe.status,
    COALESCE(oe.setup_minutes_actual, 0) + COALESCE(oe.run_minutes_actual, 0) AS actual_minutes,
    COALESCE(ro.setup_time_minutes, 0)
        + COALESCE(ro.run_time_minutes, 0)
          * (COALESCE(NULLIF(po.quantity_completed, 0), po.quantity_ordered)
             / NULLIF(r.base_quantity, 0)) AS standard_minutes,
    CASE
        WHEN (
            COALESCE(ro.setup_time_minutes, 0)
            + COALESCE(ro.run_time_minutes, 0)
              * (COALESCE(NULLIF(po.quantity_completed, 0), po.quantity_ordered)
                 / NULLIF(r.base_quantity, 0))
        ) > 0
        THEN ROUND(
            100.0 * (
                COALESCE(ro.setup_time_minutes, 0)
                + COALESCE(ro.run_time_minutes, 0)
                  * (COALESCE(NULLIF(po.quantity_completed, 0), po.quantity_ordered)
                     / NULLIF(r.base_quantity, 0))
            ) / NULLIF(
                COALESCE(oe.setup_minutes_actual, 0) + COALESCE(oe.run_minutes_actual, 0),
                0
            ),
            2
        )
        ELSE NULL
    END AS efficiency_pct
FROM manufacturing.operation_executions oe
JOIN manufacturing.production_orders po ON po.uuid = oe.production_order_uuid
LEFT JOIN manufacturing.routing_operations ro ON ro.uuid = oe.routing_operation_uuid
LEFT JOIN manufacturing.routings r ON r.uuid = ro.routing_uuid;


-- ============================================================================
-- STAGE: LABOUR RECONCILIATION (attendance vs timesheet) + payslip cost allocation
-- ============================================================================

-- Paid/present hours vs manufacturing-booked hours (by employee × day)
CREATE OR REPLACE VIEW manufacturing.v_labour_hours_reconciliation AS
SELECT
    COALESCE(a.employee_uuid, t.employee_uuid) AS employee_uuid,
    COALESCE(a.work_date, t.work_date) AS work_date,
    COALESCE(a.hours_present, 0) AS attendance_hours,
    COALESCE(t.booked_hours, 0) AS booked_hours,
    COALESCE(t.productive_hours, 0) AS productive_hours,
    COALESCE(a.hours_present, 0) - COALESCE(t.booked_hours, 0) AS unallocated_hours,
    CASE
        WHEN COALESCE(a.hours_present, 0) > 0
        THEN ROUND(
            100.0 * COALESCE(t.booked_hours, 0) / a.hours_present,
            2
        )
        ELSE NULL
    END AS booking_coverage_pct
FROM (
    SELECT employee_uuid, work_date, SUM(hours_present) AS hours_present
    FROM hr.attendance_entries
    GROUP BY employee_uuid, work_date
) a
FULL OUTER JOIN (
    SELECT
        employee_uuid,
        work_date,
        SUM(hours) FILTER (WHERE status IN ('Approved', 'Posted', 'Submitted')) AS booked_hours,
        SUM(hours) FILTER (
            WHERE status IN ('Approved', 'Posted')
              AND activity_type IN ('Direct', 'Setup', 'Rework')
        ) AS productive_hours
    FROM manufacturing.labour_time_entries
    GROUP BY employee_uuid, work_date
) t ON t.employee_uuid = a.employee_uuid AND t.work_date = a.work_date;

-- Period rollup
CREATE OR REPLACE VIEW manufacturing.v_labour_hours_reconciliation_period AS
SELECT
    employee_uuid,
    date_trunc('month', work_date)::date AS month_start,
    SUM(attendance_hours) AS attendance_hours,
    SUM(booked_hours) AS booked_hours,
    SUM(productive_hours) AS productive_hours,
    SUM(unallocated_hours) AS unallocated_hours,
    CASE
        WHEN SUM(attendance_hours) > 0
        THEN ROUND(100.0 * SUM(booked_hours) / SUM(attendance_hours), 2)
        ELSE NULL
    END AS booking_coverage_pct
FROM manufacturing.v_labour_hours_reconciliation
GROUP BY employee_uuid, date_trunc('month', work_date);

-- Allocate a processed payslip's employer labour cost to production orders
-- proportional to that employee's posted timesheet hours in the payrun period.
CREATE OR REPLACE FUNCTION manufacturing.allocate_payslip_labour_to_orders(
    p_payslip_uuid uuid,
    p_user uuid,
    p_wip_account text,
    p_labour_clearing_account text,
    p_cost_basis text DEFAULT 'employer_cost' -- employer_cost | gross | basic
) RETURNS TABLE (
    production_order_uuid uuid,
    allocated_amount numeric,
    hours_share numeric,
    cost_application_uuid uuid
)
LANGUAGE plpgsql
AS $$
DECLARE
    v_ps payroll.payslips%ROWTYPE;
    v_pr payroll.payruns%ROWTYPE;
    v_total_hours numeric(18,4);
    v_pool numeric(18,2);
    v_wip uuid;
    v_clear uuid;
    rec RECORD;
    v_amt numeric(18,2);
    v_alloc manufacturing.labour_allocations%ROWTYPE;
    v_app manufacturing.cost_applications%ROWTYPE;
BEGIN
    SELECT * INTO STRICT v_ps FROM payroll.payslips WHERE uuid = p_payslip_uuid;
    SELECT * INTO STRICT v_pr FROM payroll.payruns WHERE uuid = v_ps.payrun_uuid;

    v_pool := CASE lower(COALESCE(p_cost_basis, 'employer_cost'))
        WHEN 'basic' THEN COALESCE(v_ps.basic_pay, 0)
        WHEN 'gross' THEN COALESCE(v_ps.gross_pay, 0)
        ELSE COALESCE(NULLIF(v_ps.employer_cost_total, 0), v_ps.gross_pay, 0)
    END;

    IF v_pool IS NULL OR v_pool <= 0 THEN
        RAISE EXCEPTION 'Payslip % has no cost pool to allocate', p_payslip_uuid;
    END IF;

    SELECT COALESCE(SUM(hours), 0) INTO v_total_hours
    FROM manufacturing.labour_time_entries
    WHERE employee_uuid = v_ps.employee_uuid
      AND status = 'Posted'
      AND activity_type IN ('Direct', 'Setup', 'Rework')
      AND work_date BETWEEN v_pr.pay_period_start AND v_pr.pay_period_end
      AND production_order_uuid IS NOT NULL;

    IF v_total_hours <= 0 THEN
        RAISE EXCEPTION
            'No posted productive timesheet hours for employee % in % – %',
            v_ps.employee_uuid, v_pr.pay_period_start, v_pr.pay_period_end;
    END IF;

    v_wip := accounting.resolve_account_ref(p_wip_account);
    v_clear := accounting.resolve_account_ref(p_labour_clearing_account);

    FOR rec IN
        SELECT production_order_uuid, SUM(hours) AS hrs
        FROM manufacturing.labour_time_entries
        WHERE employee_uuid = v_ps.employee_uuid
          AND status = 'Posted'
          AND activity_type IN ('Direct', 'Setup', 'Rework')
          AND work_date BETWEEN v_pr.pay_period_start AND v_pr.pay_period_end
          AND production_order_uuid IS NOT NULL
        GROUP BY production_order_uuid
    LOOP
        v_amt := ROUND(v_pool * (rec.hrs / v_total_hours), 2);

        INSERT INTO manufacturing.labour_allocations (
            production_order_uuid, labour_class, hours, rate,
            employee_uuid, source, source_ref, memo, created_by, status
        ) VALUES (
            rec.production_order_uuid,
            'Direct',
            rec.hrs,
            CASE WHEN rec.hrs > 0 THEN ROUND(v_amt / rec.hrs, 4) ELSE 0 END,
            v_ps.employee_uuid,
            'Payroll',
            p_payslip_uuid::text,
            format('Payslip allocation %s basis=%s', p_payslip_uuid, p_cost_basis),
            p_user,
            'Draft'
        )
        RETURNING * INTO v_alloc;

        -- Post: Dr WIP / Cr labour clearing (reclass, not second wage expense)
        v_app := manufacturing.post_labour_allocation(
            v_alloc.uuid, p_user, p_wip_account, p_labour_clearing_account, NULL
        );

        production_order_uuid := rec.production_order_uuid;
        allocated_amount := v_amt;
        hours_share := rec.hrs;
        cost_application_uuid := v_app.uuid;
        RETURN NEXT;
    END LOOP;
END;
$$;


-- ============================================================================
-- STAGE: LABOUR OPS BULK — team timesheets + payrun-wide payslip allocation
-- ============================================================================

-- Supervisor records the same shift/activity for multiple employees at once.
CREATE OR REPLACE FUNCTION manufacturing.create_team_time_entries(
    p_employee_uuids uuid[],
    p_work_date date,
    p_hours numeric,
    p_activity_type text DEFAULT 'Direct',
    p_production_order_uuid uuid DEFAULT NULL,
    p_operation_execution_uuid uuid DEFAULT NULL,
    p_work_center_code text DEFAULT NULL,
    p_labour_category_uuid uuid DEFAULT NULL,
    p_memo text DEFAULT NULL,
    p_created_by uuid DEFAULT NULL
) RETURNS SETOF manufacturing.labour_time_entries
LANGUAGE plpgsql
AS $$
DECLARE
    emp uuid;
    v_row manufacturing.labour_time_entries%ROWTYPE;
BEGIN
    IF p_employee_uuids IS NULL OR array_length(p_employee_uuids, 1) IS NULL THEN
        RAISE EXCEPTION 'employee_uuids required';
    END IF;
    IF p_hours IS NULL OR p_hours <= 0 THEN
        RAISE EXCEPTION 'hours must be positive';
    END IF;

    FOREACH emp IN ARRAY p_employee_uuids
    LOOP
        v_row := manufacturing.create_labour_time_entry(
            emp,
            p_work_date,
            p_hours,
            COALESCE(p_activity_type, 'Direct'),
            p_production_order_uuid,
            p_operation_execution_uuid,
            p_work_center_code,
            p_labour_category_uuid,
            NULL, NULL, 0, 0, 0,
            COALESCE(p_memo, format('Team entry %s', p_work_date)),
            p_created_by
        );
        RETURN NEXT v_row;
    END LOOP;
END;
$$;

-- Bulk submit / approve / post by order (or explicit entry list)
CREATE OR REPLACE FUNCTION manufacturing.bulk_submit_time_entries(
    p_entry_uuids uuid[]
) RETURNS integer
LANGUAGE plpgsql
AS $$
DECLARE
    id uuid;
    n int := 0;
BEGIN
    FOREACH id IN ARRAY p_entry_uuids
    LOOP
        PERFORM manufacturing.submit_labour_time_entry(id);
        n := n + 1;
    END LOOP;
    RETURN n;
END;
$$;

CREATE OR REPLACE FUNCTION manufacturing.bulk_approve_time_entries(
    p_entry_uuids uuid[],
    p_approver uuid,
    p_hourly_rate numeric DEFAULT NULL
) RETURNS integer
LANGUAGE plpgsql
AS $$
DECLARE
    id uuid;
    n int := 0;
BEGIN
    FOREACH id IN ARRAY p_entry_uuids
    LOOP
        PERFORM manufacturing.approve_labour_time_entry(id, p_approver, p_hourly_rate);
        n := n + 1;
    END LOOP;
    RETURN n;
END;
$$;

CREATE OR REPLACE FUNCTION manufacturing.bulk_post_time_entries(
    p_entry_uuids uuid[],
    p_user uuid,
    p_wip_account text,
    p_wages_account text,
    p_oh_control text DEFAULT NULL
) RETURNS integer
LANGUAGE plpgsql
AS $$
DECLARE
    id uuid;
    n int := 0;
BEGIN
    FOREACH id IN ARRAY p_entry_uuids
    LOOP
        PERFORM manufacturing.post_labour_time_entry(
            id, p_user, p_wip_account, p_wages_account, p_oh_control
        );
        n := n + 1;
    END LOOP;
    RETURN n;
END;
$$;

-- After payroll is processed: allocate every payslip on the payrun
CREATE OR REPLACE FUNCTION manufacturing.allocate_payrun_labour_to_orders(
    p_payrun_uuid uuid,
    p_user uuid,
    p_wip_account text,
    p_labour_clearing_account text,
    p_cost_basis text DEFAULT 'employer_cost'
) RETURNS TABLE (
    payslip_uuid uuid,
    production_order_uuid uuid,
    allocated_amount numeric,
    hours_share numeric,
    cost_application_uuid uuid,
    status text
)
LANGUAGE plpgsql
AS $$
DECLARE
    v_ps RECORD;
    v_row RECORD;
BEGIN
    FOR v_ps IN
        SELECT uuid AS payslip_uuid, employee_uuid
        FROM payroll.payslips
        WHERE payrun_uuid = p_payrun_uuid
    LOOP
        BEGIN
            FOR v_row IN
                SELECT *
                FROM manufacturing.allocate_payslip_labour_to_orders(
                    v_ps.payslip_uuid, p_user, p_wip_account,
                    p_labour_clearing_account, p_cost_basis
                )
            LOOP
                payslip_uuid := v_ps.payslip_uuid;
                production_order_uuid := v_row.production_order_uuid;
                allocated_amount := v_row.allocated_amount;
                hours_share := v_row.hours_share;
                cost_application_uuid := v_row.cost_application_uuid;
                status := 'Allocated';
                RETURN NEXT;
            END LOOP;
        EXCEPTION WHEN OTHERS THEN
            payslip_uuid := v_ps.payslip_uuid;
            production_order_uuid := NULL;
            allocated_amount := 0;
            hours_share := 0;
            cost_application_uuid := NULL;
            status := format('Skipped: %s', SQLERRM);
            RETURN NEXT;
        END;
    END LOOP;
END;
$$;

-- Draft timesheets from all completed ops on an order (one entry per op with operator)
CREATE OR REPLACE FUNCTION manufacturing.draft_time_entries_from_order_ops(
    p_order_uuid uuid,
    p_created_by uuid DEFAULT NULL
) RETURNS SETOF manufacturing.labour_time_entries
LANGUAGE plpgsql
AS $$
DECLARE
    v_oe RECORD;
    v_row manufacturing.labour_time_entries%ROWTYPE;
BEGIN
    FOR v_oe IN
        SELECT uuid
        FROM manufacturing.operation_executions
        WHERE production_order_uuid = p_order_uuid
          AND status = 'Completed'
          AND (COALESCE(setup_minutes_actual, 0) + COALESCE(run_minutes_actual, 0)) > 0
    LOOP
        BEGIN
            v_row := manufacturing.draft_time_entry_from_operation(
                v_oe.uuid, NULL, p_created_by
            );
            RETURN NEXT v_row;
        EXCEPTION WHEN OTHERS THEN
            -- skip ops without operator / hours
            CONTINUE;
        END;
    END LOOP;
END;
$$;


-- ============================================================================
-- STAGE: LABOUR ACCOUNTING INTEGRITY  (base: 0decd86)
-- Standard costing: routing DirectLabor capitalises WIP; timesheet ActualLabor is
-- analytical when applied already exists (no second WIP debit). Payslip path skips
-- if ActualLabor already present. Variance never treats DirectLabor as actual.
-- ============================================================================

CREATE OR REPLACE FUNCTION manufacturing.post_labour_allocation(
    p_allocation_uuid    uuid,
    p_user               uuid,
    p_wip_account        text,
    p_wages_account      text,
    p_oh_control_account text DEFAULT NULL
) RETURNS manufacturing.cost_applications
LANGUAGE plpgsql
AS $$
DECLARE
    v_alloc     manufacturing.labour_allocations%ROWTYPE;
    v_order     manufacturing.production_orders%ROWTYPE;
    v_wip       uuid;
    v_wages     uuid;
    v_oh        uuid;
    v_dest      uuid;
    v_type      text;
    v_txn       bigint;
    v_app       manufacturing.cost_applications%ROWTYPE;
    v_applied   numeric(18,2) := 0;
    v_gl_amount numeric(18,2);
    v_memo      text;
BEGIN
    SELECT * INTO STRICT v_alloc
    FROM manufacturing.labour_allocations
    WHERE uuid = p_allocation_uuid
    FOR UPDATE;

    IF v_alloc.status <> 'Draft' THEN
        RAISE EXCEPTION 'Labour allocation % is not Draft (status=%)', p_allocation_uuid, v_alloc.status;
    END IF;

    SELECT * INTO STRICT v_order
    FROM manufacturing.production_orders
    WHERE uuid = v_alloc.production_order_uuid;

    IF v_order.status NOT IN ('Planned', 'In Progress') THEN
        RAISE EXCEPTION 'Order must be Planned or In Progress to post labour';
    END IF;

    v_wip := accounting.resolve_account_ref(p_wip_account);
    v_wages := accounting.resolve_account_ref(p_wages_account);

    SELECT COALESCE(SUM(amount), 0) INTO v_applied
    FROM manufacturing.cost_applications
    WHERE production_order_uuid = v_alloc.production_order_uuid
      AND type = 'DirectLabor';

    IF v_alloc.labour_class = 'Direct' THEN
        v_type := 'ActualLabor';
        v_dest := v_wip;
        IF v_alloc.source IN ('TimeEntry', 'Manual') AND v_applied > 0 THEN
            v_gl_amount := 0;
            v_memo := format(
                'Actual labour memo (no WIP debit; applied=%s already capitalised) %s hrs @ %s',
                v_applied, v_alloc.hours, v_alloc.rate
            );
        ELSE
            v_gl_amount := v_alloc.amount;
            v_memo := COALESCE(v_alloc.memo, 'Labour allocation');
        END IF;
    ELSE
        v_type := 'IndirectLabor';
        IF p_oh_control_account IS NULL OR btrim(p_oh_control_account) = '' THEN
            RAISE EXCEPTION
                'Indirect labour requires overhead_control_account';
        END IF;
        v_oh := accounting.resolve_account_ref(p_oh_control_account);
        v_dest := v_oh;
        v_gl_amount := v_alloc.amount;
        v_memo := COALESCE(v_alloc.memo, 'Indirect labour to OH control');
    END IF;

    IF v_gl_amount > 0 THEN
        v_txn := accounting.post_transaction(
            v_order.order_number || '-LAB-' || v_alloc.uuid::text,
            format(
                '%s labour %s hrs @ %s (%s)',
                v_alloc.labour_class, v_alloc.hours, v_alloc.rate, COALESCE(v_alloc.source, 'Manual')
            ),
            p_user,
            'manufacturing',
            CURRENT_DATE,
            jsonb_build_array(
                jsonb_build_object(
                    'account_ref', v_dest, 'debit', v_gl_amount, 'credit', 0,
                    'memo', v_memo
                ),
                jsonb_build_object(
                    'account_ref', v_wages, 'debit', 0, 'credit', v_gl_amount,
                    'memo', format('Wages clearing - %s labour', v_alloc.labour_class)
                )
            )
        );
    END IF;

    INSERT INTO manufacturing.cost_applications (
        production_order_uuid, type, amount, reference, source_account,
        destination_account, applied_at
    ) VALUES (
        v_alloc.production_order_uuid,
        v_type,
        v_alloc.amount,
        COALESCE(v_alloc.source_ref, v_alloc.uuid::text),
        v_wages,
        v_dest,
        now()
    )
    RETURNING * INTO v_app;

    UPDATE manufacturing.labour_allocations
    SET status = 'Posted',
        cost_application_uuid = v_app.uuid,
        posted_at = now()
    WHERE uuid = p_allocation_uuid;

    RETURN v_app;
END;
$$;

CREATE OR REPLACE FUNCTION manufacturing.create_labour_time_entry(
    p_employee_uuid uuid,
    p_work_date date,
    p_hours numeric DEFAULT NULL,
    p_activity_type text DEFAULT 'Direct',
    p_production_order_uuid uuid DEFAULT NULL,
    p_operation_execution_uuid uuid DEFAULT NULL,
    p_work_center_code text DEFAULT NULL,
    p_labour_category_uuid uuid DEFAULT NULL,
    p_start_time timestamptz DEFAULT NULL,
    p_end_time timestamptz DEFAULT NULL,
    p_break_minutes numeric DEFAULT 0,
    p_quantity_completed numeric DEFAULT 0,
    p_quantity_scrapped numeric DEFAULT 0,
    p_memo text DEFAULT NULL,
    p_created_by uuid DEFAULT NULL
) RETURNS manufacturing.labour_time_entries
LANGUAGE plpgsql
AS $$
DECLARE
    v_hours numeric(18, 4);
    v_row manufacturing.labour_time_entries%ROWTYPE;
    v_op manufacturing.operation_executions%ROWTYPE;
BEGIN
    IF p_activity_type NOT IN ('Direct', 'Indirect', 'Setup', 'Rework', 'Idle', 'Training', 'Other') THEN
        RAISE EXCEPTION 'Invalid activity_type %', p_activity_type;
    END IF;
    IF COALESCE(p_break_minutes, 0) < 0 THEN
        RAISE EXCEPTION 'break_minutes cannot be negative';
    END IF;
    IF p_start_time IS NOT NULL AND p_end_time IS NOT NULL AND p_end_time <= p_start_time THEN
        RAISE EXCEPTION 'end_time must be after start_time';
    END IF;

    v_hours := p_hours;
    IF v_hours IS NULL AND p_start_time IS NOT NULL AND p_end_time IS NOT NULL THEN
        v_hours := EXTRACT(EPOCH FROM (p_end_time - p_start_time)) / 3600.0
                   - COALESCE(p_break_minutes, 0) / 60.0;
    END IF;
    IF v_hours IS NULL OR v_hours <= 0 THEN
        RAISE EXCEPTION 'hours must be positive (provide hours or start/end)';
    END IF;

    IF p_activity_type IN ('Direct', 'Setup', 'Rework', 'Indirect', 'Idle', 'Training', 'Other')
       AND p_production_order_uuid IS NULL THEN
        RAISE EXCEPTION
            'production_order_uuid required for activity_type % (department OH pools not yet implemented)',
            p_activity_type;
    END IF;

    IF p_operation_execution_uuid IS NOT NULL THEN
        SELECT * INTO STRICT v_op
        FROM manufacturing.operation_executions
        WHERE uuid = p_operation_execution_uuid;
        IF p_production_order_uuid IS NOT NULL
           AND v_op.production_order_uuid IS DISTINCT FROM p_production_order_uuid THEN
            RAISE EXCEPTION 'operation_execution % does not belong to production order %',
                p_operation_execution_uuid, p_production_order_uuid;
        END IF;
        IF p_production_order_uuid IS NULL THEN
            p_production_order_uuid := v_op.production_order_uuid;
        END IF;
    END IF;

    INSERT INTO manufacturing.labour_time_entries (
        employee_uuid, production_order_uuid, operation_execution_uuid,
        work_center_code, labour_category_uuid, work_date,
        start_time, end_time, break_minutes, hours, activity_type,
        quantity_completed, quantity_scrapped, memo, created_by
    ) VALUES (
        p_employee_uuid, p_production_order_uuid, p_operation_execution_uuid,
        p_work_center_code, p_labour_category_uuid, p_work_date,
        p_start_time, p_end_time, COALESCE(p_break_minutes, 0), v_hours, p_activity_type,
        COALESCE(p_quantity_completed, 0), COALESCE(p_quantity_scrapped, 0),
        p_memo, p_created_by
    )
    RETURNING * INTO v_row;
    RETURN v_row;
END;
$$;

CREATE OR REPLACE FUNCTION manufacturing.draft_time_entry_from_operation(
    p_execution_uuid uuid,
    p_employee_uuid uuid DEFAULT NULL,
    p_created_by uuid DEFAULT NULL
) RETURNS manufacturing.labour_time_entries
LANGUAGE plpgsql
AS $$
DECLARE
    v_oe manufacturing.operation_executions%ROWTYPE;
    v_emp uuid;
    v_hours numeric(18, 4);
    v_existing manufacturing.labour_time_entries%ROWTYPE;
BEGIN
    SELECT * INTO STRICT v_oe
    FROM manufacturing.operation_executions
    WHERE uuid = p_execution_uuid;

    IF v_oe.status <> 'Completed' THEN
        RAISE EXCEPTION 'Operation must be Completed to draft time entry';
    END IF;

    SELECT * INTO v_existing
    FROM manufacturing.labour_time_entries
    WHERE operation_execution_uuid = p_execution_uuid
      AND status <> 'Rejected'
    ORDER BY created_at
    LIMIT 1;
    IF FOUND THEN
        RETURN v_existing;
    END IF;

    v_emp := COALESCE(p_employee_uuid, v_oe.operator_uuid);
    IF v_emp IS NULL THEN
        RAISE EXCEPTION 'employee_uuid or operation operator_uuid required';
    END IF;

    v_hours := (COALESCE(v_oe.setup_minutes_actual, 0) + COALESCE(v_oe.run_minutes_actual, 0)) / 60.0;
    IF v_hours <= 0 THEN
        RAISE EXCEPTION 'Operation has no actual setup/run minutes';
    END IF;

    RETURN manufacturing.create_labour_time_entry(
        v_emp,
        COALESCE(v_oe.completed_at::date, CURRENT_DATE),
        v_hours,
        'Direct',
        v_oe.production_order_uuid,
        v_oe.uuid,
        v_oe.work_center_code,
        NULL,
        v_oe.started_at,
        v_oe.completed_at,
        0,
        COALESCE(v_oe.qty_completed, 0),
        COALESCE(v_oe.qty_scrapped, 0),
        format('Auto from op %s', COALESCE(v_oe.operation_name, v_oe.uuid::text)),
        p_created_by
    );
END;
$$;

CREATE UNIQUE INDEX IF NOT EXISTS uq_lte_operation_active
    ON manufacturing.labour_time_entries (operation_execution_uuid)
    WHERE operation_execution_uuid IS NOT NULL AND status <> 'Rejected';

CREATE OR REPLACE VIEW manufacturing.v_production_order_labour AS
SELECT
    po.uuid AS production_order_uuid,
    po.order_number,
    po.status AS order_status,
    COALESCE(ca.applied_labour, 0) AS applied_labour,
    COALESCE(ca.actual_labour, 0) AS actual_labour,
    COALESCE(ca.indirect_labour, 0) AS indirect_labour,
    COALESCE(lte.actual_direct_hours, 0) AS actual_direct_hours,
    COALESCE(lte.open_timesheet_hours, 0) AS open_timesheet_hours,
    COALESCE(ca.actual_labour, 0) - COALESCE(ca.applied_labour, 0) AS labour_cost_variance,
    lv.standard_hours,
    lv.actual_hours AS variance_actual_hours,
    lv.standard_rate,
    lv.actual_rate,
    lv.rate_variance,
    lv.efficiency_variance,
    lv.total_variance AS last_computed_total_variance
FROM manufacturing.production_orders po
LEFT JOIN LATERAL (
    SELECT
        SUM(amount) FILTER (WHERE type = 'DirectLabor') AS applied_labour,
        SUM(amount) FILTER (WHERE type = 'ActualLabor') AS actual_labour,
        SUM(amount) FILTER (WHERE type = 'IndirectLabor') AS indirect_labour
    FROM manufacturing.cost_applications
    WHERE production_order_uuid = po.uuid
) ca ON true
LEFT JOIN LATERAL (
    SELECT
        SUM(hours) FILTER (
            WHERE status = 'Posted' AND activity_type IN ('Direct', 'Setup', 'Rework')
        ) AS actual_direct_hours,
        SUM(hours) FILTER (
            WHERE status IN ('Draft', 'Submitted', 'Approved')
        ) AS open_timesheet_hours
    FROM manufacturing.labour_time_entries
    WHERE production_order_uuid = po.uuid
) lte ON true
LEFT JOIN LATERAL (
    SELECT *
    FROM manufacturing.labour_variances
    WHERE production_order_uuid = po.uuid
    ORDER BY created_at DESC
    LIMIT 1
) lv ON true;

CREATE OR REPLACE FUNCTION manufacturing.allocate_payslip_labour_to_orders(
    p_payslip_uuid uuid,
    p_user uuid,
    p_wip_account text,
    p_labour_clearing_account text,
    p_cost_basis text DEFAULT 'employer_cost'
) RETURNS TABLE (
    production_order_uuid uuid,
    allocated_amount numeric,
    hours_share numeric,
    cost_application_uuid uuid
)
LANGUAGE plpgsql
AS $$
DECLARE
    v_ps payroll.payslips%ROWTYPE;
    v_pr payroll.payruns%ROWTYPE;
    v_total_hours numeric(18,4);
    v_pool numeric(18,2);
    rec RECORD;
    v_amt numeric(18,2);
    v_alloc manufacturing.labour_allocations%ROWTYPE;
    v_app manufacturing.cost_applications%ROWTYPE;
    v_applied numeric(18,2);
    v_existing_actual numeric(18,2);
    v_residual numeric(18,2);
BEGIN
    SELECT * INTO STRICT v_ps FROM payroll.payslips WHERE uuid = p_payslip_uuid;
    SELECT * INTO STRICT v_pr FROM payroll.payruns WHERE uuid = v_ps.payrun_uuid;

    v_pool := CASE lower(COALESCE(p_cost_basis, 'employer_cost'))
        WHEN 'basic' THEN COALESCE(v_ps.basic_pay, 0)
        WHEN 'gross' THEN COALESCE(v_ps.gross_pay, 0)
        ELSE COALESCE(NULLIF(v_ps.employer_cost_total, 0), v_ps.gross_pay, 0)
    END;

    IF v_pool IS NULL OR v_pool <= 0 THEN
        RAISE EXCEPTION 'Payslip % has no cost pool to allocate', p_payslip_uuid;
    END IF;

    SELECT COALESCE(SUM(hours), 0) INTO v_total_hours
    FROM manufacturing.labour_time_entries
    WHERE employee_uuid = v_ps.employee_uuid
      AND status = 'Posted'
      AND activity_type IN ('Direct', 'Setup', 'Rework')
      AND work_date BETWEEN v_pr.pay_period_start AND v_pr.pay_period_end
      AND production_order_uuid IS NOT NULL;

    IF v_total_hours <= 0 THEN
        RAISE EXCEPTION
            'No posted productive timesheet hours for employee % in % – %',
            v_ps.employee_uuid, v_pr.pay_period_start, v_pr.pay_period_end;
    END IF;

    FOR rec IN
        SELECT production_order_uuid, SUM(hours) AS hrs
        FROM manufacturing.labour_time_entries
        WHERE employee_uuid = v_ps.employee_uuid
          AND status = 'Posted'
          AND activity_type IN ('Direct', 'Setup', 'Rework')
          AND work_date BETWEEN v_pr.pay_period_start AND v_pr.pay_period_end
          AND production_order_uuid IS NOT NULL
        GROUP BY production_order_uuid
    LOOP
        v_amt := ROUND(v_pool * (rec.hrs / v_total_hours), 2);

        SELECT COALESCE(SUM(amount), 0) INTO v_existing_actual
        FROM manufacturing.cost_applications
        WHERE production_order_uuid = rec.production_order_uuid
          AND type = 'ActualLabor';

        IF v_existing_actual > 0 THEN
            CONTINUE; -- timesheet already supplied ActualLabor for this order
        END IF;

        SELECT COALESCE(SUM(amount), 0) INTO v_applied
        FROM manufacturing.cost_applications
        WHERE production_order_uuid = rec.production_order_uuid
          AND type = 'DirectLabor';

        v_residual := v_amt;
        IF v_applied > 0 THEN
            v_residual := GREATEST(v_amt - ROUND(v_applied * (rec.hrs / v_total_hours), 2), 0);
        END IF;
        IF v_residual <= 0 THEN
            CONTINUE;
        END IF;

        INSERT INTO manufacturing.labour_allocations (
            production_order_uuid, labour_class, hours, rate,
            employee_uuid, source, source_ref, memo, created_by, status
        ) VALUES (
            rec.production_order_uuid,
            'Direct',
            rec.hrs,
            CASE WHEN rec.hrs > 0 THEN ROUND(v_residual / rec.hrs, 4) ELSE 0 END,
            v_ps.employee_uuid,
            'Payroll',
            p_payslip_uuid::text,
            format('Payslip allocation %s basis=%s residual=%s', p_payslip_uuid, p_cost_basis, v_residual),
            p_user,
            'Draft'
        )
        RETURNING * INTO v_alloc;

        v_app := manufacturing.post_labour_allocation(
            v_alloc.uuid, p_user, p_wip_account, p_labour_clearing_account, NULL
        );

        production_order_uuid := rec.production_order_uuid;
        allocated_amount := v_app.amount;
        hours_share := rec.hrs;
        cost_application_uuid := v_app.uuid;
        RETURN NEXT;
    END LOOP;
END;
$$;
