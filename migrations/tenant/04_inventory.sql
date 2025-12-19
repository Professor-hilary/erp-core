-- ========================================
-- INVENTORY MODULE - FULL SCHEMA
-- Run this ONCE in a fresh DB with `accounting` schema already present
-- ========================================

-- Create schema
CREATE SCHEMA IF NOT EXISTS inventory;

-- ========================================
-- SEQUENCES
-- ========================================
CREATE SEQUENCE IF NOT EXISTS inventory.warehouses_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS inventory.item_categories_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS inventory.items_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS inventory.movements_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS inventory.adjustments_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS inventory.item_valuation_serial_id_seq;

-- ========================================
-- TABLE: warehouses
-- ========================================
CREATE TABLE IF NOT EXISTS inventory.warehouses (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('inventory.warehouses_serial_id_seq') NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    location text,
    description text,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT warehouses_pkey PRIMARY KEY (uuid),
    CONSTRAINT warehouses_serial_id_key UNIQUE (serial_id),
    CONSTRAINT warehouses_code_key UNIQUE (code)
);

-- ========================================
-- TABLE: item_categories
-- ========================================
CREATE TABLE IF NOT EXISTS inventory.item_categories (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('inventory.item_categories_serial_id_seq') NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    description text,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT item_categories_pkey PRIMARY KEY (uuid),
    CONSTRAINT item_categories_serial_id_key UNIQUE (serial_id),
    CONSTRAINT item_categories_code_key UNIQUE (code)
);

-- ========================================
-- TABLE: items
-- ========================================
CREATE TABLE IF NOT EXISTS inventory.items (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('inventory.items_serial_id_seq') NOT NULL,
    sku text NOT NULL,
    name text NOT NULL,
    category_uuid uuid,
    description text,
    unit text DEFAULT 'pcs',
    cost_price numeric(18, 2) DEFAULT 0,
    selling_price numeric(18, 2) DEFAULT 0,
    track_quantity boolean DEFAULT true,
    quantity_on_hand numeric(18, 4) DEFAULT 0,
    reorder_level numeric(18, 4) DEFAULT 0,
    valuation_method TEXT DEFAULT 'FIFO' CHECK (valuation_method IN ('FIFO', 'LIFO', 'WAVG')),
    asset_account text,
    cogs_account text,
    income_account text,
    status text DEFAULT 'Active',
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    CONSTRAINT items_pkey PRIMARY KEY (uuid),
    CONSTRAINT items_serial_id_key UNIQUE (serial_id),
    CONSTRAINT items_sku_key UNIQUE (sku),
    CONSTRAINT items_category_uuid_fkey
        FOREIGN KEY (category_uuid) REFERENCES inventory.item_categories(uuid) ON DELETE SET NULL
);

-- ========================================
-- TABLE: movements
-- ========================================
CREATE TABLE IF NOT EXISTS inventory.movements (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('inventory.movements_serial_id_seq') NOT NULL,
    item_uuid uuid NOT NULL,
    warehouse_uuid uuid,
    movement_date timestamptz DEFAULT now(),
    reference_type TEXT,
    reference_id bigint,
    quantity numeric(18, 4) NOT NULL,
    unit_cost numeric(18, 4) DEFAULT 0,
    total_cost numeric(18, 2) GENERATED ALWAYS AS (quantity * unit_cost) STORED,
    direction text NOT NULL CHECK (direction IN ('IN', 'OUT')),
    gl_transaction_uuid uuid,
    created_at timestamptz DEFAULT now(),
    CONSTRAINT movements_pkey PRIMARY KEY (uuid),
    CONSTRAINT movements_serial_id_key UNIQUE (serial_id),
    CONSTRAINT movements_item_uuid_fkey
        FOREIGN KEY (item_uuid) REFERENCES inventory.items(uuid) ON DELETE RESTRICT,
    CONSTRAINT movements_warehouse_uuid_fkey
        FOREIGN KEY (warehouse_uuid) REFERENCES inventory.warehouses(uuid) ON DELETE RESTRICT,
    CONSTRAINT movements_gl_transaction_uuid_fkey
        FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid)
);

