# Ledger Oxide — Data Model

## Overview

Ledger Oxide is a single-tenant, intranet personal finance tracking application. It
replaces the existing Rails-based [Ledger](https://github.com/DouglasMeyer/ledger)
project with a Rust backend (Axum + async-graphql + SQLx) and a React frontend
(Vite + TypeScript + urql).

The core domain is straightforward: money flows in and out of **accounts** via
**bank entries**, and those amounts are distributed across accounts through
**account entries**. **Projected entries** let you forecast future transactions
on a calendar.

Tables removed from the original Rails app:
- `users` — single-tenant, no authentication needed
- `sessions` — no server-side session storage
- `strategies` — replaced by projected entries (single-account projections)
- `tenant_ledger` — no multi-tenancy

---

## `accounts`

A named bucket that holds money. Accounts track where money lives or is owed.
Accounts are never created via a dedicated form or page — they are created
implicitly when the user types a new account name during transaction entry.

| Column | Type | Notes |
|---|---|---|
| `id` | `SERIAL` | Primary key |
| `name` | `VARCHAR(255)` | Unique, required |
| `asset` | `BOOLEAN` | `true` = asset/income, `false` = liability/expense |
| `category` | `VARCHAR(255)` | Optional grouping label (e.g. "Food", "Housing") |
| `position` | `INTEGER` | Display ordering hint |
| `deleted_at` | `TIMESTAMPTZ` | Soft-delete — accounts are never hard-deleted |
| `created_at` | `TIMESTAMPTZ` | |
| `updated_at` | `TIMESTAMPTZ` | |

**Why soft-delete?** Deleting an account would orphan its historical account
entries, breaking balance calculations. Soft-delete preserves history while
hiding the account from default views.

**Why unique name?** Account names are the primary user-facing identifier.
Duplicates would cause confusion in transaction entry (where accounts are
created implicitly by name) and in balance reports.

**How are accounts created?** Purely on-the-fly during transaction entry. The
user types a name in the account field. If it doesn't match an existing account,
the backend creates one with sensible defaults (`asset = true`, `category = null`,
`position = null`). The user can later edit the account's type, category, and
position from the account list page. There is no standalone "create account"
mutation or form.

**Asset vs liability semantics:**
- Assets (bank accounts, cash, investments) have positive balances
- Liabilities (credit cards, loans) have negative balances
- Income accounts (paychecks, interest) are assets with positive flows
- Expense accounts (groceries, rent) are liabilities with negative flows

### Computed fields

- **`balanceCents`** — running total of all account entries for this account,
  computed via SQL window function over `account_entries` joined to
  `bank_entries` (ordered by date).

- **`active`** — `true` if the account has any account entries in the last 90
  days OR has a non-zero balance. Used to hide stale accounts from the main
  list while keeping them accessible in dropdowns (listed last).

---

## `bank_entries`

A single financial transaction — typically one line from a bank statement (OFX
import) or a manually entered transaction.

| Column | Type | Notes |
|---|---|---|
| `id` | `SERIAL` | Primary key |
| `date` | `DATE` | Required |
| `amount_cents` | `INTEGER` | Required, positive for inflows, negative for outflows |
| `description` | `VARCHAR(255)` | Optional human-readable label |
| `notes` | `TEXT` | Optional free-form notes |
| `external_id` | `VARCHAR(255)` | Unique, from OFX import (used for dedup) |
| `created_at` | `TIMESTAMPTZ` | |
| `updated_at` | `TIMESTAMPTZ` | |

**Why cents as integer?** Avoid floating-point rounding errors in financial
calculations. All monetary amounts are stored as integer cents. Display
formatting (currency symbol, decimal point) is the presentation layer's
responsibility.

**Why `external_id` is unique?** Bank statement imports (OFX/QFX) assign each
transaction a unique identifier. When re-importing (e.g., downloading a new
statement that overlaps with a previous one), this field prevents duplicate
entries.

**Why no foreign key to accounts directly?** A single bank entry can be split
across multiple accounts (e.g., a $100 purchase at a department store might be
$60 "Clothing" and $40 "Household Goods"). The M:N relationship is handled by
`account_entries`.

### Computed fields

- **`amount`** — `amount_cents / 100.0` as a decimal for display.
- **`fromBank`** — `external_id IS NOT NULL`.
- **`amountRemaining`** — `amount_cents - SUM(account_entries.amount_cents)`,
  used to track how much of a bank entry hasn't been allocated to accounts yet.

---

## `account_entries`

The join table that allocates a bank entry's amount across one or more accounts.

| Column | Type | Notes |
|---|---|---|
| `id` | `SERIAL` | Primary key |
| `account_id` | `INTEGER` | FK → `accounts(id)`, required |
| `bank_entry_id` | `INTEGER` | FK → `bank_entries(id)`, required |
| `amount_cents` | `INTEGER` | Required, can be positive or negative |
| `notes` | `TEXT` | Optional per-split note |
| `created_at` | `TIMESTAMPTZ` | |
| `updated_at` | `TIMESTAMPTZ` | |

**Why not just store account_id on bank_entries?** Because transactions can be
split. A $150 grocery bill might be $100 "Groceries" and $50 "Household Goods".
The join pattern keeps it clean and extensible.

**Why is `amount_cents` per entry rather than a percentage?** Fixed amounts are
more intuitive for users to enter and don't require recalculation when the bank
entry amount changes. Percentages could be added as a convenience layer later.

---

## `projected_entries`

A predicted future transaction — either one-time or recurring (via RRULE).

| Column | Type | Notes |
|---|---|---|
| `id` | `SERIAL` | Primary key |
| `account_id` | `INTEGER` | FK → `accounts(id)`, required |
| `description` | `VARCHAR(255)` | Optional |
| `amount_cents` | `INTEGER` | Required |
| `rrule` | `VARCHAR(255)` | RRULE string; `null` means one-time |
| `active` | `BOOLEAN` | Default `true`, used to disable without deleting |
| `created_at` | `TIMESTAMPTZ` | |
| `updated_at` | `TIMESTAMPTZ` | |

**Why did projected entries replace strategies?** In the original Rails app,
strategies were allocation rules (fixed amount, percent of income, amount per
month) attached to accounts. Projected entries serve the same purpose more
directly: they project when money will flow and to which account. A single
model that handles both forecasting and allocation is simpler than two models
that overlap.

**Why single-account instead of multi-account splits?** A projected entry
predicts one line item for one account. To forecast a multi-account
distribution, you create multiple projected entries. This keeps the model flat
and matches how users think: "I get paid $2000 into Checking" and "I pay $1200
rent" are separate mental models.

**Why RRULE and not a simpler recurrence model?** RRULE (RFC 5545) is the
standard for recurring events. It handles weekly, monthly, yearly, every-N-days,
complex patterns like "third Thursday of every month", and has a well-defined
expansion algorithm. The `rrule` Rust crate can expand these into concrete
dates for the forecast view.

**What does `rrule = null` mean?** A one-time projection — it occurs once and
never repeats. Useful for known future transactions like an upcoming bill or
estimated tax payment.

### Computed / expanded fields

- **`nextOccurrence`** — the next date this projected entry fires (based on
  `rrule` expansion from today).
- **`occurrences(from, to)`** — all concrete dates within a range, computed by
  expanding the RRULE.

---

## `bank_imports`

Tracks the import of bank statements (OFX/QFX files).

| Column | Type | Notes |
|---|---|---|
| `id` | `SERIAL` | Primary key |
| `balance_cents` | `INTEGER` | Statement ending balance |
| `created_at` | `TIMESTAMPTZ` | |

**Why so sparse?** The original Rails app stored minimal import metadata. The
imported bank entries themselves carry the data (via `bank_entries.external_id`
linking back to the OFX statement). This table serves as an audit log of when
imports happened and what the stated balance was. Future improvements could
add the raw file content, import duration, number of entries created, etc.

---

## Relationship Summary

```
Account 1───* AccountEntry *───1 BankEntry
  │                                  │
  │                                  │
  └────────* ProjectedEntry     BankImport
```

- An **Account** has many **AccountEntries**, which belong to a **BankEntry**.
- A **BankEntry** has many **AccountEntries**, which belong to an **Account**.
- An **Account** has many **ProjectedEntries** (optional future transactions).
- `bank_entries` ← `account_entries` → `accounts` forms the core transaction
  model.
- `projected_entries` are forecast-only; they do not auto-generate bank entries.

---

## Balance Calculation

The running balance for an account is computed using a SQL window function:

```sql
SELECT
  be.date,
  ae.amount_cents,
  SUM(ae.amount_cents) OVER (
    ORDER BY be.date, ae.id
    ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
  ) AS running_balance_cents
FROM account_entries ae
JOIN bank_entries be ON be.id = ae.bank_entry_id
WHERE ae.account_id = $1
  AND be.deleted_at IS NULL
ORDER BY be.date, ae.id;
```

This avoids the heavy subquery approach used in the Rails app and is easily
materialized into a summary table if performance becomes an issue.

---

## Active Account Determination

An account is considered **active** (shown in default lists) if:

```sql
-- Has entries in last 90 days
EXISTS (
  SELECT 1 FROM account_entries ae
  JOIN bank_entries be ON be.id = ae.bank_entry_id
  WHERE ae.account_id = accounts.id
    AND be.date >= CURRENT_DATE - INTERVAL '90 days'
)
-- OR has a non-zero balance
OR (
  SELECT COALESCE(SUM(ae.amount_cents), 0)
  FROM account_entries ae
  WHERE ae.account_id = accounts.id
) != 0
```

Inactive accounts are hidden from the main account list but shown at the end of
dropdown selectors when entering transactions.
