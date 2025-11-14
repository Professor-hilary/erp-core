--
-- PostgreSQL database dump
--

-- Dumped from database version 17.5 (Debian 17.5-1)
-- Dumped by pg_dump version 17.5 (Debian 17.5-1)
SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;
--
-- Name: inventory; Type: SCHEMA; Schema: -; Owner: chiefalry_user
--

CREATE SCHEMA inventory;
ALTER SCHEMA inventory OWNER TO chiefalry_user;
--
-- Name: post_purchase(bigint, bigint, numeric, numeric, text, bigint, bigint); Type: FUNCTION; Schema: inventory; Owner: chiefalry_user
--

CREATE FUNCTION inventory.post_purchase(
    p_item_serial_id bigint,
    p_warehouse_serial_id bigint,
    p_quantity numeric,
    p_unit_cost numeric,
    p_reference_type text,
    p_reference_serial_id bigint,
    p_user bigint
) RETURNS void LANGUAGE plpgsql AS $$
DECLARE v_item inventory.items %ROWTYPE;
v_warehouse_uuid UUID;
v_movement_uuid UUID;
v_movement_serial_id BIGINT;
v_txn_serial_id BIGINT;
v_txn_uuid UUID;
v_total NUMERIC(18, 2);
v_lines JSONB;
BEGIN -- Resolve item
SELECT * INTO v_item
FROM inventory.items
WHERE serial_id = p_item_serial_id;
IF NOT FOUND THEN RAISE EXCEPTION 'Item with serial_id % not found',
p_item_serial_id;
END IF;
IF p_warehouse_serial_id IS NOT NULL THEN
SELECT uuid INTO v_warehouse_uuid
FROM inventory.warehouses
WHERE serial_id = p_warehouse_serial_id;
IF v_warehouse_uuid IS NULL THEN RAISE EXCEPTION 'Warehouse serial_id % not found',
p_warehouse_serial_id;
END IF;
END IF;
v_total := p_quantity * p_unit_cost;
-- Insert movement
INSERT INTO inventory.movements (
        item_uuid,
        warehouse_uuid,
        movement_date,
        reference_type,
        reference_id,
        quantity,
        unit_cost,
        direction
    )
VALUES (
        v_item.uuid,
        v_warehouse_uuid,
        now(),
        p_reference_type,
        p_reference_serial_id,
        p_quantity,
        p_unit_cost,
        'IN'
    )
RETURNING uuid,
    serial_id INTO v_movement_uuid,
    v_movement_serial_id;
-- GL: Debit Inventory, Credit A/P
v_lines := jsonb_build_array(
    jsonb_build_object(
        'account_ref',
        v_item.asset_account,
        'debit',
        v_total,
        'credit',
        0,
        'memo',
        format('Stock In - %s', v_item.sku)
    ),
    jsonb_build_object(
        'account_ref',
        '2.1.1',
        'debit',
        0,
        'credit',
        v_total,
        'memo',
        'Accounts Payable'
    )
);
-- Post to GL
v_txn_serial_id := accounting.post_transaction(
    now()::date,
    'INV-PUR-' || v_movement_serial_id,
    'Inventory Purchase',
    p_user,
    'inventory_purchase',
    v_lines
);
-- Get GL transaction UUID
SELECT uuid INTO v_txn_uuid
FROM accounting.transactions
WHERE serial_id = v_txn_serial_id;
-- Update movement with GL link
UPDATE inventory.movements
SET gl_transaction_uuid = v_txn_uuid
WHERE uuid = v_movement_uuid;
-- Record valuation
INSERT INTO inventory.item_valuation (
        item_uuid,
        movement_uuid,
        debit_account,
        credit_account,
        amount,
        gl_transaction_uuid,
        description
    )
VALUES (
        v_item.uuid,
        v_movement_uuid,
        v_item.asset_account,
        '2.1.1',
        v_total,
        v_txn_uuid,
        'Purchase'
    );
-- Update item quantity & average cost
UPDATE inventory.items
SET quantity_on_hand = quantity_on_hand + p_quantity,
    cost_price = (
        (cost_price * quantity_on_hand) + v_total
    ) / (quantity_on_hand + p_quantity),
    updated_at = now()
