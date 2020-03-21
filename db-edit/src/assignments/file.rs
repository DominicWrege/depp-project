use crate::handler::HttpResult;
use crate::State;
use actix_multipart::{Field, Multipart};
use actix_web::{http, web, HttpResponse};
use futures::StreamExt;
use std::path::PathBuf;

pub async fn download(path: web::Path<uuid::Uuid>, data: web::Data<State>) -> HttpResult {
    let client = data.db_pool.get().await?;
    let uuid = path.into_inner();
    let stmt = client
        .prepare("SELECT include_files FROM assignment WHERE uuid = $1")
        .await?;
    let row = client.query_one(&stmt, &[&uuid]).await?;

    let content: Vec<u8> = row.get("include_files");
    Ok(HttpResponse::Ok()
        .content_type("application/zip")
        .body(content))
}
pub fn ls_zip_content(buf: &[u8]) -> Result<Vec<PathBuf>, std::io::Error> {
    let reader = std::io::Cursor::new(buf);
    let mut ret = vec![];
    if let Ok(mut zip) = zip::ZipArchive::new(reader) {
        for i in 0..zip.len() {
            let file = zip.by_index(i)?;
            ret.push(PathBuf::from(file.name()));
        }
    }
    Ok(ret)
}

pub fn check_type_is_zip(field: &Field) -> bool {
    field.content_type().subtype() == "zip"
}

pub async fn update_files(
    data: web::Data<State>,
    mut payload: Multipart,
    path: web::Path<uuid::Uuid>,
) -> HttpResult {
    let uuid = path.into_inner();
    let mut zip_file: Vec<u8> = vec![];

    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();
        if check_type_is_zip(&field) {
            if let Some(Ok(s)) = field.next().await {
                zip_file = s.to_vec();
            }
        }
    }

    let client = data.db_pool.get().await?;
    let stmt = client
        .prepare("UPDATE assignment SET include_files = $1 WHERE uuid = $2")
        .await?;
    client.execute(&stmt, &[&zip_file, &uuid]).await?;
    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, format!("/assignment/{}", &uuid))
        .finish())
}
