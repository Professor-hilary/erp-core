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
```python
curl -X POST http://localhost:8080/api/auth/register -d \
    '{"email":"a@b.c","password":"123"}' -H "Content-Type: application/json"
```

# Login
```python
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "zeus@olympus.com", "password": "thunderbolt123"}' \
  | jq -r '.data')
```

# Create Account
```python
curl -X POST http://localhost:8080/api/accounts/create \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name": "Cash", "type_": "asset"}'
```

# List Accounts
```python
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/accounts/list
```

# Update
```python
curl -X PATCH http://localhost:8080/api/accounts/<id> \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name": "Petty Cash", "type_": "asset"}'
```

# Delete
```python
curl -X DELETE http://localhost:8080/api/accounts/<id> \
  -H "Authorization: Bearer $TOKEN"
```

# Create
```python
curl -X POST http://localhost:8080/api/transactions/create \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "description": "Coffee",
    "debit_account_id": "a1b2c3d4-e5f6-7890-g1h2-i3j4k5l6m7n8",
    "credit_account_id": "b2c3d4e5-f6g7-8901-h2i3-j4k5l6m7n8o9",
    "amount": 5.50
  }' | jq
```
# List
```python
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/transactions/list | jq

curl -G http://localhost:8080/api/transactions/list \
  -H "Authorization: Bearer YOUR_JWT_HERE" | jq
```

# Get a transaction ID from /list first
```python
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/transactions/list
```
# Then test:
```python
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/transactions/550e8400-e29b-41d4-a716-446655440000
```
# Invalid URI TEST → 404 with JSON
```python
curl -i http://localhost:8080/api/whatever
```

# Get Backend Server Log
  ```python
RUST_LOG=info cargo run
```

# Create a transaction (unposted)

```python
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

```python
curl -sX POST http://localhost:8080/api/transactions/create -H\ "Authorization : Bearer $TOKEN" -H
 "Content-Type: application/json" -d '{
  "txn_date": "2025-02-28",
  "reference": "PO-980",
  "description": "Investing money and property into business by owners.",
  "posted": false,
  "lines": [
    { "account_uuid": "019ae5bb-9664-7d64-b478-2ce17f26069d", "debit": "122000.00", "credit": "0", "memo": "Cash at Bank" },
{"account_uuid": "019ae5bb-9ac7-7943-aa2a-9097ed62f516", "debit": "33000.00", "credit":"0", "memo": "Land"},
{ "account_uuid": "019ae5bb-a229-79ae-b5f4-0354b80ef94e", "debit
": "0", "credit": "155000.00", "memo": "Shareholder's net ownership" }
  ]}' | jq
```

```python
curl -X POST http://localhost:8080/api/transactions/create -H "$AUTH" -H "Content-Type: application/json" -d '{
  "txn_date": "2025-12-05",
  "reference": "PO-500",
  "description": "Inventory purchase from Supplier Ltd",
  "posted": true,
  "lines": [
    { "account_uuid": "44444444-4444-4444-4444-444444444444", "debit": "3000.00", "credit": "0",     "memo": "Inventory" },
    { "account_uuid": "55555555-5555-5555-5555-555555555555", "debit": "0",      "credit": "3000.00", "memo": "Accounts Payable" }
  ]
}' | jq
```

# Create a transaction (posted)
```python
curl -X POST http://localhost:3000/accounting/create -H "$AUTH" -H "Content-Type: application/json" -d '{
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
```

# Post transactions - makes transactions immutable
```python
curl -X PUT http://localhost:8080/api/transactions/post/THE_UUID \
  -H "Authorization: Bearer YOUR_JWT_HERE" | jq
```

# Try to update a posted transaction - possible before posting
```python
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
```

# Delete an unposted transaction
```python
curl -X DELETE http://localhost:8080/api/transactions/delete/NEW_UUID_HERE \
  -H "Authorization: Bearer YOUR_JWT_HERE" | jq
