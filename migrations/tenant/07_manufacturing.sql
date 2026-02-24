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
    uuid                uuid DEFAULT uuidv7() PRIMARY KEY,
    serial_id           bigint GENERATED ALWAYS AS IDENTITY UNIQUE,
    order_number        text NOT NULL UNIQUE,
    product_item_uuid   uuid NOT NULL,          -- references inventory.items
    quantity_ordered    numeric(12,4) NOT NULL,
    quantity_completed  numeric(12,4) DEFAULT 0,
    start_date          date,
    expected_completion_date date,
    material_usage_variance numeric(18,2) DEFAULT 0,
    labor_efficiency_variance numeric(18,2) DEFAULT 0,
    total_variance      numeric(18,2) GENERATED ALWAYS AS
        (material_usage_variance + labor_efficiency_variance) STORED,
    actual_completion_date date,
    status              text DEFAULT 'Planned' CHECK (status IN ('Planned', 'In Progress', 'Completed', 'Cancelled')),
    gl_transaction_uuid uuid,                 -- link to posted entries
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now()
);

CREATE TABLE IF NOT EXISTS manufacturing.cost_entries(
    uuid            uuid DEFAULT uuidv7() PRIMARY KEY,
    production_order_uuid   uuid NOT null references manufacturing.production_orders(uuid),
    cost_type       text NOT null check(cost_type in ('MATERIAL', 'LABOR', 'OVERHEAD', 'ADJUSTMENT')),
    component_item_uuid uuid NOT NULL REFERENCES inventory.items(uuid),
    quantity        numeric(18,4),
    unit_cost       numeric(18,4),
    total_cost      numeric(18,4) NOT null generated always as (quantity * unit_cost) STORED,
    created_at       timestamptz DEFAULT now(),
    CONSTRAINT fk_cost_entries_order
        FOREIGN KEY (production_order_uuid)
        REFERENCES manufacturing.production_orders(uuid)
);

CREATE INDEX idx_cost_entries_order ON manufacturing.cost_entries(production_order_uuid);

-- helper log table (recommended for audit trail)
CREATE TABLE IF NOT EXISTS accounting.variance_proration_logs (
    uuid              uuid          DEFAULT uuidv7() PRIMARY KEY,
    proration_date    date          NOT NULL,
    variance_amount   numeric(18,2) NOT NULL,
    wip_alloc         numeric(18,2),
    fg_alloc          numeric(18,2),
    cogs_alloc        numeric(18,2),
    gl_transaction_uuid uuid        REFERENCES accounting.transactions(uuid),
    memo              text,
    created_by        uuid,
    created_at        timestamptz   DEFAULT now()
);

-- ============================================================================
-- Bill of Materials (BOM) – Header & Lines
-- ============================================================================
CREATE TABLE IF NOT EXISTS manufacturing.bom_headers (
    uuid                uuid          DEFAULT uuidv7() PRIMARY KEY,
    serial_id           bigint        GENERATED ALWAYS AS IDENTITY UNIQUE,
    bom_code            text          NOT NULL UNIQUE,          -- e.g. BOM-FG-001
    product_item_uuid   uuid          NOT NULL
        REFERENCES inventory.items(uuid) ON DELETE RESTRICT,
    description         text,
    revision            text          DEFAULT 'A' NOT NULL,
    is_active           boolean       DEFAULT true,
    is_default          boolean       DEFAULT false,            -- can have multiple BOMs per product
    created_by          uuid,
    created_at          timestamptz   DEFAULT now(),
    updated_at          timestamptz   DEFAULT now(),
    CONSTRAINT bom_product_unique_active UNIQUE (product_item_uuid)
        DEFERRABLE INITIALLY DEFERRED   -- only one active/default per product (optional constraint)
);

CREATE TABLE IF NOT EXISTS manufacturing.bom_lines (
    uuid                uuid          DEFAULT uuidv7() PRIMARY KEY,
    bom_header_uuid     uuid          NOT NULL
        REFERENCES manufacturing.bom_headers(uuid) ON DELETE CASCADE,
    line_number         smallint      NOT NULL,
    component_item_uuid uuid          NOT NULL
        REFERENCES inventory.items(uuid) ON DELETE RESTRICT,
    quantity_per        numeric(18,6) NOT NULL CHECK (quantity_per > 0),
    uom                 text          DEFAULT 'pcs',
    scrap_factor        numeric(5,4)  DEFAULT 1.0000,           -- 1.05 = 5% expected scrap
    notes               text,
    created_at          timestamptz   DEFAULT now(),
    UNIQUE (bom_header_uuid, line_number),
    UNIQUE (bom_header_uuid, component_item_uuid)   -- no duplicate components
);

-- Optional: index for fast BOM explosion
CREATE INDEX idx_bom_lines_header ON manufacturing.bom_lines(bom_header_uuid);
CREATE INDEX idx_bom_lines_component ON manufacturing.bom_lines(component_item_uuid);

-- Material Issues / Consumption (to WIP)
CREATE table if not exists manufacturing.material_issues (
    uuid                uuid DEFAULT uuidv7() PRIMARY KEY,
    production_order_uuid   uuid NOT NULL,
    stock_item_id       bigint NOT NULL,
    quantity            numeric(12,4) NOT NULL,
    unit_cost           numeric(18,4) NOT NULL,         -- from costing method
    total_cost          numeric(18,2) GENERATED ALWAYS AS (quantity * unit_cost) STORED,
    warehouse_uuid      uuid REFERENCES inventory.warehouses(uuid),
    issued_at           timestamptz DEFAULT now(),
    CONSTRAINT fk_material_issues_order FOREIGN KEY (production_order_uuid)
        REFERENCES manufacturing.production_orders(uuid)
);

-- Labor & Overhead Application (to WIP)
CREATE table if not exists manufacturing.cost_applications (
    uuid uuid DEFAULT uuidv7() PRIMARY KEY,
    production_order_uuid uuid NOT NULL,
    type text CHECK (type IN ('DirectLabor', 'Overhead')),
    amount numeric(18,2) NOT NULL,
    applied_at timestamptz DEFAULT now(),
    reference text,
    CONSTRAINT fk_applications_order FOREIGN KEY (production_order_uuid)
        REFERENCES manufacturing.production_orders(uuid)
);

