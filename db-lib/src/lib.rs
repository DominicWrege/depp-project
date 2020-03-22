use deadpool_postgres::{Manager, Pool};
use failure::ResultExt;

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
    #[serde(default = "default_db_name")]
    name: String,
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_max_connection")]
    max_connection: usize,
    #[serde(default = "default_host")]
    db_host: String,
}

fn default_max_connection() -> usize {
    16
}

// TODO maybe use URL
fn default_host() -> String {
    String::from("localhost")
}

fn default_port() -> u16 {
    5432
}
fn default_db_name() -> String {
    String::from("assignments")
}
impl Default for DbConfig {
    fn default() -> Self {
        DbConfig {
            user: "john".into(),
            password: "12345".into(),
            name: default_db_name(),
            port: default_port(),
            max_connection: default_max_connection(),
            db_host: default_host(),
        }
    }
}

pub(crate) fn get_db_config() -> DbConfig {
    match envy::from_env::<DbConfig>() {
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
        .host(&env_conf.db_host)
        .port(env_conf.port);

    let (mut client, pg) = pg_config
        .connect(tokio_postgres::NoTls)
        .await
        .context(format!(
            "DB config error. Tried to connect to host: {}." & env_conf.db_host
        ))?;
    tokio::task::spawn(pg);
    let _ = embedded::migrations::runner()
        .run_async(&mut client)
        .await
        .context("DB migration failed.")?;
    let mngr = Manager::new(pg_config.clone(), tokio_postgres::NoTls);
    Ok(Pool::new(mngr, env_conf.max_connection))
}
