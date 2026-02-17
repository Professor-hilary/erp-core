-- ========================================
-- REPORTING MODULE - FULL SCHEMA (GLOBAL-READY)
-- Run this ONCE after all core modules exist:
--   accounting, procurement, sales, inventory
-- ========================================

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

CREATE OR REPLACE FUNCTION manufacturing.get_cost_of_goods_manufactured(
    p_start_date date,
    p_end_date   date
)
RETURNS TABLE (
    description text,
    amount      numeric(18,2)
)
LANGUAGE sql
STABLE
AS $$
    WITH
    -- Beginning WIP balance (as of start_date)
    beg_wip AS (
        SELECT COALESCE(SUM(te.debit - te.credit), 0) AS beginning_wip
        FROM accounting.transaction_entries te
        JOIN accounting.transactions tx ON tx.uuid = te.transaction_uuid
        JOIN accounting.accounts a ON a.uuid = te.account_uuid
        WHERE a.code = '1310'  -- WIP account code; adjust
          AND tx.txn_date < p_start_date
    ),

    -- Direct materials used (issues to production)
    direct_materials_used AS (
        SELECT COALESCE(SUM(mi.total_cost), 0) AS amount
        FROM manufacturing.material_issues mi
        JOIN manufacturing.production_orders po ON po.uuid = mi.production_order_uuid
        WHERE po.actual_completion_date BETWEEN p_start_date AND p_end_date
          OR (po.actual_completion_date IS NULL AND po.start_date <= p_end_date)
    ),

    -- Direct labor applied
    direct_labor AS (
        SELECT COALESCE(SUM(ca.amount), 0) AS amount
        FROM manufacturing.cost_applications ca
        JOIN manufacturing.production_orders po ON po.uuid = ca.production_order_uuid
        WHERE ca.type = 'DirectLabor'
          AND (po.actual_completion_date BETWEEN p_start_date AND p_end_date
               OR (po.actual_completion_date IS NULL AND po.start_date <= p_end_date))
    ),

    -- Applied overhead
    applied_overhead AS (
        SELECT COALESCE(SUM(ca.amount), 0) AS amount
        FROM manufacturing.cost_applications ca
        JOIN manufacturing.production_orders po ON po.uuid = ca.production_order_uuid
        WHERE ca.type = 'Overhead'
          AND (po.actual_completion_date BETWEEN p_start_date AND p_end_date
               OR (po.actual_completion_date IS NULL AND po.start_date <= p_end_date))
    ),

    -- Total manufacturing costs
    total_manuf_costs AS (
        SELECT
            dm.amount + dl.amount + ao.amount AS total_manufacturing_costs
        FROM direct_materials_used dm, direct_labor dl, applied_overhead ao
    ),

    -- Ending WIP balance (as of end_date)
    end_wip AS (
        SELECT COALESCE(SUM(te.debit - te.credit), 0) AS ending_wip
        FROM accounting.transaction_entries te
        JOIN accounting.transactions tx ON tx.uuid = te.transaction_uuid
        JOIN accounting.accounts a ON a.uuid = te.account_uuid
        WHERE a.code = '1310'  -- WIP
          AND tx.txn_date <= p_end_date
    ),

    -- COGM calculation
    cogm_calc AS (
        SELECT
            bw.beginning_wip + tmc.total_manufacturing_costs - ew.ending_wip AS cogs_manufactured
        FROM beg_wip bw, total_manuf_costs tmc, end_wip ew
    )

    -- Formatted output (like a schedule)
    SELECT 'Direct Materials Used' AS description, dm.amount FROM direct_materials_used dm
    UNION ALL SELECT 'Direct Labor' , dl.amount FROM direct_labor dl
    UNION ALL SELECT 'Manufacturing Overhead Applied', ao.amount FROM applied_overhead ao
    UNION ALL SELECT 'Total Manufacturing Costs', tmc.total_manufacturing_costs FROM total_manuf_costs tmc
    UNION ALL SELECT 'Beginning WIP Inventory', bw.beginning_wip FROM beg_wip bw
    UNION ALL SELECT 'Less: Ending WIP Inventory', -ew.ending_wip FROM end_wip ew
    UNION ALL SELECT 'Cost of Goods Manufactured', cc.cogs_manufactured FROM cogm_calc cc;
$$;

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
) RETURNS void
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