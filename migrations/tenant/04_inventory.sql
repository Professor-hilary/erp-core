-- ============================================================================
-- INVENTORY MODULE - FULL SCHEMA
-- Run this ONCE in a fresh DB with `accounting` schema already present
-- ============================================================================

-- Create schema
CREATE SCHEMA IF NOT EXISTS inventory;

-- ============================================================================
-- SEQUENCES
-- ============================================================================
CREATE SEQUENCE IF NOT EXISTS inventory.warehouses_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS inventory.item_categories_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS inventory.items_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS inventory.movements_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS inventory.adjustments_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS inventory.item_valuation_serial_id_seq;

-- ============================================================================
-- TABLE: warehouses
-- ============================================================================
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

-- ============================================================================
-- TABLE: item_categories
-- ============================================================================
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

-- ============================================================================
-- TABLE: items
-- ============================================================================
CREATE TABLE IF NOT EXISTS inventory.items (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('inventory.items_serial_id_seq') NOT NULL,
    sku text NOT NULL,
    name text NOT NULL,
    category_uuid uuid,
    description text,
    unit text DEFAULT 'pcs',
    selling_price numeric(18, 2) DEFAULT 0,
    track_quantity boolean DEFAULT true,
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

-- ============================================================================
-- TABLE: movements
-- ============================================================================
CREATE TABLE IF NOT EXISTS inventory.movements (
    uuid uuid DEFAULT uuidv7() NOT NULL,
    serial_id bigint DEFAULT nextval('inventory.movements_serial_id_seq') NOT NULL,
    item_uuid uuid NOT NULL,
    warehouse_uuid uuid,
    movement_date timestamptz DEFAULT now(),
    reference_type TEXT,
    reference_id bigint,
    financial_period_uuid UUID REFERENCES accounting.financial_periods(uuid),
    movement_type TEXT CHECK (movement_type IN ('OPENING', 'PURCHASE', 'SALE', 'ADJUSTMENT', 'TRANSFER')),
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

-- ============================================================================
-- TABLE: adjustments
-- ============================================================================
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

-- ============================================================================
-- TABLE: item_valuation
-- ============================================================================
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

-- ============================================================================
-- TABLE: lots (for FIFO/LIFO cost layering)
-- ============================================================================
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

-- ============================================================================
-- VIEWS
-- ============================================================================

-- Modern item summary view, tracks using inventory management method
CREATE OR REPLACE VIEW inventory.item_summary AS
SELECT
    i.serial_id,
    i.sku,
    i.name,
    i.unit,
    inventory.get_quantity_on_hand(i.uuid) AS quantity_on_hand,
    COALESCE(
        w.current_avg_cost,
        (
            SELECT SUM(l.remaining_total_cost)/NULLIF(SUM(l.remaining_quantity),0)
            FROM inventory.lots l
            WHERE l.item_uuid = i.uuid AND l.remaining_quantity > 0
        ),
        0
    ) AS current_cost,
    COALESCE(
        (
            SELECT SUM(l.remaining_total_cost)
            FROM inventory.lots l
            WHERE l.item_uuid = i.uuid AND l.remaining_quantity > 0
        ),
        w.total_cost,
        0
    ) AS inventory_value
FROM inventory.items i
LEFT JOIN inventory.wavg_history w ON w.item_uuid = i.uuid;

-- Track stock inflows and outflows
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

-- Update current inventory cost value
CREATE OR REPLACE VIEW inventory.item_current_cost AS
SELECT
    i.uuid AS item_uuid,
    i.serial_id,
    i.sku,
    i.name,
    i.valuation_method,
    CASE
        WHEN i.valuation_method = 'WAVG' THEN
            COALESCE(w.current_avg_cost, 0)
        ELSE
            COALESCE(
                (
                    SELECT SUM(l.remaining_total_cost) / NULLIF(SUM(l.remaining_quantity), 0)
                    FROM inventory.lots l WHERE l.item_uuid = i.uuid AND l.remaining_quantity > 0
                ),
                0
            )
    END AS current_cost_per_unit,
    inventory.get_quantity_on_hand(i.uuid) AS quantity_on_hand,
    CASE
        WHEN i.valuation_method = 'WAVG' THEN
            COALESCE(w.total_cost, 0)
        ELSE
            COALESCE(
                (
                    SELECT SUM(l.remaining_total_cost) FROM inventory.lots l
                    WHERE l.item_uuid = i.uuid AND l.remaining_quantity > 0
                ),
                0
            )
    END AS inventory_value
FROM inventory.items i
LEFT JOIN inventory.wavg_history w ON w.item_uuid = i.uuid;

-- ============================================================================
-- FUNCTIONS
-- ============================================================================

-- Posting opening stock
CREATE OR REPLACE FUNCTION inventory.post_opening_stock(
    p_item_serial_id BIGINT,
    p_warehouse_serial_id BIGINT,
    p_quantity NUMERIC,
    p_unit_cost NUMERIC,
    p_financial_period_uuid UUID,
    p_user UUID
) RETURNS VOID
LANGUAGE plpgsql AS $$
DECLARE
    v_item inventory.items%ROWTYPE;
    v_warehouse_uuid UUID;
BEGIN
    SELECT * INTO v_item FROM inventory.items WHERE serial_id = p_item_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Item not found'; END IF;

    SELECT uuid INTO v_warehouse_uuid
    FROM inventory.warehouses WHERE serial_id = p_warehouse_serial_id;

    -- Inventory movement
    INSERT INTO inventory.movements (
        item_uuid, warehouse_uuid, quantity, unit_cost, direction, movement_type,
        financial_period_uuid
    ) VALUES (
        v_item.uuid, v_warehouse_uuid, p_quantity, p_unit_cost, 'IN', 'OPENING',
        p_financial_period_uuid
    );

    -- FIFO/LIFO
    IF v_item.valuation_method IN ('LIFO','FIFO') THEN
        INSERT INTO inventory.lots (
            item_uuid, warehouse_uuid, reference_id, original_quantity,
            original_cost_per_unit, remaining_quantity
        ) VALUES (
            v_item.uuid, v_warehouse_uuid, 'opening', p_item_serial_id,
            p_quantity, p_unit_cost, p_quantity
        );
    ELSE
        INSERT INTO inventory.wavg_history (item_uuid, total_quantity, total_cost)
        VALUES (v_item.uuid, p_quantity, p_quantity * p_unit_cost);
    END IF;

END;
$$;

-- Check available stock
CREATE OR REPLACE FUNCTION inventory.get_quantity_on_hand(
    p_item_uuid UUID,
    p_warehouse_uuid UUID DEFAULT NULL
) RETURNS NUMERIC(18,4)
LANGUAGE plpgsql AS $$
DECLARE
    v_qty NUMERIC(18,4);
BEGIN
    IF p_warehouse_uuid IS NULL THEN
        RAISE EXCEPTION 'Warehouse UUID is required for quantity check';
    END IF;

    SELECT COALESCE(SUM(
        CASE WHEN direction = 'IN' THEN quantity ELSE -quantity END
    ), 0)
    INTO v_qty
    FROM inventory.movements
    WHERE item_uuid = p_item_uuid
        AND warehouse_uuid = p_warehouse_uuid;

    RETURN v_qty;
END;
$$

-- Prevent negative stock quantity deduction
CREATE OR REPLACE FUNCTION inventory.chech_stock_sufficient(
    p_item_uuid UUID,
    p_warehouse_uuid UUID,
    p_quantity_out NUMERIC(18,4)
) RETURN void
LANGUAGE plpgsql AS $$
    IF inventory.get_quantity_on_hand(p_item_uuid, p_warehouse_uuid) < p_quantity_out THEN
        RETURN EXCEPTION 'Insufficient stock in warehouse for item % (requested: %, available: %)',
            p_item_uuid, p_quantity_out, inventory.get_quantity_on_hand(p_item_uuid, p_warehouse_uuid);
    END IF;
END;
$$

CREATE OR REPLACE FUNCTION inventory.prevent_negative_lot()
RETURNS trigger AS $$ BEGIN
END;


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

    IF inventory.get_quantity_on_hand(v_item.uuid) < p_quantity_needed THEN
        RAISE EXCEPTION 'Insufficient stock';
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
        item_uuid, reference_type, reference_id, movement_type,
        quantity, unit_cost, direction, gl_transaction_uuid
    ) VALUES (
        v_item.uuid, p_reference_type, p_reference_serial_id,
        'SALE', p_quantity_needed,
        v_cogs / NULLIF(p_quantity_needed, 0),  -- average unit cost for this depletion
        'OUT',
        p_gl_transaction_uuid
    );

    RETURN v_cogs;
END;
$$;

--------------------------------------------------------------------------------
-- Inventory post cash or credit sale function
--------------------------------------------------------------------------------
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

-- Function to prevent posting into locked periods
CREATE OR REPLACE FUNCTION accounting.prevent_lock_period_posting()
RETURNS trigger AS $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM accounting.financial_periods
        WHERE uuid = NEW.financial_period_uuid
        AND is_locked
    ) THEN
        RAISE EXCEPTION 'Cannot post into locked financial period';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_block_locked_periods