```
# Make posted transaction void
```python
curl -X POST http://localhost:8080/api/transactions/void/THE_UUID \
  -H "Authorization: Bearer YOUR_JWT_HERE" \
  -H "Content-Type: application/json" \
  -d '{"reason": "Customer returned the goods"}' | jq
```
# Check balance of transaction
```python
curl -G http://localhost:8080/api/transactions/balance/UUID \
  -H "Authorization: Bearer YOUR_JWT_HERE" | jq
```

# Generate payrun example (including payslips with pay items, i.e., benefits)
```python
curl -sX POST curl -sX POST http://127.0.0.1:8080/api/payroll/payrun/create \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "pay_period_start": "2025-01-01",
    "pay_period_end":   "2025-12-31",
    "payment_date":     "2025-08-28",
    "notes":            "Very first payroll for august 2025",
    "payslips": [
      {
        "employee_uuid": "019b1e69-a5e6-7172-aa1e-e3468ff9eebb",
        "gross_pay": 1540000,
        "tax_deducted": 154000,
        "social_security": 100000,
        "benefits": [
          {
            "item_type": "Allowance",
            "amount": 300000,
            "description": "Transport benefits"
          }
        ]
      },
      {
        "employee_uuid": "019b1e63-037d-7c09-90ae-9f431590a0cb",
        "gross_pay": 1740000,
        "tax_deducted": 174000,
        "social_security": 120000,
        "benefits": [
          {
            "item_type": "Allowance",
            "amount": 520000,
            "description": "Rent"
          }
        ]
      },
      {
        "employee_uuid": "019b1e5d-4e7f-7f1c-8c2d-f59833e27024",
        "gross_pay": 2500000,
        "tax_deducted": 250000,
        "social_security": 150000
      }
    ]
  }' | jq
```

# Process Payrun
```python
curl -sX POST http://127.0.0.1:8080/api/payroll/payrun/process/6 -H "Authorization: Bearer $TOKEN" -H  "Content-Type: application/json" | jq
```

# Post payroll
```python
curl -sX POST http://127.0.0.1:8080/api/payroll/payrun/post -H "Authorization: Bearer $TOKEN" -d '{"payrun_serial_id":6,"labor_expense_id":"019b1db9-75a5-7640-a3a7-614f5658c16f", "income_tax_id":"019b1e72-50b5-70fd-82ad-9bfb3d5036cb", "social_security_id":"019b1e73-844d-748f-9650-3802cbdaa644", "cash_account_uuid":"019b1db9-73b0-7654-8054-3ffaf59782ac", "payroll_payable":"019b1e05-79c8-7174-bd8d-574b06979346"}' -H "Content-Type: application/json" | jq
```

# Create Invoice
```python
curl -sX POST http://127.0.0.1:8080/api/customers/create/invoice -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{"invoice_number":"INVOICE-765JH", "customer_uuid":"019b2d55-f638-7b7f-b810-13c80fd8f3db", "issue_date":"2025-03-30", "due_date":"2025-04-30", "currency":"UGX", "items":[{"stock_item_id":2, "description":"Microcontrollers", "quantity":"20", "unit_price":"50000", "tax_rate":"18"}, {"stock_item_id":5, "description":"Microcontrollers", "quantity":"20", "unit_price":"270000", "tax_rate":"18"}, {"stock_item_id":3, "description":"Black Printable Circuit Boards", "quantity":"34", "unit_price":"6500", "tax_rate":"18"}]}' | jq
```

```python
curl -sX POST http://127.0.0.1:8080/api/vendors/create/bill -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{"bill_number":"BILL-12HD44", "vendor_uuid":"019b380e-9636-7d76-97a5-30cc179d070e", "bill_date":"2025-01-10", "due_date":"2025-01-30", "reference":"invoice", "currency":"UGX", "items":[{"stock_item_id":1, "description":"Description of nature of product A", "quantity":"100", "tax_rate":"18", "unit_price":"25000"}, {"stock_item_id":2, "description":"Description of nature of product B", "quantity":"1000", "tax_rate":"18", "unit_price":"9000"}]}' | jq
```
