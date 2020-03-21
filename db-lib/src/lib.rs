use deadpool_postgres::{Manager, Pool};
use failure::ResultExt;
use std::net::Ipv4Addr;

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
    #[error(display = "Sql error {:?}", _0)]
    Sql(tokio_postgres::error::Error),
    #[error(display = "DB returned empty rows")]
    EmptyRows,
}

// user for testing
//pub const DB_URL: &'static str = "postgres://john:12345@127.0.0.1:5432/assignments";

#[derive(serde::Deserialize, Debug)]
pub(crate) struct DbConfig {
    user: String,
    password: String,
    name: String,
    port: u16,
    #[serde(default = "default_max_connection")]
    max_connection: usize,
    host: Ipv4Addr,
}

fn default_max_connection() -> usize {
    16
}
impl Default for DbConfig {
    fn default() -> Self {
        DbConfig {
            user: "john".into(),
            password: "12345".into(),
            name: "assignments".into(),
            port: 5432,
            max_connection: 16,
            host: Ipv4Addr::new(127, 0, 0, 1),
        }
    }
}

pub(crate) fn get_db_config() -> DbConfig {
    match envy::prefixed("DEPP_DB_").from_env::<DbConfig>() {
        Ok(config) => config,
        Err(_) => DbConfig::default(),
    }
}

pub async fn connect_migrate() -> Result<Pool, failure::Error> {
    let env_conf = get_db_config();
    let mut pg_config = tokio_postgres::Config::default();
    pg_config
        .user(&env_conf.user)
        .password(&env_conf.password)
        .dbname(&env_conf.name)
        .host(&env_conf.host.to_string())
        .port(env_conf.port);

    let (mut client, pg) = pg_config
        .connect(tokio_postgres::NoTls)
        .await
        .context("DB config error")?;
    tokio::task::spawn(pg);
    let _ = embedded::migrations::runner()
        .run_async(&mut client)
        .await
        .context("DB migration failed")?;
    let mngr = Manager::new(pg_config.clone(), tokio_postgres::NoTls);
    Ok(Pool::new(mngr, env_conf.max_connection))
}
