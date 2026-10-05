# Server up first
cargo run

# One phase
cargo test --test e2e_01_onboarding -- --nocapture
cargo test --test e2e_02_entities -- --nocapture
cargo test --test e2e_03_capital -- --nocapture

# All e2e (ignored ones skip until you remove #[ignore])
cargo test --test e2e_ -- --nocapture

cargo test --test e2e_01_onboarding -- --nocapture && \
cargo test --test e2e_02_setup_stakeholders -- --nocapture && \
cargo test --test e2e_02b_api_surface -- --nocapture && \
cargo test --test e2e_03_raise_capital -- --nocapture && \
cargo test --test e2e_04_opening_balances -- --nocapture && \
cargo test --test e2e_09_financial_statements -- --nocapture