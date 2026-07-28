use async_graphql::{Context, InputObject, Object, Result, SimpleObject};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::services::ofx::parse_ofx;

#[derive(Debug, sqlx::FromRow)]
pub struct BankImportRow {
    pub id: i32,
    pub balance_cents: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(SimpleObject, Debug)]
pub struct BankImport {
    pub id: i32,
    pub balance_cents: i32,
    pub created_at: DateTime<Utc>,
}

impl BankImport {
    fn from_row(row: BankImportRow) -> Self {
        Self {
            id: row.id,
            balance_cents: row.balance_cents,
            created_at: row.created_at,
        }
    }
}

#[derive(SimpleObject, Debug)]
pub struct ImportResult {
    pub bank_import: BankImport,
    pub created_count: i32,
    pub skipped_count: i32,
    pub entries: Vec<ImportedEntry>,
}

#[derive(SimpleObject, Debug)]
pub struct ImportedEntry {
    pub id: i32,
    pub external_id: Option<String>,
    pub date: chrono::NaiveDate,
    pub amount_cents: i32,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub was_skipped: bool,
}

#[derive(InputObject)]
pub struct ImportBankStatementInput {
    pub file_content: String,
}

#[derive(Default)]
pub struct BankImportQuery;

#[Object]
impl BankImportQuery {
    async fn bank_imports(&self, ctx: &Context<'_>) -> Result<Vec<BankImport>> {
        let pool = ctx.data::<PgPool>()?;

        let rows = sqlx::query_as::<_, BankImportRow>(
            "SELECT * FROM bank_imports ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| format!("fetch bank imports: {e}"))?;

        Ok(rows.into_iter().map(BankImport::from_row).collect())
    }

    async fn bank_entries_needing_distribution(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<crate::graphql::bank_entries::BankEntry>> {
        let pool = ctx.data::<PgPool>()?;

        let rows = sqlx::query_as::<_, crate::graphql::bank_entries::BankEntryRow>(
            "SELECT be.*
             FROM bank_entries be
             WHERE be.amount_cents != 0
               AND COALESCE((
                 SELECT SUM(ae.amount_cents)
                 FROM account_entries ae
                 WHERE ae.bank_entry_id = be.id
               ), 0) != be.amount_cents
             ORDER BY be.date DESC, be.id DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| format!("fetch unallocated entries: {e}"))?;

        use crate::graphql::bank_entries::{BankEntry, fetch_account_entries_for_bank_entry};

        let mut entries = Vec::with_capacity(rows.len());
        for row in rows {
            let account_entries = fetch_account_entries_for_bank_entry(pool, row.id).await?;
            entries.push(BankEntry::from_row(row, account_entries));
        }

        Ok(entries)
    }
}

#[derive(Default)]
pub struct BankImportMutation;

#[Object]
impl BankImportMutation {
    async fn import_bank_statement(
        &self,
        ctx: &Context<'_>,
        input: ImportBankStatementInput,
    ) -> Result<ImportResult> {
        let pool = ctx.data::<PgPool>()?;

        let statement = parse_ofx(&input.file_content)
            .map_err(|e| format!("parse OFX: {e}"))?;

        let import_row = sqlx::query_as::<_, BankImportRow>(
            "INSERT INTO bank_imports (balance_cents) VALUES ($1) RETURNING *",
        )
        .bind(statement.balance_cents)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("create bank import: {e}"))?;

        let mut created_count = 0i32;
        let mut skipped_count = 0i32;
        let mut entries = Vec::new();

        for txn in &statement.transactions {
            let already_exists = if let Some(ref fitid) = txn.fitid {
                sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM bank_entries WHERE external_id = $1)",
                )
                .bind(fitid)
                .fetch_one(pool)
                .await
                .map_err(|e| format!("check duplicate: {e}"))?
            } else {
                false
            };

            if already_exists {
                skipped_count += 1;
                continue;
            }

            let description = txn.name.clone();
            let notes = match (&txn.trntype, &txn.memo) {
                (Some(trntype), Some(memo)) => Some(format!("{trntype}: {memo}")),
                (Some(trntype), None) => Some(trntype.clone()),
                (None, Some(memo)) => Some(memo.clone()),
                (None, None) => None,
            };

            let row = sqlx::query_as::<_, crate::graphql::bank_entries::BankEntryRow>(
                "INSERT INTO bank_entries (date, amount_cents, description, notes, external_id)
                 VALUES ($1, $2, $3, $4, $5) RETURNING *",
            )
            .bind(txn.date)
            .bind(txn.amount_cents)
            .bind(&description)
            .bind(&notes)
            .bind(&txn.fitid)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("insert bank entry: {e}"))?;

            created_count += 1;
            entries.push(ImportedEntry {
                id: row.id,
                external_id: row.external_id,
                date: row.date,
                amount_cents: row.amount_cents,
                description: row.description,
                notes: row.notes,
                was_skipped: false,
            });
        }

        Ok(ImportResult {
            bank_import: BankImport::from_row(import_row),
            created_count,
            skipped_count,
            entries,
        })
    }
}
