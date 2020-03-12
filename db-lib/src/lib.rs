use anyhow::Context;
use deadpool_postgres::{Manager, Pool};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./db-lib/migrations");
}

#[derive(Debug, err_derive::Error, derive_more::From)]
pub enum DbError {
    #[error(display = "{}", _0)]
    Pool(deadpool_postgres::PoolError),
    #[error(display = "could not map db type {:?}", _0)]
    TypeMap(tokio_pg_mapper::Error),
    #[error(display = "Sql error{}", _0)]
    Sql(tokio_postgres::error::Error),
    #[error(display = "DB returned empty rows")]
    EmptyRows,
}

pub const DB_URL: &'static str = "postgres://john:12345@127.0.0.1:5432/assignments";

pub async fn connect_migrate(db_url: &str) -> Result<Pool, anyhow::Error> {
    let postgres_url = String::from(db_url);
    let postgres_config: tokio_postgres::config::Config = postgres_url.parse().unwrap();
    let (mut client, pg) = postgres_config
        .connect(tokio_postgres::NoTls)
        .await
        .context("DB config error")?;
    tokio::task::spawn(pg);
    let _ = embedded::migrations::runner()
        .run_async(&mut client)
        .await
        .context("DB migration failed")?;
    let mngr = Manager::new(postgres_config.clone(), tokio_postgres::NoTls);

    Ok(Pool::new(mngr, 16))
}
