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
| Transactions (Journal Entries) | Done        | Double-entry with debit/credit validation       |
| Company Management             | Done        | Multi-company support                           |
| Customers & Vendors            | Done        | Contacts, invoices, payments                    |
| Financial Reports              | In Progress | Balance Sheet, P&L, Trial Balance               |
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
companies (id, name, period_start, user_id)
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
curl -sX POST http://localhost:8080/api/transactions/create \
  -H "Authorization : Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
  "txn_date": "2025-02-28",
  "reference": "PO-980",
  "description": "Investing money and property into business by owners.",
  "posted": false,
  "lines": [
    {
      "account_uuid": "019ae5bb-9664-7d64-b478-2ce17f26069d",
      "debit": "122000.00",
      "credit": "0",
      "memo": "Cash at Bank"
    },
    {
      "account_uuid": "019ae5bb-9ac7-7943-aa2a-9097ed62f516",
      "debit": "33000.00",
      "credit":"0",
      "memo": "Land"
    },
    {
      "account_uuid": "019ae5bb-a229-79ae-b5f4-0354b80ef94e",
      "debit": "0",
      "credit": "155000.00",
      "memo": "Shareholder's net ownership"
    }
  ]
}' | jq
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

# Cash purchase
```python
curl -sX POST http://127.0.0.1:8080/api/procurement/create/cash-purchase \
-H "Authorization: Bearer $TOKEN" \
-H "Content-Type: application/json" \
-d '{
  "bill_number": "CASH-PURCH-001",
  "vendor_uuid": "019c46a7-471c-721a-8f80-dffeb368810d",
  "bill_date": "2026-01-05",
  "due_date": "2026-01-05",
  "reference": "purchase",
  "settlement_type":"cash",
  "paid_at":"2026-01-05",
  "items": [
    {"stock_item_id": 4,"description":"Industrial Edge IoT Gateway","quantity":"7","tax_rate":"18","unit_price":"680000"},
    {"stock_item_id": 5,"description":"Smart Energy Monitoring Unit","quantity":"7","tax_rate":"18","unit_price":"320000"}
  ]
}' | jq
```

# Post purchase - Cash
```python
curl -sX POST http://127.0.0.1:8080/api/procurement/post/credit/purchase -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{  "bill_serial_id": 3,  "vat_tax_account": "110805",  "cash_account": "110100", "revenue_code":"400000"}' | jq
```

# Credit purchase
```python
curl -sX POST http://127.0.0.1:8080/api/procurement/create/credit-purchase \
-H "Authorization: Bearer $TOKEN" \
-H "Content-Type: application/json" \
-d '{
  "bill_number": "CRDT-PURCH-001",
  "vendor_uuid": "019c46a7-471c-721a-8f80-dffeb368810d",
  "bill_date": "2026-01-05",
  "due_date": "2026-01-15",
  "reference": "purchase",
  "settlement_type":"credit",
  "items": [
    {"stock_item_id": 4,"description":"Industrial Edge IoT Gateway","quantity":"7","tax_rate":"18","unit_price":"680000"},
    {"stock_item_id": 5,"description":"Smart Energy Monitoring Unit","quantity":"7","tax_rate":"18","unit_price":"320000"}
  ]
}' | jq
```

# Post purchase - Credit
```python
curl -sX POST http://127.0.0.1:8080/api/procurement/post/credit/purchase -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{  "bill_serial_id": 3,  "vat_tax_account": "110805",  "payables_account": "110300", "revenue_code":"400000"}' | jq
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
curl -sX POST http://127.0.0.1:8080/api/payroll/payrun/post \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "payrun_serial_id": 6,
    "labor_expense_id":      "019b1db9-75a5-7640-a3a7-614f5658c16f",
    "income_tax_id":         "019b1e72-50b5-70fd-82ad-9bfb3d5036cb",
    "social_security_id":    "019b1e73-844d-748f-9650-3802cbdaa644",
    "cash_account_uuid":     "019b1db9-73b0-7654-8054-3ffaf59782ac",
    "payroll_payable":       "019b1e05-79c8-7174-bd8d-574b06979346"
  }' \
  | jq
```

