fn main() {
    tonic_build::configure()
        .type_attribute("deep_project.Script", "#[derive(postgres_types::FromSql)]")
        .type_attribute(
            "deep_project.Script",
            r#"#[postgres(name = "script_type")]"#,
        )
        .type_attribute(
            "deep_project.RegexMode",
            r#"#[derive(postgres_types::FromSql, postgres_types::ToSql)]"#,
        )
        .type_attribute(
            "deep_project.RegexMode",
            r#"#[postgres(name = "regex_mode")]"#,
        )
        .type_attribute(
            "deep_project.SortStdoutBy",
            r#"#[derive(postgres_types::FromSql, postgres_types::ToSql)]"#,
        )
        .type_attribute(
            "deep_project.SortStdoutBy",
            r#"#[postgres(name = "sort_stdout_by")]"#,
        )
        .type_attribute(
            "deep_project.OptionalString",
            r#"#[derive(postgres_types::FromSql, postgres_types::ToSql)]"#,
        )
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile(&["proto/deep_project.proto"], &["proto"])
        .unwrap()
    //.type_attribute("deep_project.Assignment", "#[serde(rename_all = \"camelCase\")]")
}