-- Completions (WIP → Finished Goods)
CREATE TABLE if not exists manufacturing.completions (
    uuid uuid DEFAULT uuidv7() PRIMARY KEY,
    production_order_uuid uuid NOT NULL,
    quantity_completed numeric(12,4) NOT NULL,
    unit_cost numeric(18,4) NOT NULL,         -- calculated total cost / qty
    completed_at timestamptz DEFAULT now(),
    CONSTRAINT fk_completions_order FOREIGN KEY (production_order_uuid)
        REFERENCES manufacturing.production_orders(uuid)
);

-- Predetermined Overhead Rate Table (new table)
-- Most manufacturing systems store the rate per period/division (e.g. per direct labor hour, machine hour, or direct labor cost).
CREATE TABLE IF NOT EXISTS manufacturing.overhead_rates (
    uuid              uuid DEFAULT uuidv7() PRIMARY KEY,
    period_start      date NOT NULL,
    period_end        date NOT NULL,
    allocation_base   text NOT NULL CHECK (allocation_base IN ('DirectLaborHours', 'DirectLaborCost', 'MachineHours', 'MaterialCost')),
    estimated_overhead numeric(18,2) NOT NULL,     -- budgeted total overhead for period
    estimated_base    numeric(18,4) NOT NULL,      -- budgeted activity level (hours, cost, etc.)
    rate              numeric(18,6) GENERATED ALWAYS AS (estimated_overhead / estimated_base) STORED,
    department_code   text,                        -- optional: if multiple production departments
    is_active         boolean DEFAULT true,
    created_at        timestamptz DEFAULT now(),
    updated_at        timestamptz DEFAULT now(),
    CONSTRAINT unique_period_base UNIQUE (period_start, allocation_base, department_code)
);

CREATE TABLE IF NOT EXISTS manufacturing.overhead_actuals (
    uuid            uuid    DEFAULT uuidv7() PRIMARY KEY,
    account_uuid    uuid    NOT NULL,
    amount          numeric(18,2) NOT NULL,
    incurred_at     timestamptz DEFAULT now(),
    reference       text
);

-- Routings / Operations (for labor & machine standards)
CREATE TABLE manufacturing.routings (
    uuid            uuid DEFAULT uuidv7() PRIMARY KEY,
    product_item_uuid uuid REFERENCES inventory.items(uuid),
    routing_code    text UNIQUE,
    description     text,
    is_active       boolean DEFAULT true,
    is_default      boolean DEFAULT false,
    created_at      timestamptz DEFAULT now()
);

CREATE TABLE manufacturing.routing_operations (
    uuid            uuid DEFAULT uuidv7() PRIMARY KEY,
    routing_uuid    uuid REFERENCES manufacturing.routings(uuid) ON DELETE CASCADE,
    seq             smallint NOT NULL,
    work_center_code text NOT NULL,
    description     text,
    standard_hours  numeric(12,4) NOT NULL,
    standard_rate   numeric(18,2) NOT NULL,
    allocation_base text CHECK(allocation_base IN ('DirectLaborHours', 'MachineHours')),
    department_code text,
    UNIQUE(routing_uuid, seq)
);

-- ============================================================================
-- FUNCTIONS
-- ============================================================================

CREATE OR REPLACE FUNCTION manufacturing.calculate_standard_cost(p_item_uuid uuid)
RETURNS numeric(18,4)
LANGUAGE plpgsql
AS $$
DECLARE
    v_std numeric(18,4) := 0;
BEGIN
    SELECT COALESCE(SUM(bl.quantity_per * i.standard_cost), 0)
    INTO v_std
    FROM manufacturing.bom_headers bh
    JOIN manufacturing.bom_lines bl ON bl.bom_header_uuid = bh.uuid
    JOIN inventory.items i ON i.uuid = bl.component_item_uuid
    WHERE bh.product_item_uuid = p_item_uuid
        AND bh.is_active AND bh.is_default;

    -- You can later add routing labor cost here
    RETURN COALESCE(v_std, 0);
END;
$$;

-- Helper Function: Apply Overhead to a Production Order
CREATE OR REPLACE FUNCTION manufacturing.apply_overhead_to_order(
    p_production_order_uuid uuid,
    p_activity_amount       numeric, -- e.g. actual direct labor hours used on this order
    p_wip_account           text,
    p_overhead_control_code text,
    p_user                  uuid
) RETURNS manufacturing.cost_applications -- return full row
LANGUAGE plpgsql
AS $$
DECLARE
    v_rate              numeric(18,6);
    v_applied_amount    numeric(18,2);
    v_wip_account_uuid  uuid;
    v_applied_account_uuid uuid;
    v_txn_uuid          uuid;
    v_application       manufacturing.cost_applications%ROWTYPE;