# Create Invoice
```python
curl -sX POST http://127.0.0.1:8080/api/customers/create/invoice \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "invoice_number": "INVOICE-765JH",
    "customer_uuid":  "019b2d55-f638-7b7f-b810-13c80fd8f3db",
    "issue_date":     "2025-03-30",
    "due_date":       "2025-04-30",
    "currency":       "UGX",
    "items": [
      {
        "stock_item_id": 2,
        "description":   "Microcontrollers",
        "quantity":      "20",
        "unit_price":    "50000",
        "tax_rate":      "18"
      },
      {
        "stock_item_id": 5,
        "description":   "Microcontrollers",
        "quantity":      "20",
        "unit_price":    "270000",
        "tax_rate":      "18"
      },
      {
        "stock_item_id": 3,
        "description":   "Black Printable Circuit Boards",
        "quantity":      "34",
        "unit_price":    "6500",
        "tax_rate":      "18"
      }
    ]
  }' \
  | jq
```

# Create Bill
```python
curl -sX POST http://127.0.0.1:8080/api/vendors/create/bill \
-H "Authorization: Bearer $TOKEN" \
-H "Content-Type: application/json" \
-d '{
  "bill_number": "BILL-CR-001",
  "vendor_uuid": "019c46a7-471c-721a-8f80-dffeb368810d",
  "bill_date": "2026-01-05",
  "due_date": "2026-01-25",
  "reference": "invoice",
  "currency": "UGX",
  "items": [
    {"stock_item_id": 2,"description":"Microcontrollers","quantity":"50","tax_rate":"18","unit_price":"15000"},
    {"stock_item_id": 3,"description":"PCB Boards","quantity":"30","tax_rate":"18","unit_price":"9000"}
  ]
}' | jq
```

┌──(zeus㉿skywalker)-[~/Documents/Projects/Rust/erp-core]
└─$ curl -sX POST http://127.0.0.1:8080/api/manufacturing/apply/labor-cost -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" -d '{"production_order_uuid":"019c8bc8-24a2-7279-a867-e2e18def98c9", "wip_account_code":"122000", "rate_per_hour":"3000", "hours":"12", "is_direct":true, "control_account_code":"540000", "renumeration_code":"212000", "reference":"Assembly team test"}' | jq
{
  "error": "Internal Server Error",
  "message": "Database error: error returned from database: invalid input syntax for type uuid: \"24\"",
  "status": 500
}


{
  "status": 200,
  "message": "Direct Cash Flow Report Generated",
  "data": [
    {
      "category": "cashflow",
      "children": [
        {
          "category": "operating",
          "children": [
            {
              "category": "operating",
              "children": [],
              "code": "540600",
              "name": "Maintenance & Repairs",
              "total": "750000"
            },
            {
              "category": "operating",
              "children": [],
              "code": "540800",
              "name": "Water & Gas - Factory",
              "total": "350000"
            }
          ],
          "code": "540000",
          "name": "Manufacturing Overhead",
          "total": "14450000"
        },
        {
          "category": "working-capital",
          "children": [],
          "code": "210100",
          "name": "Accounts Payable",
          "total": "-120000000"
        },
        {
          "category": "working-capital",
          "children": [],
          "code": "210301",
          "name": "Output VAT Payable",
          "total": "-56700000"
        },
        {
          "category": "working-capital",
          "children": [],
          "code": "110701",
          "name": "Raw Materials Inventory",
          "total": "10519500"
        },
        {
          "category": "operating",
          "children": [],
          "code": "510100",
          "name": "Raw Materials Purchases",
          "total": "271580500"
        },
        {
          "category": "operating",
          "children": [],
          "code": "620200",
          "name": "Office Rent",
          "total": "5000000"
        },
        {
          "category": "operating",
          "children": [],
          "code": "620500",
          "name": "Telephone & Internet",
          "total": "200000"
        },
        {
          "category": "working-capital",
          "children": [],
          "code": "110805",
          "name": "Input VAT",
          "total": "50778000"
        }
      ],
      "code": "OPERATING",
      "name": "Cash Flows from Operating Activities",
      "total": "175828000"
    },
    {
      "category": "cashflow",
      "children": [
        {
          "category": "investing",
          "children": [],
          "code": "120102",
          "name": "Factory Building",
          "total": "240000000"
        }
      ],
      "code": "INVESTING",
      "name": "Cash Flows from Investing Activities",
      "total": "240000000"
    },
    {
      "category": "cashflow",
      "children": [
        {
          "category": "financing",
          "children": [],
          "code": "310100",
          "name": "Owner's Capital",
          "total": "-1060000000"
        },
        {
          "category": "financing",
          "children": [],
          "code": "220100",
          "name": "Long-Term Bank Loan",
          "total": "-85000000"
        }
      ],
      "code": "FINANCING",
      "name": "Cash Flows from Financing Activities",
      "total": "-1145000000"
    },
    {
      "category": "computed",
      "children": [],
      "code": "NET_CASH_FLOW",
      "name": "Net Increase / (Decrease) in Cash",
      "total": "-729172000"
    },
    {
      "category": "computed",
      "children": [],
      "code": "OPENING_CASH",
      "name": "Cash at Beginning of Period",
      "total": "0"
    },
    {
      "category": "computed",
      "children": [],
      "code": "CLOSING_CASH",
      "name": "Cash at End of Period",
      "total": "-729172000"
    }
  ]
}

