use sqlx::{Pool, Postgres};

pub type Db = Pool<Postgres>;

pub async fn init_pool() -> anyhow::Result<Db> {
    let pool = Pool::connect(&std::env::var("DATABASE_URL")?).await?;
    Ok(pool)
}