WHERE uuid = v_item.uuid;
END;
$$;
ALTER FUNCTION inventory.post_purchase(
    p_item_serial_id bigint,
    p_warehouse_serial_id bigint,
    p_quantity numeric,
    p_unit_cost numeric,
    p_reference_type text,
    p_reference_serial_id bigint,
    p_user bigint
) OWNER TO chiefalry_user;
--
-- Name: post_sale(bigint, bigint, numeric, numeric, text, bigint, bigint); Type: FUNCTION; Schema: inventory; Owner: chiefalry_user
--

CREATE FUNCTION inventory.post_sale(
    p_item_serial_id bigint,
    p_warehouse_serial_id bigint,
    p_quantity numeric,
    p_unit_cost numeric,
    p_reference_type text,
    p_reference_serial_id bigint,
    p_user bigint
) RETURNS void LANGUAGE plpgsql AS $$
DECLARE v_item inventory.items %ROWTYPE;
v_warehouse_uuid UUID;
v_movement_uuid UUID;
v_movement_serial_id BIGINT;
v_txn_serial_id BIGINT;
v_txn_uuid UUID;
v_total NUMERIC(18, 2);
v_lines JSONB;
BEGIN
SELECT * INTO v_item
FROM inventory.items
WHERE serial_id = p_item_serial_id;
IF NOT FOUND THEN RAISE EXCEPTION 'Item serial_id % not found',
p_item_serial_id;
END IF;
IF v_item.quantity_on_hand < p_quantity THEN RAISE EXCEPTION 'Insufficient stock: % < %',
v_item.quantity_on_hand,
p_quantity;
END IF;
IF p_warehouse_serial_id IS NOT NULL THEN
SELECT uuid INTO v_warehouse_uuid
FROM inventory.warehouses
WHERE serial_id = p_warehouse_serial_id;
IF v_warehouse_uuid IS NULL THEN RAISE EXCEPTION 'Warehouse serial_id % not found',
p_warehouse_serial_id;
END IF;
END IF;
v_total := p_quantity * p_unit_cost;
INSERT INTO inventory.movements (
        item_uuid,
        warehouse_uuid,
        movement_date,
        reference_type,
        reference_id,
        quantity,
        unit_cost,
        direction
    )
VALUES (
        v_item.uuid,
        v_warehouse_uuid,
        now(),
        p_reference_type,
        p_reference_serial_id,
        p_quantity,
        p_unit_cost,
        'OUT'
    )
RETURNING uuid,
    serial_id INTO v_movement_uuid,
    v_movement_serial_id;
-- GL: Debit COGS, Credit Inventory
v_lines := jsonb_build_array(
    jsonb_build_object(
        'account_ref',
        v_item.cogs_account,
        'debit',
        v_total,
        'credit',
        0,
        'memo',
        format('COGS - %s', v_item.sku)
    ),
    jsonb_build_object(
        'account_ref',
        v_item.asset_account,
        'debit',
        0,
        'credit',
        v_total,
        'memo',
        'Inventory Asset'
    )
);
v_txn_serial_id := accounting.post_transaction(
    now()::date,
    'INV-SALE-' || v_movement_serial_id,
    'Inventory Sale',
    p_user,
    'inventory_sale',
    v_lines
);
SELECT uuid INTO v_txn_uuid
FROM accounting.transactions
WHERE serial_id = v_txn_serial_id;
UPDATE inventory.movements
SET gl_transaction_uuid = v_txn_uuid
WHERE uuid = v_movement_uuid;
INSERT INTO inventory.item_valuation (
        item_uuid,
        movement_uuid,
        debit_account,
        credit_account,
        amount,
        gl_transaction_uuid,
        description
    )
VALUES (
        v_item.uuid,
        v_movement_uuid,
        v_item.cogs_account,
        v_item.asset_account,
        v_total,
        v_txn_uuid,
        'Sale'
    );
UPDATE inventory.items
SET quantity_on_hand = quantity_on_hand - p_quantity,
    updated_at = now()
WHERE uuid = v_item.uuid;
END;
$$;
ALTER FUNCTION inventory.post_sale(
    p_item_serial_id bigint,
    p_warehouse_serial_id bigint,
    p_quantity numeric,
    p_unit_cost numeric,
    p_reference_type text,
    p_reference_serial_id bigint,
    p_user bigint
) OWNER TO chiefalry_user;
SET default_tablespace = '';
SET default_table_access_method = heap;
--
-- Name: adjustments; Type: TABLE; Schema: inventory; Owner: chiefalry_user
--