BEGIN
    -- Find current applicable rate (latest or matching period)
    SELECT rate INTO STRICT v_rate
    FROM manufacturing.overhead_rates
    WHERE CURRENT_DATE BETWEEN period_start AND period_end
      AND is_active
    ORDER BY period_start DESC, created_at DESC
    LIMIT 1;

    IF v_rate IS NULL THEN
        RAISE EXCEPTION 'No active predetermined overhead rate found for current date';
    END IF;

    v_applied_amount := ROUND(p_activity_amount * v_rate, 2);

    -- Get account UUIDs (adjust codes if needed)
    SELECT uuid INTO STRICT v_wip_account_uuid
        FROM accounting.accounts WHERE code = p_wip_account;

    SELECT uuid INTO STRICT v_applied_account_uuid
        FROM accounting.accounts WHERE code = p_overhead_control_code;

    -- Create GL transaction (applied overhead)
    -- accounting.post_transaction(...) → your existing function; adapt parameters
    v_txn_uuid := accounting.post_transaction(
        'OVERHEAD-APPLY-' || p_production_order_uuid::text,
        'Applied Manufacturing Overhead',
        p_user,
        'manufacturing',
        CURRENT_DATE,
        -- '[]'::jsonb
        jsonb_build_object(
            'production_order_uuid', p_production_order_uuid,
            'base_amount', p_activity_amount,
            'rate', v_rate
        )
    );

    -- Journal entry: Dr WIP, Cr Applied Overhead
    INSERT INTO accounting.transaction_entries (
        transaction_uuid, account_uuid, line_no, amount, debit, credit, memo
    ) VALUES (
        v_txn_uuid, v_wip_account_uuid,     1, v_applied_amount, v_applied_amount, 0,
        format('Applied overhead to production order %s - base %s x rate %s',
            p_production_order_uuid, p_activity_amount, v_rate)
    ), (
        v_txn_uuid, v_applied_account_uuid, 2, v_applied_amount, 0, v_applied_amount,
        format('Applied overhead to production order %s - base %s x rate %s',
            p_production_order_uuid, p_activity_amount, v_rate)
    );

    -- Record the application (for later reporting)
    INSERT INTO manufacturing.cost_applications (
        production_order_uuid, type, amount, reference, applied_at
    ) VALUES (
        p_production_order_uuid,
        'Overhead',
        v_applied_amount,
        format(
            'Applied at rate %s x base %s (txn %s)',
            v_rate, p_activity_amount, v_txn_uuid
        ),
        now()
    )
    RETURNING * INTO v_application;

    -- Update order if needed (optional)
    UPDATE manufacturing.production_orders
    SET updated_at = now()
    WHERE uuid = p_production_order_uuid;

    RETURN v_application;
END;
$$;

-- Single professional function for both direct & indirect labor
CREATE OR REPLACE FUNCTION manufacturing.apply_labor_cost(
    p_production_order_uuid uuid,
    p_hours              CREATE OR REPLACE FUNCTION manufacturing.apply_labor_cost(
    p_production_order_uuid uuid,
    p_hours                 numeric(12,4), -- e.g. actual direct labor hours used on this order
    p_rate_per_hour         numeric(18,2),
    p_is_direct             boolean, -- true = direct (to WIP), false = indirect (to overhead/expense)
    p_user                  uuid,
    p_labor_account_code    text,
    p_control_account_code  text DEFAULT NULL,
    p_wip_account_code      text DEFAULT NULL,
    p_departrment_code      text DEFAULT NULL, -- optional if department is tracked
    p_reference             text DEFAULT NULL
) RETURNS manufacturing.cost_applications -- return full row
LANGUAGE plpgsql
AS $$
DECLARE
    v_amount                numeric(18,2) := ROUND(p_hours * p_rate_per_hour, 2);
    v_wip_or_moh_ctrl_uuid  uuid;
    v_labor_account_uuid    uuid;
    v_txn_serial            bigint;
    v_application           manufacturing.cost_applications%ROWTYPE;
    v_type_text             text;
    v_memo_suffix           text;
BEGIN
    v_amount := ROUND(p_hours * p_rate_per_hour, 2);

    v_type_text := CASE WHEN p_is_direct THEN 'DirectLabor' ELSE 'IndirectLabor' END;
    v_memo_suffix := CASE WHEN p_is_direct THEN 'direct' ELSE 'indirect' END;

    -- Is this direct or indirect labor, later goes to manufacturing overhead ctrl
    IF p_is_direct THEN -- Debit directly into Work In Progress - its traceable
        SELECT uuid INTO STRICT v_wip_or_moh_ctrl_uuid
        FROM accounting.accounts WHERE code = p_wip_account_code;

        -- Valid WIP account needed for direct labor costing
        IF NOT FOUND OR p_wip_account_code IS NULL THEN
            RAISE EXCEPTION 'Valid Work In Progress Account required for direct labor costing';
        END IF;
    ELSE -- Debit into overhead control account - its untraceable
        SELECT uuid INTO STRICT v_wip_or_moh_ctrl_uuid
        FROM accounting.accounts WHERE code = p_control_account_code;

        -- Valid WOH account required for indirect labor costing
        IF NOT FOUND OR p_control_account_code IS NULL THEN
            RAISE EXCEPTION 'Valid Overhead Control Account required for indirect labor costing';
        END IF;
    END IF;

    -- Labor account, if direct: credit wages payable, if indirect: credit salaries payable
    -- Later at closing we'll generate a payroll to pay off the accrued salaries altogether
    SELECT uuid INTO STRICT v_labor_account_uuid FROM accounting.accounts
        WHERE code = p_labor_account_code;

    -- Post GL transaction labor accrued:
    --      direct costs:
    --          DR: WIP             CR: Salaries/Wages Payable
    --      indirect costs:
    --          DR: Overhead Ctrl   CR: Salaries/Wages Payable
    v_txn_serial := accounting.post_transaction(
        format('%s-LABOR-%s', v_type_text, p_production_order_uuid::text),
        format('%s Labor Applied', CASE WHEN p_is_direct THEN 'Direct' ELSE 'Indirect' END),
        p_user,
        'manufacturing',
        CURRENT_DATE,
        jsonb_build_array(
            jsonb_build_object(
                'account_ref', v_wip_or_moh_ctrl_uuid, 'debit', v_amount, 'credit', 0,
                'memo', format(
                    '%s labor on order %s: %s hrs x %s %s',  v_memo_suffix,
                    p_production_order_uuid,
                    to_char(p_hours, 'FM999999990.00'),
                    to_char(p_rate_per_hour, 'FM999999990.00'),
                    COALESCE(' - ' || p_reference, '')
                )
            ),
            jsonb_build_object(
                'account_ref', v_labor_account_uuid, 'debit', 0, 'credit', v_amount,
                'memo', format(
                    '%s labor on order %s: %s hrs x %s %s',  v_memo_suffix,
                    p_production_order_uuid,
                    to_char(p_hours, 'FM999999990.00'),
                    to_char(p_rate_per_hour, 'FM999999990.00'),
                    COALESCE(' - ' || p_reference, '')
                )
            )
        )
    );

    -- Record in cost_applications, TODO: Add GL uuid to entries for bigger relationships
    INSERT INTO manufacturing.cost_applications (
        production_order_uuid, type, amount, reference, applied_at
    ) VALUES (
        p_production_order_uuid, v_type_text, v_amount,
        format(
            '%s labor: %s hrs @ %s%s%s',
            CASE WHEN p_is_direct THEN 'Direct' ELSE 'Indirect' END,
            to_char(p_hours, 'FM999999990.00'), to_char(p_rate_per_hour, 'FM999999990.00'),
            COALESCE('- ' || p_reference, ''),
            CASE WHEN p_departrment_code IS NOT NULL THEN ' (' || p_departrment_code || ')' ELSE '' END
        ),
        now()
    )
    RETURNING * INTO v_application;

    UPDATE manufacturing.production_orders
    SET updated_at = now()
    WHERE uuid = p_production_order_uuid;

    RETURN v_application;
END;
$$;   numeric(12,4), -- e.g. actual direct labor hours used on this order
    p_rate_per_hour         numeric(18,2),
    p_is_direct             boolean, -- true = direct (to WIP), false = indirect (to overhead/expense)
    p_user                  uuid,
    p_labor_account_code    text,
    p_control_account_code  text DEFAULT NULL,
    p_wip_account_code      text DEFAULT NULL,
    p_departrment_code      text DEFAULT NULL, -- optional if department is tracked
    p_reference             text DEFAULT NULL
) RETURNS manufacturing.cost_applications -- return full row
LANGUAGE plpgsql
AS $$
DECLARE
    v_amount                numeric(18,2) := ROUND(p_hours * p_rate_per_hour, 2);
    v_wip_or_moh_ctrl_uuid  uuid;
    v_labor_account_uuid    uuid;
    v_txn_serial            bigint;
    v_application           manufacturing.cost_applications%ROWTYPE;
    v_type_text             text;
    v_memo_suffix           text;
