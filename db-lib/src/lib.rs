use deadpool_postgres::{Manager, Pool};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}
// TODO refactor conn err
#[derive(Debug, err_derive::Error, derive_more::From)]
pub enum DbError {
    #[error(display = "{}", _0)]
    Pool(deadpool_postgres::PoolError),
    #[error(display = "could not map db type {:?}", _0)]
    TypeMap(tokio_pg_mapper::Error),
    #[error(display = "Sql error {:?}", _0)]
    Sql(tokio_postgres::error::Error),
    #[error(display = "Config error: {}", _0)]
    Config(ConfigError),
}
#[derive(Debug, err_derive::Error)]
pub enum ConfigError {
    #[error(display = "Connection failed using config {:#?}. Error: {}", _0, _1)]
    Connection(DbConfig, tokio_postgres::error::Error),
    #[error(display = "Migration failed: {}", _0)]
    Migration(refinery::Error),
}

// user for testing
//pub const DB_URL: &'static str = "postgres://john:12345@127.0.0.1:5432/assignments";

// prefix POSTGRES_
#[derive(serde::Deserialize, Debug, Clone)]
pub struct DbConfig {
    user: String,
    password: String,
    #[serde(default = "default_db_name")]
    db: String,
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_max_connection")]
    max_connection: usize,
    #[serde(default = "default_host")]
    host: String,
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
            user: "johncena".into(),
            password: "supersecret".into(),
            db: default_db_name(),
            port: default_port(),
            max_connection: default_max_connection(),
            host: default_host(),
        }
    }
}

pub(crate) fn get_db_config() -> DbConfig {
    match envy::prefixed("POSTGRES_").from_env::<DbConfig>() {
        Ok(config) => config,
        Err(_) => {
            log::info!("Using default config for the db connection.");
            DbConfig::default()
        }
    }
}

pub async fn connect_migrate() -> Result<Pool, DbError> {
    let env_conf = get_db_config();
    let mut pg_config = tokio_postgres::Config::default();
    pg_config
        .user(&env_conf.user)
        .password(&env_conf.password)
        .dbname(&env_conf.db)
        .host(&env_conf.host)
        .port(env_conf.port);

    let (mut client, pg) = pg_config
        .connect(tokio_postgres::NoTls)
        .await
        .map_err(|e| ConfigError::Connection(env_conf.clone(), e))?;
    tokio::task::spawn(pg);
    embedded::migrations::runner()
        .run_async(&mut client)
        .await
        .map_err(|e| ConfigError::Migration(e))?;
    let mngr = Manager::new(pg_config.clone(), tokio_postgres::NoTls);
    Ok(Pool::new(mngr, env_conf.max_connection))
}