CREATE TABLE inventory.adjustments (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    adjustment_number text NOT NULL,
    adjustment_date date NOT NULL,
    warehouse_uuid uuid,
    item_uuid uuid,
    old_quantity numeric(18, 4) DEFAULT 0,
    new_quantity numeric(18, 4) NOT NULL,
    difference numeric(18, 4) GENERATED ALWAYS AS ((new_quantity - old_quantity)) STORED,
    reason text,
    posted boolean DEFAULT false,
    gl_transaction_uuid uuid,
    created_at timestamp with time zone DEFAULT now()
);
ALTER TABLE inventory.adjustments OWNER TO chiefalry_user;
--
-- Name: adjustments_serial_id_seq; Type: SEQUENCE; Schema: inventory; Owner: chiefalry_user
--

CREATE SEQUENCE inventory.adjustments_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE inventory.adjustments_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: adjustments_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: inventory; Owner: chiefalry_user
--

ALTER SEQUENCE inventory.adjustments_serial_id_seq OWNED BY inventory.adjustments.serial_id;
--
-- Name: item_categories; Type: TABLE; Schema: inventory; Owner: chiefalry_user
--

CREATE TABLE inventory.item_categories (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    description text,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);
ALTER TABLE inventory.item_categories OWNER TO chiefalry_user;
--
-- Name: item_categories_serial_id_seq; Type: SEQUENCE; Schema: inventory; Owner: chiefalry_user
--

CREATE SEQUENCE inventory.item_categories_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE inventory.item_categories_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: item_categories_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: inventory; Owner: chiefalry_user
--

ALTER SEQUENCE inventory.item_categories_serial_id_seq OWNED BY inventory.item_categories.serial_id;
--
-- Name: items; Type: TABLE; Schema: inventory; Owner: chiefalry_user
--

CREATE TABLE inventory.items (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    sku text NOT NULL,
    name text NOT NULL,
    category_uuid uuid,
    description text,
    unit text DEFAULT 'pcs'::text,
    cost_price numeric(18, 2) DEFAULT 0,
    selling_price numeric(18, 2) DEFAULT 0,
    track_quantity boolean DEFAULT true,
    quantity_on_hand numeric(18, 4) DEFAULT 0,
    reorder_level numeric(18, 4) DEFAULT 0,
    asset_account text DEFAULT '1.3.1'::text,
    cogs_account text DEFAULT '5.2.1'::text,
    income_account text DEFAULT '4.1.1'::text,
    status text DEFAULT 'Active'::text,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);
ALTER TABLE inventory.items OWNER TO chiefalry_user;
--
-- Name: item_summary; Type: VIEW; Schema: inventory; Owner: chiefalry_user
--

CREATE VIEW inventory.item_summary AS
SELECT i.serial_id AS item_serial_id,
    i.sku,
    i.name AS item_name,
    c.name AS category_name,
    i.unit,
    i.quantity_on_hand,
    i.cost_price,
    (i.quantity_on_hand * i.cost_price) AS inventory_value,
    i.reorder_level,
    (i.quantity_on_hand <= i.reorder_level) AS needs_reorder
FROM (
        inventory.items i
        LEFT JOIN inventory.item_categories c ON ((c.uuid = i.category_uuid))
    )
ORDER BY i.sku;
ALTER VIEW inventory.item_summary OWNER TO chiefalry_user;
--
-- Name: item_valuation; Type: TABLE; Schema: inventory; Owner: chiefalry_user
--

CREATE TABLE inventory.item_valuation (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    item_uuid uuid NOT NULL,
    movement_uuid uuid,
    valuation_date timestamp with time zone DEFAULT now(),
    debit_account text,
    credit_account text,
    amount numeric(18, 2) NOT NULL,
    gl_transaction_uuid uuid,
    description text
);
ALTER TABLE inventory.item_valuation OWNER TO chiefalry_user;
--
-- Name: item_valuation_serial_id_seq; Type: SEQUENCE; Schema: inventory; Owner: chiefalry_user
--

CREATE SEQUENCE inventory.item_valuation_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE inventory.item_valuation_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: item_valuation_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: inventory; Owner: chiefalry_user
--