BEGIN
    v_amount := ROUND(p_hours * p_rate_per_hour, 2);

    v_type_text := CASE WHEN p_is_direct THEN 'DirectLabor' ELSE 'IndirectLabor' END;
    v_memo_suffix := CASE WHEN p_is_direct THEN 'direct' ELSE 'indirect' END;

    -- Is this direct or indirect labor, later goes to manufacturing overhead ctrl
    IF p_is_direct THEN -- Debit directly into Work In Progress - its traceable
        SELECT uuid INTO STRICT v_wip_or_moh_ctrl_uuid
        FROM accounting.accounts WHERE code = p_wip_account_code;

        -- Valid WIP account needed for direct labor costing
        IF NOT FOUND OR p_wip_account_code IS NULL THEN
            RAISE EXCEPTION 'Valid Work In Progress Account required for direct labor costing';
        END IF;
    ELSE -- Debit into overhead control account - its untraceable
        SELECT uuid INTO STRICT v_wip_or_moh_ctrl_uuid
        FROM accounting.accounts WHERE code = p_control_account_code;

        -- Valid WOH account required for indirect labor costing
        IF NOT FOUND OR p_control_account_code IS NULL THEN
            RAISE EXCEPTION 'Valid Overhead Control Account required for indirect labor costing';
        END IF;
    END IF;

    -- Labor account, if direct: credit wages payable, if indirect: credit salaries payable
    -- Later at closing we'll generate a payroll to pay off the accrued salaries altogether
    SELECT uuid INTO STRICT v_labor_account_uuid FROM accounting.accounts
        WHERE code = p_labor_account_code;

    -- Post GL transaction labor accrued:
    --      direct costs:
    --          DR: WIP             CR: Salaries/Wages Payable
    --      indirect costs:
    --          DR: Overhead Ctrl   CR: Salaries/Wages Payable
    v_txn_serial := accounting.post_transaction(
        format('%s-LABOR-%s', v_type_text, p_production_order_uuid::text),
        format('%s Labor Applied', CASE WHEN p_is_direct THEN 'Direct' ELSE 'Indirect' END),
        p_user,
        'manufacturing',
        CURRENT_DATE,
        jsonb_build_array(
            jsonb_build_object(
                'account_ref', v_wip_or_moh_ctrl_uuid, 'debit', v_amount, 'credit', 0,
                'memo', format(
                    '%s labor on order %s: %s hrs x %s %s',  v_memo_suffix,
                    p_production_order_uuid,
                    to_char(p_hours, 'FM999999990.00'),
                    to_char(p_rate_per_hour, 'FM999999990.00'),
                    COALESCE(' - ' || p_reference, '')
                )
            ),
            jsonb_build_object(
                'account_ref', v_labor_account_uuid, 'debit', 0, 'credit', v_amount,
                'memo', format(
                    '%s labor on order %s: %s hrs x %s %s',  v_memo_suffix,
                    p_production_order_uuid,
                    to_char(p_hours, 'FM999999990.00'),
                    to_char(p_rate_per_hour, 'FM999999990.00'),
                    COALESCE(' - ' || p_reference, '')
                )
            )
        )
    );

    -- Record in cost_applications, TODO: Add GL uuid to entries for bigger relationships
    INSERT INTO manufacturing.cost_applications (
        production_order_uuid, type, amount, reference, applied_at
    ) VALUES (
        p_production_order_uuid, v_type_text, v_amount,
        format(
            '%s labor: %s hrs @ %s%s%s',
            CASE WHEN p_is_direct THEN 'Direct' ELSE 'Indirect' END,
            to_char(p_hours, 'FM999999990.00'), to_char(p_rate_per_hour, 'FM999999990.00'),
            COALESCE('- ' || p_reference, ''),
            CASE WHEN p_departrment_code IS NOT NULL THEN ' (' || p_departrment_code || ')' ELSE '' END
        ),
        now()
    )
    RETURNING * INTO v_application;

    UPDATE manufacturing.production_orders
    SET updated_at = now()
    WHERE uuid = p_production_order_uuid;

    RETURN v_application;
