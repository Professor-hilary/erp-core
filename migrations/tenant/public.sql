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
-- Name: public; Type: SCHEMA; Schema: -; Owner: chiefalry_user
--

CREATE SCHEMA public;
ALTER SCHEMA public OWNER TO chiefalry_user;
--
-- Name: SCHEMA public; Type: COMMENT; Schema: -; Owner: chiefalry_user
--

COMMENT ON SCHEMA public IS 'standard public schema';
SET default_tablespace = '';
SET default_table_access_method = heap;
--
-- Name: accounts; Type: TABLE; Schema: public; Owner: chiefalry_user
--

CREATE TABLE public.accounts (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    name character varying NOT NULL,
    type character varying NOT NULL,
    balance numeric(15, 2) DEFAULT 0.00,
    user_id uuid,
    created_at timestamp with time zone DEFAULT now(),
    CONSTRAINT accounts_type_check CHECK (
        (
            (type)::text = ANY (
                (
                    ARRAY ['asset'::character varying, 'liability'::character varying, 'equity'::character varying, 'revenue'::character varying, 'expense'::character varying]
                )::text []
            )
        )
    )
);
ALTER TABLE public.accounts OWNER TO chiefalry_user;
--
-- Name: transactions; Type: TABLE; Schema: public; Owner: chiefalry_user
--

CREATE TABLE public.transactions (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    description text,
    debit_account_id uuid,
    credit_account_id uuid,
    amount numeric(15, 2) NOT NULL,
    date date DEFAULT CURRENT_DATE NOT NULL,
    user_id uuid,
    created_at timestamp with time zone DEFAULT now(),
    CONSTRAINT transactions_amount_check CHECK ((amount > (0)::numeric))
);
ALTER TABLE public.transactions OWNER TO chiefalry_user;
--
-- Name: users; Type: TABLE; Schema: public; Owner: chiefalry_user
--

CREATE TABLE public.users (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    email character varying NOT NULL,
    password_hash character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now()
);
ALTER TABLE public.users OWNER TO chiefalry_user;
--
-- Name: accounts accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: chiefalry_user
--

ALTER TABLE ONLY public.accounts
ADD CONSTRAINT accounts_pkey PRIMARY KEY (id);
--
-- Name: transactions transactions_pkey; Type: CONSTRAINT; Schema: public; Owner: chiefalry_user
--

ALTER TABLE ONLY public.transactions
ADD CONSTRAINT transactions_pkey PRIMARY KEY (id);
--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: chiefalry_user
--

ALTER TABLE ONLY public.users
ADD CONSTRAINT users_email_key UNIQUE (email);
--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: chiefalry_user
--

ALTER TABLE ONLY public.users
ADD CONSTRAINT users_pkey PRIMARY KEY (id);
--
-- Name: idx_accounts_user; Type: INDEX; Schema: public; Owner: chiefalry_user
--

CREATE INDEX idx_accounts_user ON public.accounts USING btree (user_id);
--
-- Name: idx_transactions_user_date; Type: INDEX; Schema: public; Owner: chiefalry_user
--

CREATE INDEX idx_transactions_user_date ON public.transactions USING btree (user_id, date);
--
-- Name: accounts accounts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: chiefalry_user
--

ALTER TABLE ONLY public.accounts
ADD CONSTRAINT accounts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
--
-- Name: transactions transactions_credit_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: chiefalry_user
--

ALTER TABLE ONLY public.transactions
ADD CONSTRAINT transactions_credit_account_id_fkey FOREIGN KEY (credit_account_id) REFERENCES public.accounts(id) ON DELETE CASCADE;
--
-- Name: transactions transactions_debit_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: chiefalry_user
--

ALTER TABLE ONLY public.transactions
ADD CONSTRAINT transactions_debit_account_id_fkey FOREIGN KEY (debit_account_id) REFERENCES public.accounts(id) ON DELETE CASCADE;
--
-- Name: transactions transactions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: chiefalry_user
--

ALTER TABLE ONLY public.transactions
ADD CONSTRAINT transactions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
--
-- PostgreSQL database dump complete
--