This is the full, consistent, professional algorithm for a Standard Costing Manufacturing ERP — the kind that doesn’t collapse after 2 months in production.

# 🧠 CORE PRINCIPLE

Every cost belongs to exactly one of these buckets:

1. `Actual` → what really happened
2. `Applied (Standard)` → what should have happened
3. `Variance` → the difference

👉 If you mix these, your system breaks. Period.

⚔️ THE FULL MANUFACTURING ALGORITHM

We divide the system into `4 phases`:

1. Production Start
2. During Production (Cost Accumulation)
3. Production Completion
4. Period-End Adjustment

# 🧱 1. PRODUCTION START

### Inputs:

- Product

- Quantity ordered

- BOM (materials)

- Routing (labor)

- Standard cost (already set)


### System does:

Creates production order

```
status = 'In Progress'
quantity_ordered = X
quantity_completed = 0
```
### 👉 NO GL ENTRY HERE

# 🔄 2. DURING PRODUCTION (ACTUAL COST FLOW)

This is where real costs accumulate


## 2.1 Material Issue (Actual Material)

### Event:

Raw materials issued to production

### Journal:
```
Dr WIP
Cr Raw Material Inventory
```
### Data:
```
manufacturing.material_issues.total_cost
```
👉 This is actual material cost


## 2.2 Direct Labor (Actual Labor)

### Event:

Labor is incurred

### Journal:
```
Dr WIP
Cr Wages Payable / Cash
```
### Data:
```
manufacturing.cost_applications
type = 'DirectLabor'
amount = actual cost
```
👉 No rate. No standard. Pure actual.

## 2.3 Overhead Incurred (Actual Overhead)

### Event:

Utilities, rent, indirect labor, etc.

### Journal:
```
Dr Manufacturing Overhead Control
Cr Cash / Payables
```
### Data:
```
manufacturing.cost_applications
type = 'OverheadActual'
amount = actual cost
```
👉 NOT posted to WIP

👉 Sits in MOH Control

## 2.4 Overhead Applied (Standard Overhead)

### Event:

Apply overhead to production using a rate

### Formula:
```
Applied Overhead = Activity Base × Predetermined Rate
```
### Journal:
```
Dr WIP
Cr Manufacturing Overhead Control
```
### Data:
```
manufacturing.cost_applications
type = 'OverheadApplied'
amount = applied amount
```
👉 This is NOT actual

👉 This is standard allocation




# 🧠 STATE AFTER PRODUCTION

At this point:

### WIP contains:

- Actual materials

- Actual labor

- Applied overhead


### MOH Control contains:

- Actual overhead (debits)

- Applied overhead (credits)


👉 Balance = Over/Under applied overhead




# 🏁 3. PRODUCTION COMPLETION (STANDARD COSTING)

⚠️ This is where people mess up.

You DO NOT move actual cost to Finished Goods.

You move `STANDARD COST.`


## 3.1 Compute Standard Cost
```
Standard Cost = standard_cost_per_unit × quantity_completed
```

## 3.2 Move WIP → Finished Goods

Journal:
```
Dr Finished Goods
Cr WIP
```
👉 Amount = STANDARD COST


## 3.3 Record Inventory Movement
```
quantity = completed quantity
unit_cost = standard_cost
```

### ⚠️ IMPORTANT

At this point:

> WIP ≠ Standard Cost



Because WIP contains actual costs

👉 That difference becomes variance


# ⚖️ 4. VARIANCE CALCULATION (THE HEART)

This is done after completion


## 4.1 Gather Actual Costs

👉 ONLY from cost tables

NOT GL
```
Actual Material = SUM(material_issues.total_cost)

Actual Labor = SUM(cost_applications WHERE type='DirectLabor')

Actual Overhead = SUM(cost_applications WHERE type='OverheadActual')

Applied Overhead = SUM(cost_applications WHERE type='OverheadApplied')
```



## 4.2 Compute Standard Costs

From:

- BOM

- Routing

- Overhead rates


### Material Standard
```
Standard Material =
Σ(BOM qty × standard price × completed qty)
```

### Labor Standard
```
Standard Labor =
Σ(routing hours × standard rate × completed qty)
```

### Overhead Standard
```
Standard Overhead = Applied Overhead
```
👉 In standard costing:

> Applied = Standard


## 4.3 Compute Variances

### Material Variance
```
Material Variance = Actual Material - Standard Material
```

### Labor Variance
```
Labor Variance = Actual Labor - Standard Labor
```

### Overhead Variance
```
Overhead Variance = Actual Overhead - Applied Overhead
```

## 4.4 Post Variances to GL

### Material Variance
```
Dr/Cr Material Variance
Dr/Cr WIP (offset)
```

### Labor Variance
```
Dr/Cr Labor Variance
Dr/Cr WIP (offset)
```

### Overhead Variance
```
Dr/Cr Overhead Variance
Dr/Cr MOH Control (or WIP depending on design)
```



# 📅 5. PERIOD-END ADJUSTMENT (CRITICAL)

MOH Control must be cleared.

## Compute:

```
MOH Balance = Actual Overhead - Applied Overhead
```

## Close:
```
Dr/Cr Overhead Variance
Cr/Dr MOH Control
```

👉 Now MOH = 0


# 🧠 FINAL DATA FLOW SUMMARY

WHAT EACH TABLE REPRESENTS

| Table                          | Meaning                            |
| ------------------------------ | ---------------------------------- |
| material_issues                | Actual material                    |
| cost_applications              | (DirectLabor) Actual labor         |
| cost_applications              | (OverheadActual)	Actual overhead   |
| cost_applications              | (OverheadApplied) Applied overhead |
| accounting.transaction_entries | Ledger only                        |

# ⚠️ ABSOLUTE RULES (NO EXCEPTIONS)

❌ DO NOT use GL to compute cost

No:
```
SUM(debit - credit)
```
👉 That’s accounting, not costing


❌ DO NOT mix actual and applied

Never:
```
Actual + Applied → FG
```

❌ DO NOT skip overhead application

If you skip:
```
WIP is incomplete
Variance becomes meaningless
```

❌ DO NOT compute variance from WIP

WIP is a mix of flows

Always compute from:

material_issues

cost_applications


# 🧭 EXECUTION ORDER (THIS IS YOUR CHECKLIST)

When running production:

### During Production

1. Issue materials → WIP
2. Record labor → WIP
3. Record overhead actual → MOH
4. Apply overhead → WIP

### At Completion

5. Move standard cost → FG
6. Compute variances
7. Post variances

### Period End
8. Clear MOH control
