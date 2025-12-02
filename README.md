# Chiefalry Accountant – Backend Server

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-316192?style=for-the-badge&logo=postgresql&logoColor=white)
![Axum](https://img.shields.io/badge/Axum-000000?style=for-the-badge&logo=rust&logoColor=white)
![SQLx](https://img.shields.io/badge/SQLx-000000?style=for-the-badge&logo=rust&logoColor=white)

---

## Overview

**Chiefalry Accountant** is a **high-performance, secure, and scalable backend** for a full-featured **double-entry accounting system**, built in **Rust** using **Axum**, **SQLx**, and **PostgreSQL**.

Designed to power a **Flutter-based desktop application**, this backend delivers a **modern alternative to QuickBooks and Tally**, targeting **Small and Medium Enterprises (SMEs)** and **growing businesses** with straightforward accounting needs.

> **MVP Goal**: A **standalone, offline-capable**, **cost-effective**, and **reliable** accounting suite — no cloud lock-in, no subscriptions, no complexity.

---

## Core Features (Current & Planned)

| Feature                        | Status      | Description                                     |
| ------------------------------ | ----------- | ----------------------------------------------- |
| User Auth (Sign up / Login)    | Done        | JWT-based secure authentication                 |
| Chart of Accounts (CRUD)       | Done        | Asset, Liability, Equity, Revenue, Expense      |
| Transactions (Journal Entries) | In Progress | Double-entry with debit/credit validation       |
| Company Management             | Planned     | Multi-company support                           |
| Customers & Vendors            | Planned     | Contacts, invoices, payments                    |
| Financial Reports              | Planned     | Balance Sheet, P&L, Trial Balance               |
| AI-Powered Insights            | Future      | Forecasting, anomaly detection, recommendations |

---

## Tech Stack

| Layer        | Technology                                            |
| ------------ | ----------------------------------------------------- |
| Language     | **Rust** (safe, fast, zero-cost abstractions)         |
| Framework    | **Axum** (async, modular, production-ready)           |
| Database     | **PostgreSQL** (`NUMERIC` for money, ACID compliance) |
| ORM/Query    | **SQLx** (type-safe, compile-time checked)            |
| Auth         | **JWT + Argon2**                                      |
| Decimal Math | **rust_decimal** (no floating-point errors)           |
| Frontend     | **Flutter Desktop** (Windows, macOS, Linux)           |

---

## Why Rust?

- **Financial Accuracy**: No floating-point errors (`f64`) → `rust_decimal::Decimal`
- **Performance**: Blazing fast transaction processing
- **Safety**: Memory safety without GC
- **Reliability**: Compile-time SQL checks, zero runtime crashes
- **Future-Proof**: Ready for AI/ML integration (via Python or ONNX)

---

## API Endpoints (v1)

Base URL: `http://localhost:8080/api`

### Authentication
```
POST   /auth/signup
POST   /auth/login
POST   /auth/refresh
```

### Accounts (Chart of Accounts)
```
POST   /accounts/create
GET    /accounts/list
GET    /accounts/{id}
PATCH  /accounts/{id}
DELETE /accounts/{id}
```

### Transactions (Planned)
```
POST   /transactions
GET    /transactions
GET    /transactions/{id}
PATCH  /transactions/{id}
DELETE /transactions/{id}
```

### Companies (Planned)
```
POST   /companies
GET    /companies
GET    /companies/{id}
PATCH  /companies/{id}
```

### Customers & Vendors (Planned)
```
POST   /customers
POST   /vendors
GET    /customers
GET    /vendors
```

### Reports (Planned)
```
GET    /reports/balance-sheet
GET    /reports/profit-loss
GET    /reports/trial-balance
GET    /reports/ledger/{account_id}
```

---

## Running Locally

### Prerequisites
- Rust (`rustup stable`)
- PostgreSQL 15+
- Flutter (for frontend)

### Setup

```bash
# 1. Clone & enter
git clone https://github.com/yourorg/chiefalry-accountant-backend.git
cd chiefalry-accountant-backend

# 2. Set environment
cp .env.example .env
# Edit .env → DATABASE_URL=postgres://user:pass@localhost/chiefalry

# 3. Run migrations
sqlx migrate run

# 4. Start server
cargo run --release
```

Server runs on: [**http://localhost:8080**](http://localhost:8080)

---

## Database Schema (Key Tables)

```sql
users (id, email, password_hash, created_at)
accounts (id, name, type, balance NUMERIC, user_id, created_at)
transactions (id, date, description, user_id, created_at)
transaction_lines (id, transaction_id, account_id, debit, credit)
companies (id, name, fiscal_year_start, user_id)
customers (id, name, email, phone, company_id)
vendors (id, name, email, phone, company_id)
```

> All money fields use `NUMERIC` + `rust_decimal::Decimal`

---

## Security

- **Password Hashing**: Argon2id
- **JWT Tokens**: Short-lived + refresh
- **Input Validation**: Service-layer checks
- **SQL Injection**: Impossible (SQLx compile-time queries)
- **Rate Limiting**: Planned (Tower middleware)

---

## Testing

```bash
# Unit + Integration
cargo test

# With database
DATABASE_URL=postgres://... cargo test -- --include-ignored
```

Mockable repository traits → easy unit testing.

---

## Roadmap

| Phase    | Features                                           |
| -------- | -------------------------------------------------- |
| **MVP**  | Auth, Accounts, Transactions, Basic Reports        |
| **v1.0** | Multi-company, Customers/Vendors, Invoices         |
| **v2.0** | Import/Export (CSV, QIF), Audit Log                |
| **v3.0** | AI Recommendations, Anomaly Detection, Forecasting |

---

## Contributing

We welcome contributions! See [`CONTRIBUTING.md`](./CONTRIBUTING.md)

1. Fork
2. Create feature branch
3. Write tests
4. Open PR

---

## License

```
MIT License
```

---

## Contact

**Chiefalry** – *Accounting, Simplified.*

- Website: [https://chiefalry.com](https://chiefalry.com) *(coming soon)*
- Twitter: [@chiefalry](https://twitter.com/chiefalry)
- Email: dev@chiefalry.com

---

> **"Accounting doesn’t have to be expensive or complicated.
> Chiefalry brings enterprise-grade accuracy to every small business."**

---

*Built with love in Rust. Deployed for real businesses.*

## Usefull Routes
# Sign Up
curl -X POST http://localhost:8080/api/auth/register -d \
    '{"email":"a@b.c","password":"123"}' -H "Content-Type: application/json"

# Login
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "zeus@olympus.com", "password": "thunderbolt123"}' \
  | jq -r '.data')

# Create Account
curl -X POST http://localhost:8080/api/accounts/create \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name": "Cash", "type_": "asset"}'

# List Accounts
```python
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/accounts/list
```

# Update
curl -X PATCH http://localhost:8080/api/accounts/<id> \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name": "Petty Cash", "type_": "asset"}'

# Delete
```python
curl -X DELETE http://localhost:8080/api/accounts/<id> \
  -H "Authorization: Bearer $TOKEN"
```

# Create
curl -X POST http://localhost:8080/api/transactions/create \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "description": "Coffee",
    "debit_account_id": "a1b2c3d4-e5f6-7890-g1h2-i3j4k5l6m7n8",
    "credit_account_id": "b2c3d4e5-f6g7-8901-h2i3-j4k5l6m7n8o9",
    "amount": 5.50
  }' | jq

# List
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/transactions/list | jq

# Get a transaction ID from /list first
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/transactions/list

# Then test:
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/transactions/550e8400-e29b-41d4-a716-446655440000

# Invalid URI TEST → 404 with JSON
curl -i http://localhost:8080/api/whatever

# Get Backend Server Log
  RUST_LOG=info cargo run