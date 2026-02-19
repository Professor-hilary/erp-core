-- ==============================================================================
-- MANUFACTURING MODULE - FULL SCHEMA (GLOBAL-READY)
-- Run this ONCE after all core modules exist:
--   accounting, procurement, sales, inventory
-- ==============================================================================

-- Create schema
CREATE SCHEMA IF NOT EXISTS manufacturing;

-- Raw Materials, WIP, Finished Goods – separate asset accounts or sub-ledgers
-- (You can use accounting.accounts with codes like 1300-Raw, 1310-WIP, 1320-FG)

-- Production Orders / Jobs (header)
CREATE table if not exists manufacturing.production_orders (
    uuid uuid DEFAULT uuidv7() PRIMARY KEY,
    order_number text NOT NULL UNIQUE,
    product_item_id bigint NOT NULL,          -- references inventory.items
    quantity_ordered numeric(12,4) NOT NULL,
    quantity_completed numeric(12,4) DEFAULT 0,
    start_date date,
    expected_completion_date date,
    actual_completion_date date,
    status text DEFAULT 'Planned'
        CHECK (status IN ('Planned', 'In Progress', 'Completed', 'Cancelled')),
    gl_transaction_uuid uuid,                 -- link to posted entries
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now()
);

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

CREATE TABLE IF NOT EXISTS manufacturing.bom_headers (
    uuid            UUID PRIMARY KEY  DEFAULT uuidv7(),
    product_item_uuid UUID NOT NULL REFERENCES inventory.items(uuid),
    bom_code        text UNIQUE NOT NULL,
    description     text,
    revision        text DEFAULT 'A',
    is_active       boolean DEFAULT true,
    created_at      timestamptz DEFAULT now()
);

CREATE TABLE IF NOT EXISTS manufacturing.bom_lines (
    uuid            UUID PRIMARY KEY DEFAULT uuidv7(),
    bom_header_uuid UUID NOT NULL REFERENCES manufacturing.bom_headers(uuid) ON DELETE CASCADE,
    component_item_uuid UUID NOT NULL REFERENCES inventory.items(uuid),
    quantity_per    numeric(18, 6) NOT NULL CHECK (quantity_per > 0),
    unit_of_measure TEXT DEFAULT 'pcs',
    scrap_factor    numeric(5, 4) DEFAULT 1.0,  -- e.g. 1.05 = 5% scrap expected
    line_number     smallint NOT NULL,
    UNIQUE (bom_header_uuid, line_number)
);

-- Material Issues / Consumption (to WIP)
CREATE table if not exists manufacturing.material_issues (
    uuid uuid DEFAULT uuidv7() PRIMARY KEY,
    production_order_uuid uuid NOT NULL,
    stock_item_id bigint NOT NULL,
    quantity numeric(12,4) NOT NULL,
    unit_cost numeric(18,4) NOT NULL,         -- from costing method
    total_cost numeric(18,2) GENERATED ALWAYS AS (quantity * unit_cost) STORED,
    issued_at timestamptz DEFAULT now(),
    CONSTRAINT fk_material_issues_order FOREIGN KEY (production_order_uuid) REFERENCES manufacturing.production_orders(uuid)
);

-- Labor & Overhead Application (to WIP)
CREATE table if not exists manufacturing.cost_applications (
    uuid uuid DEFAULT uuidv7() PRIMARY KEY,
    production_order_uuid uuid NOT NULL,
    type text CHECK (type IN ('DirectLabor', 'Overhead')),
    amount numeric(18,2) NOT NULL,
    applied_at timestamptz DEFAULT now(),
    reference text,
    CONSTRAINT fk_applications_order FOREIGN KEY (production_order_uuid) REFERENCES manufacturing.production_orders(uuid)
);

