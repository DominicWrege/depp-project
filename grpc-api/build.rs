fn main() {
    tonic_build::configure()
        .type_attribute("deep_project.Script", "#[derive(postgres_types::FromSql)]")
        .type_attribute(
            "deep_project.Script",
            r#"#[postgres(name = "script_type")]"#,
        )
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile(&["proto/deep_project.proto"], &["proto"])
        .unwrap()
    //.type_attribute("deep_project.Assignment", "#[serde(rename_all = \"camelCase\")]")
}
