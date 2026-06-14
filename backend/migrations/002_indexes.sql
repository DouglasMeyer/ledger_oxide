CREATE INDEX idx_bank_entries_date ON bank_entries(date DESC);
CREATE INDEX idx_account_entries_bank_entry ON account_entries(bank_entry_id);
CREATE INDEX idx_account_entries_account ON account_entries(account_id);
CREATE INDEX idx_account_entries_account_date ON account_entries(account_id, created_at DESC);
