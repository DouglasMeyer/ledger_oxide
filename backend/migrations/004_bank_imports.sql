CREATE TABLE bank_imports (
    id SERIAL PRIMARY KEY,
    balance_cents INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_bank_imports_created_at ON bank_imports(created_at DESC);