-- Completions (WIP → Finished Goods)
CREATE TABLE if not exists manufacturing.completions (
    uuid uuid DEFAULT uuidv7() PRIMARY KEY,
    production_order_uuid uuid NOT NULL,
    quantity_completed numeric(12,4) NOT NULL,
    unit_cost numeric(18,4) NOT NULL,         -- calculated total cost / qty
    completed_at timestamptz DEFAULT now(),
    CONSTRAINT fk_completions_order FOREIGN KEY (production_order_uuid) REFERENCES manufacturing.production_orders(uuid)
);

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

-- Example insert (yearly rate)
-- INSERT INTO manufacturing.overhead_rates (period_start, period_end, allocation_base, estimated_overhead, estimated_base)
-- VALUES ('2026-01-01', '2026-12-31', 'DirectLaborHours', 1200000.00, 40000.00);  -- → rate = 30.00 per DLH

-- Helper Function: Apply Overhead to a Production Order
CREATE OR REPLACE FUNCTION manufacturing.apply_overhead_to_order(
    p_production_order_uuid uuid,
    p_activity_amount numeric,               -- e.g. actual direct labor hours used on this order
    p_user              uuid
) RETURNS numeric(18,2)     -- applied amount
LANGUAGE plpgsql
AS $$
DECLARE
    v_rate              numeric(18,6);
    v_applied_amount    numeric(18,2);
    v_wip_account_uuid  uuid;
    v_applied_account_uuid uuid;
    v_txn_uuid          uuid;
BEGIN
    -- Find current applicable rate (latest or matching period)
    SELECT rate INTO v_rate
    FROM manufacturing.overhead_rates
    WHERE CURRENT_DATE BETWEEN period_start AND period_end
      AND is_active
    ORDER BY period_start DESC
    LIMIT 1;

    IF v_rate IS NULL THEN
        RAISE EXCEPTION 'No active predetermined overhead rate found for current date';
    END IF;

    v_applied_amount := p_activity_amount * v_rate;

    -- Get account UUIDs (adjust codes if needed)
    SELECT uuid INTO v_wip_account_uuid     FROM accounting.accounts WHERE code = '1310';
    SELECT uuid INTO v_applied_account_uuid FROM accounting.accounts WHERE code = '5110';

    -- Create GL transaction (applied overhead)
    -- accounting.post_transaction(...) → your existing function; adapt parameters
    v_txn_uuid := accounting.post_transaction(
        'OVERHEAD-APPLY-' || p_production_order_uuid::text,
        'Applied Manufacturing Overhead',
        p_user,
        'manufacturing',
        CURRENT_DATE,
        '[]'::jsonb
    );

    -- Journal entry: Dr WIP, Cr Applied Overhead
    INSERT INTO accounting.transaction_entries (
        transaction_uuid, account_uuid, line_no, amount, debit, credit, memo
    ) VALUES
        (v_txn_uuid, v_wip_account_uuid,     1, v_applied_amount, v_applied_amount, 0, 'Applied overhead to production order'),
        (v_txn_uuid, v_applied_account_uuid, 2, v_applied_amount, 0, v_applied_amount, 'Applied overhead to production order');

    -- Record the application (for later reporting)
    INSERT INTO manufacturing.cost_applications (
        production_order_uuid, type, amount, reference
    ) VALUES (
        p_production_order_uuid, 'Overhead', v_applied_amount, 'Applied at rate ' || v_rate
    );

    -- Update order if needed (optional)
    UPDATE manufacturing.production_orders
    SET updated_at = now()
    WHERE uuid = p_production_order_uuid;

    RETURN v_applied_amount;
END;
$$;

-- Move accumulated cost from WIP to Finished Goods
CREATE OR REPLACE FUNCTION manufacturing.complete_production_order(
    p_order_uuid          uuid,
    p_completed_quantity  numeric(18,4),
    p_completion_date     date DEFAULT CURRENT_DATE,
    p_user                uuid
) RETURNS uuid   -- returns the GL transaction uuid
LANGUAGE plpgsql
AS $$
DECLARE
    v_order       manufacturing.production_orders%ROWTYPE;
    v_product     inventory.items%ROWTYPE;
    v_total_cost  numeric(18,2);
    v_unit_cost   numeric(18,4);
    v_fg_account  uuid;
    v_wip_account uuid;
    v_txn_uuid    uuid;
    v_movement_uuid uuid;
