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
    warehouse_serial  bigint      NOT NULL,
    category_uuid     uuid        REFERENCES inventory.item_categories(uuid) ON DELETE SET NULL,
    description       text,
    unit              text        DEFAULT 'pcs',
    item_type text not null default 'Purchased' check (item_type in ('Purchased', 'Manufactured', 'Service', 'Non-inventory')),
    selling_price     numeric(18,2) DEFAULT 0,
    standard_cost     numeric(18,4),
    track_quantity    boolean     DEFAULT true,
    reorder_level     numeric(18,4) DEFAULT 0,
    valuation_method  text        NOT NULL DEFAULT 'FIFO' CHECK (valuation_method IN ('FIFO', 'LIFO', 'WAVG')),
    bom_uuid uuid,          -- For manufacturing bill of materials
    routing_uuid uuid,      -- For routing production
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
    movement_type       text,
    quantity            numeric(18,4) NOT NULL,
    unit_cost           numeric(18,4) DEFAULT 0,
    total_cost          numeric(18,2) GENERATED ALWAYS AS (quantity * unit_cost) STORED,
    direction           text        NOT NULL CHECK (direction IN ('IN', 'OUT')),
    gl_transaction_uuid uuid        REFERENCES accounting.transactions(uuid),
    created_at          timestamptz DEFAULT now(),

    constraint movements_movement_type_check
		check(movement_type in (
			'OPENING', 'PURCHASE', 'SALE', 'ADJUSTMENT', 'TRANSFER',
			'ISSUE_TO_PROD',	-- Raw -> WIP (direct materials)
			'LABOR_APPLIED', 	-- Direct labor to WIP
			'OVERHEAD_APPLIED',	-- Overhead applied to WIP
			'PROD_COMPLETION',	-- WIP -> Finished Goods
			'PROD_SCRAP',		-- scrap / yield loss
			'PROD_RETURN'		-- return unused materials from prod to raw
		))
);

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
    p_gl_transaction_uuid uuid DEFAULT NULL,
    p_wip_code            text default NULL
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
    v_wip_account    uuid;
BEGIN
    SELECT * INTO v_item FROM inventory.items WHERE serial_id = p_item_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Item not found'; END IF;

    SELECT uuid INTO v_warehouse_uuid
    FROM inventory.warehouses WHERE serial_id = p_warehouse_serial_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'Warehouse not found'; END IF;

    PERFORM inventory.check_stock_sufficient(v_item.uuid, v_warehouse_uuid, p_quantity_needed);

    ----------------------------------------------------------------------
    -- Valuation logic
    ----------------------------------------------------------------------
    IF v_item.valuation_method = 'WAVG' THEN
        -- Lock the wavg row to prevent concurrent depletions
        SELECT current_avg_cost INTO v_avg_cost
        FROM inventory.wavg_warehouse
        WHERE item_uuid = v_item.uuid AND warehouse_uuid = v_warehouse_uuid
        FOR UPDATE;

        IF v_avg_cost IS NULL AND p_quantity_needed > 0 THEN
            RAISE EXCEPTION 'No WAVG cost history for item % in warehouse %', p_item_serial_id, p_warehouse_serial_id;
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
        v_item.uuid, v_warehouse_uuid, p_reference_type, p_reference_serial_id,
        CASE WHEN UPPER(p_reference_type) = 'PRODUCTION' THEN 'ISSUE_TO_PROD' ELSE 'SALE' END,
        p_quantity_needed, v_cogs / NULLIF(p_quantity_needed, 0), 'OUT', p_gl_transaction_uuid
    );

    -- For production Issue -> post to WIP (if GL provided)
    IF p_gl_transaction_uuid IS NOT NULL AND UPPER(p_reference_type) = 'PRODUCTION' THEN
        SELECT uuid INTO v_wip_account FROM accounting.accounts WHERE code = p_wip_code;

        INSERT INTO accounting.transaction_entries (
            transaction_uuid, account_uuid, line_no, amount, debit, credit, memo
        ) VALUES
            (p_gl_transaction_uuid, v_wip_account, 1, v_cogs, v_cogs, 0, 'Materials Issued to production'),
            (p_gl_transaction_uuid,
                (SELECT uuid FROM accounting.accounts WHERE code = v_item.asset_account),
                2, v_cogs, 0, v_cogs, 'Raw materials issued'
            );
	END IF;

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
    p_item_serial_id        bigint,
    p_warehouse_serial_id   bigint,
    p_quantity              numeric,
    p_unit_cost             numeric,
    p_reference_type        text, -- 'bill', 'cash_purchase', 'adjustment', etc.
    p_reference_serial_id   bigint,
    p_user                  uuid,
    p_source_account        text DEFAULT NULL, -- Cash/Bank  purchase
    p_gl_transaction_uuid   uuid DEFAULT NULL  -- For Credit purchase
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
            'INV-PUR-' || p_reference_serial_id, 'Inventory Purchase (Cash)',
            p_user, 'inventory_purchase', CURRENT_DATE,
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
    -- p_selling_price       numeric,
    p_reference_type      text, -- 'invoice', 'cash-sale', 'mobile-money', 'virtual-card'
    p_reference_serial_id bigint,
    p_user                uuid,
    -- p_cash_code           text DEFAULT NULL,
    p_gl_transaction_uuid uuid DEFAULT NULL
) RETURNS numeric
LANGUAGE plpgsql AS $$
DECLARE
    v_item           inventory.items%ROWTYPE;
    v_cogs           numeric(18,2);
BEGIN
    SELECT * INTO v_item FROM inventory.items WHERE serial_id = p_item_serial_id;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Item not found for serial id %', p_item_serial_id;
    END IF;

    -- Deplete inventory
    v_cogs := inventory.deplete_inventory(
        p_item_serial_id, p_warehouse_serial_id, p_quantity,
        p_reference_type, p_reference_serial_id, p_user, p_gl_transaction_uuid
    );

    -- Post COGS and inventory reduction using correct column names
    INSERT INTO accounting.transaction_entries(
        transaction_uuid, account_uuid, line_no, amount, debit, credit, memo
    ) VALUES (
        p_gl_transaction_uuid,
            (SELECT uuid FROM accounting.accounts WHERE code = v_item.cogs_account),
        1, v_cogs, v_cogs, 0, 'Cost Of Goods Sold'
    ),(
        p_gl_transaction_uuid,
            (SELECT uuid FROM accounting.accounts WHERE code = v_item.asset_account),
        2, -v_cogs, 0, v_cogs, 'Inventory reduction'
    );

    RETURN v_cogs;
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
