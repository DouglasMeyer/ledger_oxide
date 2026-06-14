use sqlx::PgPool;

#[derive(Clone)]
pub struct DbPool {
    pub pool: PgPool,
}

impl DbPool {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
