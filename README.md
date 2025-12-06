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

curl -G http://localhost:8080/api/transactions/list \
  -H "Authorization: Bearer YOUR_JWT_HERE" | jq

# Get a transaction ID from /list first
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/transactions/list

# Then test:
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/transactions/550e8400-e29b-41d4-a716-446655440000

# Invalid URI TEST → 404 with JSON
curl -i http://localhost:8080/api/whatever

# Get Backend Server Log
  RUST_LOG=info cargo run

# Create a transaction (unposted)

```json
curl -X POST http://localhost:8080/api/
transactions/create \
  -H "Authorization: Bearer YOUR_JWT_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "txn_date": "2025-04-01",
    "reference": "TEST-001",
    "description": "Test sale - draft",
    "module": "journal",
    "posted": false,
    "lines": [
      {
        "account_uuid": "11111111-1111-1111-1111-111111111111",
        "debit": "500.00",
        "credit": "0.00",
        "memo": "Cash received"
      },
      {
        "account_uuid": "22222222-2222-2222-2222-222222222222",
        "debit": "0.00",
        "credit": "500.00",
        "memo": "Sales revenue"
      }
    ]
  }' | jq
```

`curl -sX POST http://localhost:8080/api/transactions/create -H\ "Authorization : Bearer $TOKEN" -H
 "Content-Type: application/json" -d '{
  "txn_date": "2025-02-28",
  "reference": "PO-980",
  "description": "Investing money and property into business by owners.",
  "posted": false,
  "lines": [
    { "account_uuid": "019ae5bb-9664-7d64-b478-2ce17f26069d", "debit": "122000.00", "credit": "0", "memo": "Cash at Bank" }, {"account_uuid": "019ae5
bb-9ac7-7943-aa2a-9097ed62f516", "debit": "33000.00", "credit":"0", "memo": "Land"}, { "account_uuid": "019ae5bb-a229-79ae-b5f4-0354b80ef94e", "debit
": "0", "credit": "155000.00", "memo": "Shareholder's net ownership" }
  ]}' | jq`

`curl -X POST http://localhost:8080/api/transactions/create -H "$AUTH" -H "Content-Type: application/json" -d '{
  "txn_date": "2025-12-05",
  "reference": "PO-500",
  "description": "Inventory purchase from Supplier Ltd",
  "posted": true,
  "lines": [
    { "account_uuid": "44444444-4444-4444-4444-444444444444", "debit": "3000.00", "credit": "0",     "memo": "Inventory" },
    { "account_uuid": "55555555-5555-5555-5555-555555555555", "debit": "0",      "credit": "3000.00", "memo": "Accounts Payable" }
  ]
}' | jq`

# Create a transaction (posted)
`curl -X POST http://localhost:3000/accounting/create -H "$AUTH" -H "Content-Type: application/json" -d '{
  "txn_date": "2025-12-02",
  "reference": "CASH-001",
  "description": "Cash sale to walk-in customer",
  "module": "sales",
  "posted": true,
  "lines": [
    { "account_uuid": "11111111-1111-1111-1111-111111111111", "debit": "1200.00", "credit": "0",    "memo": "Cash received" },
    { "account_uuid": "66666666-6666-6666-6666-666666666666", "debit": "0",      "credit": "1200.00", "memo": "Sales revenue" }
  ]
}' | jq
`

# Post transactions - makes transactions immutable
curl -X PUT http://localhost:8080/api/transactions/post/THE_UUID \
  -H "Authorization: Bearer YOUR_JWT_HERE" | jq

# Try to update a posted transaction - possible before posting
curl -X PUT http://localhost:8080/api/transactions/update/NEW_UUID_HERE \
  -H "Authorization: Bearer YOUR_JWT_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Updated description - still draft!",
    "lines": [
      { "account_uuid": "11111111-1111-1111-1111-111111111111", "debit": "1500.00", "credit": "0.00", "memo": "More cash" },
      { "account_uuid": "22222222-2222-2222-2222-222222222222", "debit": "0.00", "credit": "1500.00", "memo": "More revenue" }
    ]
  }' | jq

# Delete an unposted transaction
curl -X DELETE http://localhost:8080/api/transactions/delete/NEW_UUID_HERE \
  -H "Authorization: Bearer YOUR_JWT_HERE" | jq

# Make posted transaction void
curl -X POST http://localhost:8080/api/transactions/void/THE_UUID \
  -H "Authorization: Bearer YOUR_JWT_HERE" \
  -H "Content-Type: application/json" \
  -d '{"reason": "Customer returned the goods"}' | jq

# Check balance of transaction
`curl -G http://localhost:8080/api/transactions/balance/UUID \
  -H "Authorization: Bearer YOUR_JWT_HERE" | jq`