ALTER SEQUENCE inventory.item_valuation_serial_id_seq OWNED BY inventory.item_valuation.serial_id;
--
-- Name: items_serial_id_seq; Type: SEQUENCE; Schema: inventory; Owner: chiefalry_user
--

CREATE SEQUENCE inventory.items_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE inventory.items_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: items_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: inventory; Owner: chiefalry_user
--

ALTER SEQUENCE inventory.items_serial_id_seq OWNED BY inventory.items.serial_id;
--
-- Name: movements; Type: TABLE; Schema: inventory; Owner: chiefalry_user
--

CREATE TABLE inventory.movements (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    item_uuid uuid NOT NULL,
    warehouse_uuid uuid,
    movement_date timestamp with time zone DEFAULT now(),
    reference_type text,
    reference_id bigint,
    quantity numeric(18, 4) NOT NULL,
    unit_cost numeric(18, 4) DEFAULT 0,
    total_cost numeric(18, 2) GENERATED ALWAYS AS ((quantity * unit_cost)) STORED,
    direction text NOT NULL,
    gl_transaction_uuid uuid,
    created_at timestamp with time zone DEFAULT now(),
    CONSTRAINT movements_direction_check CHECK (
        (direction = ANY (ARRAY ['IN'::text, 'OUT'::text]))
    )
);
ALTER TABLE inventory.movements OWNER TO chiefalry_user;
--
-- Name: warehouses; Type: TABLE; Schema: inventory; Owner: chiefalry_user
--

CREATE TABLE inventory.warehouses (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    location text,
    description text,
    created_at timestamp with time zone DEFAULT now()
);
ALTER TABLE inventory.warehouses OWNER TO chiefalry_user;
--
-- Name: movement_history; Type: VIEW; Schema: inventory; Owner: chiefalry_user
--

CREATE VIEW inventory.movement_history AS
SELECT m.serial_id AS movement_serial_id,
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
FROM (
        (
            (
                inventory.movements m
                JOIN inventory.items i ON ((i.uuid = m.item_uuid))
            )
            LEFT JOIN inventory.warehouses w ON ((w.uuid = m.warehouse_uuid))
        )
        LEFT JOIN accounting.transactions t ON ((t.uuid = m.gl_transaction_uuid))
    )
ORDER BY m.movement_date DESC;
ALTER VIEW inventory.movement_history OWNER TO chiefalry_user;
--
-- Name: movements_serial_id_seq; Type: SEQUENCE; Schema: inventory; Owner: chiefalry_user
--

CREATE SEQUENCE inventory.movements_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE inventory.movements_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: movements_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: inventory; Owner: chiefalry_user
--

ALTER SEQUENCE inventory.movements_serial_id_seq OWNED BY inventory.movements.serial_id;
--
-- Name: warehouses_serial_id_seq; Type: SEQUENCE; Schema: inventory; Owner: chiefalry_user
--

CREATE SEQUENCE inventory.warehouses_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE inventory.warehouses_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: warehouses_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: inventory; Owner: chiefalry_user
--

ALTER SEQUENCE inventory.warehouses_serial_id_seq OWNED BY inventory.warehouses.serial_id;
--
-- Name: adjustments serial_id; Type: DEFAULT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.adjustments
ALTER COLUMN serial_id
SET DEFAULT nextval('inventory.adjustments_serial_id_seq'::regclass);
--
-- Name: item_categories serial_id; Type: DEFAULT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.item_categories
ALTER COLUMN serial_id
SET DEFAULT nextval(
        'inventory.item_categories_serial_id_seq'::regclass
    );
--
-- Name: item_valuation serial_id; Type: DEFAULT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.item_valuation
ALTER COLUMN serial_id
SET DEFAULT nextval(
        'inventory.item_valuation_serial_id_seq'::regclass
    );
