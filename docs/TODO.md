# Ledger Oxide — TODO

## Priority Legend

- **[P0]** — Must have for MVP
- **[P1]** — Important, soon after MVP
- **[P2]** — Nice to have
- **[P3]** — Future / stretch

---

## Phase 1: Project Scaffolding & Database {P0} ✓

- [x] Initialize Rust workspace with Cargo (axum, async-graphql, sqlx, tokio, etc.)
- [x] Initialize Vite + React + TypeScript project
- [x] Create `docker-compose.yml` with PostgreSQL + backend + frontend
- [x] Write SQLx migrations (5 tables + indexes + projected_entry date column)
- [x] Set up Axum server + SQLx pool + async-graphql + CORS
- [x] Derive in `docker-compose.yml`: backend/frontend Dockerfiles, cargo-watch, hot-reload

---

## Phase 2: Core GraphQL CRUD {P0} ✓

### Accounts
- [x] `Query::accounts(active: Boolean)` — with balance + active computed fields
- [x] `Query::account(id: ID!)`
- [x] No `createAccount` mutation — accounts created implicitly during entry
- [x] `Mutation::updateAccount` (name, asset, category, position)
- [x] `Mutation::deleteAccount` (soft delete via `deleted_at`)

### Bank Entries
- [x] `Query::bankEntries` (with date range + account filter via dynamic SQL)
- [x] `Query::bankEntry(id: ID!)` — includes nested account entries with account detail
- [x] `Mutation::createBankEntry` — inline account creation via `account_name`
- [x] `Mutation::updateBankEntry` — with account entry add/remove/modify
- [x] `Mutation::deleteBankEntry` (hard delete, returns deleted entry + its children)

### Account Entries
- [x] Nested within BankEntry mutations (no standalone CRUD)
- [x] Running balance via `COALESCE(SUM(amount_cents), 0)` per account

---

## Phase 3: Projected Entries & Forecast {P0} ✓

- [x] `Query::projectedEntries` (with accountId + active filters)
- [x] `Query::projectedEntry(id: ID!)`
- [x] `Mutation::createProjectedEntry` — with date, optional rrule
- [x] `Mutation::updateProjectedEntry`
- [x] `Mutation::deleteProjectedEntry`
- [x] `rrule` crate integrated — `expand_rrule()` parses RRULE, filters by date range
- [x] `Query::forecast(from: Date!, to: Date!)` — merges real bank entries with
      expanded projected entries, sorted by date
- [x] `date` column added to `projected_entries` (migration 003) — required for both
      one-time entries (no rrule) and DTSTART for recurring entries
- [ ] Write tests for RRULE expansion edge cases

---

## Phase 4: OFX/QFX Import {P1} ✓

- [x] Build OFX file parser (custom regex-based, same approach as original Rails app)
- [x] Handle QFX format (same structure, different wrapper)
- [x] Implement `Mutation::importBankStatement(fileContent: String!)` — client reads
      file and sends content as string (avoids multipart upload complexity)
- [x] External ID dedup: skip bank entries with matching `external_id`
- [x] Auto-create BankEntries with zero account entries allocated
- [x] Implement `Query::bankImports` (import history)
- [x] Implement `Query::bankEntriesNeedingDistribution` (bank entries where
      `amountCents != SUM(accountEntries.amountCents)`)
- [x] Bank imports table (migration 004)

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