-- ========================================
-- TABLE: adjustments
-- ========================================
CREATE TABLE IF NOT EXISTS inventory.adjustments (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('inventory.adjustments_serial_id_seq') NOT NULL,
    adjustment_number text NOT NULL,
    adjustment_date date NOT NULL,
    warehouse_uuid uuid,
    item_uuid uuid,
    old_quantity numeric(18, 4) DEFAULT 0,
    new_quantity numeric(18, 4) NOT NULL,
    difference numeric(18, 4) GENERATED ALWAYS AS (new_quantity - old_quantity) STORED,
    reason text,
    posted boolean DEFAULT false,
    gl_transaction_uuid uuid,
    created_at timestamptz DEFAULT now(),
    CONSTRAINT adjustments_pkey PRIMARY KEY (uuid),
    CONSTRAINT adjustments_serial_id_key UNIQUE (serial_id),
    CONSTRAINT adjustments_adjustment_number_key UNIQUE (adjustment_number),
    CONSTRAINT adjustments_item_uuid_fkey
        FOREIGN KEY (item_uuid) REFERENCES inventory.items(uuid),
    CONSTRAINT adjustments_warehouse_uuid_fkey
        FOREIGN KEY (warehouse_uuid) REFERENCES inventory.warehouses(uuid),
    CONSTRAINT adjustments_gl_transaction_uuid_fkey
        FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid)
);

-- ========================================
-- TABLE: item_valuation
-- ========================================
CREATE TABLE IF NOT EXISTS inventory.item_valuation (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('inventory.item_valuation_serial_id_seq') NOT NULL,
    item_uuid uuid NOT NULL,
    movement_uuid uuid,
    valuation_date timestamptz DEFAULT now(),
    debit_account text,
    credit_account text,
    amount numeric(18, 2) NOT NULL,
    gl_transaction_uuid uuid,
    description text,
    CONSTRAINT item_valuation_pkey PRIMARY KEY (uuid),
    CONSTRAINT item_valuation_serial_id_key UNIQUE (serial_id),
    CONSTRAINT item_valuation_item_uuid_fkey
        FOREIGN KEY (item_uuid) REFERENCES inventory.items(uuid),
    CONSTRAINT item_valuation_movement_uuid_fkey
        FOREIGN KEY (movement_uuid) REFERENCES inventory.movements(uuid),
    CONSTRAINT item_valuation_gl_transaction_uuid_fkey
        FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid)
);

-- ========================================
-- TABLE: lots (for FIFO/LIFO cost layering)
-- ========================================
CREATE TABLE IF NOT EXISTS inventory.lots (
    uuid UUID DEFAULT uuidv7() PRIMARY KEY,
    serial_id BIGSERIAL NOT NULL UNIQUE,
    item_uuid UUID NOT NULL REFERENCES inventory.items(uuid) ON DELETE CASCADE,
    warehouse_uuid UUID REFERENCES inventory.warehouses(uuid) ON DELETE SET NULL,
    -- Source of the lot
    reference_type TEXT NOT NULL,           -- 'bill', 'purchase', 'adjustment', etc.
    reference_id BIGINT,                    -- e.g., bill_serial_id
    received_date DATE NOT NULL DEFAULT CURRENT_DATE,
    -- Original batch info
    original_quantity NUMERIC(18, 4) NOT NULL,
    original_cost_per_unit NUMERIC(18, 4) NOT NULL,
    original_total_cost NUMERIC(18, 2) GENERATED ALWAYS AS (original_quantity * original_cost_per_unit) STORED,
    -- Remaining
    remaining_quantity NUMERIC(18, 4) NOT NULL CHECK (remaining_quantity >= 0),
    remaining_total_cost NUMERIC(18, 2) GENERATED ALWAYS AS (remaining_quantity * original_cost_per_unit) STORED,
    -- Optional: expiry, batch number, etc.
    batch_number TEXT,
    expiry_date DATE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT lots_item_warehouse_ref UNIQUE (item_uuid, warehouse_uuid, reference_type, reference_id) -- prevent duplicates
);