--
-- Name: items serial_id; Type: DEFAULT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.items
ALTER COLUMN serial_id
SET DEFAULT nextval('inventory.items_serial_id_seq'::regclass);
--
-- Name: movements serial_id; Type: DEFAULT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.movements
ALTER COLUMN serial_id
SET DEFAULT nextval('inventory.movements_serial_id_seq'::regclass);
--
-- Name: warehouses serial_id; Type: DEFAULT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.warehouses
ALTER COLUMN serial_id
SET DEFAULT nextval('inventory.warehouses_serial_id_seq'::regclass);
--
-- Name: adjustments adjustments_adjustment_number_key; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.adjustments
ADD CONSTRAINT adjustments_adjustment_number_key UNIQUE (adjustment_number);
--
-- Name: adjustments adjustments_pkey; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.adjustments
ADD CONSTRAINT adjustments_pkey PRIMARY KEY (uuid);
--
-- Name: adjustments adjustments_serial_id_key; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.adjustments
ADD CONSTRAINT adjustments_serial_id_key UNIQUE (serial_id);
--
-- Name: item_categories item_categories_code_key; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.item_categories
ADD CONSTRAINT item_categories_code_key UNIQUE (code);
--
-- Name: item_categories item_categories_pkey; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.item_categories
ADD CONSTRAINT item_categories_pkey PRIMARY KEY (uuid);
--
-- Name: item_categories item_categories_serial_id_key; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.item_categories
ADD CONSTRAINT item_categories_serial_id_key UNIQUE (serial_id);
--
-- Name: item_valuation item_valuation_pkey; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.item_valuation
ADD CONSTRAINT item_valuation_pkey PRIMARY KEY (uuid);
--
-- Name: item_valuation item_valuation_serial_id_key; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.item_valuation
ADD CONSTRAINT item_valuation_serial_id_key UNIQUE (serial_id);
--
-- Name: items items_pkey; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.items
ADD CONSTRAINT items_pkey PRIMARY KEY (uuid);
--
-- Name: items items_serial_id_key; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.items
ADD CONSTRAINT items_serial_id_key UNIQUE (serial_id);
--
-- Name: items items_sku_key; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.items
ADD CONSTRAINT items_sku_key UNIQUE (sku);
--
-- Name: movements movements_pkey; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.movements
ADD CONSTRAINT movements_pkey PRIMARY KEY (uuid);
--
-- Name: movements movements_serial_id_key; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.movements
ADD CONSTRAINT movements_serial_id_key UNIQUE (serial_id);
--
-- Name: warehouses warehouses_code_key; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.warehouses
ADD CONSTRAINT warehouses_code_key UNIQUE (code);
--
-- Name: warehouses warehouses_pkey; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.warehouses
ADD CONSTRAINT warehouses_pkey PRIMARY KEY (uuid);
--
-- Name: warehouses warehouses_serial_id_key; Type: CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.warehouses
ADD CONSTRAINT warehouses_serial_id_key UNIQUE (serial_id);
--
-- Name: adjustments_adjustment_date_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX adjustments_adjustment_date_idx ON inventory.adjustments USING btree (adjustment_date);
--
-- Name: adjustments_adjustment_number_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX adjustments_adjustment_number_idx ON inventory.adjustments USING btree (adjustment_number);
--
-- Name: adjustments_item_uuid_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX adjustments_item_uuid_idx ON inventory.adjustments USING btree (item_uuid);
--
-- Name: adjustments_posted_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX adjustments_posted_idx ON inventory.adjustments USING btree (posted);
--
-- Name: adjustments_serial_id_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX adjustments_serial_id_idx ON inventory.adjustments USING btree (serial_id);
--
-- Name: adjustments_warehouse_uuid_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX adjustments_warehouse_uuid_idx ON inventory.adjustments USING btree (warehouse_uuid);
--
-- Name: item_categories_code_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX item_categories_code_idx ON inventory.item_categories USING btree (code);
--
-- Name: item_categories_name_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX item_categories_name_idx ON inventory.item_categories USING btree (name);
--
-- Name: item_categories_serial_id_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX item_categories_serial_id_idx ON inventory.item_categories USING btree (serial_id);
--
-- Name: item_valuation_item_uuid_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX item_valuation_item_uuid_idx ON inventory.item_valuation USING btree (item_uuid);
--
-- Name: item_valuation_movement_uuid_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX item_valuation_movement_uuid_idx ON inventory.item_valuation USING btree (movement_uuid);
--
-- Name: item_valuation_serial_id_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX item_valuation_serial_id_idx ON inventory.item_valuation USING btree (serial_id);
--
-- Name: item_valuation_valuation_date_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX item_valuation_valuation_date_idx ON inventory.item_valuation USING btree (valuation_date);
--
-- Name: items_category_uuid_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX items_category_uuid_idx ON inventory.items USING btree (category_uuid);
--
-- Name: items_name_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX items_name_idx ON inventory.items USING btree (name);
--
-- Name: items_quantity_on_hand_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX items_quantity_on_hand_idx ON inventory.items USING btree (quantity_on_hand);
--
-- Name: items_serial_id_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX items_serial_id_idx ON inventory.items USING btree (serial_id);
--
-- Name: items_sku_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX items_sku_idx ON inventory.items USING btree (sku);
--
-- Name: items_status_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX items_status_idx ON inventory.items USING btree (status);
--
-- Name: movements_direction_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX movements_direction_idx ON inventory.movements USING btree (direction);
--
-- Name: movements_item_uuid_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX movements_item_uuid_idx ON inventory.movements USING btree (item_uuid);
--
-- Name: movements_movement_date_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX movements_movement_date_idx ON inventory.movements USING btree (movement_date);
--
-- Name: movements_reference_type_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX movements_reference_type_idx ON inventory.movements USING btree (reference_type);
--
-- Name: movements_serial_id_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX movements_serial_id_idx ON inventory.movements USING btree (serial_id);
--
-- Name: movements_warehouse_uuid_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX movements_warehouse_uuid_idx ON inventory.movements USING btree (warehouse_uuid);
--
-- Name: warehouses_code_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX warehouses_code_idx ON inventory.warehouses USING btree (code);
--
-- Name: warehouses_name_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX warehouses_name_idx ON inventory.warehouses USING btree (name);
--
-- Name: warehouses_serial_id_idx; Type: INDEX; Schema: inventory; Owner: chiefalry_user
--