END;
$$;

-- Move accumulated cost from WIP to Finished Goods
CREATE OR REPLACE FUNCTION manufacturing.complete_production_order(
    p_order_uuid          uuid,
    p_completed_quantity  numeric(18,4),
    p_user                uuid,
    p_wip_account         text,
    p_completion_date     date DEFAULT CURRENT_DATE
) RETURNS uuid   -- returns the GL transaction uuid
LANGUAGE plpgsql
AS $$
DECLARE
    v_order             manufacturing.production_orders%ROWTYPE;
    v_product           inventory.items%ROWTYPE;
    -- v_total_cost        numeric(18,2);
    v_standard_total    numeric(18,2);
    v_unit_cost         numeric(18,4);
    v_fg_account        uuid;
    v_warehouse_account uuid;
    v_wip_account       uuid;
    v_txn_uuid          uuid;
    v_movement_uuid     uuid;
BEGIN
    SELECT * INTO v_order FROM manufacturing.production_orders WHERE uuid = p_order_uuid;
    IF NOT FOUND THEN RAISE EXCEPTION 'Production order not found'; END IF;

    IF v_order.status != 'In Progress' THEN
        RAISE EXCEPTION 'Order must be In Progress to complete';
    END IF;

    SELECT * INTO v_product
    FROM inventory.items WHERE uuid = v_order.product_item_uuid;

    -- Calculate total cost accumulated in WIP for this order
    -- SELECT
    --     COALESCE(SUM(mi.total_cost), 0) + COALESCE(SUM(ca.amount), 0)
    -- INTO v_total_cost FROM manufacturing.material_issues mi
    -- LEFT JOIN manufacturing.cost_applications ca ON ca.production_order_uuid = mi.production_order_uuid
    -- WHERE mi.production_order_uuid = p_order_uuid;

    -- IF v_total_cost <= 0 THEN
    --     RAISE EXCEPTION 'No costs accumulated in WIP for this order';
    -- END IF;

    v_standard_total := p_completed_quantity * v_product.standard_cost;

    IF v_standard_total IS NULL OR v_standard_total = 0 THEN
        RAISE EXCEPTION 'Standard cost not set on product';
    END IF;

    -- Get unit cost
    v_unit_cost := v_standard_total / p_completed_quantity;

    -- Compute material usage variance
    -- SELECT
    --     COALESCE(SUM(mi.quantity), 0) AS actual_qty,
    --     (SELECT SUM(quantity_per * v_order.quantity_ordered * (1 + scrap_factor - 1))
    --     FROM manufacturing.bom_lines bl
    --     WHERE bl.bom_header_uuid = v_bom_header) AS std_qty
    -- INTO v_actual_mat_qty, v_std_mat_qty
    -- FROM manufacturing.material_issues mi
    -- WHERE mi.production_order_uuid = p_order_uuid;

    -- RECORD VARIANCES BEFORE MOVING TO FG
    PERFORM manufacturing.record_standard_vs_actual_variance(p_order_uuid, p_user, p_completion_date);

    -- Create GL transaction
    v_txn_uuid := accounting.post_transaction(
        v_order.order_number || '-COMPLETION',
        'Production Completion',
        p_user,
        'manufacturing_completion',
        p_completion_date,
        '[]'::jsonb
    );

    -- Get uuid for posting transaction_entries
    SELECT uuid INTO v_wip_account FROM accounting.accounts WHERE code = p_wip_account;
    SELECT uuid INTO v_fg_account  FROM accounting.accounts WHERE code = v_product.asset_account;
    SELECT uuid INTO v_warehouse_account FROM inventory.warehouses WHERE serial_id = v_product.warehouse_serial;

    -- Dr FG Inventory, Cr WIP
    INSERT INTO accounting.transaction_entries (
        transaction_uuid, account_uuid, line_no, amount, debit, credit, memo
    ) VALUES
        (v_txn_uuid, v_fg_account,  1, v_standard_total, v_standard_total, 0, 'FG at standard cost'),
        (v_txn_uuid, v_wip_account, 2, v_standard_total, 0, v_standard_total, 'WIP relieved at standard');

    -- Record inventory movement IN to FG
    INSERT INTO inventory.movements (
        item_uuid, warehouse_uuid, movement_type, reference_type, reference_id,
        quantity, unit_cost, direction, gl_transaction_uuid, movement_date
    ) VALUES (
        v_product.uuid,
        (SELECT uuid FROM inventory.warehouses WHERE serial_id = v_product.warehouse_serial),
        'PROD_COMPLETION',
        'production_order',
        v_order.serial_id,
        p_completed_quantity,
        v_unit_cost,
        'IN',
        v_txn_uuid,
        p_completion_date
    ) RETURNING uuid INTO v_movement_uuid;

    -- Record completion (WIP -> FG)
    INSERT INTO manufacturing.completions (
        production_order_uuid, quantity_completed, unit_cost, completed_at
    ) VALUES (
        p_order_uuid, p_completed_quantity, v_unit_cost, p_completion_date
    );

    -- If FIFO/LIFO → create new lot for FG
    IF v_product.valuation_method IN ('FIFO','LIFO') THEN
        INSERT INTO inventory.lots (
            item_uuid, warehouse_uuid, reference_type, reference_id,
            received_date, original_quantity, original_cost_per_unit, remaining_quantity
        ) VALUES (
            v_product.uuid,
            (SELECT uuid FROM inventory.warehouses WHERE serial_id = v_product.warehouse_serial),
            'PROD_COMPLETION', v_movement_uuid,
            p_completion_date, p_completed_quantity, v_unit_cost, p_completed_quantity
        );
    ELSIF v_product.valuation_method = 'WAVG' THEN
        INSERT INTO inventory.wavg_warehouse (item_uuid, warehouse_uuid, total_quantity, total_cost)
        VALUES (v_product.uuid, v_warehouse_account, p_completed_quantity, v_standard_total)
        ON CONFLICT (item_uuid, warehouse_uuid) DO UPDATE SET
            total_quantity = inventory.wavg_warehouse.total_quantity + p_completed_quantity,
            total_cost     = inventory.wavg_warehouse.total_cost     + (p_completed_quantity * v_unit_cost),
            last_updated_at = now();
    END IF;

    -- Update order
    UPDATE manufacturing.production_orders
    SET quantity_completed = quantity_completed + p_completed_quantity,
        actual_completion_date = p_completion_date,
        status = CASE WHEN quantity_completed + p_completed_quantity >= quantity_ordered THEN 'Completed' ELSE 'In Progress' END,
        updated_at = now()
    WHERE uuid = p_order_uuid;

    RETURN v_txn_uuid;
