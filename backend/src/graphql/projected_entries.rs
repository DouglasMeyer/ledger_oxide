use async_graphql::{Context, InputObject, Object, Result, SimpleObject};
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use rrule::{RRuleSet, Tz};
use sqlx::PgPool;

#[derive(Debug, sqlx::FromRow)]
pub struct ProjectedEntryRow {
    pub id: i32,
    pub account_id: i32,
    pub description: Option<String>,
    pub amount_cents: i32,
    pub date: NaiveDate,
    pub rrule: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(SimpleObject, Debug)]
pub struct ProjectedEntry {
    pub id: i32,
    pub account_id: i32,
    pub description: Option<String>,
    pub amount_cents: i32,
    pub date: NaiveDate,
    pub rrule: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProjectedEntry {
    fn from_row(row: ProjectedEntryRow) -> Self {
        Self {
            id: row.id,
            account_id: row.account_id,
            description: row.description,
            amount_cents: row.amount_cents,
            date: row.date,
            rrule: row.rrule,
            active: row.active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(SimpleObject, Debug)]
pub struct ForecastEntry {
    pub date: NaiveDate,
    pub description: Option<String>,
    pub amount_cents: i32,
    pub source: ForecastSource,
    pub account_id: Option<i32>,
    pub bank_entry_id: Option<i32>,
    pub projected_entry_id: Option<i32>,
}

#[derive(SimpleObject, Debug)]
pub struct ForecastSource {
    pub kind: String,
}

#[derive(InputObject)]
pub struct CreateProjectedEntryInput {
    pub account_id: i32,
    pub description: Option<String>,
    pub amount_cents: i32,
    pub date: NaiveDate,
    pub rrule: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateProjectedEntryInput {
    pub account_id: Option<i32>,
    pub description: Option<String>,
    pub amount_cents: Option<i32>,
    pub date: Option<NaiveDate>,
    pub rrule: Option<String>,
    pub active: Option<bool>,
}

fn expand_rrule(date: NaiveDate, rrule_str: &str, from: NaiveDate, to: NaiveDate) -> Vec<NaiveDate> {
    let dtstart = Tz::UTC
        .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
        .single()
        .unwrap();

    let rrule_set_str = format!(
        "DTSTART:{}\nRRULE:{}",
        dtstart.format("%Y%m%dT%H%M%SZ"),
        rrule_str
    );

    let Ok(rrule_set) = rrule_set_str.parse::<RRuleSet>() else {
        return Vec::new();
    };

    let after = Tz::UTC
        .with_ymd_and_hms(from.year(), from.month(), from.day(), 0, 0, 0)
        .single()
        .unwrap();

    let before = Tz::UTC
        .with_ymd_and_hms(to.year(), to.month(), to.day(), 23, 59, 59)
        .single()
        .unwrap();

    let result = rrule_set.after(after).before(before).all(1000);

    result
        .dates
        .into_iter()
        .map(|dt| dt.date_naive())
        .collect()
}

#[derive(Default)]
pub struct ProjectedEntryQuery;

#[Object]
impl ProjectedEntryQuery {
    async fn projected_entries(
        &self,
        ctx: &Context<'_>,
        account_id: Option<i32>,
        active: Option<bool>,
    ) -> Result<Vec<ProjectedEntry>> {
        let pool = ctx.data::<PgPool>()?;

        let mut sql = String::from("SELECT * FROM projected_entries WHERE 1=1");
        let mut bind_idx = 1u32;

        if account_id.is_some() {
            sql.push_str(&format!(" AND account_id = ${}", bind_idx));
            bind_idx += 1;
        }
        if active.is_some() {
            sql.push_str(&format!(" AND active = ${}", bind_idx));
            bind_idx += 1;
        }

        let _ = bind_idx;

        sql.push_str(" ORDER BY date ASC, id ASC");

        let mut query = sqlx::query_as::<_, ProjectedEntryRow>(&sql);

        if let Some(aid) = account_id {
            query = query.bind(aid);
        }
        if let Some(a) = active {
            query = query.bind(a);
        }

        let rows = query
            .fetch_all(pool)
            .await
            .map_err(|e| format!("fetch projected entries: {e}"))?;

        Ok(rows.into_iter().map(ProjectedEntry::from_row).collect())
    }

    async fn projected_entry(&self, ctx: &Context<'_>, id: i32) -> Result<Option<ProjectedEntry>> {
        let pool = ctx.data::<PgPool>()?;

        let row = sqlx::query_as::<_, ProjectedEntryRow>(
            "SELECT * FROM projected_entries WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("fetch projected entry: {e}"))?;

        Ok(row.map(ProjectedEntry::from_row))
    }

    async fn forecast(
        &self,
        ctx: &Context<'_>,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<ForecastEntry>> {
        let pool = ctx.data::<PgPool>()?;

        let bank_rows = sqlx::query_as::<_, (i32, NaiveDate, Option<String>, i32)>(
            "SELECT id, date, description, amount_cents FROM bank_entries
             WHERE date >= $1 AND date <= $2
             ORDER BY date ASC, id ASC",
        )
        .bind(from)
        .bind(to)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("fetch bank entries for forecast: {e}"))?;

        let proj_rows = sqlx::query_as::<_, ProjectedEntryRow>(
            "SELECT * FROM projected_entries WHERE active = true ORDER BY date ASC, id ASC",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| format!("fetch projected entries for forecast: {e}"))?;

        let mut entries: Vec<ForecastEntry> = bank_rows
            .into_iter()
            .map(|(id, date, desc, amount)| ForecastEntry {
                date,
                description: desc,
                amount_cents: amount,
                source: ForecastSource {
                    kind: "BANK_ENTRY".to_string(),
                },
                account_id: None,
                bank_entry_id: Some(id),
                projected_entry_id: None,
            })
            .collect();

        for proj in proj_rows {
            let dates: Vec<NaiveDate> = if let Some(ref rrule) = proj.rrule {
                expand_rrule(proj.date, rrule, from, to)
            } else if proj.date >= from && proj.date <= to {
                vec![proj.date]
            } else {
                continue;
            };

            for d in dates {
                entries.push(ForecastEntry {
                    date: d,
                    description: proj.description.clone(),
                    amount_cents: proj.amount_cents,
                    source: ForecastSource {
                        kind: "PROJECTED".to_string(),
                    },
                    account_id: Some(proj.account_id),
                    bank_entry_id: None,
                    projected_entry_id: Some(proj.id),
                });
            }
        }

        entries.sort_by(|a, b| a.date.cmp(&b.date));

        Ok(entries)
    }
}

#[derive(Default)]
pub struct ProjectedEntryMutation;

#[Object]
impl ProjectedEntryMutation {
    async fn create_projected_entry(
        &self,
        ctx: &Context<'_>,
        input: CreateProjectedEntryInput,
    ) -> Result<ProjectedEntry> {
        let pool = ctx.data::<PgPool>()?;

        let row = sqlx::query_as::<_, ProjectedEntryRow>(
            "INSERT INTO projected_entries (account_id, description, amount_cents, date, rrule)
             VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(input.account_id)
        .bind(&input.description)
        .bind(input.amount_cents)
        .bind(input.date)
        .bind(&input.rrule)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("create projected entry: {e}"))?;

        Ok(ProjectedEntry::from_row(row))
    }

    async fn update_projected_entry(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateProjectedEntryInput,
    ) -> Result<ProjectedEntry> {
        let pool = ctx.data::<PgPool>()?;

        let current = sqlx::query_as::<_, ProjectedEntryRow>(
            "SELECT * FROM projected_entries WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("fetch projected entry: {e}"))?
        .ok_or_else(|| format!("projected entry {id} not found"))?;

        let account_id = input.account_id.unwrap_or(current.account_id);
        let description = input.description.or(current.description);
        let amount_cents = input.amount_cents.unwrap_or(current.amount_cents);
        let date = input.date.unwrap_or(current.date);
        let rrule = input.rrule.or(current.rrule);
        let active = input.active.unwrap_or(current.active);

        let row = sqlx::query_as::<_, ProjectedEntryRow>(
            "UPDATE projected_entries
             SET account_id = $1, description = $2, amount_cents = $3, date = $4,
                 rrule = $5, active = $6, updated_at = NOW()
             WHERE id = $7 RETURNING *",
        )
        .bind(account_id)
        .bind(&description)
        .bind(amount_cents)
        .bind(date)
        .bind(&rrule)
        .bind(active)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("update projected entry: {e}"))?;

        Ok(ProjectedEntry::from_row(row))
    }

    async fn delete_projected_entry(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> Result<ProjectedEntry> {
        let pool = ctx.data::<PgPool>()?;

        let row = sqlx::query_as::<_, ProjectedEntryRow>(
            "DELETE FROM projected_entries WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("delete projected entry: {e}"))?
        .ok_or_else(|| format!("projected entry {id} not found"))?;

        Ok(ProjectedEntry::from_row(row))
    }
}
