# Server up first
cargo run

# One phase
cargo test --test e2e_01_onboarding -- --nocapture
cargo test --test e2e_02_entities -- --nocapture
cargo test --test e2e_03_capital -- --nocapture

# All e2e (ignored ones skip until you remove #[ignore])
cargo test --test e2e_ -- --nocapture