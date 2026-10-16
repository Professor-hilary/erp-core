# Server up first
cargo run

# One phase
cargo test --test e2e_01_onboarding -- --nocapture
cargo test --test e2e_02_setup_stakeholders -- --nocapture
cargo test --test e2e_02b_api_surface -- --nocapture
cargo test --test e2e_03_raise_capital -- --nocapture
cargo test --test e2e_04_opening_balances -- --nocapture
cargo test --test e2e_09_financial_statements -- --nocapture

# All e2e (ignored ones skip until you remove #[ignore])
cargo test --tests -- --nocapture --test-threads=1

# Delete test roles and dbs
SELECT format('DROP DATABASE IF EXISTS %I WITH (FORCE);', datname) FROM pg_database WHERE datname LIKE 'tenant_test%'\gexec
SELECT format('REASSIGN OWNED BY %I TO postgres; DROP OWNED BY %I', rolname, rolname) FROM pg_roles WHERE rolname LIKE 'role%' ESCAPE '\'\gexec
SELECT format('DROP ROLE IF EXISTS %I;', rolname) FROM pg_roles WHERE rolname LIKE 'role%' ESCAPE '\'\gexec