CREATE TABLE IF NOT EXISTS inventory.wavg_history (
    item_uuid UUID PRIMARY KEY REFERENCES inventory.items(uuid) ON DELETE CASCADE,
    total_quantity NUMERIC(18, 4) NOT NULL DEFAULT 0 CHECK (total_quantity >= 0),
    total_cost NUMERIC(18, 2) NOT NULL DEFAULT 0 CHECK (total_cost >= 0),
    -- Current average cost (computed for convenience)
    current_avg_cost NUMERIC(18, 4) GENERATED ALWAYS AS (
        CASE WHEN total_quantity > 0 THEN total_cost / total_quantity ELSE 0 END
    ) STORED,
    last_updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS inventory.lot_movements (
    uuid UUID DEFAULT uuidv7() PRIMARY KEY,
    lot_uuid UUID NOT NULL REFERENCES inventory.lots(uuid) ON DELETE RESTRICT,
    movement_uuid UUID REFERENCES inventory.movements(uuid) ON DELETE CASCADE, -- links to OUT movement

    quantity_used NUMERIC(18, 4) NOT NULL,
    cost_per_unit NUMERIC(18, 4) NOT NULL,
    total_cost NUMERIC(18, 2) NOT NULL,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_lot_movements_lot ON inventory.lot_movements(lot_uuid);
CREATE INDEX IF NOT EXISTS idx_lot_movements_movement ON inventory.lot_movements(movement_uuid);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_lots_item ON inventory.lots(item_uuid);
CREATE INDEX IF NOT EXISTS idx_lots_item_received ON inventory.lots(item_uuid, received_date);
CREATE INDEX IF NOT EXISTS idx_lots_remaining ON inventory.lots(item_uuid) WHERE remaining_quantity > 0;

-- ========================================
-- VIEWS
-- ========================================
CREATE OR REPLACE VIEW inventory.item_summary AS
SELECT
    i.serial_id AS item_serial_id,
    i.sku,
    i.name AS item_name,
    c.name AS category_name,
    i.unit,
    i.quantity_on_hand,
    i.cost_price,
    (i.quantity_on_hand * i.cost_price) AS inventory_value,
    i.reorder_level,
    (i.quantity_on_hand <= i.reorder_level) AS needs_reorder
FROM inventory.items i
LEFT JOIN inventory.item_categories c ON c.uuid = i.category_uuid
ORDER BY i.sku;

CREATE OR REPLACE VIEW inventory.movement_history AS
SELECT
    m.serial_id AS movement_serial_id,
    i.sku,
    i.name AS item_name,
    w.name AS warehouse_name,
    m.movement_date,
    m.reference_type,
    m.reference_id AS reference_serial_id,
    m.direction,
    m.quantity,
    m.unit_cost,
    m.total_cost,
    t.txn_date AS gl_posted_date,
    m.gl_transaction_uuid
FROM inventory.movements m
JOIN inventory.items i ON i.uuid = m.item_uuid
LEFT JOIN inventory.warehouses w ON w.uuid = m.warehouse_uuid
LEFT JOIN accounting.transactions t ON t.uuid = m.gl_transaction_uuid
ORDER BY m.movement_date DESC;

/*
CREATE OR REPLACE VIEW inventory.item_current_cost AS
SELECT
    i.uuid,
    i.serial_id,
    i.sku,
    i.name,
    i.valuation_method,
    COALESCE(
        -- For WAVG items: use the running average
        w.current_avg_cost,
        -- For FIFO/LIFO: weighted average of remaining lots
        (SELECT SUM(l.remaining_total_cost) / NULLIF(SUM(l.remaining_quantity), 0)
         FROM inventory.lots l WHERE l.item_uuid = i.uuid AND l.remaining_quantity > 0),
        0
    ) AS current_cost_price,
    i.quantity_on_hand,
    (i.quantity_on_hand * COALESCE(
        (SELECT SUM(remaining_total_cost) / NULLIF(SUM(remaining_quantity), 0)
            FROM inventory.lots
            WHERE item_uuid = items.uuid AND remaining_quantity > 0
        ), 0)) AS current_inventory_value
FROM inventory.items i
LEFT JOIN inventory.wavg_history w ON w.item_uuid = i.uuid;
*/
-- ========================================
-- FUNCTIONS
-- ========================================

CREATE OR REPLACE FUNCTION inventory.deplete_inventory(
    p_item_serial_id BIGINT,
    p_quantity_needed NUMERIC(18,4),
    p_reference_type TEXT,
    p_reference_serial_id BIGINT,
    p_user UUID,
    p_gl_transaction_uuid UUID DEFAULT NULL
) RETURNS NUMERIC(18,2)  -- returns total COGS
LANGUAGE plpgsql AS $$
DECLARE
    v_item inventory.items%ROWTYPE;
    v_cogs NUMERIC(18,2) := 0;
    v_lot RECORD;
    v_remaining NUMERIC(18,4) := p_quantity_needed;
    v_order TEXT;
    v_avg_cost NUMERIC(18,4);
BEGIN
    SELECT * INTO v_item FROM inventory.items WHERE serial_id = p_item_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Item not found'; END IF;

    IF v_item.quantity_on_hand < p_quantity_needed THEN
        RAISE EXCEPTION 'Insufficient stock: have %, need %', v_item.quantity_on_hand, p_quantity_needed;
    END IF;

    -- Branch by valuation method
    IF v_item.valuation_method = 'WAVG' THEN
        -- Get current moving average
        SELECT current_avg_cost INTO v_avg_cost
        FROM inventory.wavg_history
        WHERE item_uuid = v_item.uuid;

        IF v_avg_cost IS NULL THEN
            RAISE EXCEPTION 'No cost history for WAVG item %', p_item_serial_id;
        END IF;

        v_cogs := p_quantity_needed * v_avg_cost;

        -- Update running totals (reduce quantity and cost)
        UPDATE inventory.wavg_history
        SET
            total_quantity = total_quantity - p_quantity_needed,
            total_cost = total_cost - v_cogs,
            last_updated_at = NOW()
        WHERE item_uuid = v_item.uuid;

    ELSIF v_item.valuation_method IN ('FIFO', 'LIFO') THEN
        v_order := CASE WHEN v_item.valuation_method = 'LIFO' THEN 'DESC' ELSE 'ASC' END;

        PERFORM 1 FROM inventory.lots WHERE item_uuid = v_item.uuid FOR UPDATE;

        FOR v_lot IN
            SELECT * FROM inventory.lots
            WHERE item_uuid = v_item.uuid AND remaining_quantity > 0
            ORDER BY received_date || v_order
        LOOP
            EXIT WHEN v_remaining <= 0;

            IF v_lot.remaining_quantity >= v_remaining THEN
                v_cogs := v_cogs + (v_remaining * v_lot.original_cost_per_unit);

                INSERT INTO inventory.lot_movements (
                    lot_uuid, quantity_used, cost_per_unit, total_cost
                ) VALUES (
                    v_lot.uuid, v_remaining, v_lot.original_cost_per_unit,
                    v_remaining * v_lot.original_cost_per_unit
                );

                UPDATE inventory.lots
                SET remaining_quantity = remaining_quantity - v_remaining
                WHERE uuid = v_lot.uuid;

                v_remaining := 0;
            ELSE
                v_cogs := v_cogs + v_lot.remaining_total_cost;

                INSERT INTO inventory.lot_movements (
                    lot_uuid, quantity_used, cost_per_unit, total_cost
                ) VALUES (
                    v_lot.uuid, v_lot.remaining_quantity, v_lot.original_cost_per_unit,
                    v_lot.remaining_total_cost
                );

                UPDATE inventory.lots
                SET remaining_quantity = 0
                WHERE uuid = v_lot.uuid;

                v_remaining := v_remaining - v_lot.remaining_quantity;
            END IF;
        END LOOP;

        IF v_remaining > 0 THEN
            RAISE EXCEPTION 'Inventory layer exhaustion error';
        END IF;

    END IF;

    -- Record OUT movement
    INSERT INTO inventory.movements (
        item_uuid, reference_type, reference_id,
        quantity, unit_cost, direction, gl_transaction_uuid
    ) VALUES (
        v_item.uuid, p_reference_type, p_reference_serial_id,
        p_quantity_needed,
        v_cogs / NULLIF(p_quantity_needed, 0),  -- average unit cost for this depletion
        'OUT',
        p_gl_transaction_uuid
    );

    -- Update quantity on hand
    UPDATE inventory.items
    SET quantity_on_hand = quantity_on_hand - p_quantity_needed
    WHERE uuid = v_item.uuid;

    RETURN v_cogs;
END;
$$;

-- CREATE OR REPLACE FUNCTION inventory.post_sale(
    -- p_item_serial_id bigint,
    -- p_warehouse_serial_id bigint,
    -- p_quantity numeric,
    -- p_unit_cost numeric,
    -- p_reference_type text,
    -- p_reference_serial_id bigint,
    -- p_user uuid,
    -- p_cash_code text DEFAULT NULL,
    -- p_gl_transaction_uuid uuid DEFAULT NULL
-- ) RETURNS uuid
-- LANGUAGE plpgsql AS $$
-- DECLARE
    -- v_item inventory.items%ROWTYPE;
    -- v_warehouse_uuid uuid;
    -- v_movement_uuid uuid;
    -- v_txn_uuid uuid;
    -- v_txn_serial_id bigint;
    -- v_total numeric(18,2);
    -- v_lines jsonb;
-- BEGIN
    -- SELECT * INTO v_item
    -- FROM inventory.items
    -- WHERE serial_id = p_item_serial_id;
--
    -- IF NOT FOUND THEN
        -- RAISE EXCEPTION 'Item % not found', p_item_serial_id;
    -- END IF;
--
    -- IF v_item.quantity_on_hand < p_quantity THEN
        -- RAISE EXCEPTION 'Insufficient stock';
    -- END IF;
--
    -- IF p_warehouse_serial_id IS NOT NULL THEN
        -- SELECT uuid INTO v_warehouse_uuid
        -- FROM inventory.warehouses
        -- WHERE serial_id = p_warehouse_serial_id;
    -- END IF;
--
    -- v_total := p_quantity * p_unit_cost;
--
    -- INSERT INTO inventory.movements (
        -- item_uuid, warehouse_uuid,
        -- reference_type, reference_id,
        -- quantity, unit_cost, direction
    -- ) VALUES (
        -- v_item.uuid, v_warehouse_uuid,
        -- p_reference_type, p_reference_serial_id,
        -- p_quantity, p_unit_cost, 'OUT'
    -- )
    -- RETURNING uuid INTO v_movement_uuid;
--
    -- GL handling
    -- IF p_gl_transaction_uuid IS NULL THEN
        -- Cash sale
        -- v_lines := jsonb_build_array(
            -- jsonb_build_object(
                -- 'account_ref', v_item.cogs_account,
                -- 'debit', v_total, 'credit', 0,
                -- 'memo', 'COGS'
            -- ),
            -- jsonb_build_object(
                -- 'account_ref', v_item.asset_account,
                -- 'debit', 0, 'credit', v_total,
                -- 'memo', 'Inventory Asset'
            -- ),
            -- jsonb_build_object(
                -- 'account_ref', p_cash_code,
                -- 'debit', v_total, 'credit', 0,
                -- 'memo', 'Cash Received'
            -- ),
            -- jsonb_build_object(
                -- 'account_ref', v_item.income_account,
                -- 'debit', 0, 'credit', v_total,
                -- 'memo', 'Revenue received'
            -- )
        -- );
--
        -- v_txn_serial_id := accounting.post_transaction(
            -- now()::date,
            -- 'INV-SALE-' || p_reference_serial_id,
            -- 'Inventory Sale',
            -- p_user,
            -- 'inventory_sale',
            -- v_lines
        -- );
--
        -- SELECT uuid INTO v_txn_uuid
        -- FROM accounting.transactions
        -- WHERE serial_id = v_txn_serial_id;
    -- ELSE
        -- Credit sale (invoice already posted)
        -- v_txn_uuid := p_gl_transaction_uuid;
--
        -- v_lines := jsonb_build_array(
            -- jsonb_build_object(
                -- 'account_ref', v_item.cogs_account,
                -- 'debit', v_total, 'credit', 0,
                -- 'memo', 'COGS'
            -- ),
            -- jsonb_build_object(
                -- 'account_ref', v_item.asset_account,
                -- 'debit', 0, 'credit', v_total,
                -- 'memo', 'Inventory Adjustment'
            -- )
        -- );
--
        -- PERFORM accounting.post_transaction(
            -- now()::date,
            -- 'INV-COGS-' || p_reference_serial_id,
            -- 'COGS Recognition',
            -- p_user,
            -- 'cogs',
            -- v_lines
        -- );
    -- END IF;
--
    -- UPDATE inventory.movements
    -- SET gl_transaction_uuid = v_txn_uuid
    -- WHERE uuid = v_movement_uuid;
--
    -- UPDATE inventory.items
    -- SET quantity_on_hand = quantity_on_hand - p_quantity,
        -- updated_at = now()
    -- WHERE uuid = v_item.uuid;
--
    -- RETURN v_txn_uuid;
-- END;
-- $$;

CREATE OR REPLACE FUNCTION inventory.post_sale(
    p_item_serial_id BIGINT,
    p_warehouse_serial_id BIGINT,
    p_quantity NUMERIC,
    p_selling_price NUMERIC,
    p_reference_type TEXT, -- 'invoice', 'cash-sale', 'mobile-money', 'virtual-card'
    p_reference_serial_id BIGINT,
    p_user UUID,
    p_cash_code TEXT DEFAULT NULL, -- Provided only for cash sales
    p_gl_transaction_uuid UUID DEFAULT NULL -- Provided for credit sales
) RETURNS UUID
LANGUAGE plpgsql AS $$
DECLARE
    v_item inventory.items%ROWTYPE;
    v_total_revenue NUMERIC(18,2) := p_quantity * p_selling_price;
    v_cogs NUMERIC(18,2);
BEGIN
    SELECT * INTO v_item FROM inventory.items WHERE serial_id = p_item_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Item % not found', p_item_serial_id; END IF;

    -- Calculate and record COGS using correct valuation method
    v_cogs := inventory.deplete_inventory(
        p_item_serial_id, p_quantity,
        p_reference_type, p_reference_serial_id, p_user, p_gl_transaction_uuid
    );

    -- Always post COGS entry (separate from revenue)
    PERFORM accounting.post_transaction(
        CURRENT_DATE,
        'COGS-' || p_reference_serial_id,
        'Cost of Goods Sold',
        p_user,
        'cogs',
        jsonb_build_array(
            jsonb_build_object('account_ref', v_item.cogs_account, 'debit', v_cogs, 'credit', 0),
            jsonb_build_object('account_ref', v_item.asset_account, 'debit', 0, 'credit', v_cogs)
        )
    );

    -- Cash sale revenue (if not credit sale)
    IF p_cash_code IS NOT NULL AND p_gl_transaction_uuid IS NULL THEN
        PERFORM accounting.post_transaction(
            CURRENT_DATE,
            'SALE-' || p_reference_serial_id,
            'Cash Sale',
            p_user,
            'sale',
            jsonb_build_array(
                jsonb_build_object('account_ref', p_cash_code, 'debit', v_total_revenue, 'credit', 0),
                jsonb_build_object('account_ref', v_item.income_account, 'debit', 0, 'credit', v_total_revenue)
            )
        );
    END IF;

    -- For credit sales: Revenue/AR already posted by receivables.post_invoice()

    RETURN p_gl_transaction_uuid;
END;
$$;

-- Inventory post cash or credit purchase function
CREATE OR REPLACE FUNCTION inventory.post_purchase(
    p_item_serial_id BIGINT,
    p_warehouse_serial_id BIGINT,
    p_quantity NUMERIC,
    p_unit_cost NUMERIC,
    p_reference_type TEXT,          -- 'bill', 'cash_purchase', 'adjustment', etc.
    p_reference_serial_id BIGINT,
    p_user UUID,
    p_source_account TEXT,          -- Cash/Bank account for cash purchases
    p_gl_transaction_uuid UUID DEFAULT NULL -- Provided for credit purchases (bill)
) RETURNS UUID
LANGUAGE plpgsql AS $$
DECLARE
    v_item inventory.items%ROWTYPE;
    v_warehouse_uuid UUID;
    v_lot_uuid UUID;
    v_movement_uuid UUID;
    v_txn_uuid UUID := p_gl_transaction_uuid;
    v_total NUMERIC(18,2) := p_quantity * p_unit_cost;
BEGIN
    SELECT * INTO v_item FROM inventory.items WHERE serial_id = p_item_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Item not found'; END IF;

    IF p_warehouse_serial_id IS NOT NULL THEN
        SELECT uuid INTO v_warehouse_uuid
        FROM inventory.warehouses WHERE serial_id = p_warehouse_serial_id;
    END IF;

    -- Record movement IN (common to all methods)
    INSERT INTO inventory.movements (
        item_uuid, warehouse_uuid,
        reference_type, reference_id,
        quantity, unit_cost, direction,
        gl_transaction_uuid
    ) VALUES (
        v_item.uuid, v_warehouse_uuid,
        p_reference_type, p_reference_serial_id,
        p_quantity, p_unit_cost, 'IN',
        v_txn_uuid
    ) RETURNING uuid INTO v_movement_uuid;

    -- Valuation method branching
    IF v_item.valuation_method IN ('FIFO', 'LIFO') THEN
        -- Create new lot
        INSERT INTO inventory.lots (
            item_uuid, warehouse_uuid,
            reference_type, reference_id,
            received_date,
            original_quantity, original_cost_per_unit,
            remaining_quantity
        ) VALUES (
            v_item.uuid, v_warehouse_uuid,
            p_reference_type, p_reference_serial_id,
            CURRENT_DATE,
            p_quantity, p_unit_cost,
            p_quantity
        );

    ELSIF v_item.valuation_method = 'WAVG' THEN
        -- Update or insert running average
        INSERT INTO inventory.wavg_history (item_uuid, total_quantity, total_cost)
        VALUES (v_item.uuid, p_quantity, v_total)
        ON CONFLICT (item_uuid) DO UPDATE
        SET
            total_quantity = wavg_history.total_quantity + p_quantity,
            total_cost = wavg_history.total_cost + v_total,
            last_updated_at = NOW();

    ELSE
        RAISE EXCEPTION 'Unsupported valuation method: %', v_item.valuation_method;
    END IF;

    -- GL posting (only if not from bill)
    IF p_gl_transaction_uuid IS NULL THEN
        PERFORM accounting.post_transaction(
            CURRENT_DATE,
            'INV-PUR-' || p_reference_serial_id,
            'Inventory Purchase',
            p_user,
            'inventory_purchase',
            jsonb_build_array(
                jsonb_build_object('account_ref', v_item.asset_account, 'debit', v_total, 'credit', 0),
                jsonb_build_object('account_ref', p_source_account, 'debit', 0, 'credit', v_total)
            )
        );
    END IF;

    -- Update total quantity on hand (common)
    UPDATE inventory.items
    SET quantity_on_hand = quantity_on_hand + p_quantity,
        updated_at = NOW()
    WHERE uuid = v_item.uuid;

    RETURN v_txn_uuid;
END;
$$;

-- CREATE OR REPLACE FUNCTION inventory.post_purchase(
    -- p_item_serial_id bigint,
    -- p_warehouse_serial_id bigint,
    -- p_quantity numeric,
    -- p_unit_cost numeric,
    -- p_reference_type text,
    -- p_reference_serial_id bigint,
    -- p_user uuid,
    -- p_source_account text,
    -- p_gl_transaction_uuid uuid DEFAULT NULL
-- ) RETURNS uuid
-- LANGUAGE plpgsql
-- AS $$
-- DECLARE
    -- v_item inventory.items%ROWTYPE;
    -- v_warehouse_uuid uuid;
    -- v_movement_uuid uuid;
    -- v_exp_account_uuid uuid;
    -- v_payable_account_uuid uuid;
    -- v_txn_uuid uuid;
    -- v_txn_serial_id bigint;
    -- v_total numeric(18,2);
    -- v_lines jsonb;
-- BEGIN
    -- SELECT * INTO v_item FROM inventory.items WHERE serial_id = p_item_serial_id;
    -- IF NOT FOUND THEN RAISE EXCEPTION 'Item not found'; END IF;
--
    -- IF p_warehouse_serial_id IS NOT NULL THEN
        -- SELECT uuid INTO v_warehouse_uuid
        -- FROM inventory.warehouses WHERE serial_id = p_warehouse_serial_id;
    -- END IF;
--
    -- v_total := p_quantity * p_unit_cost;
--
    -- INSERT INTO inventory.movements (
        -- item_uuid, warehouse_uuid, movement_date,
        -- reference_type, reference_id,
        -- quantity, unit_cost, direction
    -- ) VALUES (
        -- v_item.uuid, v_warehouse_uuid, now(),
        -- p_reference_type, p_reference_serial_id,
        -- p_quantity, p_unit_cost, 'IN'
    -- ) RETURNING uuid INTO v_movement_uuid;
--
    -- =====================================================================
    -- GL handling
    -- =====================================================================
    -- IF p_gl_transaction_uuid IS NULL THEN
        -- v_lines := jsonb_build_array(
            -- jsonb_build_object(
                -- 'account_ref', v_item.asset_account, 'debit', v_total, 'credit', 0,
                -- 'memo','Purchases Account'
            -- ),
            -- jsonb_build_object(
                -- 'account_ref', p_source_account, 'debit', 0, 'credit', v_total,
                -- 'memo','Cash or equivalent Account'
            -- )
        -- );
--
        -- v_txn_serial_id := accounting.post_transaction(
            -- now()::date,
            -- 'INV-PUR-' || p_reference_serial_id,
            -- 'Inventory Purchase',
            -- p_user,
            -- 'inventory_purchase',
            -- v_lines
        -- );
--
        -- SELECT uuid INTO v_txn_uuid
        -- FROM accounting.transactions WHERE serial_id = v_txn_serial_id;
    -- ELSE
        -- Bill posted to ledger from payables.post_bill(...)
        -- v_txn_uuid := p_gl_transaction_uuid;
    -- END IF;
--
    -- UPDATE inventory.movements
    -- SET gl_transaction_uuid = v_txn_uuid
    -- WHERE uuid = v_movement_uuid;
--
    -- UPDATE inventory.items
    -- SET quantity_on_hand = quantity_on_hand + p_quantity,
        -- updated_at = now()
    -- WHERE uuid = v_item.uuid;
--
    -- RETURN v_txn_uuid;
-- END;
-- $$;

-- ========================================
-- INDEXES (Performance)
-- ========================================
CREATE INDEX IF NOT EXISTS idx_items_sku ON inventory.items(sku);
CREATE INDEX IF NOT EXISTS idx_items_category ON inventory.items(category_uuid);
CREATE INDEX IF NOT EXISTS idx_movements_item ON inventory.movements(item_uuid);
CREATE INDEX IF NOT EXISTS idx_movements_date ON inventory.movements(movement_date);
CREATE INDEX IF NOT EXISTS idx_movements_direction ON inventory.movements(direction);
CREATE INDEX IF NOT EXISTS idx_adjustments_item ON inventory.adjustments(item_uuid);
CREATE INDEX IF NOT EXISTS idx_wavg_history_cost ON inventory.wavg_history(current_avg_cost);
