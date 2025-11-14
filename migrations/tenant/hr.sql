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
-- Name: hr; Type: SCHEMA; Schema: -; Owner: chiefalry_user
--

CREATE SCHEMA hr;
ALTER SCHEMA hr OWNER TO chiefalry_user;
SET default_tablespace = '';
SET default_table_access_method = heap;
--
-- Name: departments; Type: TABLE; Schema: hr; Owner: chiefalry_user
--

CREATE TABLE hr.departments (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    name character varying(100) NOT NULL,
    description text,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);
ALTER TABLE hr.departments OWNER TO chiefalry_user;
--
-- Name: departments_serial_id_seq; Type: SEQUENCE; Schema: hr; Owner: chiefalry_user
--

CREATE SEQUENCE hr.departments_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE hr.departments_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: departments_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: hr; Owner: chiefalry_user
--

ALTER SEQUENCE hr.departments_serial_id_seq OWNED BY hr.departments.serial_id;
--
-- Name: employees; Type: TABLE; Schema: hr; Owner: chiefalry_user
--

CREATE TABLE hr.employees (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    first_name character varying(50) NOT NULL,
    last_name character varying(50) NOT NULL,
    email character varying(100),
    phone_number character varying(20),
    hire_date date NOT NULL,
    termination_date date,
    job_title character varying(100),
    department_uuid uuid,
    supervisor_uuid uuid,
    employment_type character varying(20),
    salary numeric(14, 2),
    pay_frequency character varying(20),
    status character varying(20) DEFAULT 'Active'::character varying,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now(),
    CONSTRAINT employees_employment_type_check CHECK (
        (
            (employment_type)::text = ANY (
                (
                    ARRAY ['Full-time'::character varying, 'Part-time'::character varying, 'Contract'::character varying]
                )::text []
            )
        )
    ),
    CONSTRAINT employees_pay_frequency_check CHECK (
        (
            (pay_frequency)::text = ANY (
                (
                    ARRAY ['Monthly'::character varying, 'Weekly'::character varying, 'Bi-weekly'::character varying]
                )::text []
            )
        )
    )
);
ALTER TABLE hr.employees OWNER TO chiefalry_user;
--
-- Name: employees_serial_id_seq; Type: SEQUENCE; Schema: hr; Owner: chiefalry_user
--

CREATE SEQUENCE hr.employees_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE hr.employees_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: employees_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: hr; Owner: chiefalry_user
--

ALTER SEQUENCE hr.employees_serial_id_seq OWNED BY hr.employees.serial_id;
--
-- Name: job_titles; Type: TABLE; Schema: hr; Owner: chiefalry_user
--

CREATE TABLE hr.job_titles (
    uuid uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    serial_id bigint NOT NULL,
    title character varying(100) NOT NULL,
    description text,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);
ALTER TABLE hr.job_titles OWNER TO chiefalry_user;
--
-- Name: job_titles_serial_id_seq; Type: SEQUENCE; Schema: hr; Owner: chiefalry_user
--

CREATE SEQUENCE hr.job_titles_serial_id_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE hr.job_titles_serial_id_seq OWNER TO chiefalry_user;
--
-- Name: job_titles_serial_id_seq; Type: SEQUENCE OWNED BY; Schema: hr; Owner: chiefalry_user
--

ALTER SEQUENCE hr.job_titles_serial_id_seq OWNED BY hr.job_titles.serial_id;
--
-- Name: departments serial_id; Type: DEFAULT; Schema: hr; Owner: chiefalry_user
--

ALTER TABLE ONLY hr.departments
ALTER COLUMN serial_id
SET DEFAULT nextval('hr.departments_serial_id_seq'::regclass);
--
-- Name: employees serial_id; Type: DEFAULT; Schema: hr; Owner: chiefalry_user
--

ALTER TABLE ONLY hr.employees
ALTER COLUMN serial_id
SET DEFAULT nextval('hr.employees_serial_id_seq'::regclass);
--
-- Name: job_titles serial_id; Type: DEFAULT; Schema: hr; Owner: chiefalry_user
--

ALTER TABLE ONLY hr.job_titles
ALTER COLUMN serial_id
SET DEFAULT nextval('hr.job_titles_serial_id_seq'::regclass);
--
-- Name: departments departments_pkey; Type: CONSTRAINT; Schema: hr; Owner: chiefalry_user
--

