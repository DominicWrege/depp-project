use crate::assignments::get::Assignment;
use crate::error::HttpError;
use db_lib::DbError;
use deadpool_postgres::Pool;
use postgres_types::{FromSql, ToSql};
use std::str::FromStr;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use tokio_postgres::Row;

#[derive(Debug, Clone, ToSql, FromSql, serde::Deserialize, serde::Serialize)]
#[postgres(name = "script_type")]
pub enum ScriptType {
    PowerShell,
    Batch,
    Python3,
    Shell,
    Bash,
    Awk,
    Sed,
}

impl FromStr for ScriptType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PowerShell" => Ok(ScriptType::PowerShell),
            "Batch" => Ok(ScriptType::Batch),
            "Python3" => Ok(ScriptType::Python3),
            "Shell" => Ok(ScriptType::Shell),
            "Bash" => Ok(ScriptType::Bash),
            "Awk" => Ok(ScriptType::Awk),
            "Sed" => Ok(ScriptType::Sed),
            _ => Err(()),
        }
    }
}

//type Exercises = Result<Vec<Exercise>, DbError>;
#[derive(Debug, Clone, PostgresMapper, serde::Serialize)]
#[pg_mapper(table = "exercise")]
pub struct Exercise {
    id: i32,
    description: String,
}

pub async fn get_all_exercises(pool: &Pool) -> Result<Vec<Exercise>, HttpError> {
    let client = pool.get().await?;
    let query = "SELECT id, description FROM exercise ORDER BY description;";
    let rows = client.query(query, &[]).await?;
    Ok(rows_into(rows))
}

pub fn rows_into<S>(rows: Vec<Row>) -> Vec<S>
where
    S: FromTokioPostgresRow,
{
    rows.into_iter()
        .map(S::from_row)
        .filter_map(Result::ok)
        .collect::<Vec<S>>()
}

//"SELECT unnest(enum_range(NULL::script_type))::text as name ORDER BY name;"
pub async fn get_script_types(pool: &Pool) -> Result<Vec<ScriptType>, DbError> {
    let client = pool.get().await?;
    let rows = client
        .query("SELECT unnest(enum_range(NULL::script_type))", &[])
        .await?;

    let ret = if rows.is_empty() {
        vec![]
    } else {
        rows.into_iter()
            .map(|row| row.get(0))
            .collect::<Vec<ScriptType>>()
    };
    Ok(ret)
}

pub async fn insert_assignment(pool: &Pool, assign: &Assignment) -> Result<(), DbError> {
    let client = pool.get().await?;
    let stmt = client.prepare(r#"INSERT INTO assignment(assignment_name, script_type, solution, exercise_id, args, description, 
                                                                    include_files, compare_fs_solution, compare_stdout_solution, custom_script, regex, regex_check_mode)
                                                  Values($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#).await?;
    client
        .execute(
            &stmt,
            &[
                &assign.name,
                &assign.script_type,
                &assign.solution,
                &assign.exercise_id,
                &assign.args,
                &assign.description,
                &assign.include_files,
                &assign.compare_fs_solution,
                &assign.compare_stdout_solution,
                &assign.custom_script,
                &assign.regex,
                &assign.regex_check_mode,
            ],
        )
        .await?;
    Ok(())
}
