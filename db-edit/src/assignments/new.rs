use crate::assignments::file::check_type_is_zip;
use crate::assignments::get::Assignment;
use crate::db::ScriptType;
use crate::handler::{redirect, render_template, HttpResult};
use crate::template::TEMPLATES;
use crate::{db, State};
use actix_multipart::{Field, Multipart};
use actix_web::web;
use futures::StreamExt;
use std::collections::HashMap;
use std::str::FromStr;

fn into_assignment_form(h: &mut HashMap<String, String>, zip: &[u8]) -> Assignment {
    let fixed_args = match h.get("args") {
        Some(a) => a.split(",").map(&remove_whitespace).collect(),
        None => vec![],
    };
    Assignment {
        uuid: Default::default(),
        name: h.get("name").unwrap_or(&String::new()).into(),
        solution: fix_newlines(h.get("solution").unwrap_or(&String::new())),
        script_type: ScriptType::from_str(h.get("script_type").unwrap()).unwrap(),
        description: h.get("description").unwrap_or(&String::new()).into(),
        active: true,
        args: fixed_args,
        exercise_id: h
            .get("exercise_id")
            .unwrap_or(&String::default())
            .parse::<i32>()
            .unwrap(),
        include_files: zip.to_vec(),
    }
}

pub async fn insert(data: web::Data<State>, mut payload: Multipart) -> HttpResult {
    let mut text_fields: HashMap<_, _> = HashMap::new();
    let mut zip_file: Vec<u8> = vec![];
    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();

        if field.content_type().subtype() == mime::OCTET_STREAM {
            if let Some((k, v)) = convert_field(&mut field).await {
                text_fields.insert(k, v);
            }
        } else if check_type_is_zip(&field) {
            if let Some(Ok(s)) = field.next().await {
                zip_file = s.to_vec();
            }
        }
    }

    let assign = into_assignment_form(&mut text_fields, &zip_file);
    db::insert_assignment(&data.db_pool, &assign).await?;

    Ok(redirect(format!("exercise/{}", &assign.exercise_id)))
}

fn remove_whitespace(s: &str) -> String {
    s.split_whitespace().collect()
}

fn fix_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

async fn convert_field(field: &mut Field) -> Option<(String, String)> {
    let content_disposition = &field.content_disposition().unwrap();
    if let Some(f_name) = content_disposition.get_name() {
        if let Some(s) = field.next().await {
            if let Ok(v) = String::from_utf8(s.unwrap_or_default().to_vec()) {
                return Some((f_name.to_string(), v));
            } else {
                return None;
            }
        }
    }
    None
}

pub async fn get_form(data: web::Data<State>) -> HttpResult {
    let pool = &data.db_pool;
    let scripts = db::get_script_types(&pool).await?;
    let exercises = db::get_all_exercises(&pool).await?;
    let mut context = tera::Context::new();
    context.insert("exercises", &exercises);
    context.insert("scripts", &scripts);
    render_template(&TEMPLATES, "assignment_form.html", &context)
}
