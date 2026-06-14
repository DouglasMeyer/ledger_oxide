CREATE TABLE accounts (
    id          SERIAL PRIMARY KEY,
    name        VARCHAR(255) NOT NULL UNIQUE,
    asset       BOOLEAN NOT NULL DEFAULT true,
    category    VARCHAR(255),
    position    INTEGER,
    deleted_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE bank_entries (
    id            SERIAL PRIMARY KEY,
    date          DATE NOT NULL,
    amount_cents  INTEGER NOT NULL,
    description   VARCHAR(255),
    notes         TEXT,
    external_id   VARCHAR(255) UNIQUE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE account_entries (
    id            SERIAL PRIMARY KEY,
    account_id    INTEGER NOT NULL REFERENCES accounts(id),
    bank_entry_id INTEGER NOT NULL REFERENCES bank_entries(id),
    amount_cents  INTEGER NOT NULL,
    notes         TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE projected_entries (
    id            SERIAL PRIMARY KEY,
    account_id    INTEGER NOT NULL REFERENCES accounts(id),
    description   VARCHAR(255),
    amount_cents  INTEGER NOT NULL,
    rrule         VARCHAR(255),
    active        BOOLEAN NOT NULL DEFAULT true,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE bank_imports (
    id            SERIAL PRIMARY KEY,
    balance_cents INTEGER NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