{
  "status": 200,
  "message": "Fetched ledger for account",
  "data": [
    {
      "amount": "50000000",
      "created_at": "2026-04-16T19:36:06.506100Z",
      "credit": "0",
      "debit": "50000000",
      "description": "Initial capital injection by owner",
      "entry_serial_id": 1,
      "entry_uuid": "019d97cb-27a9-795c-81f6-d11b405377da",
      "memo": "Capital injected in cash",
      "reference": "CAP-001",
      "running_balance": "50000000",
      "transaction_serial_id": 1,
      "transaction_uuid": "019d97cb-274c-7a2a-84f2-f8690841c38d",
      "txn_date": "2026-01-01"
    },
    {
      "amount": "-350000",
      "created_at": "2026-04-16T19:54:34.848039Z",
      "credit": "350000",
      "debit": "0",
      "description": "Monthly water bill payment",
      "entry_serial_id": 5,
      "entry_uuid": "019d97dc-0fe2-70f3-8b1d-89a9f3adf3a8",
      "memo": "Payment to water utility",
      "reference": "UTIL-002",
      "running_balance": "49650000",
      "transaction_serial_id": 2,
      "transaction_uuid": "019d97dc-0fe0-77e8-b83e-237a6a06f461",
      "txn_date": "2026-02-01"
    },
    {
      "amount": "-5000000",
      "created_at": "2026-04-16T19:57:10.843153Z",
      "credit": "5000000",
      "debit": "0",
      "description": "Monthly office rent payment",
      "entry_serial_id": 7,
      "entry_uuid": "019d97de-713d-765f-9a84-26136fbbd3af",
      "memo": "Rent paid to landlord",
      "reference": "RENT-001",
      "running_balance": "44650000",
      "transaction_serial_id": 3,
      "transaction_uuid": "019d97de-713b-7a39-b695-9e5d57d25c95",
      "txn_date": "2026-02-01"
    },
    {
      "amount": "-750000",
      "created_at": "2026-04-16T20:01:51.118903Z",
      "credit": "750000",
      "debit": "0",
      "description": "Office repair and maintenance work",
      "entry_serial_id": 9,
      "entry_uuid": "019d97e2-b824-7c41-a161-d0bf718d4302",
      "memo": "Cash paid to technician",
      "reference": "MAINT-001",
      "running_balance": "43900000",
      "transaction_serial_id": 4,
      "transaction_uuid": "019d97e2-b811-73e6-9f36-40f9329b14ec",
      "txn_date": "2026-02-08"
    },
    {
      "amount": "-30000000",
      "created_at": "2026-04-16T20:10:31.762835Z",
      "credit": "30000000",
      "debit": "0",
      "description": "Purchase of warehouse and land property with split payment",
      "entry_serial_id": 13,
      "entry_uuid": "019d97ea-a9f2-72b7-a81e-2b705be8fc7a",
      "memo": "Cash payment toward property",
      "reference": "CAPEX-002",
      "running_balance": "13900000",
      "transaction_serial_id": 6,
      "transaction_uuid": "019d97ea-a9d5-7fad-a9a2-0441967bd2c1",
      "txn_date": "2026-02-10"
    },
    {
      "amount": "45000000",
      "created_at": "2026-04-19T13:35:52.797883Z",
      "credit": "0",
      "debit": "45000000",
      "description": "Issuing 450,000 shares to a venture capitalist valued at 1000 UGX each",
      "entry_serial_id": 28,
      "entry_uuid": "019da5f4-8304-73a3-ae7d-ab8c77d88cab",
      "memo": "Working capital buffer from issuance of shares",
      "reference": "CAP-002",
      "running_balance": "58900000",
      "transaction_serial_id": 11,
      "transaction_uuid": "019da5f4-7fbf-72ef-ab8d-5e33d7be6a1e",
      "txn_date": "2026-03-04"
    },
    {
      "amount": "-13000000",
      "created_at": "2026-04-28T20:16:04.148019Z",
      "credit": "13000000",
      "debit": "0",
      "description": "Actual Manufacturing Overhead",
      "entry_serial_id": 81,
      "entry_uuid": "019dd5bc-32fe-7f27-8b68-02c953cc57e9",
      "memo": "Actual overhead (ActualOverhead) for order PO-SG-001",
      "reference": "OVERHEAD-ACTUAL-PO-SG-001",
      "running_balance": "45900000",
      "transaction_serial_id": 35,
      "transaction_uuid": "019dd5bc-126c-7d63-a3c0-e63cea9cef96",
      "txn_date": "2026-04-28"
    },
    {
      "amount": "-350000",
      "created_at": "2026-04-29T18:11:28.201206Z",
      "credit": "350000",
      "debit": "0",
      "description": "Actual Manufacturing Overhead",
      "entry_serial_id": 83,
      "entry_uuid": "019dda70-59d4-72df-9995-4bc2c6e1e38d",
      "memo": "Off-setting account for order PO-SG-001: Payment for lighting",
      "reference": "OVERHEAD-ACTUAL-PO-SG-001",
      "running_balance": "45550000",
      "transaction_serial_id": 36,
      "transaction_uuid": "019dda70-574e-75f2-870c-91bf981ed9dd",
      "txn_date": "2026-04-29"
    }
  ]
}

