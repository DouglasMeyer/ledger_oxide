use async_graphql::{Context, InputObject, Object, Result, SimpleObject};
use chrono::{NaiveDate, DateTime, Utc};
use sqlx::PgPool;

use crate::graphql::accounts::{AccountRow, Account};

#[derive(Debug, sqlx::FromRow)]
pub struct BankEntryRow {
    pub id: i32,
    pub date: NaiveDate,
    pub amount_cents: i32,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(SimpleObject, Debug)]
pub struct BankEntry {
    pub id: i32,
    pub date: NaiveDate,
    pub amount_cents: i32,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub account_entries: Vec<AccountEntry>,
}

impl BankEntry {
    fn from_row(row: BankEntryRow, account_entries: Vec<AccountEntry>) -> Self {
        Self {
            id: row.id,
            date: row.date,
            amount_cents: row.amount_cents,
            description: row.description,
            notes: row.notes,
            external_id: row.external_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            account_entries,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct AccountEntryRow {
    pub id: i32,
    pub account_id: i32,
    pub bank_entry_id: i32,
    pub amount_cents: i32,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(SimpleObject, Debug)]
pub struct AccountEntry {
    pub id: i32,
    pub account_id: i32,
    pub bank_entry_id: i32,
    pub amount_cents: i32,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub account: Option<Account>,
}

impl AccountEntry {
    fn from_row(row: AccountEntryRow, account: Option<Account>) -> Self {
        Self {
            id: row.id,
            account_id: row.account_id,
            bank_entry_id: row.bank_entry_id,
            amount_cents: row.amount_cents,
            notes: row.notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
            account,
        }
    }
}

#[derive(InputObject)]
pub struct CreateBankEntryInput {
    pub date: NaiveDate,
    pub amount_cents: i32,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub account_entries: Vec<CreateAccountEntryInput>,
}

#[derive(InputObject)]
pub struct UpdateBankEntryInput {
    pub date: Option<NaiveDate>,
    pub amount_cents: Option<i32>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub account_entries: Option<Vec<UpdateAccountEntryInput>>,
}

#[derive(InputObject)]
pub struct CreateAccountEntryInput {
    pub account_name: String,
    pub amount_cents: i32,
    pub notes: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateAccountEntryInput {
    pub id: Option<i32>,
    pub account_name: Option<String>,
    pub amount_cents: Option<i32>,
    pub notes: Option<String>,
    pub _destroy: Option<bool>,
}

async fn ensure_account(pool: &PgPool, name: &str) -> Result<(AccountRow, bool)> {
    let existing = sqlx::query_as::<_, AccountRow>(
        "SELECT * FROM accounts WHERE name = $1 AND deleted_at IS NULL",
    )
    .bind(name)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("find account: {e}"))?;

    if let Some(account) = existing {
        Ok((account, false))
    } else {
        let row = sqlx::query_as::<_, AccountRow>(
            "INSERT INTO accounts (name) VALUES ($1) RETURNING *",
        )
        .bind(name)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("create account: {e}"))?;
        Ok((row, true))
    }
}

async fn fetch_account_entries_for_bank_entry(
    pool: &PgPool,
    bank_entry_id: i32,
) -> Result<Vec<AccountEntry>> {
    let rows = sqlx::query_as::<_, AccountEntryRow>(
        "SELECT * FROM account_entries WHERE bank_entry_id = $1",
    )
    .bind(bank_entry_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("fetch account entries: {e}"))?;

    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let account = sqlx::query_as::<_, AccountRow>(
            "SELECT * FROM accounts WHERE id = $1",
        )
        .bind(row.account_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("fetch account: {e}"))?
        .map(|r| {
            let balance_cents = 0i32;
            let active = false;
            Account::from_row(r, balance_cents, active)
        });

        entries.push(AccountEntry::from_row(row, account));
    }

    Ok(entries)
}

#[derive(Default)]
pub struct BankEntryQuery;

#[Object]
impl BankEntryQuery {
    async fn bank_entries(
        &self,
        ctx: &Context<'_>,
        date_from: Option<NaiveDate>,
        date_to: Option<NaiveDate>,
        account_id: Option<i32>,
    ) -> Result<Vec<BankEntry>> {
        let pool = ctx.data::<PgPool>()?;

        let mut sql = String::from(
            "SELECT be.* FROM bank_entries be",
        );

        if account_id.is_some() {
            sql.push_str(" JOIN account_entries ae ON ae.bank_entry_id = be.id");
        }

        let mut conditions: Vec<String> = Vec::new();
        let mut param_idx = 1u32;

        if date_from.is_some() {
            conditions.push(format!("be.date >= ${}", param_idx));
            param_idx += 1;
        }
        if date_to.is_some() {
            conditions.push(format!("be.date <= ${}", param_idx));
            param_idx += 1;
        }
        if account_id.is_some() {
            conditions.push(format!("ae.account_id = ${}", param_idx));
            param_idx += 1;
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(" ORDER BY be.date DESC, be.id DESC");

        let mut query = sqlx::query_as::<_, BankEntryRow>(&sql);

        if let Some(from) = date_from {
            query = query.bind(from);
        }
        if let Some(to) = date_to {
            query = query.bind(to);
        }
        if let Some(acct_id) = account_id {
            query = query.bind(acct_id);
        }

        let rows = query
            .fetch_all(pool)
            .await
            .map_err(|e| format!("fetch bank entries: {e}"))?;

        let mut entries = Vec::with_capacity(rows.len());
        for row in rows {
            let account_entries = fetch_account_entries_for_bank_entry(pool, row.id).await?;
            entries.push(BankEntry::from_row(row, account_entries));
        }

        Ok(entries)
    }

    async fn bank_entry(&self, ctx: &Context<'_>, id: i32) -> Result<Option<BankEntry>> {
        let pool = ctx.data::<PgPool>()?;

        let row = sqlx::query_as::<_, BankEntryRow>(
            "SELECT * FROM bank_entries WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("fetch bank entry: {e}"))?;

        match row {
            Some(row) => {
                let account_entries = fetch_account_entries_for_bank_entry(pool, row.id).await?;
                Ok(Some(BankEntry::from_row(row, account_entries)))
            }
            None => Ok(None),
        }
    }
}

#[derive(Default)]
pub struct BankEntryMutation;

#[Object]
impl BankEntryMutation {
    async fn create_bank_entry(
        &self,
        ctx: &Context<'_>,
        input: CreateBankEntryInput,
    ) -> Result<BankEntry> {
        let pool = ctx.data::<PgPool>()?;

        let row = sqlx::query_as::<_, BankEntryRow>(
            "INSERT INTO bank_entries (date, amount_cents, description, notes)
             VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(input.date)
        .bind(input.amount_cents)
        .bind(&input.description)
        .bind(&input.notes)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("create bank entry: {e}"))?;

        let mut account_entries = Vec::with_capacity(input.account_entries.len());
        for ae_input in input.account_entries {
            let (account, _) = ensure_account(pool, &ae_input.account_name).await?;

            let entry_row = sqlx::query_as::<_, AccountEntryRow>(
                "INSERT INTO account_entries (account_id, bank_entry_id, amount_cents, notes)
                 VALUES ($1, $2, $3, $4) RETURNING *",
            )
            .bind(account.id)
            .bind(row.id)
            .bind(ae_input.amount_cents)
            .bind(&ae_input.notes)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("create account entry: {e}"))?;

            let balance_cents = 0i32;
            let active = false;
            let acc = Account::from_row(account, balance_cents, active);
            account_entries.push(AccountEntry::from_row(entry_row, Some(acc)));
        }

        Ok(BankEntry::from_row(row, account_entries))
    }

    async fn update_bank_entry(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateBankEntryInput,
    ) -> Result<BankEntry> {
        let pool = ctx.data::<PgPool>()?;

        let current = sqlx::query_as::<_, BankEntryRow>(
            "SELECT * FROM bank_entries WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("fetch bank entry: {e}"))?
        .ok_or_else(|| format!("bank entry {id} not found"))?;

        let date = input.date.unwrap_or(current.date);
        let amount_cents = input.amount_cents.unwrap_or(current.amount_cents);
        let description = input.description.or(current.description);
        let notes = input.notes.or(current.notes);

        let row = sqlx::query_as::<_, BankEntryRow>(
            "UPDATE bank_entries
             SET date = $1, amount_cents = $2, description = $3, notes = $4, updated_at = NOW()
             WHERE id = $5 RETURNING *",
        )
        .bind(date)
        .bind(amount_cents)
        .bind(&description)
        .bind(&notes)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("update bank entry: {e}"))?;

        if let Some(ae_updates) = input.account_entries {
            for ae_input in ae_updates {
                if ae_input._destroy.unwrap_or(false) {
                    if let Some(ae_id) = ae_input.id {
                        sqlx::query("DELETE FROM account_entries WHERE id = $1")
                            .bind(ae_id)
                            .execute(pool)
                            .await
                            .map_err(|e| format!("delete account entry: {e}"))?;
                    }
                } else if let Some(ae_id) = ae_input.id {
                    if let Some(amount) = ae_input.amount_cents {
                        sqlx::query(
                            "UPDATE account_entries SET amount_cents = $1, updated_at = NOW()
                             WHERE id = $2",
                        )
                        .bind(amount)
                        .bind(ae_id)
                        .execute(pool)
                        .await
                        .map_err(|e| format!("update account entry: {e}"))?;
                    }
                    if let Some(notes) = &ae_input.notes {
                        sqlx::query(
                            "UPDATE account_entries SET notes = $1, updated_at = NOW()
                             WHERE id = $2",
                        )
                        .bind(notes)
                        .bind(ae_id)
                        .execute(pool)
                        .await
                        .map_err(|e| format!("update account entry notes: {e}"))?;
                    }
                    if let Some(ref name) = ae_input.account_name {
                        let (account, _) = ensure_account(pool, name).await?;
                        sqlx::query(
                            "UPDATE account_entries SET account_id = $1, updated_at = NOW()
                             WHERE id = $2",
                        )
                        .bind(account.id)
                        .bind(ae_id)
                        .execute(pool)
                        .await
                        .map_err(|e| format!("update account entry account: {e}"))?;
                    }
                } else if let Some(ref name) = ae_input.account_name {
                    let (account, _) = ensure_account(pool, name).await?;
                    let amount = ae_input.amount_cents.unwrap_or(0);

                    let _ = sqlx::query_as::<_, AccountEntryRow>(
                        "INSERT INTO account_entries (account_id, bank_entry_id, amount_cents, notes)
                         VALUES ($1, $2, $3, $4) RETURNING *",
                    )
                    .bind(account.id)
                    .bind(row.id)
                    .bind(amount)
                    .bind(&ae_input.notes)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| format!("create account entry: {e}"))?;
                }
            }
        }

        let account_entries = fetch_account_entries_for_bank_entry(pool, row.id).await?;
        Ok(BankEntry::from_row(row, account_entries))
    }

    async fn delete_bank_entry(&self, ctx: &Context<'_>, id: i32) -> Result<BankEntry> {
        let pool = ctx.data::<PgPool>()?;

        let account_entries = fetch_account_entries_for_bank_entry(pool, id).await?;

        let row = sqlx::query_as::<_, BankEntryRow>(
            "DELETE FROM bank_entries WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("delete bank entry: {e}"))?
        .ok_or_else(|| format!("bank entry {id} not found"))?;

        Ok(BankEntry::from_row(row, account_entries))
    }
}