END;
$$;

-- Prepare RM for production, deplete inventory, update materials issued table
CREATE OR REPLACE FUNCTION manufacturing.issue_material_to_order(
    p_item_serial_id    bigint, -- Serial of raw maaterial inventory item
    p_warehouse_serial  bigint, -- Serial of warehouse for the raw material
    p_quantity          numeric(18,4), -- Quantity to be issued for prod
    p_production_order_uuid uuid,
    p_user_uuid         uuid
) RETURNS manufacturing.material_issues -- Return actual issued material
LANGUAGE plpgsql
AS $$
DECLARE
    v_order         manufacturing.production_orders%ROWTYPE;
    v_item          inventory.items%ROWTYPE;
    v_warehouse     inventory.warehouses%ROWTYPE;
    v_total_cost    numeric(18,4);
    v_unit_cost     numeric(18,4);
    v_issue_row     manufacturing.material_issues%ROWTYPE;
    v_movement_uuid uuid;
BEGIN
    -- 1. Lock & validate production order
    SELECT * INTO STRICT v_order
    FROM manufacturing.production_orders
    WHERE uuid = p_production_order_uuid
    FOR UPDATE;

    IF v_order.status NOT IN ('Planned', 'In Progress') THEN
        RAISE EXCEPTION 'Production order must be Planned or In Progress to issue materials';
    END IF;

    -- 2. Validate item and warehouse
    SELECT * INTO STRICT v_item
    FROM inventory.items WHERE serial_id = p_item_serial_id;

    SELECT * INTO STRICT v_warehouse
    FROM inventory.warehouses WHERE serial_id = p_warehouse_serial;

    -- 3. Deplete inventory (call your existing function)
    -- deplete_inventory returns a total value of inventory
    SELECT * INTO v_total_cost
    FROM inventory.deplete_inventory(
        p_item_serial_id, p_warehouse_serial, p_quantity, 'PRODUCTION', v_order.serial_id,
        p_user_uuid, NULL, NULL
    );

    -- 4. Calculate unit cost safely
    v_unit_cost := CASE WHEN p_quantity > 0 THEN v_total_cost / p_quantity ELSE 0 END;

    -- 5. Insert into material_issues
    INSERT INTO manufacturing.material_issues(
        production_order_uuid, stock_item_id, warehouse_uuid, quantity, unit_cost, issued_at
    ) VALUES (
        p_production_order_uuid, v_item.serial_id, v_warehouse.uuid, p_quantity, v_unit_cost, now()
    ) RETURNING * INTO v_issue_row;

    -- 6. Update production order status
    UPDATE manufacturing.production_orders
    SET status = 'In Progress', updated_at = now()
    WHERE uuid = p_production_order_uuid;

    RETURN v_issue_row;
END;
$$;

-- Core function for recording variances in the production
CREATE OR REPLACE FUNCTION manufacturing.record_standard_vs_actual_variance(
    p_order_uuid uuid,
    p_user uuid,
    p_manufacturing_variance text,
    p_wip_account text,
    p_mfg_mat_usage_var text,
    p_mfg_lab_eff_var text,
    p_date date DEFAULT CURRENT_DATE
) RETURNS void
LANGUAGE plpgsql
AS $$
DECLARE
    v_order             manufacturing.production_orders%ROWTYPE;
    v_bom_uuid          uuid;
    v_std_material_cost numeric(18,2) := 0;
    v_act_material_cost numeric(18,2);
    v_material_variance numeric(18,2);
    v_std_labor_hours   numeric(18,2) := 0;
    v_act_labor_hours   numeric(18,2);
    v_labor_variance    numeric(18,2);
    v_variance_account  uuid;
    v_mat_var_uuid      uuid;
    v_wip_uuid          uuid;
    v_txn_uuid          uuid;
    v_lab_var_uuid      uuid;