CREATE INDEX warehouses_serial_id_idx ON inventory.warehouses USING btree (serial_id);
--
-- Name: adjustments adjustments_gl_transaction_uuid_fkey; Type: FK CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.adjustments
ADD CONSTRAINT adjustments_gl_transaction_uuid_fkey FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid);
--
-- Name: adjustments adjustments_item_uuid_fkey; Type: FK CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.adjustments
ADD CONSTRAINT adjustments_item_uuid_fkey FOREIGN KEY (item_uuid) REFERENCES inventory.items(uuid);
--
-- Name: adjustments adjustments_warehouse_uuid_fkey; Type: FK CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.adjustments
ADD CONSTRAINT adjustments_warehouse_uuid_fkey FOREIGN KEY (warehouse_uuid) REFERENCES inventory.warehouses(uuid);
--
-- Name: item_valuation item_valuation_gl_transaction_uuid_fkey; Type: FK CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.item_valuation
ADD CONSTRAINT item_valuation_gl_transaction_uuid_fkey FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid);
--
-- Name: item_valuation item_valuation_item_uuid_fkey; Type: FK CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.item_valuation
ADD CONSTRAINT item_valuation_item_uuid_fkey FOREIGN KEY (item_uuid) REFERENCES inventory.items(uuid);
--
-- Name: item_valuation item_valuation_movement_uuid_fkey; Type: FK CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.item_valuation
ADD CONSTRAINT item_valuation_movement_uuid_fkey FOREIGN KEY (movement_uuid) REFERENCES inventory.movements(uuid);
--
-- Name: items items_category_uuid_fkey; Type: FK CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.items
ADD CONSTRAINT items_category_uuid_fkey FOREIGN KEY (category_uuid) REFERENCES inventory.item_categories(uuid) ON DELETE
SET NULL;
--
-- Name: movements movements_gl_transaction_uuid_fkey; Type: FK CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.movements
ADD CONSTRAINT movements_gl_transaction_uuid_fkey FOREIGN KEY (gl_transaction_uuid) REFERENCES accounting.transactions(uuid);
--
-- Name: movements movements_item_uuid_fkey; Type: FK CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.movements
ADD CONSTRAINT movements_item_uuid_fkey FOREIGN KEY (item_uuid) REFERENCES inventory.items(uuid) ON DELETE RESTRICT;
--
-- Name: movements movements_warehouse_uuid_fkey; Type: FK CONSTRAINT; Schema: inventory; Owner: chiefalry_user
--

ALTER TABLE ONLY inventory.movements
ADD CONSTRAINT movements_warehouse_uuid_fkey FOREIGN KEY (warehouse_uuid) REFERENCES inventory.warehouses(uuid) ON DELETE RESTRICT;
--
-- PostgreSQL database dump complete
--