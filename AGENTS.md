# Ledger Oxide — AGENTS.md

Single-tenant personal finance app (no auth, no users). Rust backend (axum + async-graphql 7 + sqlx/Postgres) in `backend/`, React 19 + Vite + TypeScript + urql + Tailwind v4 in `frontend/`. Domain conventions live in `docs/MODEL.md`, workflows in `docs/WORKFLOWS.md`, phase status in `docs/TODO.md` (Phases 1–6 done). No README; no CI.

## Run the stack
- `docker compose up -d` from repo root → postgres `:5432`, backend `:4000`, frontend `:5173`.
- Backend container runs `cargo watch -x run` and auto-rebuilds on any change under `backend/`; frontend container runs `npm install && npm run dev` (Vite HMR). SQLx migrations auto-apply on backend startup.
- Local backend without docker: `cargo run` in `backend/` — `backend/.env` points `DATABASE_URL` at `localhost:5432`. Run postgres alone with `docker compose up -d postgres`.
- This machine has no system npm; node/npm come from nix: `nix-env -iA nixpkgs.nodejs`.

## Verify
- Backend: `cargo build`; `cargo test` (only unit test is the OFX parser). No linter configured.
- Frontend: `npx tsc -b` (strict typecheck; there is no `typecheck`/`lint` script) or `npm run build` (= `tsc -b && vite build`).

## Backend gotchas (all hit in practice)
- async-graphql v7 `chrono` feature exposes `NaiveDate` as the GraphQL scalar **`NaiveDate`, not `Date`**. Queries must use `NaiveDate` (e.g. `bankEntries($dateFrom: NaiveDate)`; `forecast(from: NaiveDate, to: NaiveDate)`). `DateTime<Utc>` → `DateTime`.
- Rust field `_destroy` is exposed as GraphQL **`destroy`** (leading underscore stripped); frontend sends `destroy: true`.
- PG `SUM()` returns BIGINT — decoding into `i32` panics ("mismatched types … INT8"). Cast: `COALESCE(SUM(amount_cents),0)::INTEGER`.
- `bank_entries` → `account_entries` FK has no cascade; `deleteBankEntry` deletes child rows first (already implemented).
- All money is integer cents (`amount_cents`); display formatting is the frontend's job.
- Benign expected warnings: `DbPool`/`new` dead code in `src/db.rs`.

## Docker root-ownership gotchas
- Backend container builds as root into the mounted `backend/target/`. If `cargo build` fails with `Permission denied … .d`, delete root-owned `backend/target/debug/*.d` and rebuild.
- Host `npm install` in `frontend/` can fail EACCES if `node_modules` is root-owned from an earlier container run; `rm -rf node_modules` and retry.
- `docker-compose.yml` still has the obsolete `version:` key (harmless warning).

## Conventions
- No `createAccount` mutation — accounts are created implicitly when a new name is typed in an account field (`AccountSelect` frontend, `ensure_account` backend).
- GraphQL wire names are camelCase (Rust snake_case auto-converted).
- OFX/QFX import sends raw file text as `importBankStatement(input: { fileContent: String })` — no multipart `Upload`.
- Schema changes go in new numbered `backend/migrations/NNN_*.sql`; they apply automatically at backend startup.
- Keep `docs/TODO.md` in sync as phases land.
