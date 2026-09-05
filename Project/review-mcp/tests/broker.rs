use review_runner::{
    contract::{Contract, Requirement},
    execution::broker::Broker,
    response::Expected,
};
use serde_json::Value;
use std::{io::Read, os::unix::net::UnixStream, path::Path};

fn expected() -> Expected {
    Expected {
        run_id: "run".into(),
        candidate: "a".repeat(40),
        contract_digest: "b".repeat(64),
        role: "critic".into(),
        contract: Contract {
            schema_version: 1,
            requirements: vec![Requirement {
                id: "C-1".into(),
                text: "Check".into(),
                reviewers: vec!["critic".into()],
                allow_na: false,
            }],
        },
    }
}
fn request(socket: &Path) -> Value {
    let mut stream = UnixStream::connect(socket).unwrap();
    let mut reply = String::new();
    stream.read_to_string(&mut reply).unwrap();
    serde_json::from_str(&reply).unwrap()
}
#[test]
fn parent_owned_attempts_exhaust_without_mutable_counter_file() {
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("control.sock");
    let broker = Broker::start(expected(), directory.path().to_owned(), &socket, 2).unwrap();
    assert_eq!(request(&socket)["decision"], "block");
    assert_eq!(request(&socket)["continue"], false);
    assert_eq!(broker.finish().unwrap(), (2, true));
    let names: Vec<_> = std::fs::read_dir(directory.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, ["control.sock"]);
}
#[test]
fn successful_correction_uses_the_same_file_validator() {
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("control.sock");
    let broker = Broker::start(expected(), directory.path().to_owned(), &socket, 3).unwrap();
    assert_eq!(request(&socket)["decision"], "block");
    let answer = serde_json::json!({"run_id":"run","candidate":"a".repeat(40),"contract_digest":"b".repeat(64),
        "role":"critic","verdict":"PASS","checks":[{"contract_id":"C-1","status":"PASS","evidence":"src/a:1","finding":"","minimal_fix":""}],"observations":[]});
    std::fs::write(
        directory.path().join("review.json"),
        serde_json::to_vec(&answer).unwrap(),
    )
    .unwrap();
    assert_eq!(request(&socket), serde_json::json!({}));
    assert_eq!(broker.finish().unwrap(), (2, false));
}
