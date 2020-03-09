use crate::handler::AssignmentForm;
use anyhow::Context;
use db_lib::DbError;
use deadpool_postgres::Pool;
use postgres_types::{FromSql, ToSql};
use std::str::FromStr;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use tokio_postgres::Row;

#[derive(serde::Deserialize, serde::Serialize, Debug, PostgresMapper)]
#[pg_mapper(table = "assignment_exercise")]
pub struct AssignmentExercise {
    name: String,
    script_type: ScriptType,
    active: bool,
    solution: String,
    args: Vec<String>,
    exercise_name: String,
    pub(crate) include_files: Vec<u8>,
}

//assignment_name, script_type, active, solution, args, description

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

#[derive(Debug, Clone, PostgresMapper, serde::Serialize)]
#[pg_mapper(table = "course")]
pub struct Exercise {
    id: i32,
    description: String,
}

type Exercises = Result<Vec<Exercise>, DbError>;

pub async fn get_all_exercises(pool: &Pool) -> Exercises {
    let client = pool.get().await?;

    let query = "Select id, description From exercise";
    let rows = client.query(query, &[]).await?;
    rows_into(rows)
}

pub async fn get_exercises_with_assignments(pool: &Pool) -> Exercises {
    let client = pool.get().await?;
    let query = r#"
        SELECT e.id, e.description
        FROM assignment a LEFT JOIN exercise e ON a.exercise_id = e.id
        GROUP BY e.id, e.description
        ORDER BY  e.description
    "#;
    let rows = client.query(query, &[]).await?;
    rows_into(rows)
}

fn rows_into<S>(rows: Vec<Row>) -> Result<Vec<S>, DbError>
where
    S: FromTokioPostgresRow,
{
    let ret = if rows.is_empty() {
        vec![]
    } else {
        rows.into_iter()
            .map(S::from_row)
            .filter_map(Result::ok)
            .collect::<Vec<S>>()
    };
    Ok(ret)
}

pub async fn get_script_types(pool: &Pool) -> Result<Vec<ScriptType>, DbError> {
    let client = pool.get().await?;
    let rows = client
        .query("SELECT unnest(enum_range(NULL::script_type))", &[])
        .await?;

    if rows.is_empty() {
        panic!("enum script_type does not exits");
    } else {
        let ret = rows
            .into_iter()
            .map(|row| row.get(0))
            .collect::<Vec<ScriptType>>();
        Ok(ret)
    }
}

pub async fn get_assignments_for_exercise(
    pool: &Pool,
    e_id: i32,
) -> Result<Vec<AssignmentExercise>, DbError> {
    let client = pool.get().await?;
    let stmt = client
        .prepare(
            r#"
            select assignment_name as name, script_type, active, solution, args, description as exercise_name, include_files
            FROM assignment INNER JOIN exercise ON assignment.exercise_id = exercise.id
            WHERE exercise.id = $1"#
        )
        .await?;
    let rows = client.query(&stmt, &[&e_id]).await?;
    dbg!(&rows.is_empty());
    rows_into(rows)
}

/*pub async fn get_assignments(pool: &Pool)-> Vec<AssignmentCourse>{
    let client = pool.get().await.expect("33");
    let rows = client
        .query("SELECT name, script_type, active, solution, args, course_name from assignment_course", &[])
        .await
        .unwrap();

    rows
        .into_iter()
        .filter_map(|row| AssignmentCourse::from_row(row).ok())
        .collect::<Vec<_>>()
}*/

pub async fn insert_assignment(pool: &Pool, assign: &AssignmentForm<'_>) -> Result<(), DbError> {
    //dbg!(&assign);
    let client = pool.get().await?;
    let stmt = client.prepare(r#"INSERT INTO assignment(assignment_name, script_type, solution, exercise_id, args, include_files)
                                                  Values($1, $2, $3, $4, $5, $6)"#).await?;
    client
        .execute(
            &stmt,
            &[
                &assign.name,
                &assign.script_type,
                &assign.solution,
                &assign.exercise_id,
                &assign.args,
                &assign.include_files,
            ],
        )
        .await?;
    Ok(())
}
