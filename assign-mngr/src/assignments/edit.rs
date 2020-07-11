use crate::assignments::get::Assignment;
use crate::handler::HttpResult;
use crate::State;
use actix_web::{http, web, HttpRequest, HttpResponse};

pub async fn update(
    form: web::Form<Assignment>,
    data: web::Data<State>,
    path: web::Path<uuid::Uuid>,
    req: HttpRequest,
) -> HttpResult {
    let client = data.db_pool.get().await?;
    let uuid = path.into_inner();
    let asign = form.into_inner();

    let stmt = client.prepare(r#"
    UPDATE assignment
    SET assignment_name = $1, solution = $2, script_type = $3, description = $4, active = $5, 
                    exercise_id = $6, args = $7, compare_fs_solution = $8, compare_stdout_solution = $9,
                    custom_script = $10, regex = $11, regex_check_mode = $12, sort_stdout = $13
    WHERE uuid = $14
    "#).await?;
    client
        .execute(
            &stmt,
            &[
                &asign.name,
                &asign.solution,
                &asign.script_type,
                &asign.description,
                &asign.active,
                &asign.exercise_id,
                &asign.args,
                &asign.compare_fs_solution,
                &asign.compare_stdout_solution,
                &asign.custom_script,
                &asign.regex,
                &asign.regex_check_mode,
                &asign.sort_stdout,
                &uuid,
            ],
        )
        .await?;
    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, req.path())
        .finish())
}
