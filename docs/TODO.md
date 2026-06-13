# Ledger Oxide — TODO

## Priority Legend

- **[P0]** — Must have for MVP
- **[P1]** — Important, soon after MVP
- **[P2]** — Nice to have
- **[P3]** — Future / stretch

---

## Phase 1: Project Scaffolding & Database {P0}

- [ ] Initialize Rust workspace (`backend/`) with Cargo
  - Dependencies: `axum`, `async-graphql`, `sqlx` (postgres), `tokio`, `serde`,
    `chrono`, `tower-http`
- [ ] Initialize Vite + React + TypeScript project (`frontend/`)
  - Dependencies: `react`, `urql`, `graphql`, `date-fns`, `tailwindcss`
- [ ] Create `docker-compose.yml` with PostgreSQL
- [ ] Write SQLx migrations for the five tables:
  - `accounts`
  - `bank_entries`
  - `account_entries`
  - `projected_entries`
  - `bank_imports`
- [ ] Write performance indexes migration
- [ ] Set up Axum server skeleton with health check endpoint
- [ ] Set up SQLx connection pool
- [ ] Set up async-graphql with a placeholder `{ ok }` schema
- [ ] Configure CORS for frontend dev server

---

## Phase 2: Core GraphQL CRUD {P0}

### Accounts
- [ ] Implement `Query::accounts(active: Boolean)`
- [ ] Implement `Query::account(id: ID!)`
- [ ] **No `createAccount` mutation** — accounts are created implicitly during
      bank entry creation (see Bank Entries below)
- [ ] Implement `Mutation::updateAccount`
- [ ] Implement `Mutation::deleteAccount` (soft delete)
- [ ] Implement computed fields: `balanceCents`, `active`

### Bank Entries
- [ ] Implement `Query::bankEntries` (with date range, account filter)
- [ ] Implement `Query::bankEntry(id: ID!)`
- [ ] Implement `Mutation::createBankEntry` (with nested account entries)
- [ ] Implement account-on-the-fly creation in bank entry mutation — if an
      account name doesn't exist, create it with defaults (`asset = true`,
      `category = null`) and use it
- [ ] Implement `Mutation::updateBankEntry`
- [ ] Implement `Mutation::deleteBankEntry`

### Account Entries
- [ ] Handled as nested mutations within BankEntry (no standalone CRUD)
- [ ] Implement running balance window function query

---

## Phase 3: Projected Entries & Forecast {P0}

- [ ] Implement `Query::projectedEntries` (with accountId, active filters)
- [ ] Implement `Query::projectedEntry(id: ID!)`
- [ ] Implement `Mutation::createProjectedEntry`
- [ ] Implement `Mutation::updateProjectedEntry`
- [ ] Implement `Mutation::deleteProjectedEntry`
- [ ] Integrate `rrule` crate for RRULE expansion
- [ ] Implement `Query::forecast(from: Date!, to: Date!)` — merges real bank
      entries with projected entries expanded into date range
- [ ] Write tests for RRULE expansion edge cases

---

## Phase 4: OFX/QFX Import {P1}

- [ ] Build OFX file parser (custom or wrap existing OFX parser)
- [ ] Handle QFX format (same structure, different wrapper)
- [ ] Implement `Mutation::importBankStatement(file: Upload!)`
- [ ] External ID dedup: skip bank entries with matching `external_id`
- [ ] Auto-create BankEntries with zero account entries allocated
- [ ] Implement `Query::bankImports` (import history)
- [ ] Implement "needs distribution" query (bank entries where
      `amountCents != SUM(accountEntries.amountCents)`)

---

## Phase 5: Frontend — Skeleton & Navigation {P1}

- [ ] Set up routing: `/accounts`, `/transactions`, `/forecast`, `/import`,
      `/projections`
- [ ] Set up urql client pointing at backend GraphQL endpoint
- [ ] Set up Tailwind CSS theme
- [ ] Create shared layout (sidebar or top nav)
- [ ] Create shared UI components:
  - [ ] CurrencyInput (cents ↔ formatted display)
  - [ ] DatePicker
  - [ ] AccountSelect (type-to-create — no separate create form; typing a
        new name and pressing Enter creates the account implicitly)
  - [ ] SplitRowsWidget (dynamic add/remove split rows)
  - [ ] DataTable (sortable, filterable)
  - [ ] Modal / Drawer
- [ ] Create account list page
- [ ] Create transaction list page (with date + account filters)

---

## Phase 6: Frontend — Transaction Entry {P1}

- [ ] Build new transaction form:
  - [ ] Date, amount, description fields
  - [ ] Split rows with inline account creation
  - [ ] Running allocation total vs. transaction total indicator
- [ ] Build edit transaction form (reuse create form)
- [ ] Build transaction detail view
- [ ] Handle account-on-the-fly creation in the dropdown component

---

## Phase 7: Frontend — Forecast & Projections {P1}

- [ ] Build forecast view: timeline or calendar merging real + projected entries
- [ ] Build projected entry form (with RRULE builder UI)
- [ ] Build projected entry list / management page
- [ ] Handle RRULE representation: human-readable summary + edit

---

## Phase 8: Frontend — Import & Polish {P2}

- [ ] Build import page with file upload
- [ ] Build import preview (transactions found vs. already imported)
- [ ] Build "needs distribution" queue view
- [ ] Add account detail page with balance history chart
- [ ] Add balance trend visualization (simple line chart)
- [ ] Keyboard shortcuts for fast data entry (Tab to next field, Ctrl+Enter to
      save)
- [ ] PWA: manifest.json, service worker for offline access
- [ ] Responsive layout for mobile/tablet

---

## Phase 9: Testing & Hardening {P1}

- [ ] Write Rust integration tests for GraphQL mutations
- [ ] Write Rust unit tests for balance calculation
- [ ] Write Rust unit tests for OFX parser
- [ ] Write Rust unit tests for RRULE expansion
- [ ] Set up frontend testing (Vitest + React Testing Library)
- [ ] Write frontend tests for critical forms (transaction entry)
- [ ] Load test balance calculation with 100k+ account entries
- [ ] Add SQLx compile-time query checking

---

## Future / Stretch {P3}

- [ ] **Reconciliation** — cleared/uncleared flags, statement matching UI
- [ ] **Budgets** — monthly spending targets by account/category, with progress
      tracking
- [ ] **Rules engine** — auto-categorize transactions based on description
      patterns (e.g., "COSTCO*" → Groceries)
- [ ] **Categories as first-class model** — CRUD for categories, bulk rename
- [ ] **Multi-currency** — exchange rate support, per-account currency
- [ ] **Scheduled auto-import** — Plaid integration or scheduled OFX download
- [ ] **Reports** — spending by category over time, income v. expense, net
      worth trend
- [ ] **Data export** — CSV/JSON export for all entities
- [ ] **API tokens** — for scripted access / external tools
- [ ] **Dark mode** — theme toggle
- [ ] **i18n** — localization support
- [ ] **Notifications** — email or in-app reminders for upcoming projected
      entries
- [ ] **Attachments** — attach receipts/images to bank entries