BEFORE INSERT ON inventory.movements
FOR EACH ROW EXECUTE FUNCTION accounting.prevent_lock_period_posting();

--------------------------------------------------------------------------------
-- Inventory post cash or credit purchase function
--------------------------------------------------------------------------------
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
        reference_type, movement_type, reference_id,
        quantity, unit_cost, direction,
        gl_transaction_uuid
    ) VALUES (
        v_item.uuid, v_warehouse_uuid,
        p_reference_type, 'PURCHASE' p_reference_serial_id,
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

    RETURN v_txn_uuid;
END;
$$;

-- ============================================================================
-- INDEXES (Performance)
-- ============================================================================
CREATE INDEX IF NOT EXISTS idx_items_sku ON inventory.items(sku);
CREATE INDEX IF NOT EXISTS idx_items_category ON inventory.items(category_uuid);
CREATE INDEX IF NOT EXISTS idx_movements_item ON inventory.movements(item_uuid);
CREATE INDEX IF NOT EXISTS idx_movements_date ON inventory.movements(movement_date);
CREATE INDEX IF NOT EXISTS idx_movements_direction ON inventory.movements(direction);
CREATE INDEX IF NOT EXISTS idx_adjustments_item ON inventory.adjustments(item_uuid);
CREATE INDEX IF NOT EXISTS idx_wavg_history_cost ON inventory.wavg_history(current_avg_cost);

/*
-- ============================================================================
-- INVENTORY MODULE - FIXED & MVP-READY (FIFO + LIFO + per-warehouse WAVG)
-- ============================================================================
-- Run ONCE in a fresh DB with accounting schema present

CREATE SCHEMA IF NOT EXISTS inventory;

-- ============================================================================
-- SEQUENCES
-- ============================================================================
CREATE SEQUENCE IF NOT EXISTS inventory.warehouses_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS inventory.item_categories_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS inventory.items_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS inventory.movements_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS inventory.adjustments_serial_id_seq;
CREATE SEQUENCE IF NOT EXISTS inventory.lots_serial_id_seq;

-- ============================================================================
-- TABLES
-- ============================================================================

CREATE TABLE IF NOT EXISTS inventory.warehouses (
    uuid          uuid        DEFAULT uuidv7() NOT NULL PRIMARY KEY,
    serial_id     bigint      DEFAULT nextval('inventory.warehouses_serial_id_seq') NOT NULL UNIQUE,
    code          text        NOT NULL UNIQUE,
    name          text        NOT NULL,
    location      text,
    description   text,
    created_at    timestamptz DEFAULT now(),
    updated_at    timestamptz DEFAULT now()
);

CREATE TABLE IF NOT EXISTS inventory.item_categories (
    uuid          uuid        DEFAULT uuidv7() NOT NULL PRIMARY KEY,
    serial_id     bigint      DEFAULT nextval('inventory.item_categories_serial_id_seq') NOT NULL UNIQUE,
    code          text        NOT NULL UNIQUE,
    name          text        NOT NULL,
    description   text,
    created_at    timestamptz DEFAULT now(),
    updated_at    timestamptz DEFAULT now()
);

CREATE TABLE IF NOT EXISTS inventory.items (
    uuid              uuid        DEFAULT uuidv7() NOT NULL PRIMARY KEY,
    serial_id         bigint      DEFAULT nextval('inventory.items_serial_id_seq') NOT NULL UNIQUE,
    sku               text        NOT NULL UNIQUE,
    name              text        NOT NULL,
    category_uuid     uuid        REFERENCES inventory.item_categories(uuid) ON DELETE SET NULL,
    description       text,
    unit              text        DEFAULT 'pcs',
    selling_price     numeric(18,2) DEFAULT 0,
    track_quantity    boolean     DEFAULT true,
    reorder_level     numeric(18,4) DEFAULT 0,
    valuation_method  text        NOT NULL DEFAULT 'FIFO'
        CHECK (valuation_method IN ('FIFO', 'LIFO', 'WAVG')),
    asset_account     text,
    cogs_account      text,
    income_account    text,
    status            text        DEFAULT 'Active',
    created_at        timestamptz DEFAULT now(),
    updated_at        timestamptz DEFAULT now()
);

CREATE TABLE IF NOT EXISTS inventory.movements (
    uuid                uuid        DEFAULT uuidv7() NOT NULL PRIMARY KEY,
    serial_id           bigint      DEFAULT nextval('inventory.movements_serial_id_seq') NOT NULL UNIQUE,
    item_uuid           uuid        NOT NULL REFERENCES inventory.items(uuid) ON DELETE RESTRICT,
    warehouse_uuid      uuid        NOT NULL REFERENCES inventory.warehouses(uuid) ON DELETE RESTRICT,
    movement_date       timestamptz DEFAULT now(),
    reference_type      text,
    reference_id        bigint,
    financial_period_uuid uuid      REFERENCES accounting.financial_periods(uuid),
    movement_type       text        CHECK (movement_type IN ('OPENING', 'PURCHASE', 'SALE', 'ADJUSTMENT', 'TRANSFER')),
    quantity            numeric(18,4) NOT NULL,
    unit_cost           numeric(18,4) DEFAULT 0,
    total_cost          numeric(18,2) GENERATED ALWAYS AS (quantity * unit_cost) STORED,
    direction           text        NOT NULL CHECK (direction IN ('IN', 'OUT')),
    gl_transaction_uuid uuid        REFERENCES accounting.transactions(uuid),
    created_at          timestamptz DEFAULT now()
);

-- Single inventory stock adjustment not reliable - updates one inventory item per adjustment
--CREATE TABLE IF NOT EXISTS inventory.adjustments (
--    uuid                uuid        DEFAULT uuidv7() NOT NULL PRIMARY KEY,
--    serial_id           bigint      DEFAULT nextval('inventory.adjustments_serial_id_seq') NOT NULL UNIQUE,
--    adjustment_number   text        NOT NULL UNIQUE,
--    adjustment_date     date        NOT NULL,
--    warehouse_uuid      uuid        REFERENCES inventory.warehouses(uuid),
--    item_uuid           uuid        REFERENCES inventory.items(uuid),
--    old_quantity        numeric(18,4) DEFAULT 0,
--    new_quantity        numeric(18,4) NOT NULL,
--    difference          numeric(18,4) GENERATED ALWAYS AS (new_quantity - old_quantity) STORED,
--    reason              text,
--    posted              boolean     DEFAULT false,
--    gl_transaction_uuid uuid        REFERENCES accounting.transactions(uuid),
--    created_at          timestamptz DEFAULT now()
--);

-- ============================================================================
-- ADJUSTMENTS - UPGRADED TO HEADER + MULTI-ITEM LINES (full horsepower!)
-- ============================================================================
-- Header: one adjustment document (e.g. "Cycle Count Jan 2026")
CREATE TABLE IF NOT EXISTS inventory.adjustment_headers (
    uuid                uuid          DEFAULT uuidv7() NOT NULL PRIMARY KEY,
    serial_id           bigint        DEFAULT nextval('inventory.adjustments_serial_id_seq') NOT NULL UNIQUE,
    adjustment_number   text          NOT NULL UNIQUE,  -- e.g. 'ADJ-2026-001'
    adjustment_date     date          NOT NULL,
    warehouse_uuid      uuid          REFERENCES inventory.warehouses(uuid) ON DELETE SET NULL,
    reason              text,                               -- overall reason, e.g. "Monthly physical count"
    posted              boolean       DEFAULT false,
    gl_transaction_uuid uuid          REFERENCES accounting.transactions(uuid),
    created_at          timestamptz   DEFAULT now(),
    updated_at          timestamptz   DEFAULT now()
);

-- Lines: one per item in the adjustment
CREATE TABLE IF NOT EXISTS inventory.adjustment_lines (
    uuid                uuid          DEFAULT uuidv7() NOT NULL PRIMARY KEY,
    adjustment_header_uuid uuid       NOT NULL REFERENCES inventory.adjustment_headers(uuid) ON DELETE CASCADE,
    item_uuid           uuid          NOT NULL REFERENCES inventory.items(uuid) ON DELETE RESTRICT,
    old_quantity        numeric(18,4) DEFAULT 0,
    new_quantity        numeric(18,4) NOT NULL,
    difference          numeric(18,4) GENERATED ALWAYS AS (new_quantity - old_quantity) STORED,
    line_reason         text,                               -- optional per-item note, e.g. "Damaged 5 pcs"
    created_at          timestamptz   DEFAULT now()
);


-- FIFO / LIFO layering (per item + warehouse)
CREATE TABLE IF NOT EXISTS inventory.lots (
    uuid                    uuid        DEFAULT uuidv7() PRIMARY KEY,
    serial_id               bigserial   NOT NULL UNIQUE,
    item_uuid               uuid        NOT NULL REFERENCES inventory.items(uuid) ON DELETE CASCADE,
    warehouse_uuid          uuid        REFERENCES inventory.warehouses(uuid) ON DELETE SET NULL,
    reference_type          text        NOT NULL,
    reference_id            bigint,
    received_date           date        NOT NULL DEFAULT CURRENT_DATE,
    original_quantity       numeric(18,4) NOT NULL,
    original_cost_per_unit  numeric(18,4) NOT NULL,
    original_total_cost     numeric(18,2) GENERATED ALWAYS AS (original_quantity * original_cost_per_unit) STORED,
    remaining_quantity      numeric(18,4) NOT NULL CHECK (remaining_quantity >= 0),
    remaining_total_cost    numeric(18,2) GENERATED ALWAYS AS (remaining_quantity * original_cost_per_unit) STORED,
    batch_number            text,
    expiry_date             date,
    created_at              timestamptz DEFAULT now(),

    CONSTRAINT lots_item_warehouse_ref UNIQUE (item_uuid, warehouse_uuid, reference_type, reference_id)
);

-- Weighted Average — per item + warehouse
CREATE TABLE IF NOT EXISTS inventory.wavg_warehouse (
    item_uuid       uuid NOT NULL,
    warehouse_uuid  uuid NOT NULL,
    total_quantity  numeric(18,4) NOT NULL DEFAULT 0 CHECK (total_quantity >= 0),
    total_cost      numeric(18,2) NOT NULL DEFAULT 0 CHECK (total_cost >= 0),
    current_avg_cost numeric(18,4) GENERATED ALWAYS AS (
        CASE WHEN total_quantity > 0 THEN total_cost / total_quantity ELSE 0 END
    ) STORED,
    last_updated_at timestamptz DEFAULT now(),
    PRIMARY KEY (item_uuid, warehouse_uuid),
    FOREIGN KEY (item_uuid) REFERENCES inventory.items(uuid) ON DELETE CASCADE,
    FOREIGN KEY (warehouse_uuid) REFERENCES inventory.warehouses(uuid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS inventory.lot_movements (
    uuid            uuid        DEFAULT uuidv7() PRIMARY KEY,
    lot_uuid        uuid        NOT NULL REFERENCES inventory.lots(uuid) ON DELETE RESTRICT,
    movement_uuid   uuid        REFERENCES inventory.movements(uuid) ON DELETE CASCADE,
    quantity_used   numeric(18,4) NOT NULL,
    cost_per_unit   numeric(18,4) NOT NULL,
    total_cost      numeric(18,2) NOT NULL,
    created_at      timestamptz DEFAULT now()
);

-- Indexes
CREATE INDEX idx_lots_item           ON inventory.lots(item_uuid);
CREATE INDEX idx_lots_item_received  ON inventory.lots(item_uuid, received_date);
CREATE INDEX idx_lots_remaining      ON inventory.lots(item_uuid) WHERE remaining_quantity > 0;
CREATE INDEX idx_lot_movements_lot   ON inventory.lot_movements(lot_uuid);
CREATE INDEX idx_movements_item      ON inventory.movements(item_uuid);
CREATE INDEX idx_movements_date      ON inventory.movements(movement_date);
CREATE INDEX idx_movements_warehouse ON inventory.movements(warehouse_uuid);

-- ============================================================================
-- FUNCTIONS
-- ============================================================================

-- Quantity on hand — warehouse REQUIRED
CREATE OR REPLACE FUNCTION inventory.get_quantity_on_hand(
    p_item_uuid     UUID,
    p_warehouse_uuid UUID
) RETURNS numeric(18,4)
LANGUAGE sql STABLE AS $$
    SELECT COALESCE(SUM(CASE WHEN direction = 'IN' THEN quantity ELSE -quantity END), 0)
    FROM inventory.movements
    WHERE item_uuid = p_item_uuid
      AND warehouse_uuid = p_warehouse_uuid;
$$;

-- Guard against overselling
CREATE OR REPLACE FUNCTION inventory.check_stock_sufficient(
    p_item_uuid      UUID,
    p_warehouse_uuid UUID,
    p_quantity_out   numeric(18,4)
) RETURNS void
LANGUAGE plpgsql AS $$
DECLARE
    v_available numeric(18,4);
BEGIN
    v_available := inventory.get_quantity_on_hand(p_item_uuid, p_warehouse_uuid);
    IF v_available < p_quantity_out THEN
        RAISE EXCEPTION 'Insufficient stock. Item: %, Warehouse: %, Requested: %, Available: %',
            p_item_uuid, p_warehouse_uuid, p_quantity_out, v_available;
    END IF;
END;
$$;

-- Prevent negative lots
CREATE OR REPLACE FUNCTION inventory.prevent_negative_lot()
RETURNS trigger AS $$
BEGIN
    IF NEW.remaining_quantity < 0 THEN
        RAISE EXCEPTION 'Cannot set negative remaining quantity on lot %', NEW.uuid;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_lots_no_negative
    BEFORE UPDATE OF remaining_quantity ON inventory.lots
    FOR EACH ROW EXECUTE FUNCTION inventory.prevent_negative_lot();

-- Deplete inventory — supports FIFO / LIFO / WAVG
CREATE OR REPLACE FUNCTION inventory.deplete_inventory(
    p_item_serial_id      bigint,
    p_warehouse_serial_id bigint,
    p_quantity_needed     numeric(18,4),
    p_reference_type      text,
    p_reference_serial_id bigint,
    p_user                uuid,
    p_gl_transaction_uuid uuid DEFAULT NULL
) RETURNS numeric(18,2)  -- total COGS
LANGUAGE plpgsql AS $$
DECLARE
    v_item           inventory.items%ROWTYPE;
    v_warehouse_uuid uuid;
    v_cogs           numeric(18,2) := 0;
    v_lot            RECORD;
    v_remaining      numeric(18,4) := p_quantity_needed;
    v_avg_cost       numeric(18,4);
    v_order          text;
BEGIN
    SELECT * INTO v_item FROM inventory.items WHERE serial_id = p_item_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Item not found'; END IF;

    SELECT uuid INTO v_warehouse_uuid
    FROM inventory.warehouses WHERE serial_id = p_warehouse_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Warehouse not found'; END IF;

    PERFORM inventory.check_stock_sufficient(v_item.uuid, v_warehouse_uuid, p_quantity_needed);

    IF v_item.valuation_method = 'WAVG' THEN
        -- Lock the wavg row to prevent concurrent depletions
        SELECT current_avg_cost INTO v_avg_cost
        FROM inventory.wavg_warehouse
        WHERE item_uuid = v_item.uuid AND warehouse_uuid = v_warehouse_uuid
        FOR UPDATE;

        IF v_avg_cost IS NULL AND p_quantity_needed > 0 THEN
            RAISE EXCEPTION 'No weighted average cost history for item % in warehouse %', p_item_serial_id, p_warehouse_serial_id;
        END IF;

        v_cogs := p_quantity_needed * v_avg_cost;

        UPDATE inventory.wavg_warehouse
        SET
            total_quantity  = total_quantity - p_quantity_needed,
            total_cost      = total_cost - v_cogs,
            last_updated_at = now()
        WHERE item_uuid = v_item.uuid AND warehouse_uuid = v_warehouse_uuid;

    ELSE  -- FIFO or LIFO
        v_order := CASE WHEN v_item.valuation_method = 'LIFO' THEN 'DESC' ELSE 'ASC' END;

        PERFORM 1 FROM inventory.lots
        WHERE item_uuid = v_item.uuid AND warehouse_uuid = v_warehouse_uuid AND remaining_quantity > 0
        FOR UPDATE;

        FOR v_lot IN
            SELECT * FROM inventory.lots
            WHERE item_uuid = v_item.uuid
              AND warehouse_uuid = v_warehouse_uuid
              AND remaining_quantity > 0
            ORDER BY received_date || v_order
        LOOP
            EXIT WHEN v_remaining <= 0;

            IF v_lot.remaining_quantity >= v_remaining THEN
                v_cogs := v_cogs + (v_remaining * v_lot.original_cost_per_unit);
                INSERT INTO inventory.lot_movements (lot_uuid, quantity_used, cost_per_unit, total_cost)
                VALUES (v_lot.uuid, v_remaining, v_lot.original_cost_per_unit, v_remaining * v_lot.original_cost_per_unit);

                UPDATE inventory.lots SET remaining_quantity = remaining_quantity - v_remaining
                WHERE uuid = v_lot.uuid;

                v_remaining := 0;
            ELSE
                v_cogs := v_cogs + v_lot.remaining_total_cost;
                INSERT INTO inventory.lot_movements (lot_uuid, quantity_used, cost_per_unit, total_cost)
                VALUES (v_lot.uuid, v_lot.remaining_quantity, v_lot.original_cost_per_unit, v_lot.remaining_total_cost);

                UPDATE inventory.lots SET remaining_quantity = 0
                WHERE uuid = v_lot.uuid;

                v_remaining := v_remaining - v_lot.remaining_quantity;
            END IF;
        END LOOP;

        IF v_remaining > 0 THEN
            RAISE EXCEPTION 'Inventory layer exhaustion';
        END IF;
    END IF;

    -- Record OUT movement (unit_cost = effective average for this depletion)
    INSERT INTO inventory.movements (
        item_uuid, warehouse_uuid, reference_type, reference_id, movement_type,
        quantity, unit_cost, direction, gl_transaction_uuid
    ) VALUES (
        v_item.uuid, v_warehouse_uuid, p_reference_type, p_reference_serial_id, 'SALE',
        p_quantity_needed, v_cogs / NULLIF(p_quantity_needed, 0), 'OUT', p_gl_transaction_uuid
    );

    RETURN v_cogs;
END;
$$;

-- Post opening stock
CREATE OR REPLACE FUNCTION inventory.post_opening_stock(
    p_item_serial_id      bigint,
    p_warehouse_serial_id bigint,
    p_quantity            numeric,
    p_unit_cost           numeric,
    p_financial_period_uuid uuid,
    p_user                uuid
) RETURNS void
LANGUAGE plpgsql AS $$
DECLARE
    v_item           inventory.items%ROWTYPE;
    v_warehouse_uuid uuid;
BEGIN
    SELECT * INTO v_item FROM inventory.items WHERE serial_id = p_item_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Item not found'; END IF;

    SELECT uuid INTO v_warehouse_uuid
    FROM inventory.warehouses WHERE serial_id = p_warehouse_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Warehouse not found'; END IF;

    INSERT INTO inventory.movements (
        item_uuid, warehouse_uuid, quantity, unit_cost, direction, movement_type,
        financial_period_uuid
    ) VALUES (
        v_item.uuid, v_warehouse_uuid, p_quantity, p_unit_cost, 'IN', 'OPENING',
        p_financial_period_uuid
    );

    IF v_item.valuation_method IN ('FIFO', 'LIFO') THEN
        INSERT INTO inventory.lots (
            item_uuid, warehouse_uuid, reference_type, reference_id,
            received_date, original_quantity, original_cost_per_unit, remaining_quantity
        ) VALUES (
            v_item.uuid, v_warehouse_uuid, 'opening', p_item_serial_id,
            CURRENT_DATE, p_quantity, p_unit_cost, p_quantity
        );
    ELSIF v_item.valuation_method = 'WAVG' THEN
        INSERT INTO inventory.wavg_warehouse (item_uuid, warehouse_uuid, total_quantity, total_cost)
        VALUES (v_item.uuid, v_warehouse_uuid, p_quantity, p_quantity * p_unit_cost)
        ON CONFLICT (item_uuid, warehouse_uuid) DO UPDATE SET
            total_quantity = inventory.wavg_warehouse.total_quantity + p_quantity,
            total_cost     = inventory.wavg_warehouse.total_cost     + (p_quantity * p_unit_cost),
            last_updated_at = now();
    END IF;
END;
$$;

-- Post purchase (cash or credit)
CREATE OR REPLACE FUNCTION inventory.post_purchase(
    p_item_serial_id      bigint,
    p_warehouse_serial_id bigint,
    p_quantity            numeric,
    p_unit_cost           numeric,
    p_reference_type      text,
    p_reference_serial_id bigint,
    p_user                uuid,
    p_source_account      text DEFAULT NULL,
    p_gl_transaction_uuid uuid DEFAULT NULL
) RETURNS uuid
LANGUAGE plpgsql AS $$
DECLARE
    v_item           inventory.items%ROWTYPE;
    v_warehouse_uuid uuid;
    v_total          numeric(18,2) := p_quantity * p_unit_cost;
    v_movement_uuid  uuid;
BEGIN
    SELECT * INTO v_item FROM inventory.items WHERE serial_id = p_item_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Item not found'; END IF;

    SELECT uuid INTO v_warehouse_uuid
    FROM inventory.warehouses WHERE serial_id = p_warehouse_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Warehouse not found'; END IF;

    INSERT INTO inventory.movements (
        item_uuid, warehouse_uuid, reference_type, reference_id, movement_type,
        quantity, unit_cost, direction, gl_transaction_uuid
    ) VALUES (
        v_item.uuid, v_warehouse_uuid, p_reference_type, p_reference_serial_id, 'PURCHASE',
        p_quantity, p_unit_cost, 'IN', p_gl_transaction_uuid
    ) RETURNING uuid INTO v_movement_uuid;

    IF v_item.valuation_method IN ('FIFO', 'LIFO') THEN
        INSERT INTO inventory.lots (
            item_uuid, warehouse_uuid, reference_type, reference_id,
            received_date, original_quantity, original_cost_per_unit, remaining_quantity
        ) VALUES (
            v_item.uuid, v_warehouse_uuid, p_reference_type, p_reference_serial_id,
            CURRENT_DATE, p_quantity, p_unit_cost, p_quantity
        );
    ELSIF v_item.valuation_method = 'WAVG' THEN
        INSERT INTO inventory.wavg_warehouse (item_uuid, warehouse_uuid, total_quantity, total_cost)
        VALUES (v_item.uuid, v_warehouse_uuid, p_quantity, v_total)
        ON CONFLICT (item_uuid, warehouse_uuid) DO UPDATE SET
            total_quantity = inventory.wavg_warehouse.total_quantity + p_quantity,
            total_cost     = inventory.wavg_warehouse.total_cost     + v_total,
            last_updated_at = now();
    END IF;

    -- Cash purchase GL
    IF p_gl_transaction_uuid IS NULL AND p_source_account IS NOT NULL THEN
        PERFORM accounting.post_transaction(
            CURRENT_DATE, 'INV-PUR-' || p_reference_serial_id, 'Inventory Purchase (Cash)',
            p_user, 'inventory_purchase',
            jsonb_build_array(
                jsonb_build_object('account_ref', v_item.asset_account, 'debit', v_total, 'credit', 0),
                jsonb_build_object('account_ref', p_source_account, 'debit', 0, 'credit', v_total)
            )
        );
    END IF;

    RETURN p_gl_transaction_uuid;
END;
$$;

-- Post sale (cash or credit) — COGS always posted here
CREATE OR REPLACE FUNCTION inventory.post_sale(
    p_item_serial_id      bigint,
    p_warehouse_serial_id bigint,
    p_quantity            numeric,
    p_selling_price       numeric,
    p_reference_type      text,
    p_reference_serial_id bigint,
    p_user                uuid,
    p_cash_code           text DEFAULT NULL,
    p_gl_transaction_uuid uuid DEFAULT NULL
) RETURNS uuid
LANGUAGE plpgsql AS $$
DECLARE
    v_item           inventory.items%ROWTYPE;
    v_warehouse_uuid uuid;
    v_total_revenue  numeric(18,2) := p_quantity * p_selling_price;
    v_cogs           numeric(18,2);
BEGIN
    SELECT * INTO v_item FROM inventory.items WHERE serial_id = p_item_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Item not found'; END IF;

    SELECT uuid INTO v_warehouse_uuid
    FROM inventory.warehouses WHERE serial_id = p_warehouse_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Warehouse not found'; END IF;

    v_cogs := inventory.deplete_inventory(
        p_item_serial_id, p_warehouse_serial_id, p_quantity,
        p_reference_type, p_reference_serial_id, p_user, p_gl_transaction_uuid
    );

    -- Post COGS
    PERFORM accounting.post_transaction(
        CURRENT_DATE, 'COGS-' || p_reference_serial_id, 'Cost of Goods Sold',
        p_user, 'cogs',
        jsonb_build_array(
            jsonb_build_object('account_ref', v_item.cogs_account, 'debit', v_cogs, 'credit', 0),
            jsonb_build_object('account_ref', v_item.asset_account, 'debit', 0, 'credit', v_cogs)
        )
    );

    -- Cash sale revenue
    IF p_cash_code IS NOT NULL AND p_gl_transaction_uuid IS NULL THEN
        PERFORM accounting.post_transaction(
            CURRENT_DATE, 'SALE-' || p_reference_serial_id, 'Cash Sale',
            p_user, 'sale',
            jsonb_build_array(
                jsonb_build_object('account_ref', p_cash_code, 'debit', v_total_revenue, 'credit', 0),
                jsonb_build_object('account_ref', v_item.income_account, 'debit', 0, 'credit', v_total_revenue)
            )
        );
    END IF;

    RETURN p_gl_transaction_uuid;
END;
$$;

-- Lock-period guard (only on movements — see warning below)
CREATE OR REPLACE FUNCTION accounting.prevent_lock_period_posting()
RETURNS trigger AS $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM accounting.financial_periods
        WHERE uuid = NEW.financial_period_uuid AND is_locked
    ) THEN
        RAISE EXCEPTION 'Cannot post into locked financial period';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_block_locked_periods
    BEFORE INSERT ON inventory.movements
    FOR EACH ROW EXECUTE FUNCTION accounting.prevent_lock_period_posting();


-- Indexes for fast lookups / reporting
CREATE INDEX idx_adjustment_lines_header ON inventory.adjustment_lines(adjustment_header_uuid);
CREATE INDEX idx_adjustment_lines_item   ON inventory.adjustment_lines(item_uuid);

-- Optional: prevent duplicate items in same adjustment
CREATE UNIQUE INDEX idx_adjustment_lines_unique_item
    ON inventory.adjustment_lines (adjustment_header_uuid, item_uuid);

-- ============================================================================
-- IMPORTANT MVP WARNINGS
-- ============================================================================
/*
  - Only movements table blocks posting into locked periods.
  - Updates to lots / wavg_warehouse / adjustments can still happen after lock → use application logic / UI restrictions.
  - Adjustments table supports multiple items per record (good for MVP).
  - Test thoroughly: opening → purchase → sale → check lots / wavg / GL balance.
*/