BEGIN
    SELECT * INTO v_order FROM manufacturing.production_orders WHERE uuid = p_order_uuid;

    -- 1. Material usage Variance (standard from current BOM)
    SELECT uuid INTO v_bom_uuid
    FROM manufacturing.bom_headers
    WHERE product_item_uuid = v_order.product_item_uuid
        AND is_active AND is_default
    LIMIT 1;

    IF v_bom_uuid IS NOT NULL THEN
        SELECT COALESCE(SUM(bl.quantity_per * v_order.quantity_ordered* i.standard_cost), 0)
        INTO v_std_material_cost
        FROM manufacturing.bom_lines bl
        JOIN inventory.items i ON i.uuid = bl.component_item_uuid
        WHERE bl.bom_header_uuid = v_bom_uuid;

        SELECT COALESCE(SUM(total_cost), 0) INTO v_act_material_cost
        FROM manufacturing.material_issues
        WHERE production_order_uuid = p_order_uuid;

        v_material_variance := v_act_material_cost - v_std_material_cost;

        -- UPDATE manufacturing.production_orders
        -- SET material_usage_variance = v_material_variance
        -- WHERE uuid = p_order_uuid;
    END IF;

    -- 2. Labor Effeciency Variance (from routing, if innexistent, skip)
    SELECT COALESCE(SUM(ro.standard_hours * v_order.quantity_ordered::numeric), 0)
        INTO v_std_labor_hours
    FROM manufacturing.routings r
    JOIN manufacturing.routing_operations ro ON ro.routing_uuid = r.uuid
    WHERE r.product_item_uuid = v_order.product_item_uuid
        AND r.is_default AND r.is_active;

    SELECT COALESCE(SUM(CASE WHEN type = 'DirectLabor' THEN amount / NULLIF(rate_per_hour,0) ELSE 0 END), 0)
    -- Note: we can store rate in cost-applications; for now we approcimate with hours
    INTO v_act_labor_hours
    FROM manufacturing.cost_applications
    WHERE production_order_uuid = p_order_uuid AND type = 'DirectLabor';

    v_labor_variance := (v_act_labor_hours - v_std_labor_hours) *
                        (SELECT COALESCE(AVG(rate_per_hour), 0)
                        FROM manufacturing.cost_applications
                        WHERE production_order_uuid = p_order_uuid AND type = 'DirectLabor');

    -- Update order for reporting
    UPDATE manufacturing.production_orders
        SET material_usage_variance = v_material_variance,
            labor_efficiency_variance = v_labor_variance
    WHERE uuid = p_order_uuid;

    -- JOURNAL ENTRIES
    SELECT uuid INTO v_wip_uuid     FROM accounting.accounts WHERE code = p_wip_account;
    SELECT uuid INTO v_mat_var_uuid FROM accounting.accounts WHERE code = p_mfg_mat_usage_var;
    SELECT uuid INTO v_lab_var_uuid FROM accounting.accounts WHERE code = p_mfg_lab_eff_var;

    -- Material Variance Journal
    IF ABS(v_material_variance) > 0.01 THEN
        v_txn_uuid := accounting.post_transaction (
            v_order.order_number || '-MAT-VAR',
            'Material Usage Variance at Completion',
            p_user, 'manufacturing', p_date,
            jsonb_build_object('order_uuid', p_order_uuid, 'variance', v_material_variance)
        );

        INSERT INTO accounting.transaction_entries (
            transaction_uuid, account_uuid, line_no, amount, debit, credit, memo
        ) VALUES
            (
                v_txn_uuid, v_mat_var_uuid, 1, ABS(v_material_variance),
                CASE WHEN v_material_variance > 0 THEN ABS(v_material_variance) ELSE 0 END,
                CASE WHEN v_material_variance < 0 THEN ABS(v_material_variance) ELSE 0 END,
                format('Material usage variance on order %s', v_order.order_number)
            ),
            (
                v_txn_uuid, v_wip_uuid,     2, ABS(v_material_variance),
                CASE WHEN v_material_variance < 0 THEN ABS(v_material_variance) ELSE 0 END,
                CASE WHEN v_material_variance > 0 THEN ABS(v_material_variance) ELSE 0 END,
                format('Offset to WIP - order %s', v_order.order_number)
            );
	END IF;

     -- Labor Variance Journal
    IF ABS(v_labor_variance) > 0.01 THEN
        v_txn_uuid := accounting.post_transaction (
            v_order.order_number || '-LAB-VAR',
            'Labor Efficiency Variance at Completion',
            p_user, 'manufacturing', p_date,
            jsonb_build_object('order_uuid', p_order_uuid, 'variance', v_labor_variance)
        );

        INSERT INTO accounting.transaction_entries (
            transaction_uuid, account_uuid, line_no, amount, debit, credit, memo
        ) VALUES
            (
                v_txn_uuid, v_lab_var_uuid, 1, ABS(v_labor_variance),
                CASE WHEN v_labor_variance > 0 THEN ABS(v_labor_variance) ELSE 0 END,
                CASE WHEN v_labor_variance < 0 THEN ABS(v_labor_variance) ELSE 0 END,
                format('Labor efficiency variance on order %s', v_order.order_number)
            ),
            (
                v_txn_uuid, v_wip_uuid,     2, ABS(v_labor_variance),
                CASE WHEN v_labor_variance < 0 THEN ABS(v_labor_variance) ELSE 0 END,
                CASE WHEN v_labor_variance > 0 THEN ABS(v_labor_variance) ELSE 0 END,
                format('Offset to WIP - order %s', v_order.order_number)
            );
	END IF;
END;
$$;

-- Transfer raw materials to production process for new inventory
CREATE OR REPLACE FUNCTION manufacturing.initialize_material_cost(
    p_order_uuid uuid
) RETURNS void
LANGUAGE plpgsql
AS $$
DECLARE
    v_order manufacturing.production_orders%ROWTYPE;
    v_bom_header uuid;
    v_line RECORD;
BEGIN
    -- Fetch order
    SELECT * INTO v_order FROM manufacturing.production_orders
    WHERE uuid = p_order_uuid;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Production order not found: %', p_order_uuid;
    END IF;

    -- Get default BOM for product
    SELECT uuid INTO v_bom_header FROM manufacturing.bom_headers
    WHERE product_item_uuid = v_order.product_item_uuid AND is_active AND is_default LIMIT 1;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'No default BOM found for product %', v_order.product_item_uuid;
    END IF;

    -- Loop BOM lines
    FOR v_line IN
        SELECT bl.*, i.unit_cost AS current_unit_cost
        FROM manufacturing.bom_lines bl
        JOIN inventory.items i ON i.uuid = bl.component_item_uuid
        WHERE bl.bom_header_uuid = v_bom_header
    LOOP
        INSERT INTO manufacturing.cost_entries (
            production_order_uuid, component_item_uuid, quantity, unit_cost, cost_type
        ) VALUES (
            p_order_uuid, v_line.component_item_uuid,
            v_line.quantity_per * v_order.quantity_ordered, -- multiply by order qty
            v_line.current_unit_cost, 'MATERIAL'
        );
    END LOOP;
