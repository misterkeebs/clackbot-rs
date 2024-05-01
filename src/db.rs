use diesel_async::{
    pooled_connection::{
        deadpool::{self, Object, PoolError},
        AsyncDieselConnectionManager,
    },
    AsyncPgConnection,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not connect to database: {0}")]
    CannotConnect(#[from] PoolError),
}

#[derive(Clone)]
pub struct Pool {
    inner: deadpool::Pool<AsyncPgConnection>,
}

impl Pool {
    pub async fn get(&self) -> Result<Object<AsyncPgConnection>, Error> {
        let conn = self.inner.get().await?;
        Ok(conn)
    }
}

pub async fn connect() -> anyhow::Result<Pool> {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(db_url);
    let inner = deadpool::Pool::builder(config).build()?;

    Ok(Pool { inner })
}
