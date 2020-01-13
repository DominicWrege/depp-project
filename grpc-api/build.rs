fn main() {
    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(
            "deep_project.AssignmentShort",
            "#[serde(rename_all = \"camelCase\")]",
        )
        .compile(&["proto/deep_project.proto"], &["proto"])
        .unwrap()

    //.type_attribute("deep_project.Assignment", "#[serde(rename_all = \"camelCase\")]")
}