END;
$$;

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
    SELECT uuid INTO STRICT v_wip_account_uuid  FROM accounting.accounts WHERE code = p_wip_code;
    SELECT uuid INTO STRICT v_fg_account_uuid   FROM accounting.accounts WHERE code = p_fg_code;
    SELECT uuid INTO STRICT v_cogs_account_uuid FROM accounting.accounts WHERE code = p_cogs_code;

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
SELECT
    po.order_number,
    i.name as product_name,
    po.quantity_ordered,
    po.quantity_completed,
    po.status,
    -- Actual costs
    COALESCE(SUM(mi.total_cost), 0) as actual_material,
    COALESCE(SUM(CASE WHEN ca.type = 'DirectLabor' THEN ca.amount END), 0) as actual_labor,
    COALESCE(SUM(CASE WHEN ca.type = 'Overhead' THEN ca.amount END), 0) as applied_overhead,
    -- Variances
    po.material_usage_variance,
    po.labor_efficiency_variance,
    (po.material_usage_variance + po.labor_efficiency_variance) as total_variance,
    -- Unit cost
    (COALESCE(SUM(mi.total_cost),0) + COALESCE(SUM(ca.amount),0)) / NULLIF(po.quantity_completed,0)
            as actual_unit_cost,
    i.standard_cost as standard_unit_cost
FROM manufacturing.production_orders po
JOIN inventory.items i ON i.uuid = po.product_item_uuid
LEFT JOIN manufacturing.material_issues mi ON mi.production_order_uuid = po.uuid
LEFT JOIN manufacturing.cost_applications ca ON ca.production_order_uuid = po.uuid
GROUP BY po.uuid, i.uuid, po.material_usage_variance, po.labor_efficiency_variance;

-- Period-End Overhead Adjustment Procedure (Over/Under Applied) - Not Accurate, Proration WINS
-- CREATE OR REPLACE PROCEDURE manufacturing.adjust_over_under_applied_overhead(
--     p_period_start  date,
--     p_period_end    date,
--     p_cogs_code     text,
--     p_control_code  text,
--     p_control_applied  text,
--     p_user          uuid
-- )
-- LANGUAGE plpgsql
-- AS $$
-- DECLARE
--     v_actual_overhead   numeric(18,2);
--     v_applied_overhead  numeric(18,2);
--     v_variance          numeric(18,2);
--     v_cogs_uuid         uuid;
--     v_control_uuid      uuid;
--     v_txn_uuid          uuid;
-- BEGIN
--     -- Total actual overhead (debits to control account in period)
--     SELECT COALESCE(SUM(te.debit - te.credit), 0) INTO v_actual_overhead
--     FROM accounting.transaction_entries te
--     JOIN accounting.transactions tx ON tx.uuid = te.transaction_uuid
--     JOIN accounting.accounts a ON a.uuid = te.account_uuid
--     WHERE a.code = p_control_code  -- Manufacturing Overhead Control
--       AND tx.transaction_date BETWEEN p_period_start AND p_period_end;

--     -- Total applied overhead (credits to applied account in period)
--     SELECT COALESCE(SUM(te.credit - te.debit), 0) INTO v_applied_overhead
--     FROM accounting.transaction_entries te
--     JOIN accounting.transactions tx ON tx.uuid = te.transaction_uuid
--     JOIN accounting.accounts a ON a.uuid = te.account_uuid
--     WHERE a.code = p_control_applied  -- Manufacturing Overhead Applied
--       AND tx.transaction_date BETWEEN p_period_start AND p_period_end;

--     v_variance := v_actual_overhead - v_applied_overhead;

--     IF v_variance = 0 THEN
--         RAISE NOTICE 'Overhead perfectly applied - no adjustment needed';
--         RETURN;
--     END IF;

--     SELECT uuid INTO v_cogs_uuid   FROM accounting.accounts WHERE code = p_cogs_code; -- COGS
--     SELECT uuid INTO v_control_uuid FROM accounting.accounts WHERE code = p_control_code;

--     v_txn_uuid := accounting.post_transaction(
--         'OVERHEAD-ADJ-' || to_char(p_period_end, 'YYYYMM'),
--         'Over/Under Applied Overhead Adjustment',
--         p_user,
--         'adjustment',
--         p_period_end,
--         '[]'::jsonb
--     );

--     -- Simple method: close variance to COGS (most common for small/medium businesses)
--     IF v_variance > 0 THEN
--         -- Under-applied → increase COGS
--         INSERT INTO accounting.transaction_entries (transaction_uuid, account_uuid, line_no, amount, debit, credit, memo)
--         VALUES
--             (v_txn_uuid, v_cogs_uuid,   1, v_variance, v_variance, 0, 'Under-applied overhead adjustment'),
--             (v_txn_uuid, v_control_uuid,2, v_variance, 0, v_variance, 'Under-applied overhead adjustment');
--     ELSE
--         -- Over-applied → reduce COGS
--         INSERT INTO accounting.transaction_entries (transaction_uuid, account_uuid, line_no, amount, debit, credit, memo)
--         VALUES
--             (v_txn_uuid, v_cogs_uuid,   1, -v_variance, 0, -v_variance, 'Over-applied overhead adjustment'),
--             (v_txn_uuid, v_control_uuid,2, -v_variance, -v_variance, 0, 'Over-applied overhead adjustment');
--     END IF;

--     -- Optional: clear applied account (or leave for audit trail)
-- END;
-- $$;

-- Typical codes for manufacturing (adjust numbering to fit your CoA)
-- Raw Materials, WIP, Finished Goods → separate sub-accounts under Inventory (Asset)
-- Overhead control + applied → temporary / clearing accounts

-- INSERT INTO accounting.accounts (code, name, category, parent_code, normal_balance, is_contra)
-- VALUES
    -- Assets – Inventory sub-ledgers
    -- ('1300', 'Raw Materials Inventory',     'Asset', '1200', 'DR', false),   -- parent = main Inventory or Current Assets
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


/*
    NOTES:
        Three standard methods of overhead adjustments at the end of a financial period:
        1. Close Entire Variance to COGS
            All over or under applied overhead goes to Cost of Goods Sold:
            Fast, Simple, Common
        2. Prorate Across WIP, FG, and COGS
            Spread the variance accross WIP, FG and COGS Used When:
            inventory is material, desire for reasonable accuracy
        3. The Pure Method
            Recomputation using actual overhead rate, involves bactracking to get
            actual overhead and reapply corrected ones
*/