{
  "status": 200,
  "message": "Fetched ledger for account",
  "data": [
    {
      "amount": "30000000",
      "created_at": "2026-04-16T19:36:06.506100Z",
      "credit": "0",
      "debit": "30000000",
      "description": "Initial capital injection by owner",
      "entry_serial_id": 2,
      "entry_uuid": "019d97cb-280c-7e4f-b19e-d6c86fcf5757",
      "memo": "Capital injected via bank transfer",
      "reference": "CAP-001",
      "running_balance": "30000000",
      "transaction_serial_id": 1,
      "transaction_uuid": "019d97cb-274c-7a2a-84f2-f8690841c38d",
      "txn_date": "2026-01-01"
    },
    {
      "amount": "371700000",
      "created_at": "2026-05-02T19:34:45.727330Z",
      "credit": "0",
      "debit": "371700000",
      "description": "Cash Sale",
      "entry_serial_id": 100,
      "entry_uuid": "019dea2f-abef-75c3-a5aa-72fae76628f9",
      "memo": "Cash received – sale",
      "reference": "SALE-CASH-001",
      "running_balance": "401700000",
      "transaction_serial_id": 45,
      "transaction_uuid": "019dea2f-ab27-77d3-9b50-7471892a1844",
      "txn_date": "2026-01-30"
    },
    {
      "amount": "-200000",
      "created_at": "2026-04-16T20:04:44.632011Z",
      "credit": "200000",
      "debit": "0",
      "description": "Monthly internet subscription",
      "entry_serial_id": 11,
      "entry_uuid": "019d97e5-5dd9-790b-ba1d-10d60168545e",
      "memo": "Paid via mobile money",
      "reference": "UTIL-003",
      "running_balance": "401500000",
      "transaction_serial_id": 5,
      "transaction_uuid": "019d97e5-5dd8-778c-ac63-5e0d1fa53986",
      "txn_date": "2026-02-01"
    },
    {
      "amount": "-30000000",
      "created_at": "2026-04-16T20:10:31.762835Z",
      "credit": "30000000",
      "debit": "0",
      "description": "Purchase of warehouse and land property with split payment",
      "entry_serial_id": 14,
      "entry_uuid": "019d97ea-aa0d-7075-9557-659a9f432d11",
      "memo": "Bank transfer toward property",
      "reference": "CAPEX-002",
      "running_balance": "371500000",
      "transaction_serial_id": 6,
      "transaction_uuid": "019d97ea-a9d5-7fad-a9a2-0441967bd2c1",
      "txn_date": "2026-02-10"
    },
    {
      "amount": "85000000",
      "created_at": "2026-04-18T17:10:55.543638Z",
      "credit": "0",
      "debit": "85000000",
      "description": "Taking out a bank loan to finance purchase of raw materials",
      "entry_serial_id": 16,
      "entry_uuid": "019da192-f47e-76c5-8563-a0014f841165",
      "memo": "Banking money from bank loan",
      "reference": "LOAN-001",
      "running_balance": "456500000",
      "transaction_serial_id": 7,
      "transaction_uuid": "019da192-f454-7293-9e72-fa1145336823",
      "txn_date": "2026-02-11"
    },
    {
      "amount": "405000000",
      "created_at": "2026-04-19T13:35:52.797883Z",
      "credit": "0",
      "debit": "405000000",
      "description": "Issuing 450,000 shares to a venture capitalist valued at 1000 UGX each",
      "entry_serial_id": 27,
      "entry_uuid": "019da5f4-8291-72f8-8681-5c5eae4db533",
      "memo": "Bank transfer from Venture Capital Limited",
      "reference": "CAP-002",
      "running_balance": "861500000",
      "transaction_serial_id": 11,
      "transaction_uuid": "019da5f4-7fbf-72ef-ab8d-5e33d7be6a1e",
      "txn_date": "2026-03-04"
    },
    {
      "amount": "-91225800",
      "created_at": "2026-04-18T17:42:42.354691Z",
      "credit": "91225800",
      "debit": "0",
      "description": "Vendor Cash Purchase",
      "entry_serial_id": 20,
      "entry_uuid": "019da1b0-0c1e-754e-ab03-240283bb642d",
      "memo": "Account payable credit purchase",
      "reference": "CASH-PURCH-001",
      "running_balance": "770274200",
      "transaction_serial_id": 8,
      "transaction_uuid": "019da1b0-0bdc-76d1-98c9-7758a96fbd76",
      "txn_date": "2026-04-18"
    },
    {
      "amount": "-62646200",
      "created_at": "2026-04-18T17:42:54.884912Z",
      "credit": "62646200",
      "debit": "0",
      "description": "Vendor Cash Purchase",
      "entry_serial_id": 23,
      "entry_uuid": "019da1b0-3cb9-7864-966c-7fd864c94128",
      "memo": "Account payable credit purchase",
      "reference": "CASH-PURCH-002",
      "running_balance": "707628000",
      "transaction_serial_id": 9,
      "transaction_uuid": "019da1b0-3ca8-79e9-af8c-6f621555f78b",
      "txn_date": "2026-04-19"
    },
    {
      "amount": "-130095000",
      "created_at": "2026-04-19T15:48:19.111042Z",
      "credit": "130095000",
      "debit": "0",
      "description": "Vendor Cash Purchase",
      "entry_serial_id": 32,
      "entry_uuid": "019da66d-b049-7fe0-80a4-85bc6a258ec3",
      "memo": "Account payable credit purchase",
      "reference": "CASH-PURCH-003",
      "running_balance": "577533000",
      "transaction_serial_id": 12,
      "transaction_uuid": "019da66d-af75-7ebe-ae94-24eea2f74f58",
      "txn_date": "2026-04-20"
    },
    {
      "amount": "-48911000",
      "created_at": "2026-04-19T15:48:25.701939Z",
      "credit": "48911000",
      "debit": "0",
      "description": "Vendor Cash Purchase",
      "entry_serial_id": 35,
      "entry_uuid": "019da66d-c7fa-7ba7-aca8-4c84623cfb46",
      "memo": "Account payable credit purchase",
      "reference": "CASH-PURCH-004",
      "running_balance": "528622000",
      "transaction_serial_id": 13,
      "transaction_uuid": "019da66d-c7ed-7917-90a7-15bd827e55e8",
      "txn_date": "2026-04-20"
    }
  ]
}