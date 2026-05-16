{
  "status": 200,
  "message": "Journal entries fetched",
  "data": [
    {
      "header": {
        "created_at": "2026-04-16T19:36:06.506100Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Initial capital injection by owner",
        "module": "equity",
        "posted": true,
        "reference": "CAP-001",
        "serial_id": 1,
        "txn_date": "2026-01-01",
        "uuid": "019d97cb-274c-7a2a-84f2-f8690841c38d"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110100",
          "account_name": "Cash on Hand",
          "account_uuid": "019d8d8a-9a9f-735d-92db-dd06e264055a",
          "amount": "50000000",
          "credit": "0",
          "debit": "50000000",
          "line_no": 1,
          "memo": "Capital injected in cash",
          "transaction_uuid": "019d97cb-274c-7a2a-84f2-f8690841c38d",
          "uuid": "019d97cb-27a9-795c-81f6-d11b405377da"
        },
        {
          "account_category": "asset",
          "account_code": "110200",
          "account_name": "Bank Accounts",
          "account_uuid": "019d8d8a-9aaa-76c8-b593-6ea15e978ac4",
          "amount": "30000000",
          "credit": "0",
          "debit": "30000000",
          "line_no": 2,
          "memo": "Capital injected via bank transfer",
          "transaction_uuid": "019d97cb-274c-7a2a-84f2-f8690841c38d",
          "uuid": "019d97cb-280c-7e4f-b19e-d6c86fcf5757"
        },
        {
          "account_category": "equity",
          "account_code": "310100",
          "account_name": "Owner's Capital",
          "account_uuid": "019d8d8a-9d24-7ab6-b280-fc26a81cc918",
          "amount": "-80000000",
          "credit": "80000000",
          "debit": "0",
          "line_no": 3,
          "memo": "Owner capital contribution",
          "transaction_uuid": "019d97cb-274c-7a2a-84f2-f8690841c38d",
          "uuid": "019d97cb-280e-76e3-9448-c4245b219ee4"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-16T19:54:34.848039Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Monthly water bill payment",
        "module": "utilities",
        "posted": true,
        "reference": "UTIL-002",
        "serial_id": 2,
        "txn_date": "2026-02-01",
        "uuid": "019d97dc-0fe0-77e8-b83e-237a6a06f461"
      },
      "lines": [
        {
          "account_category": "expense",
          "account_code": "540800",
          "account_name": "Water & Gas - Factory",
          "account_uuid": "019d8d8a-9e9f-79cb-ada7-719446b0d2d5",
          "amount": "350000",
          "credit": "0",
          "debit": "350000",
          "line_no": 1,
          "memo": "Water usage for office and storage",
          "transaction_uuid": "019d97dc-0fe0-77e8-b83e-237a6a06f461",
          "uuid": "019d97dc-0fe0-7fa1-b33c-3c1c4c7a8169"
        },
        {
          "account_category": "asset",
          "account_code": "110100",
          "account_name": "Cash on Hand",
          "account_uuid": "019d8d8a-9a9f-735d-92db-dd06e264055a",
          "amount": "-350000",
          "credit": "350000",
          "debit": "0",
          "line_no": 2,
          "memo": "Payment to water utility",
          "transaction_uuid": "019d97dc-0fe0-77e8-b83e-237a6a06f461",
          "uuid": "019d97dc-0fe2-70f3-8b1d-89a9f3adf3a8"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-16T19:57:10.843153Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Monthly office rent payment",
        "module": "rent",
        "posted": true,
        "reference": "RENT-001",
        "serial_id": 3,
        "txn_date": "2026-02-01",
        "uuid": "019d97de-713b-7a39-b695-9e5d57d25c95"
      },
      "lines": [
        {
          "account_category": "expense",
          "account_code": "620200",
          "account_name": "Office Rent",
          "account_uuid": "019d8d8a-9f5c-767b-993d-b2d294fb13d7",
          "amount": "5000000",
          "credit": "0",
          "debit": "5000000",
          "line_no": 1,
          "memo": "Office space rental",
          "transaction_uuid": "019d97de-713b-7a39-b695-9e5d57d25c95",
          "uuid": "019d97de-713c-7432-afc8-6f0313f8afb8"
        },
        {
          "account_category": "asset",
          "account_code": "110100",
          "account_name": "Cash on Hand",
          "account_uuid": "019d8d8a-9a9f-735d-92db-dd06e264055a",
          "amount": "-5000000",
          "credit": "5000000",
          "debit": "0",
          "line_no": 2,
          "memo": "Rent paid to landlord",
          "transaction_uuid": "019d97de-713b-7a39-b695-9e5d57d25c95",
          "uuid": "019d97de-713d-765f-9a84-26136fbbd3af"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-16T20:01:51.118903Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Office repair and maintenance work",
        "module": "maintenance",
        "posted": true,
        "reference": "MAINT-001",
        "serial_id": 4,
        "txn_date": "2026-02-08",
        "uuid": "019d97e2-b811-73e6-9f36-40f9329b14ec"
      },
      "lines": [
        {
          "account_category": "expense",
          "account_code": "540600",
          "account_name": "Maintenance & Repairs",
          "account_uuid": "019d8d8a-9e89-79b6-af8d-31116a10c945",
          "amount": "750000",
          "credit": "0",
          "debit": "750000",
          "line_no": 1,
          "memo": "Repairs to office plumbing and wiring",
          "transaction_uuid": "019d97e2-b811-73e6-9f36-40f9329b14ec",
          "uuid": "019d97e2-b813-73f6-9113-e93505199bbf"
        },
        {
          "account_category": "asset",
          "account_code": "110100",
          "account_name": "Cash on Hand",
          "account_uuid": "019d8d8a-9a9f-735d-92db-dd06e264055a",
          "amount": "-750000",
          "credit": "750000",
          "debit": "0",
          "line_no": 2,
          "memo": "Cash paid to technician",
          "transaction_uuid": "019d97e2-b811-73e6-9f36-40f9329b14ec",
          "uuid": "019d97e2-b824-7c41-a161-d0bf718d4302"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-16T20:04:44.632011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Monthly internet subscription",
        "module": "utilities",
        "posted": true,
        "reference": "UTIL-003",
        "serial_id": 5,
        "txn_date": "2026-02-01",
        "uuid": "019d97e5-5dd8-778c-ac63-5e0d1fa53986"
      },
      "lines": [
        {
          "account_category": "expense",
          "account_code": "620500",
          "account_name": "Telephone & Internet",
          "account_uuid": "019d8d8a-9f7d-79dd-9fd8-afa19280f322",
          "amount": "200000",
          "credit": "0",
          "debit": "200000",
          "line_no": 1,
          "memo": "Business internet connectivity",
          "transaction_uuid": "019d97e5-5dd8-778c-ac63-5e0d1fa53986",
          "uuid": "019d97e5-5dd8-7e5c-8afe-db64fbf8106b"
        },
        {
          "account_category": "asset",
          "account_code": "110200",
          "account_name": "Bank Accounts",
          "account_uuid": "019d8d8a-9aaa-76c8-b593-6ea15e978ac4",
          "amount": "-200000",
          "credit": "200000",
          "debit": "0",
          "line_no": 2,
          "memo": "Paid via mobile money",
          "transaction_uuid": "019d97e5-5dd8-778c-ac63-5e0d1fa53986",
          "uuid": "019d97e5-5dd9-790b-ba1d-10d60168545e"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-16T20:10:31.762835Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Purchase of warehouse and land property with split payment",
        "module": "fixed_assets",
        "posted": true,
        "reference": "CAPEX-002",
        "serial_id": 6,
        "txn_date": "2026-02-10",
        "uuid": "019d97ea-a9d5-7fad-a9a2-0441967bd2c1"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "120102",
          "account_name": "Factory Building",
          "account_uuid": "019d8d8a-9bab-7248-92f9-3c114b76974d",
          "amount": "120000000",
          "credit": "0",
          "debit": "120000000",
          "line_no": 1,
          "memo": "Warehouse acquisition",
          "transaction_uuid": "019d97ea-a9d5-7fad-a9a2-0441967bd2c1",
          "uuid": "019d97ea-a9ed-7f05-a1e8-ccc3e6161e6d"
        },
        {
          "account_category": "asset",
          "account_code": "110100",
          "account_name": "Cash on Hand",
          "account_uuid": "019d8d8a-9a9f-735d-92db-dd06e264055a",
          "amount": "-30000000",
          "credit": "30000000",
          "debit": "0",
          "line_no": 2,
          "memo": "Cash payment toward property",
          "transaction_uuid": "019d97ea-a9d5-7fad-a9a2-0441967bd2c1",
          "uuid": "019d97ea-a9f2-72b7-a81e-2b705be8fc7a"
        },
        {
          "account_category": "asset",
          "account_code": "110200",
          "account_name": "Bank Accounts",
          "account_uuid": "019d8d8a-9aaa-76c8-b593-6ea15e978ac4",
          "amount": "-30000000",
          "credit": "30000000",
          "debit": "0",
          "line_no": 3,
          "memo": "Bank transfer toward property",
          "transaction_uuid": "019d97ea-a9d5-7fad-a9a2-0441967bd2c1",
          "uuid": "019d97ea-aa0d-7075-9557-659a9f432d11"
        },
        {
          "account_category": "liability",
          "account_code": "210100",
          "account_name": "Accounts Payable",
          "account_uuid": "019d8d8a-9c67-7fed-9251-2908315208b9",
          "amount": "-60000000",
          "credit": "60000000",
          "debit": "0",
          "line_no": 4,
          "memo": "Outstanding balance owed to seller",
          "transaction_uuid": "019d97ea-a9d5-7fad-a9a2-0441967bd2c1",
          "uuid": "019d97ea-aa0e-7443-824e-feb9371f34ba"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-18T17:10:55.543638Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Taking out a bank loan to finance purchase of raw materials",
        "module": "equity",
        "posted": true,
        "reference": "LOAN-001",
        "serial_id": 7,
        "txn_date": "2026-02-11",
        "uuid": "019da192-f454-7293-9e72-fa1145336823"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110200",
          "account_name": "Bank Accounts",
          "account_uuid": "019d8d8a-9aaa-76c8-b593-6ea15e978ac4",
          "amount": "85000000",
          "credit": "0",
          "debit": "85000000",
          "line_no": 1,
          "memo": "Banking money from bank loan",
          "transaction_uuid": "019da192-f454-7293-9e72-fa1145336823",
          "uuid": "019da192-f47e-76c5-8563-a0014f841165"
        },
        {
          "account_category": "liability",
          "account_code": "220100",
          "account_name": "Long-Term Bank Loan",
          "account_uuid": "019d8d8a-9ccc-7d60-9026-13dd5daf8f7f",
          "amount": "-85000000",
          "credit": "85000000",
          "debit": "0",
          "line_no": 2,
          "memo": "Receiving bank loan from bank",
          "transaction_uuid": "019da192-f454-7293-9e72-fa1145336823",
          "uuid": "019da192-f49c-7bc3-a7f1-cb80519c01e9"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-18T17:42:42.354691Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Vendor Cash Purchase",
        "module": "bill",
        "posted": true,
        "reference": "CASH-PURCH-001",
        "serial_id": 8,
        "txn_date": "2026-04-18",
        "uuid": "019da1b0-0bdc-76d1-98c9-7758a96fbd76"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "77310000",
          "credit": "0",
          "debit": "77310000",
          "line_no": 1,
          "memo": "Inventory billed on credit",
          "transaction_uuid": "019da1b0-0bdc-76d1-98c9-7758a96fbd76",
          "uuid": "019da1b0-0c1d-744e-85e3-8bebd9bb84ea"
        },
        {
          "account_category": "asset",
          "account_code": "110805",
          "account_name": "Input VAT",
          "account_uuid": "019da1ad-c0a8-7f77-9c51-8011edfd3d9c",
          "amount": "13915800",
          "credit": "0",
          "debit": "13915800",
          "line_no": 2,
          "memo": "Inventory billed on credit",
          "transaction_uuid": "019da1b0-0bdc-76d1-98c9-7758a96fbd76",
          "uuid": "019da1b0-0c1e-73d5-8aa2-8a988162cb7c"
        },
        {
          "account_category": "asset",
          "account_code": "110200",
          "account_name": "Bank Accounts",
          "account_uuid": "019d8d8a-9aaa-76c8-b593-6ea15e978ac4",
          "amount": "-91225800",
          "credit": "91225800",
          "debit": "0",
          "line_no": 3,
          "memo": "Account payable credit purchase",
          "transaction_uuid": "019da1b0-0bdc-76d1-98c9-7758a96fbd76",
          "uuid": "019da1b0-0c1e-754e-ab03-240283bb642d"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-18T17:42:54.884912Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Vendor Cash Purchase",
        "module": "bill",
        "posted": true,
        "reference": "CASH-PURCH-002",
        "serial_id": 9,
        "txn_date": "2026-04-19",
        "uuid": "019da1b0-3ca8-79e9-af8c-6f621555f78b"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "53090000",
          "credit": "0",
          "debit": "53090000",
          "line_no": 1,
          "memo": "Inventory billed on credit",
          "transaction_uuid": "019da1b0-3ca8-79e9-af8c-6f621555f78b",
          "uuid": "019da1b0-3cb8-7991-ac29-688526071770"
        },
        {
          "account_category": "asset",
          "account_code": "110805",
          "account_name": "Input VAT",
          "account_uuid": "019da1ad-c0a8-7f77-9c51-8011edfd3d9c",
          "amount": "9556200",
          "credit": "0",
          "debit": "9556200",
          "line_no": 2,
          "memo": "Inventory billed on credit",
          "transaction_uuid": "019da1b0-3ca8-79e9-af8c-6f621555f78b",
          "uuid": "019da1b0-3cb9-771e-b8cc-f9ed35925cb2"
        },
        {
          "account_category": "asset",
          "account_code": "110200",
          "account_name": "Bank Accounts",
          "account_uuid": "019d8d8a-9aaa-76c8-b593-6ea15e978ac4",
          "amount": "-62646200",
          "credit": "62646200",
          "debit": "0",
          "line_no": 3,
          "memo": "Account payable credit purchase",
          "transaction_uuid": "019da1b0-3ca8-79e9-af8c-6f621555f78b",
          "uuid": "019da1b0-3cb9-7864-966c-7fd864c94128"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-18T17:47:41.794781Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Vendor Credit Purchase",
        "module": "bill",
        "posted": true,
        "reference": "CRDT-PURCH-001",
        "serial_id": 10,
        "txn_date": "2026-04-18",
        "uuid": "019da1b4-9d63-7d89-9dba-c86b91b8ad05"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "76000000",
          "credit": "0",
          "debit": "76000000",
          "line_no": 1,
          "memo": "Inventory billed on credit",
          "transaction_uuid": "019da1b4-9d63-7d89-9dba-c86b91b8ad05",
          "uuid": "019da1b4-9d6e-76d4-a35b-63570297d270"
        },
        {
          "account_category": "asset",
          "account_code": "110805",
          "account_name": "Input VAT",
          "account_uuid": "019da1ad-c0a8-7f77-9c51-8011edfd3d9c",
          "amount": "13680000",
          "credit": "0",
          "debit": "13680000",
          "line_no": 2,
          "memo": "Inventory billed on credit",
          "transaction_uuid": "019da1b4-9d63-7d89-9dba-c86b91b8ad05",
          "uuid": "019da1b4-9d6e-7b2e-ae39-d7dd642f48f2"
        },
        {
          "account_category": "liability",
          "account_code": "210100",
          "account_name": "Accounts Payable",
          "account_uuid": "019d8d8a-9c67-7fed-9251-2908315208b9",
          "amount": "-89680000",
          "credit": "89680000",
          "debit": "0",
          "line_no": 3,
          "memo": "Account payable credit purchase",
          "transaction_uuid": "019da1b4-9d63-7d89-9dba-c86b91b8ad05",
          "uuid": "019da1b4-9d6f-7205-9879-16d30a1906ae"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-19T13:35:52.797883Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Issuing 450,000 shares to a venture capitalist valued at 1000 UGX each",
        "module": "equity",
        "posted": true,
        "reference": "CAP-002",
        "serial_id": 11,
        "txn_date": "2026-03-04",
        "uuid": "019da5f4-7fbf-72ef-ab8d-5e33d7be6a1e"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110200",
          "account_name": "Bank Accounts",
          "account_uuid": "019d8d8a-9aaa-76c8-b593-6ea15e978ac4",
          "amount": "405000000",
          "credit": "0",
          "debit": "405000000",
          "line_no": 1,
          "memo": "Bank transfer from Venture Capital Limited",
          "transaction_uuid": "019da5f4-7fbf-72ef-ab8d-5e33d7be6a1e",
          "uuid": "019da5f4-8291-72f8-8681-5c5eae4db533"
        },
        {
          "account_category": "asset",
          "account_code": "110100",
          "account_name": "Cash on Hand",
          "account_uuid": "019d8d8a-9a9f-735d-92db-dd06e264055a",
          "amount": "45000000",
          "credit": "0",
          "debit": "45000000",
          "line_no": 2,
          "memo": "Working capital buffer from issuance of shares",
          "transaction_uuid": "019da5f4-7fbf-72ef-ab8d-5e33d7be6a1e",
          "uuid": "019da5f4-8304-73a3-ae7d-ab8c77d88cab"
        },
        {
          "account_category": "equity",
          "account_code": "310100",
          "account_name": "Owner's Capital",
          "account_uuid": "019d8d8a-9d24-7ab6-b280-fc26a81cc918",
          "amount": "-450000000",
          "credit": "450000000",
          "debit": "0",
          "line_no": 3,
          "memo": "Money from issuing shares to venture capitalist",
          "transaction_uuid": "019da5f4-7fbf-72ef-ab8d-5e33d7be6a1e",
          "uuid": "019da5f4-8306-76b1-bcbc-ee72a1791111"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-19T15:48:19.111042Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Vendor Cash Purchase",
        "module": "bill",
        "posted": true,
        "reference": "CASH-PURCH-003",
        "serial_id": 12,
        "txn_date": "2026-04-20",
        "uuid": "019da66d-af75-7ebe-ae94-24eea2f74f58"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "110250000",
          "credit": "0",
          "debit": "110250000",
          "line_no": 1,
          "memo": "Inventory billed on credit",
          "transaction_uuid": "019da66d-af75-7ebe-ae94-24eea2f74f58",
          "uuid": "019da66d-b01f-752f-b92b-55fce7a0d3a6"
        },
        {
          "account_category": "asset",
          "account_code": "110805",
          "account_name": "Input VAT",
          "account_uuid": "019da1ad-c0a8-7f77-9c51-8011edfd3d9c",
          "amount": "19845000",
          "credit": "0",
          "debit": "19845000",
          "line_no": 2,
          "memo": "Inventory billed on credit",
          "transaction_uuid": "019da66d-af75-7ebe-ae94-24eea2f74f58",
          "uuid": "019da66d-b049-7dab-a174-948931776cba"
        },
        {
          "account_category": "asset",
          "account_code": "110200",
          "account_name": "Bank Accounts",
          "account_uuid": "019d8d8a-9aaa-76c8-b593-6ea15e978ac4",
          "amount": "-130095000",
          "credit": "130095000",
          "debit": "0",
          "line_no": 3,
          "memo": "Account payable credit purchase",
          "transaction_uuid": "019da66d-af75-7ebe-ae94-24eea2f74f58",
          "uuid": "019da66d-b049-7fe0-80a4-85bc6a258ec3"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-19T15:48:25.701939Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Vendor Cash Purchase",
        "module": "bill",
        "posted": true,
        "reference": "CASH-PURCH-004",
        "serial_id": 13,
        "txn_date": "2026-04-20",
        "uuid": "019da66d-c7ed-7917-90a7-15bd827e55e8"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "41450000",
          "credit": "0",
          "debit": "41450000",
          "line_no": 1,
          "memo": "Inventory billed on credit",
          "transaction_uuid": "019da66d-c7ed-7917-90a7-15bd827e55e8",
          "uuid": "019da66d-c7f9-7c57-a383-381a2c1c50d1"
        },
        {
          "account_category": "asset",
          "account_code": "110805",
          "account_name": "Input VAT",
          "account_uuid": "019da1ad-c0a8-7f77-9c51-8011edfd3d9c",
          "amount": "7461000",
          "credit": "0",
          "debit": "7461000",
          "line_no": 2,
          "memo": "Inventory billed on credit",
          "transaction_uuid": "019da66d-c7ed-7917-90a7-15bd827e55e8",
          "uuid": "019da66d-c7fa-7a7b-acaf-0656fbb6b311"
        },
        {
          "account_category": "asset",
          "account_code": "110200",
          "account_name": "Bank Accounts",
          "account_uuid": "019d8d8a-9aaa-76c8-b593-6ea15e978ac4",
          "amount": "-48911000",
          "credit": "48911000",
          "debit": "0",
          "line_no": 3,
          "memo": "Account payable credit purchase",
          "transaction_uuid": "019da66d-c7ed-7917-90a7-15bd827e55e8",
          "uuid": "019da66d-c7fa-7ba7-aca8-4c84623cfb46"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-19T15:48:45.712828Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Vendor Credit Purchase",
        "module": "bill",
        "posted": true,
        "reference": "CRDT-PURCH-002",
        "serial_id": 14,
        "txn_date": "2026-04-21",
        "uuid": "019da66e-1612-796c-893e-adaa7ae7b783"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "53050000",
          "credit": "0",
          "debit": "53050000",
          "line_no": 1,
          "memo": "Inventory billed on credit",
          "transaction_uuid": "019da66e-1612-796c-893e-adaa7ae7b783",
          "uuid": "019da66e-1619-7348-90b3-d290eaa258b5"
        },
        {
          "account_category": "asset",
          "account_code": "110805",
          "account_name": "Input VAT",
          "account_uuid": "019da1ad-c0a8-7f77-9c51-8011edfd3d9c",
          "amount": "9549000",
          "credit": "0",
          "debit": "9549000",
          "line_no": 2,
          "memo": "Inventory billed on credit",
          "transaction_uuid": "019da66e-1612-796c-893e-adaa7ae7b783",
          "uuid": "019da66e-1621-72a8-abae-296ae17c3bc5"
        },
        {
          "account_category": "liability",
          "account_code": "210100",
          "account_name": "Accounts Payable",
          "account_uuid": "019d8d8a-9c67-7fed-9251-2908315208b9",
          "amount": "-62599000",
          "credit": "62599000",
          "debit": "0",
          "line_no": 3,
          "memo": "Account payable credit purchase",
          "transaction_uuid": "019da66e-1612-796c-893e-adaa7ae7b783",
          "uuid": "019da66e-1621-744b-a73e-725749923f06"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-19T15:48:53.787379Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Vendor Credit Purchase",
        "module": "bill",
        "posted": true,
        "reference": "CRDT-PURCH-003",
        "serial_id": 15,
        "txn_date": "2026-04-21",
        "uuid": "019da66e-35c3-7e4e-a558-5c0fe003b092"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "56900000",
          "credit": "0",
          "debit": "56900000",
          "line_no": 1,
          "memo": "Inventory billed on credit",
          "transaction_uuid": "019da66e-35c3-7e4e-a558-5c0fe003b092",
          "uuid": "019da66e-35c6-70a2-82f6-ce889b8d5301"
        },
        {
          "account_category": "asset",
          "account_code": "110805",
          "account_name": "Input VAT",
          "account_uuid": "019da1ad-c0a8-7f77-9c51-8011edfd3d9c",
          "amount": "10242000",
          "credit": "0",
          "debit": "10242000",
          "line_no": 2,
          "memo": "Inventory billed on credit",
          "transaction_uuid": "019da66e-35c3-7e4e-a558-5c0fe003b092",
          "uuid": "019da66e-35c6-73e6-967f-cc4280e99fd9"
        },
        {
          "account_category": "liability",
          "account_code": "210100",
          "account_name": "Accounts Payable",
          "account_uuid": "019d8d8a-9c67-7fed-9251-2908315208b9",
          "amount": "-67142000",
          "credit": "67142000",
          "debit": "0",
          "line_no": 3,
          "memo": "Account payable credit purchase",
          "transaction_uuid": "019da66e-35c3-7e4e-a558-5c0fe003b092",
          "uuid": "019da66e-35c6-74aa-8687-6f3afc38dc7f"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 16,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-b73e-773b-8ba7-3c0db3497347"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "40825000",
          "credit": "0",
          "debit": "40825000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-b73e-773b-8ba7-3c0db3497347",
          "uuid": "019dc073-b89b-7644-8361-dbea1b69dbcf"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-40825000",
          "credit": "40825000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-b73e-773b-8ba7-3c0db3497347",
          "uuid": "019dc073-ba1e-71fa-8fca-cdbf302adb31"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 17,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-ba51-7ae9-88ab-c0c8c5a974a8"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "25800000",
          "credit": "0",
          "debit": "25800000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-ba51-7ae9-88ab-c0c8c5a974a8",
          "uuid": "019dc073-ba52-7362-996f-1c32c5cc3ac7"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-25800000",
          "credit": "25800000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-ba51-7ae9-88ab-c0c8c5a974a8",
          "uuid": "019dc073-bae8-769c-9e7a-8783928f0c0f"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 18,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-baec-7600-af30-7ca63fa81705"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "14400000",
          "credit": "0",
          "debit": "14400000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-baec-7600-af30-7ca63fa81705",
          "uuid": "019dc073-baec-7b51-a3c3-6966f90705ed"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-14400000",
          "credit": "14400000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-baec-7600-af30-7ca63fa81705",
          "uuid": "019dc073-baed-7fe8-b580-578a1d66393c"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 19,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-baf4-7592-ac90-3cc6bcede2d6"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "58625000",
          "credit": "0",
          "debit": "58625000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-baf4-7592-ac90-3cc6bcede2d6",
          "uuid": "019dc073-baf4-7994-a67d-2d65dd4d7bf1"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-58625000",
          "credit": "58625000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-baf4-7592-ac90-3cc6bcede2d6",
          "uuid": "019dc073-baf5-71fe-becd-e5a24387c1a9"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 20,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-baf8-7e0f-9b1d-3b8bf430ab3a"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "23100000",
          "credit": "0",
          "debit": "23100000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-baf8-7e0f-9b1d-3b8bf430ab3a",
          "uuid": "019dc073-baf9-70ef-b6da-aa6e017087c7"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-23100000",
          "credit": "23100000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-baf8-7e0f-9b1d-3b8bf430ab3a",
          "uuid": "019dc073-baf9-778d-b91b-f7b7e1fcae83"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 21,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-bafd-7241-a0ae-154dc73adac0"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "11600000",
          "credit": "0",
          "debit": "11600000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-bafd-7241-a0ae-154dc73adac0",
          "uuid": "019dc073-bafd-7596-a4c7-3ce22ae3971e"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-11600000",
          "credit": "11600000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-bafd-7241-a0ae-154dc73adac0",
          "uuid": "019dc073-bafd-7f1a-b181-c17ce05eedbd"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 22,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-bb01-7c88-b1b0-58bb63452e43"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "69240000",
          "credit": "0",
          "debit": "69240000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-bb01-7c88-b1b0-58bb63452e43",
          "uuid": "019dc073-bb02-7118-b16a-7e25ee19a45f"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-69240000",
          "credit": "69240000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-bb01-7c88-b1b0-58bb63452e43",
          "uuid": "019dc073-bb02-7bd3-b067-74cb5761729a"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 23,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-bb04-7acd-b16f-2d6f5efb0617"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "5800000",
          "credit": "0",
          "debit": "5800000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-bb04-7acd-b16f-2d6f5efb0617",
          "uuid": "019dc073-bb04-7d42-bb37-6d7a699c8bf5"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-5800000",
          "credit": "5800000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-bb04-7acd-b16f-2d6f5efb0617",
          "uuid": "019dc073-bb05-7320-89bd-86e9f46295c5"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 24,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-bb07-731e-96f8-91f7d653538d"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "9150000",
          "credit": "0",
          "debit": "9150000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-bb07-731e-96f8-91f7d653538d",
          "uuid": "019dc073-bb07-7567-9ca0-6404280bcdee"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-9150000",
          "credit": "9150000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-bb07-731e-96f8-91f7d653538d",
          "uuid": "019dc073-bb07-7ee0-83cf-a4502843c8b0"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 25,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-bb09-7fe8-b85c-95453104769d"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "30650000",
          "credit": "0",
          "debit": "30650000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-bb09-7fe8-b85c-95453104769d",
          "uuid": "019dc073-bb0a-7244-a11b-089faae01b15"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-30650000",
          "credit": "30650000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-bb09-7fe8-b85c-95453104769d",
          "uuid": "019dc073-bb0a-7897-a640-ec222a3d906c"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 26,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-bb0c-76ed-8a79-9017499262e9"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "17500000",
          "credit": "0",
          "debit": "17500000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-bb0c-76ed-8a79-9017499262e9",
          "uuid": "019dc073-bb0c-7930-bc9f-ce7dd058cf5f"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-17500000",
          "credit": "17500000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-bb0c-76ed-8a79-9017499262e9",
          "uuid": "019dc073-bb0c-7f04-93b6-6df7279e78ca"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 27,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-bb0e-7c49-a47c-8a6fedc1726d"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "11800000",
          "credit": "0",
          "debit": "11800000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-bb0e-7c49-a47c-8a6fedc1726d",
          "uuid": "019dc073-bb0e-7f81-989f-9c6335591603"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-11800000",
          "credit": "11800000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-bb0e-7c49-a47c-8a6fedc1726d",
          "uuid": "019dc073-bb0f-77c6-a0b9-14814bc79670"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 28,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-bb12-7c69-9ece-1415f732e3c5"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "3900000",
          "credit": "0",
          "debit": "3900000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-bb12-7c69-9ece-1415f732e3c5",
          "uuid": "019dc073-bb13-70a5-9a2d-05f61f707908"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-3900000",
          "credit": "3900000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-bb12-7c69-9ece-1415f732e3c5",
          "uuid": "019dc073-bb13-798c-b458-99afda641731"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 29,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-bb15-781e-9dfe-0fd0e19ce57b"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "5700000",
          "credit": "0",
          "debit": "5700000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-bb15-781e-9dfe-0fd0e19ce57b",
          "uuid": "019dc073-bb15-7a97-a625-3c18698e3ae2"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-5700000",
          "credit": "5700000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-bb15-781e-9dfe-0fd0e19ce57b",
          "uuid": "019dc073-bb16-7094-80b3-28dc1e4bd380"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 30,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-bb17-79bb-8c62-b61a665b0bc9"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "9500000",
          "credit": "0",
          "debit": "9500000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-bb17-79bb-8c62-b61a665b0bc9",
          "uuid": "019dc073-bb17-7bee-84a3-590685a3c527"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-9500000",
          "credit": "9500000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-bb17-79bb-8c62-b61a665b0bc9",
          "uuid": "019dc073-bb18-71c6-8ea9-7b9bf25527d6"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 31,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-bb19-7be3-b12a-d2760f9326e8"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "3900000",
          "credit": "0",
          "debit": "3900000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-bb19-7be3-b12a-d2760f9326e8",
          "uuid": "019dc073-bb19-7e2e-9d95-a8bef4567520"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-3900000",
          "credit": "3900000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-bb19-7be3-b12a-d2760f9326e8",
          "uuid": "019dc073-bb1a-7412-9685-5f6462cb5779"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 32,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-bb1b-7e72-a153-979626e2ea85"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "1950000",
          "credit": "0",
          "debit": "1950000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-bb1b-7e72-a153-979626e2ea85",
          "uuid": "019dc073-bb1c-70b5-939f-463a73649bb3"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-1950000",
          "credit": "1950000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-bb1b-7e72-a153-979626e2ea85",
          "uuid": "019dc073-bb1c-777e-8ec6-4e4713659c31"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-24T17:04:58.060011Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 33,
        "txn_date": "2026-04-24",
        "uuid": "019dc073-bb1e-7281-9e61-74c58afdec0f"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "9500000",
          "credit": "0",
          "debit": "9500000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019dc073-bb1e-7281-9e61-74c58afdec0f",
          "uuid": "019dc073-bb1e-75b9-84ef-ad66562148f6"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-9500000",
          "credit": "9500000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019dc073-bb1e-7281-9e61-74c58afdec0f",
          "uuid": "019dc073-bb1e-7cb3-b54f-aab7c0b29404"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-27T18:17:40.375648Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Applied Manufacturing Overhead",
        "module": "manufacturing",
        "posted": true,
        "reference": "OVERHEAD-APPLY-PO-SG-001",
        "serial_id": 34,
        "txn_date": "2026-04-27",
        "uuid": "019dd029-4c06-745c-ad6d-3ae569d9bd45"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "12900000",
          "credit": "0",
          "debit": "12900000",
          "line_no": 1,
          "memo": "Applied overhead to production order 019da5fc-1cdc-7fc2-9274-99d2b49275e7 - base 2150 x rate 6000.000000",
          "transaction_uuid": "019dd029-4c06-745c-ad6d-3ae569d9bd45",
          "uuid": "019dd029-4c51-7bd9-9ab1-3e23f0d2705b"
        },
        {
          "account_category": "expense",
          "account_code": "540000",
          "account_name": "Manufacturing Overhead",
          "account_uuid": "019d8d8a-9e45-7bd6-9bb8-7e9cda2b3f40",
          "amount": "-12900000",
          "credit": "12900000",
          "debit": "0",
          "line_no": 2,
          "memo": "Applied overhead to production order 019da5fc-1cdc-7fc2-9274-99d2b49275e7 - base 2150 x rate 6000.000000",
          "transaction_uuid": "019dd029-4c06-745c-ad6d-3ae569d9bd45",
          "uuid": "019dd029-4cad-7f6b-80d0-f65bc0952f44"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-28T20:16:04.148019Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Actual Manufacturing Overhead",
        "module": "manufacturing",
        "posted": true,
        "reference": "OVERHEAD-ACTUAL-PO-SG-001",
        "serial_id": 35,
        "txn_date": "2026-04-28",
        "uuid": "019dd5bc-126c-7d63-a3c0-e63cea9cef96"
      },
      "lines": [
        {
          "account_category": "expense",
          "account_code": "540000",
          "account_name": "Manufacturing Overhead",
          "account_uuid": "019d8d8a-9e45-7bd6-9bb8-7e9cda2b3f40",
          "amount": "13000000",
          "credit": "0",
          "debit": "13000000",
          "line_no": 1,
          "memo": "Actual overhead (ActualOverhead) for order PO-SG-001",
          "transaction_uuid": "019dd5bc-126c-7d63-a3c0-e63cea9cef96",
          "uuid": "019dd5bc-1e0c-7716-bbc8-0c6e7eaa22f7"
        },
        {
          "account_category": "asset",
          "account_code": "110100",
          "account_name": "Cash on Hand",
          "account_uuid": "019d8d8a-9a9f-735d-92db-dd06e264055a",
          "amount": "-13000000",
          "credit": "13000000",
          "debit": "0",
          "line_no": 2,
          "memo": "Actual overhead (ActualOverhead) for order PO-SG-001",
          "transaction_uuid": "019dd5bc-126c-7d63-a3c0-e63cea9cef96",
          "uuid": "019dd5bc-32fe-7f27-8b68-02c953cc57e9"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-29T18:11:28.201206Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Actual Manufacturing Overhead",
        "module": "manufacturing",
        "posted": true,
        "reference": "OVERHEAD-ACTUAL-PO-SG-001",
        "serial_id": 36,
        "txn_date": "2026-04-29",
        "uuid": "019dda70-574e-75f2-870c-91bf981ed9dd"
      },
      "lines": [
        {
          "account_category": "expense",
          "account_code": "540000",
          "account_name": "Manufacturing Overhead",
          "account_uuid": "019d8d8a-9e45-7bd6-9bb8-7e9cda2b3f40",
          "amount": "350000",
          "credit": "0",
          "debit": "350000",
          "line_no": 1,
          "memo": "Actual overhead (ActualOverhead) for order PO-SG-001: Payment for lighting",
          "transaction_uuid": "019dda70-574e-75f2-870c-91bf981ed9dd",
          "uuid": "019dda70-591f-7f19-885c-c72cf459fb79"
        },
        {
          "account_category": "asset",
          "account_code": "110100",
          "account_name": "Cash on Hand",
          "account_uuid": "019d8d8a-9a9f-735d-92db-dd06e264055a",
          "amount": "-350000",
          "credit": "350000",
          "debit": "0",
          "line_no": 2,
          "memo": "Off-setting account for order PO-SG-001: Payment for lighting",
          "transaction_uuid": "019dda70-574e-75f2-870c-91bf981ed9dd",
          "uuid": "019dda70-59d4-72df-9995-4bc2c6e1e38d"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-04-30T20:36:47.615892Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 37,
        "txn_date": "2026-04-30",
        "uuid": "019de01b-cf29-7792-8713-ee226b486896"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "740000",
          "credit": "0",
          "debit": "740000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019de01b-cf29-7792-8713-ee226b486896",
          "uuid": "019de01b-d01d-7bd7-9215-e03a4aadfef5"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-740000",
          "credit": "740000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019de01b-cf29-7792-8713-ee226b486896",
          "uuid": "019de01b-d13f-74a0-8d93-a1aa19050b73"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-05-01T05:43:28.602083Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material To Work In Progress",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-IN",
        "serial_id": 38,
        "txn_date": "2026-05-01",
        "uuid": "019de210-47df-71c6-82a7-de62e9d78833"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "185000",
          "credit": "0",
          "debit": "185000",
          "line_no": 1,
          "memo": "Material issue on order PO-SG-001",
          "transaction_uuid": "019de210-47df-71c6-82a7-de62e9d78833",
          "uuid": "019de210-4837-761b-951a-0f9f788bbe12"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-185000",
          "credit": "185000",
          "debit": "0",
          "line_no": 2,
          "memo": "Offset to WIP - order PO-SG-001",
          "transaction_uuid": "019de210-47df-71c6-82a7-de62e9d78833",
          "uuid": "019de210-486f-7fbd-93d4-eac2ef7836e0"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-05-01T06:51:43.921426Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Posting Manufacturing Overhead to WIP",
        "module": "manufacturing",
        "posted": true,
        "reference": "OH-TRF-019da5fc-1cdc-7fc2-9274-99d2b49275e7-20260501",
        "serial_id": 41,
        "txn_date": "2026-05-01",
        "uuid": "019de24e-bbc5-7d66-9599-7c579200e4c5"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "12900000",
          "credit": "0",
          "debit": "12900000",
          "line_no": 1,
          "memo": "Applied overhead to production order",
          "transaction_uuid": "019de24e-bbc5-7d66-9599-7c579200e4c5",
          "uuid": "019de24e-bbc8-752e-8740-60a0300a355f"
        },
        {
          "account_category": "expense",
          "account_code": "540000",
          "account_name": "Manufacturing Overhead",
          "account_uuid": "019d8d8a-9e45-7bd6-9bb8-7e9cda2b3f40",
          "amount": "-12900000",
          "credit": "12900000",
          "debit": "0",
          "line_no": 2,
          "memo": "Applied overhead to production order",
          "transaction_uuid": "019de24e-bbc5-7d66-9599-7c579200e4c5",
          "uuid": "019de24e-bbe7-7e8d-9e11-643d95831ffe"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-05-01T06:51:43.921426Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Production Completion - Standard Cost",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-COMPLETION",
        "serial_id": 42,
        "txn_date": "2026-05-01",
        "uuid": "019de24e-bbec-701f-8620-0158506fb0ff"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110703",
          "account_name": "Finished Goods Inventory",
          "account_uuid": "019d8d8a-9b1a-7843-830e-189ad7d4d383",
          "amount": "452634166.6700",
          "credit": "0",
          "debit": "452634166.6700",
          "line_no": 1,
          "memo": "FG at standard cost",
          "transaction_uuid": "019de24e-bbec-701f-8620-0158506fb0ff",
          "uuid": "019de24e-bbec-7a8d-8a69-5b4c0e3e3fdd"
        },
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "-452634166.6700",
          "credit": "452634166.6700",
          "debit": "0",
          "line_no": 2,
          "memo": "Relieve WIP at standard",
          "transaction_uuid": "019de24e-bbec-701f-8620-0158506fb0ff",
          "uuid": "019de24e-bbee-7206-977a-a451e43bc8c3"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-05-01T06:51:43.921426Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Material Usage/Price Variance",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-MAT-VAR",
        "serial_id": 43,
        "txn_date": "2026-05-01",
        "uuid": "019de24e-bc04-7802-b610-d43871419325"
      },
      "lines": [
        {
          "account_category": "expense",
          "account_code": "550100",
          "account_name": "Material Usage Variance",
          "account_uuid": "019d8d8a-9ed7-75cd-b029-6956ee00df2e",
          "amount": "-94285000",
          "credit": "94285000",
          "debit": "0",
          "line_no": 1,
          "memo": null,
          "transaction_uuid": "019de24e-bc04-7802-b610-d43871419325",
          "uuid": "019de24e-bc06-7c8e-a99b-06d74d044cf9"
        },
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "94285000",
          "credit": "0",
          "debit": "94285000",
          "line_no": 2,
          "memo": null,
          "transaction_uuid": "019de24e-bc04-7802-b610-d43871419325",
          "uuid": "019de24e-bc09-721e-af99-9a1303ff4366"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-05-01T06:51:43.921426Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Labor Rate/Efficiency Variance",
        "module": "manufacturing",
        "posted": true,
        "reference": "PO-SG-001-LAB-VAR",
        "serial_id": 44,
        "txn_date": "2026-05-01",
        "uuid": "019de24e-bc0d-79d4-bae4-58744f36cbbf"
      },
      "lines": [
        {
          "account_category": "expense",
          "account_code": "550200",
          "account_name": "Labor Efficiency Variance",
          "account_uuid": "019d8d8a-9ee2-796d-9894-bf0dae225dca",
          "amount": "-3746916.6700",
          "credit": "3746916.6700",
          "debit": "0",
          "line_no": 1,
          "memo": null,
          "transaction_uuid": "019de24e-bc0d-79d4-bae4-58744f36cbbf",
          "uuid": "019de24e-bc0e-73a6-b064-1737ae4a1f89"
        },
        {
          "account_category": "asset",
          "account_code": "110702",
          "account_name": "Work in Progress (WIP)",
          "account_uuid": "019d8d8a-9b0f-77c4-826f-34bb052c844c",
          "amount": "3746916.6700",
          "credit": "0",
          "debit": "3746916.6700",
          "line_no": 2,
          "memo": null,
          "transaction_uuid": "019de24e-bc0d-79d4-bae4-58744f36cbbf",
          "uuid": "019de24e-bc10-713d-8790-dd3339daa6f3"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-05-02T19:34:45.727330Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Cash Sale",
        "module": "turnover",
        "posted": true,
        "reference": "SALE-CASH-001",
        "serial_id": 45,
        "txn_date": "2026-01-30",
        "uuid": "019dea2f-ab27-77d3-9b50-7471892a1844"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110200",
          "account_name": "Bank Accounts",
          "account_uuid": "019d8d8a-9aaa-76c8-b593-6ea15e978ac4",
          "amount": "371700000",
          "credit": "0",
          "debit": "371700000",
          "line_no": 1,
          "memo": "Cash received – sale",
          "transaction_uuid": "019dea2f-ab27-77d3-9b50-7471892a1844",
          "uuid": "019dea2f-abef-75c3-a5aa-72fae76628f9"
        },
        {
          "account_category": "expense",
          "account_code": "510100",
          "account_name": "Raw Materials Purchases",
          "account_uuid": "019d8d8a-9dcb-7bab-a661-2a0def01309e",
          "amount": "271580500",
          "credit": "0",
          "debit": "271580500",
          "line_no": 1,
          "memo": "Cost Of Goods Sold",
          "transaction_uuid": "019dea2f-ab27-77d3-9b50-7471892a1844",
          "uuid": "019dea2f-ac8e-772c-9709-1cfba46fe53d"
        },
        {
          "account_category": "liability",
          "account_code": "210301",
          "account_name": "Output VAT Payable",
          "account_uuid": "019da1ae-c7a7-7e41-ad97-1425151c1663",
          "amount": "-56700000",
          "credit": "56700000",
          "debit": "0",
          "line_no": 2,
          "memo": "Output VAT collected",
          "transaction_uuid": "019dea2f-ab27-77d3-9b50-7471892a1844",
          "uuid": "019dea2f-ac08-7e8f-8146-9e07d07211d2"
        },
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "-271580500",
          "credit": "271580500",
          "debit": "0",
          "line_no": 2,
          "memo": "Inventory reduction",
          "transaction_uuid": "019dea2f-ab27-77d3-9b50-7471892a1844",
          "uuid": "019dea2f-ac8e-7d95-a7c1-d41d81f794e2"
        },
        {
          "account_category": "income",
          "account_code": "400000",
          "account_name": "Revenue",
          "account_uuid": "019d8d8a-9d51-7605-bbba-ea526ecc40a7",
          "amount": "-315000000",
          "credit": "315000000",
          "debit": "0",
          "line_no": 3,
          "memo": "Sales revenue",
          "transaction_uuid": "019dea2f-ab27-77d3-9b50-7471892a1844",
          "uuid": "019dea2f-ac08-7ff7-8224-08ee3bc21859"
        }
      ]
    },
    {
      "header": {
        "created_at": "2026-05-03T07:21:42.689979Z",
        "created_by": "019d8d89-e444-74b4-82c8-64a2142ef7cc",
        "description": "Transfering erroneous debit of credit of raw materials in place of finished goods",
        "module": "adjustment",
        "posted": true,
        "reference": "ADJ-001",
        "serial_id": 46,
        "txn_date": "2026-03-05",
        "uuid": "019decb6-e6b2-7e83-b218-a43a0616ede6"
      },
      "lines": [
        {
          "account_category": "asset",
          "account_code": "110701",
          "account_name": "Raw Materials Inventory",
          "account_uuid": "019d8d8a-9b04-735e-814d-b462acbabcde",
          "amount": "271580500",
          "credit": "0",
          "debit": "271580500",
          "line_no": 1,
          "memo": "Reversing figure credited from RM",
          "transaction_uuid": "019decb6-e6b2-7e83-b218-a43a0616ede6",
          "uuid": "019decb6-e6dc-72e3-9cd9-df185d42d679"
        },
        {
          "account_category": "asset",
          "account_code": "110703",
          "account_name": "Finished Goods Inventory",
          "account_uuid": "019d8d8a-9b1a-7843-830e-189ad7d4d383",
          "amount": "-271580500",
          "credit": "271580500",
          "debit": "0",
          "line_no": 2,
          "memo": "Crediting finished goods",
          "transaction_uuid": "019decb6-e6b2-7e83-b218-a43a0616ede6",
          "uuid": "019decb6-e703-7156-8dd2-66d8980423f1"
        }
      ]
    }
  ]
}