# Ledger Oxide — User Workflows

## Overview

This document captures the primary workflows a user performs in Ledger Oxide.
These are the user-facing interactions that the frontend must support and the
backend must enable.

---

## 1. Managing Accounts

### 1.1 Viewing Accounts

The user opens the accounts page and sees a list of all active accounts.

- Accounts are sorted: active accounts first (used in last 90 days or non-zero
  balance), then inactive accounts.
- Each row shows: name, type (asset/liability), category, current balance.
- A toggle lets the user switch between "Active only" and "All accounts"
  (including inactive and soft-deleted).

**Note:** There is no "Create Account" form or page. Accounts are created
implicitly when the user types a new name during transaction entry (see §2.1).
New accounts get sensible defaults (`asset = true`, no category) and can be
edited afterward.

### 1.2 Editing an Account

From the account list, the user clicks an account to edit its name, type,
category, or position. Name uniqueness is enforced.

### 1.3 Deleting an Account

Soft-delete: the account is hidden from all default views. Its historical
account entries remain for balance calculations. A "Show deleted" toggle
reveals soft-deleted accounts for potential restoration.

---

## 2. Entering Transactions

### 2.1 Simple Transaction (Single Account)

The most common workflow:

1. Click "New Transaction".
2. Enter the **date** (defaults to today).
3. Enter the **total amount** (positive for inflow, negative for outflow, or
   use a sign toggle).
4. Enter a **description** (e.g. "Groceries at Costco").
5. Under "Split", a single row appears by default:
   - **Account**: start typing an account name. The dropdown shows matching
     active accounts first, then inactive accounts. If the typed name doesn't
     match any account, the user can press Enter to **create the account on the
     fly**. The account is created with default settings (`asset = true`,
     no category) — the user never sees a creation form.
   - **Amount**: auto-filled to the total, editable for splits.
   - **Notes**: optional per-split note.
6. Click "Save".

Result: a BankEntry is created with one AccountEntry. If a new account was
typed, it's created automatically with no additional steps.

### 2.2 Split Transaction (Multiple Accounts)

Same as above, but the user clicks "Add Split" to add more rows. Each row is
an account + amount pair. The UI shows a running total of the allocated amount
vs. the transaction total, with a visual indicator when they match.

Example: A $100 department store purchase split:
- $60 → Clothing (account created on the fly)
- $40 → Household Goods (existing account)

### 2.3 Editing a Transaction

From the transaction list, click a transaction to edit it. The user can:
- Change the date, amount, description, or notes.
- Add, remove, or modify account entry splits.
- If a split's account is set to a new name, a new account is created.

### 2.4 Deleting a Transaction

Removes the BankEntry and all associated AccountEntries. Balances update
immediately.

### 2.5 Viewing the Transaction List

Paginated list of all BankEntries, sorted by date (descending). Each row shows:
- Date
- Description
- Total amount
- Account names (as tags/badges)
- Running account-level or overall balance

Filtering:
- By date range (date picker)
- By account (select from dropdown)
- By text search on description

---

## 3. Importing Bank Statements (OFX/QFX)

1. Navigate to "Import" page.
2. Upload an OFX or QFX file.
3. The system parses the file and displays a preview of all transactions found.
4. For each transaction:
   - If `external_id` matches an existing BankEntry, it's shown as
     "Already imported" (skipped).
   - New transactions are listed as ready to import.
   - The user can edit description/amount before confirming.
5. Click "Confirm Import" to create all new BankEntries.
   - New BankEntries are created with zero account entries allocated.
   - The user is directed to a page showing unallocated transactions
     (transactions that need splitting).

### 3.1 Bulk Allocation After Import

After import, the "Needs Distribution" view shows all BankEntries where
`amount_remaining != 0`. The user can quickly split each one:

- Click "Distribute" on a row.
- The split UI opens (same as 2.1/2.2).
- Save to create the AccountEntries.

---

## 4. Projecting Future Transactions

### 4.1 Creating a One-Time Projection

1. Navigate to "Forecast" or "Projections".
2. Click "New Projection".
3. Fill in:
   - **Description** (e.g. "Q4 estimated tax payment")
   - **Amount** ($5000)
   - **Account** (select from dropdown)
   - **Date** (specific date, e.g. Jan 15, 2027)
4. Leave "Repeat" off (one-time).
5. Save.

This creates a projected entry that appears on the forecast for that date.

### 4.2 Creating a Recurring Projection

Same flow, but toggle "Repeat" on and configure:
- **RRULE** via a friendly UI:
  - Frequency: Daily / Weekly / Monthly / Yearly / Custom
  - Interval: Every 1 / 2 / 3 ... (e.g., every 2 weeks)
  - End: Never / After N occurrences / On date
  - Day-of-week / day-of-month selectors for complex patterns
- Example: "Every 2 weeks on Friday starting Sep 1, 2026" →
  `FREQ=WEEKLY;INTERVAL=2;BYDAY=FR`

### 4.3 Viewing the Forecast

The forecast page shows a calendar or timeline view merging two data sources:

1. **Real transactions** (BankEntries) that have already occurred.
2. **Projected entries** expanded into concrete dates via RRULE.

The user sees a unified view: past transactions on the left, future projections
on the right, with a running balance projection line.

### 4.4 Editing / Disabling a Projection

- Edit any field (description, amount, account, RRULE).
- Toggle `active` off to keep the projection in the database but hide it from
  the forecast (useful for paused subscriptions).

### 4.5 Deleting a Projection

Permanently removes the projected entry. Historical bank entries that were
matched to it are not affected.

---

## 5. Reconciliation / Balancing

### 5.1 Viewing Account Balances

Each account shows its computed balance (from AccountEntries). The user can
see:
- Current balance.
- Balance history over time (chart or table).
- Statement ending balance from the last BankImport.

### 5.2 Reconciling with a Statement

1. Import the latest bank statement (OFX/QFX).
2. The system shows the statement's ending balance.
3. The user can run a reconciliation report:
   - Statement balance vs. calculated balance
   - List of transactions in the system not on the statement
   - List of transactions on the statement not in the system
4. The user resolves discrepancies:
   - Add missing transactions manually.
   - Mark matched transactions as cleared.

*(Note: reconciliation is a v1 simplification. The initial version tracks
balances via imports and manual entry; formal reconciliation with cleared/
uncleared flags is a future addition.)*

---

## 6. Administrative Tasks

### 6.1 Managing Categories

Categories are free-text on accounts (not a separate model). To rename a
category, the user edits each account. A future improvement could promote
categories to a first-class model with bulk rename.

### 6.2 Data Export

Export all data as CSV or JSON:
- Accounts list
- Transactions (BankEntries + AccountEntries)
- Account balances as of a given date

---

## Workflow Dependency Graph

```
Transaction Entry ─── accounts created implicitly
    │
    ├──→ Balance Calculation
    │
    ├──→ Forecast View ←── Projections
    │
    └──← Bank Import ──→ Needs Distribution
```

- Accounts are created implicitly during transaction entry; there is no
  separate account creation step.
- Bank imports create transactions that need distribution.
- Projections and transactions feed into the forecast.
- All account entries feed into balance calculations.
