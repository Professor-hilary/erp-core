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
-- Name: system; Type: SCHEMA; Schema: -; Owner: chiefalry_user
--

CREATE SCHEMA system;
ALTER SCHEMA system OWNER TO chiefalry_user;
--
-- Name: audit_trigger_fn(); Type: FUNCTION; Schema: system; Owner: chiefalry_user
--

CREATE FUNCTION system.audit_trigger_fn() RETURNS trigger LANGUAGE plpgsql AS $_$
DECLARE v_record_uuid UUID;
v_record_serial_id BIGINT;
v_changed_by BIGINT;
BEGIN -- Get current user from app context (set via SET app.current_user_id = 123;)
BEGIN v_changed_by := NULLIF(current_setting('app.current_user_id'), '')::BIGINT;
EXCEPTION
WHEN OTHERS THEN v_changed_by := NULL;
END;
-- Extract uuid and serial_id from NEW or OLD
IF TG_OP = 'DELETE' THEN v_record_uuid := (OLD.*)::uuid;
v_record_serial_id := (OLD.*)::jsonb->>'serial_id';
ELSE v_record_uuid := (NEW.*)::uuid;
v_record_serial_id := (NEW.*)::jsonb->>'serial_id';
END IF;
-- Fallback: try direct column access if casting fails
IF v_record_uuid IS NULL THEN EXECUTE format('SELECT $1.%I FROM (SELECT $1.*) s', 'uuid') USING (
    CASE
        WHEN TG_OP = 'DELETE' THEN OLD
        ELSE NEW
    END
) INTO v_record_uuid;
END IF;
IF v_record_serial_id IS NULL THEN EXECUTE format('SELECT $1.%I FROM (SELECT $1.*) s', 'serial_id') USING (
    CASE
        WHEN TG_OP = 'DELETE' THEN OLD
        ELSE NEW
    END
) INTO v_record_serial_id;
END IF;
-- Insert audit record
IF TG_OP = 'INSERT' THEN
INSERT INTO system.audit_logs(
        schema_name,
        table_name,
        record_uuid,
        record_serial_id,
        operation,
        changed_by,
        new_data
    )
VALUES (
        TG_TABLE_SCHEMA,
        TG_TABLE_NAME,
        v_record_uuid,
        v_record_serial_id,
        TG_OP,
        v_changed_by,
        to_jsonb(NEW)
    );
ELSIF TG_OP = 'UPDATE' THEN
INSERT INTO system.audit_logs(
        schema_name,
        table_name,
        record_uuid,
        record_serial_id,
        operation,
        changed_by,
        old_data,
        new_data
    )
VALUES (
        TG_TABLE_SCHEMA,
        TG_TABLE_NAME,
        v_record_uuid,
        v_record_serial_id,
        TG_OP,
        v_changed_by,
        to_jsonb(OLD),
        to_jsonb(NEW)
    );
ELSIF TG_OP = 'DELETE' THEN
INSERT INTO system.audit_logs(
        schema_name,
        table_name,
        record_uuid,
        record_serial_id,
        operation,
        changed_by,
        old_data
    )
VALUES (
        TG_TABLE_SCHEMA,
        TG_TABLE_NAME,
        v_record_uuid,
        v_record_serial_id,
        TG_OP,
        v_changed_by,
        to_jsonb(OLD)
    );
END IF;
RETURN NULL;
END;
$_$;
ALTER FUNCTION system.audit_trigger_fn() OWNER TO chiefalry_user;
SET default_tablespace = '';
SET default_table_access_method = heap;
--
-- Name: audit_logs; Type: TABLE; Schema: system; Owner: chiefalry_user
--

CREATE TABLE system.audit_logs (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    schema_name text NOT NULL,
    table_name text NOT NULL,
    record_uuid uuid NOT NULL,
    record_serial_id bigint,
    operation text NOT NULL,
    changed_by bigint,
    changed_at timestamp with time zone DEFAULT now(),
    old_data jsonb,
    new_data jsonb
);
ALTER TABLE system.audit_logs OWNER TO chiefalry_user;
--
-- Name: audit_logs_serial_id_seq; Type: SEQUENCE; Schema: system; Owner: chiefalry_user
--

CREATE SEQUENCE system.audit_logs_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE system.audit_logs_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: audit_logs_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: system; Owner: chiefalry_user
--

ALTER SEQUENCE system.audit_logs_serial_id_seq OWNED BY system.audit_logs.serial_id;
--
-- Name: audit_logs serial_id; Type: DEFAULT; Schema: system; Owner: chiefalry_user
--

ALTER TABLE ONLY system.audit_logs
ALTER COLUMN serial_id
SET DEFAULT nextval('system.audit_logs_serial_id_seq'::regclass);
--
-- Name: audit_logs audit_logs_pkey; Type: CONSTRAINT; Schema: system; Owner: chiefalry_user
--

ALTER TABLE ONLY system.audit_logs
ADD CONSTRAINT audit_logs_pkey PRIMARY KEY (uuid);
--
-- Name: audit_logs audit_logs_serial_id_key; Type: CONSTRAINT; Schema: system; Owner: chiefalry_user
--

ALTER TABLE ONLY system.audit_logs
ADD CONSTRAINT audit_logs_serial_id_key UNIQUE (serial_id);
--
-- PostgreSQL database dump complete
--