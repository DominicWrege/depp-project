use deep_project::test_client::TestClient;
use deep_project::{AssignmentMsg, Script};

use std::path::Path;

pub mod deep_project {
    tonic::include_proto!("deep_project");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = TestClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(AssignmentMsg {
            name: "Hello python3".into(),
            solution_path: "examples/pk7-Aufgabe1.sh".into(),
            include_files : vec!["examples/akademisches_jahrbuch.txt".into()],
            script_type: Script::Python3 as i32,
            src_code: "ZWNobyAiSGFsbG9Xb3JsZCIgPj4gaGFsbG8udHh0Cm1rZGlyIGRpcjIyCmVjaG8gInNvbWUgdGhpbmcgZGlmZmVybnQgLi4uIiA+PiAiZGlyMjIvbG9ybWUudHh0Ig==".into(),
            args: vec![]
    });

    let response = client.run_test(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