ALTER TABLE ONLY hr.departments
ADD CONSTRAINT departments_pkey PRIMARY KEY (uuid);
--
-- Name: departments departments_serial_id_key; Type: CONSTRAINT; Schema: hr; Owner: chiefalry_user
--

ALTER TABLE ONLY hr.departments
ADD CONSTRAINT departments_serial_id_key UNIQUE (serial_id);
--
-- Name: employees employees_email_key; Type: CONSTRAINT; Schema: hr; Owner: chiefalry_user
--

ALTER TABLE ONLY hr.employees
ADD CONSTRAINT employees_email_key UNIQUE (email);
--
-- Name: employees employees_pkey; Type: CONSTRAINT; Schema: hr; Owner: chiefalry_user
--

ALTER TABLE ONLY hr.employees
ADD CONSTRAINT employees_pkey PRIMARY KEY (uuid);
--
-- Name: employees employees_serial_id_key; Type: CONSTRAINT; Schema: hr; Owner: chiefalry_user
--

ALTER TABLE ONLY hr.employees
ADD CONSTRAINT employees_serial_id_key UNIQUE (serial_id);
--
-- Name: job_titles job_titles_pkey; Type: CONSTRAINT; Schema: hr; Owner: chiefalry_user
--

ALTER TABLE ONLY hr.job_titles
ADD CONSTRAINT job_titles_pkey PRIMARY KEY (uuid);
--
-- Name: job_titles job_titles_serial_id_key; Type: CONSTRAINT; Schema: hr; Owner: chiefalry_user
--

ALTER TABLE ONLY hr.job_titles
ADD CONSTRAINT job_titles_serial_id_key UNIQUE (serial_id);
--
-- Name: departments_name_idx; Type: INDEX; Schema: hr; Owner: chiefalry_user
--

CREATE INDEX departments_name_idx ON hr.departments USING btree (name);
--
-- Name: departments_serial_id_idx; Type: INDEX; Schema: hr; Owner: chiefalry_user
--

CREATE INDEX departments_serial_id_idx ON hr.departments USING btree (serial_id);
--
-- Name: employees_department_uuid_idx; Type: INDEX; Schema: hr; Owner: chiefalry_user
--

CREATE INDEX employees_department_uuid_idx ON hr.employees USING btree (department_uuid);
--
-- Name: employees_email_idx; Type: INDEX; Schema: hr; Owner: chiefalry_user
--

CREATE INDEX employees_email_idx ON hr.employees USING btree (email);
--
-- Name: employees_last_name_first_name_idx; Type: INDEX; Schema: hr; Owner: chiefalry_user
--

CREATE INDEX employees_last_name_first_name_idx ON hr.employees USING btree (last_name, first_name);
--
-- Name: employees_serial_id_idx; Type: INDEX; Schema: hr; Owner: chiefalry_user
--

CREATE INDEX employees_serial_id_idx ON hr.employees USING btree (serial_id);
--
-- Name: employees_status_idx; Type: INDEX; Schema: hr; Owner: chiefalry_user
--

CREATE INDEX employees_status_idx ON hr.employees USING btree (status);
--
-- Name: job_titles_serial_id_idx; Type: INDEX; Schema: hr; Owner: chiefalry_user
--

CREATE INDEX job_titles_serial_id_idx ON hr.job_titles USING btree (serial_id);
--
-- Name: job_titles_title_idx; Type: INDEX; Schema: hr; Owner: chiefalry_user
--

CREATE INDEX job_titles_title_idx ON hr.job_titles USING btree (title);
--
-- Name: employees employees_department_uuid_fkey; Type: FK CONSTRAINT; Schema: hr; Owner: chiefalry_user
--

ALTER TABLE ONLY hr.employees
ADD CONSTRAINT employees_department_uuid_fkey FOREIGN KEY (department_uuid) REFERENCES hr.departments(uuid) ON DELETE
SET NULL;
--
-- Name: employees employees_supervisor_uuid_fkey; Type: FK CONSTRAINT; Schema: hr; Owner: chiefalry_user
--

ALTER TABLE ONLY hr.employees
ADD CONSTRAINT employees_supervisor_uuid_fkey FOREIGN KEY (supervisor_uuid) REFERENCES hr.employees(uuid) ON DELETE
SET NULL;
--
-- PostgreSQL database dump complete
--