BEGIN
    SELECT * INTO v_order FROM manufacturing.production_orders WHERE uuid = p_order_uuid;
    IF NOT FOUND THEN RAISE EXCEPTION 'Production order not found'; END IF;

    IF v_order.status != 'In Progress' THEN
        RAISE EXCEPTION 'Order must be In Progress to complete';
    END IF;

    SELECT * INTO v_product
    FROM inventory.items WHERE uuid = v_order.product_item_id;

    -- Calculate total cost accumulated in WIP for this order
    -- (in real system you would sum from material_issues + cost_applications)
    -- Here we assume you have a helper or column — for demo we query GL
    SELECT COALESCE(SUM(te.debit - te.credit), 0) INTO v_total_cost
    FROM accounting.transaction_entries te
    JOIN accounting.transactions tx ON tx.uuid = te.transaction_uuid
    WHERE te.account_uuid = (SELECT uuid FROM accounting.accounts WHERE code = '1310')
      AND tx.memo LIKE '%' || v_order.order_number || '%';   -- rough filter - improve!

    IF v_total_cost <= 0 THEN
        RAISE EXCEPTION 'No costs accumulated in WIP for this order';
    END IF;

    v_unit_cost := v_total_cost / p_completed_quantity;

    -- Create GL transaction
    v_txn_uuid := accounting.post_transaction(
        v_order.order_number || '-COMPLETION',
        'Production Completion',
        p_user,
        'manufacturing_completion',
        p_completion_date,
        '[]'::jsonb
    );

    SELECT uuid INTO v_wip_account FROM accounting.accounts WHERE code = '1310';
    SELECT uuid INTO v_fg_account  FROM accounting.accounts WHERE code = v_product.asset_account;

    -- Dr FG Inventory, Cr WIP
    INSERT INTO accounting.transaction_entries (
        transaction_uuid, account_uuid, line_no, amount, debit, credit, memo
    ) VALUES
        (v_txn_uuid, v_fg_account,  1, v_total_cost, v_total_cost, 0, 'FG completion from production'),
        (v_txn_uuid, v_wip_account, 2, v_total_cost, 0, v_total_cost, 'WIP relieved on completion');

    -- Record inventory movement IN to FG
    INSERT INTO inventory.movements (
        item_uuid, warehouse_uuid, movement_type, reference_type, reference_id,
        quantity, unit_cost, direction, gl_transaction_uuid, movement_date
    ) VALUES (
        v_product.uuid,
        (SELECT uuid FROM inventory.warehouses WHERE serial_id = v_product.warehouse_serial),
        'PROD_COMPLETION',
        'production_order',
        v_order.serial_id,   -- assuming production_orders has serial_id
        p_completed_quantity,
        v_unit_cost,
        'IN',
        v_txn_uuid,
        p_completion_date
    ) RETURNING uuid INTO v_movement_uuid;

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
        VALUES (v_product.uuid, ..., p_completed_quantity, v_total_cost)
        ON CONFLICT ... DO UPDATE ...;
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

-- Manual Variance Proration (full version)
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

-- Period-End Overhead Adjustment Procedure (Over/Under Applied)
CREATE OR REPLACE PROCEDURE manufacturing.adjust_over_under_applied_overhead(
    p_period_start date,
    p_period_end   date,
    p_user         uuid
)
LANGUAGE plpgsql
AS $$
DECLARE
    v_actual_overhead   numeric(18,2);
    v_applied_overhead  numeric(18,2);
    v_variance          numeric(18,2);
    v_cogs_uuid         uuid;
    v_control_uuid      uuid;
    v_txn_uuid          uuid;
