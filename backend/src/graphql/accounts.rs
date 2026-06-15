use async_graphql::{Context, InputObject, Object, Result, SimpleObject};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, sqlx::FromRow)]
pub struct AccountRow {
    pub id: i32,
    pub name: String,
    pub asset: bool,
    pub category: Option<String>,
    pub position: Option<i32>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(SimpleObject, Debug)]
pub struct Account {
    pub id: i32,
    pub name: String,
    pub asset: bool,
    pub category: Option<String>,
    pub position: Option<i32>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub balance_cents: i32,
    pub active: bool,
}

impl Account {
    pub(crate) fn from_row(row: AccountRow, balance_cents: i32, active: bool) -> Self {
        Self {
            id: row.id,
            name: row.name,
            asset: row.asset,
            category: row.category,
            position: row.position,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            balance_cents,
            active,
        }
    }
}

#[derive(InputObject)]
pub struct UpdateAccountInput {
    pub name: Option<String>,
    pub asset: Option<bool>,
    pub category: Option<String>,
    pub position: Option<i32>,
}

async fn fetch_account_balance(pool: &PgPool, account_id: i32) -> Result<i32> {
    let balance: Option<(i32,)> = sqlx::query_as(
        "SELECT COALESCE(SUM(amount_cents), 0) FROM account_entries WHERE account_id = $1",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("fetch balance: {e}"))?;

    Ok(balance.map(|b| b.0).unwrap_or(0))
}

async fn fetch_account_active(pool: &PgPool, account_id: i32, balance_cents: i32) -> Result<bool> {
    if balance_cents != 0 {
        return Ok(true);
    }

    let recent: Option<(bool,)> = sqlx::query_as(
        "SELECT EXISTS (
           SELECT 1 FROM account_entries ae
           JOIN bank_entries be ON be.id = ae.bank_entry_id
           WHERE ae.account_id = $1
             AND be.date >= CURRENT_DATE - INTERVAL '90 days'
         )",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("fetch active: {e}"))?;

    Ok(recent.map(|r| r.0).unwrap_or(false))
}

#[derive(Default)]
pub struct AccountQuery;

#[Object]
impl AccountQuery {
    async fn accounts(
        &self,
        ctx: &Context<'_>,
        active: Option<bool>,
    ) -> Result<Vec<Account>> {
        let pool = ctx.data::<PgPool>()?;

        let rows = if active.unwrap_or(true) {
            sqlx::query_as::<_, AccountRow>(
                "SELECT * FROM accounts
                 WHERE deleted_at IS NULL
                   AND (
                     EXISTS (
                       SELECT 1 FROM account_entries ae
                       JOIN bank_entries be ON be.id = ae.bank_entry_id
                       WHERE ae.account_id = accounts.id
                         AND be.date >= CURRENT_DATE - INTERVAL '90 days'
                     )
                     OR (
                       SELECT COALESCE(SUM(ae2.amount_cents), 0)
                       FROM account_entries ae2
                       WHERE ae2.account_id = accounts.id
                     ) != 0
                   )
                 ORDER BY position ASC, name ASC",
            )
            .fetch_all(pool)
            .await
            .map_err(|e| format!("fetch active accounts: {e}"))?
        } else {
            sqlx::query_as::<_, AccountRow>(
                "SELECT * FROM accounts
                 WHERE deleted_at IS NULL
                 ORDER BY position ASC, name ASC",
            )
            .fetch_all(pool)
            .await
            .map_err(|e| format!("fetch all accounts: {e}"))?
        };

        let mut accounts = Vec::with_capacity(rows.len());
        for row in rows {
            let balance_cents = fetch_account_balance(pool, row.id).await?;
            let active_flag = fetch_account_active(pool, row.id, balance_cents).await?;
            accounts.push(Account::from_row(row, balance_cents, active_flag));
        }

        Ok(accounts)
    }

    async fn account(&self, ctx: &Context<'_>, id: i32) -> Result<Option<Account>> {
        let pool = ctx.data::<PgPool>()?;

        let row = sqlx::query_as::<_, AccountRow>(
            "SELECT * FROM accounts WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("fetch account: {e}"))?;

        match row {
            Some(row) => {
                let balance_cents = fetch_account_balance(pool, row.id).await?;
                let active_flag = fetch_account_active(pool, row.id, balance_cents).await?;
                Ok(Some(Account::from_row(row, balance_cents, active_flag)))
            }
            None => Ok(None),
        }
    }
}

#[derive(Default)]
pub struct AccountMutation;

#[Object]
impl AccountMutation {
    async fn update_account(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateAccountInput,
    ) -> Result<Account> {
        let pool = ctx.data::<PgPool>()?;

        let current = sqlx::query_as::<_, AccountRow>(
            "SELECT * FROM accounts WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("fetch account: {e}"))?
        .ok_or_else(|| format!("account {id} not found"))?;

        let name = input.name.unwrap_or(current.name);
        let asset = input.asset.unwrap_or(current.asset);
        let category = input.category.or(current.category);
        let position = input.position.or(current.position);

        let row = sqlx::query_as::<_, AccountRow>(
            "UPDATE accounts
             SET name = $1, asset = $2, category = $3, position = $4, updated_at = NOW()
             WHERE id = $5 AND deleted_at IS NULL
             RETURNING *",
        )
        .bind(&name)
        .bind(asset)
        .bind(&category)
        .bind(position)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("update account: {e}"))?;

        let balance_cents = fetch_account_balance(pool, row.id).await?;
        let active_flag = fetch_account_active(pool, row.id, balance_cents).await?;
        Ok(Account::from_row(row, balance_cents, active_flag))
    }

    async fn delete_account(&self, ctx: &Context<'_>, id: i32) -> Result<Account> {
        let pool = ctx.data::<PgPool>()?;

        let row = sqlx::query_as::<_, AccountRow>(
            "UPDATE accounts
             SET deleted_at = NOW(), updated_at = NOW()
             WHERE id = $1 AND deleted_at IS NULL
             RETURNING *",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("delete account: {e}"))?
        .ok_or_else(|| format!("account {id} not found"))?;

        let balance_cents = fetch_account_balance(pool, row.id).await?;
        let active_flag = fetch_account_active(pool, row.id, balance_cents).await?;
        Ok(Account::from_row(row, balance_cents, active_flag))
    }
}
