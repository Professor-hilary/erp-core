WITH cash_entries AS (
    SELECT 
        te.transaction_uuid,
        SUM(te.debit - te.credit) AS cash_delta   -- Total cash movement in this transaction
    FROM accounting.transaction_entries te
    JOIN accounting.accounts acc ON acc.uuid = te.account_uuid
    WHERE acc.cash_flow_category = 'cash'
      AND te.created_at >= $1 
      AND te.created_at < ($2 + INTERVAL '1 day')
    GROUP BY te.transaction_uuid
)
SELECT 
    acc.code,
    acc.name,
    acc.parent_code,
    acc.cash_flow_category AS category,
    SUM(te2.debit - te2.credit) AS balance
FROM cash_entries ce
JOIN accounting.transaction_entries te2 
    ON te2.transaction_uuid = ce.transaction_uuid
JOIN accounting.accounts acc 
    ON acc.uuid = te2.account_uuid
WHERE acc.cash_flow_category IS NOT NULL 
  AND acc.cash_flow_category != 'cash'
  AND acc.cash_flow_category != 'non-cash'
GROUP BY acc.code, acc.name, acc.parent_code, acc.cash_flow_category
ORDER BY acc.code;