BEGIN
    -- Total actual overhead (debits to control account in period)
    SELECT COALESCE(SUM(te.debit - te.credit), 0) INTO v_actual_overhead
    FROM accounting.transaction_entries te
    JOIN accounting.transactions tx ON tx.uuid = te.transaction_uuid
    JOIN accounting.accounts a ON a.uuid = te.account_uuid
    WHERE a.code = '5100'  -- Manufacturing Overhead Control
      AND tx.transaction_date BETWEEN p_period_start AND p_period_end;

    -- Total applied overhead (credits to applied account in period)
    SELECT COALESCE(SUM(te.credit - te.debit), 0) INTO v_applied_overhead
    FROM accounting.transaction_entries te
    JOIN accounting.transactions tx ON tx.uuid = te.transaction_uuid
    JOIN accounting.accounts a ON a.uuid = te.account_uuid
    WHERE a.code = '5110'  -- Manufacturing Overhead Applied
      AND tx.transaction_date BETWEEN p_period_start AND p_period_end;

    v_variance := v_actual_overhead - v_applied_overhead;

    IF v_variance = 0 THEN
        RAISE NOTICE 'Overhead perfectly applied - no adjustment needed';
        RETURN;
    END IF;

    SELECT uuid INTO v_cogs_uuid   FROM accounting.accounts WHERE code = '5200'; -- COGS
    SELECT uuid INTO v_control_uuid FROM accounting.accounts WHERE code = '5100';

    v_txn_uuid := accounting.post_transaction(
        'OVERHEAD-ADJ-' || to_char(p_period_end, 'YYYYMM'),
        'Over/Under Applied Overhead Adjustment',
        p_user,
        'adjustment',
        p_period_end,
        '[]'::jsonb
    );

    -- Simple method: close variance to COGS (most common for small/medium businesses)
    IF v_variance > 0 THEN
        -- Under-applied → increase COGS
        INSERT INTO accounting.transaction_entries (transaction_uuid, account_uuid, line_no, amount, debit, credit, memo)
        VALUES
            (v_txn_uuid, v_cogs_uuid,   1, v_variance, v_variance, 0, 'Under-applied overhead adjustment'),
            (v_txn_uuid, v_control_uuid,2, v_variance, 0, v_variance, 'Under-applied overhead adjustment');
    ELSE
        -- Over-applied → reduce COGS
        INSERT INTO accounting.transaction_entries (transaction_uuid, account_uuid, line_no, amount, debit, credit, memo)
        VALUES
            (v_txn_uuid, v_cogs_uuid,   1, -v_variance, 0, -v_variance, 'Over-applied overhead adjustment'),
            (v_txn_uuid, v_control_uuid,2, -v_variance, -v_variance, 0, 'Over-applied overhead adjustment');
    END IF;

    -- Optional: clear applied account (or leave for audit trail)
    -- You can also prorate variance to WIP/FG/COGS if material – more accurate but complex
END;
$$;

-- View: Total inventory by stage
CREATE OR REPLACE VIEW inventory.manufacturing_inventory_summary AS
SELECT
    a.code,
    a.name,
    COALESCE(SUM(te.debit - te.credit), 0) AS current_balance
FROM accounting.accounts a
LEFT JOIN accounting.transaction_entries te ON te.account_uuid = a.uuid
WHERE a.code IN ('1300','1310','1320')
GROUP BY a.code, a.name;

-- View: Overhead variance quick check (for current open period)
-- CREATE OR REPLACE VIEW manufacturing.overhead_variance_current AS
-- SELECT
--     (SELECT COALESCE(SUM(debit - credit),0) FROM ... WHERE code='5100' AND tx.transaction_date >= date_trunc('month', CURRENT_DATE)) AS actual,
--     (SELECT COALESCE(SUM(credit - debit),0) FROM ... WHERE code='5110' AND tx.transaction_date >= date_trunc('month', CURRENT_DATE)) AS applied,
--     actual - applied AS variance;