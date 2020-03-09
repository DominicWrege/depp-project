use crate::api::AssignmentShort;
use db_lib::DbError;
use deadpool_postgres::Pool;
use grpc_api::Assignment;
use std::convert::From;
use tokio_postgres::row::Row;
use uuid::Uuid;

pub async fn get_assignments(pool: &Pool) -> Result<Vec<AssignmentShort>, DbError> {
    let client = pool.get().await?;
    let query = "select uuid, assignment_name from assignment";
    let rows = client.query(query, &[]).await?;

    Ok(rows
        .into_iter()
        .map(|r| AssignmentShort::from(r))
        .collect::<Vec<_>>())
}

impl From<Row> for AssignmentShort {
    fn from(r: Row) -> Self {
        Self {
            id: r.get("uuid"),
            name: r.get("assignment_name"),
        }
    }
}

pub async fn get_assignment(pool: &Pool, uuid: &Uuid) -> Result<Assignment, DbError> {
    let client = pool.get().await?;
    let stmt = client
        .prepare(
            r#"select assignment_name, script_type, include_files, solution, args 
                                                                    from assignment 
                                                                    where assignment.uuid = $1"#,
        )
        .await?;
    let rows = client.query(&stmt, &[uuid]).await?;
    if let Some(row) = rows.get(0) {
        Ok(Assignment::from(row))
    } else {
        Err(DbError::EmptyRows)
    }
}
