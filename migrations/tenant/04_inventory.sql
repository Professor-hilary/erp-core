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

-- ========================================
-- FUNCTIONS
-- ========================================
CREATE OR REPLACE FUNCTION inventory.post_purchase(
    p_item_serial_id bigint,
    p_warehouse_serial_id bigint,
    p_quantity numeric,
    p_unit_cost numeric,
    p_reference_type text,
    p_reference_serial_id bigint,
    p_user uuid,
    p_payables_uuid uuid
    -- p_data JSONB
) RETURNS void
LANGUAGE plpgsql
AS $$
DECLARE
    v_item inventory.items%ROWTYPE;
    v_warehouse_uuid UUID;
    v_movement_uuid UUID;
    v_movement_serial_id BIGINT;
    v_txn_serial_id BIGINT;
    v_txn_uuid UUID;
    v_total NUMERIC(18, 2);
    v_lines JSONB;
BEGIN
    -- Resolve item
    SELECT * INTO v_item FROM inventory.items WHERE serial_id = p_item_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Item serial_id % not found', p_item_serial_id; END IF;

    -- Resolve warehouse
    IF p_warehouse_serial_id IS NOT NULL THEN
        SELECT uuid INTO v_warehouse_uuid FROM inventory.warehouses WHERE serial_id = p_warehouse_serial_id;
        IF v_warehouse_uuid IS NULL THEN RAISE EXCEPTION 'Warehouse serial_id % not found', p_warehouse_serial_id; END IF;
    END IF;

    v_total := p_quantity * p_unit_cost;

    -- Insert movement
    INSERT INTO inventory.movements (
        item_uuid, warehouse_uuid, movement_date, reference_type, reference_id,
        quantity, unit_cost, direction
    ) VALUES (
        v_item.uuid, v_warehouse_uuid, now(), p_reference_type, p_reference_serial_id,
        p_quantity, p_unit_cost, 'IN'
    ) RETURNING uuid, serial_id INTO v_movement_uuid, v_movement_serial_id;

    -- Post to GL: Debit Inventory, Credit A/P
    v_lines := jsonb_build_array(
        jsonb_build_object('account_ref', v_item.asset_account, 'debit', v_total, 'credit', 0, 'memo', format('Stock In - %s', v_item.sku)),
        jsonb_build_object('account_ref', p_payables_uuid, 'debit', 0, 'credit', v_total, 'memo', 'Accounts Payable')
    );

    v_txn_serial_id := accounting.post_transaction(
        now()::date, 'INV-PUR-' || v_movement_serial_id, 'Inventory Purchase', p_user, 'inventory_purchase', v_lines
    );

    SELECT uuid INTO v_txn_uuid FROM accounting.transactions WHERE serial_id = v_txn_serial_id;

    UPDATE inventory.movements SET gl_transaction_uuid = v_txn_uuid WHERE uuid = v_movement_uuid;

    -- Record valuation
    INSERT INTO inventory.item_valuation (
        item_uuid, movement_uuid, debit_account, credit_account, amount, gl_transaction_uuid, description
    ) VALUES (
        v_item.uuid, v_movement_uuid, v_item.asset_account, v_item.category_uuid, v_total, v_txn_uuid, 'Purchase'
    );

    -- Update item
    UPDATE inventory.items
    SET
        quantity_on_hand = quantity_on_hand + p_quantity,
        cost_price = ((cost_price * quantity_on_hand) + v_total) / (quantity_on_hand + p_quantity),
        updated_at = now()
    WHERE uuid = v_item.uuid;
END;
$$;

CREATE OR REPLACE FUNCTION inventory.post_sale(
    p_item_serial_id bigint,
    p_warehouse_serial_id bigint,
    p_quantity numeric,
    p_unit_cost numeric,
    p_reference_type text,
    p_reference_serial_id bigint,
    p_user UUID
) RETURNS void
LANGUAGE plpgsql
AS $$
DECLARE
    v_item inventory.items%ROWTYPE;
    v_warehouse_uuid UUID;
    v_movement_uuid UUID;
    v_movement_serial_id BIGINT;
    v_txn_serial_id BIGINT;
    v_txn_uuid UUID;
    v_total NUMERIC(18, 2);
    v_lines JSONB;
BEGIN
    SELECT * INTO v_item FROM inventory.items WHERE serial_id = p_item_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Item serial_id % not found', p_item_serial_id; END IF;

    IF v_item.quantity_on_hand < p_quantity THEN
        RAISE EXCEPTION 'Insufficient stock: % < %', v_item.quantity_on_hand, p_quantity;
    END IF;

    IF p_warehouse_serial_id IS NOT NULL THEN
        SELECT uuid INTO v_warehouse_uuid FROM inventory.warehouses WHERE serial_id = p_warehouse_serial_id;
        IF v_warehouse_uuid IS NULL THEN RAISE EXCEPTION 'Warehouse serial_id % not found', p_warehouse_serial_id; END IF;
    END IF;

    v_total := p_quantity * p_unit_cost;

    INSERT INTO inventory.movements (
        item_uuid, warehouse_uuid, movement_date, reference_type, reference_id,
        quantity, unit_cost, direction
    ) VALUES (
        v_item.uuid, v_warehouse_uuid, now(), p_reference_type, p_reference_serial_id,
        p_quantity, p_unit_cost, 'OUT'
    ) RETURNING uuid, serial_id INTO v_movement_uuid, v_movement_serial_id;

    v_lines := jsonb_build_array(
        jsonb_build_object('account_ref', v_item.cogs_account, 'debit', v_total, 'credit', 0, 'memo', format('COGS - %s', v_item.sku)),
        jsonb_build_object('account_ref', v_item.asset_account, 'debit', 0, 'credit', v_total, 'memo', 'Inventory Asset')
    );

    v_txn_serial_id := accounting.post_transaction(
        now()::date, 'INV-SALE-' || v_movement_serial_id, 'Inventory Sale', p_user, 'inventory_sale', v_lines
    );

    SELECT uuid INTO v_txn_uuid FROM accounting.transactions WHERE serial_id = v_txn_serial_id;

    UPDATE inventory.movements SET gl_transaction_uuid = v_txn_uuid WHERE uuid = v_movement_uuid;

    INSERT INTO inventory.item_valuation (
        item_uuid, movement_uuid, debit_account, credit_account, amount, gl_transaction_uuid, description
    ) VALUES (
        v_item.uuid, v_movement_uuid, v_item.cogs_account, v_item.asset_account, v_total, v_txn_uuid, 'Sale'
    );

    UPDATE inventory.items
    SET quantity_on_hand = quantity_on_hand - p_quantity, updated_at = now()
    WHERE uuid = v_item.uuid;
END;
$$;

-- ========================================
-- INDEXES (Performance)
-- ========================================
CREATE INDEX IF NOT EXISTS idx_items_sku ON inventory.items(sku);
CREATE INDEX IF NOT EXISTS idx_items_category ON inventory.items(category_uuid);
CREATE INDEX IF NOT EXISTS idx_movements_item ON inventory.movements(item_uuid);
CREATE INDEX IF NOT EXISTS idx_movements_date ON inventory.movements(movement_date);
CREATE INDEX IF NOT EXISTS idx_movements_direction ON inventory.movements(direction);
CREATE INDEX IF NOT EXISTS idx_adjustments_item ON inventory.adjustments(item_uuid);
