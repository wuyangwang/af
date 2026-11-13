use std::fs;
use std::process::Command;

fn af() -> Command {
    Command::new(env!("CARGO_BIN_EXE_af"))
}

#[test]
fn read_write_append_patch_roundtrip_json() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.txt");

    let _out = af().args(["--json", "write", file.to_str().unwrap(), "--force", "--stdin"]).stdin(std::process::Stdio::piped()).spawn();
    // simpler: use .output with input via echo-like by writing temp input file
    let input = dir.path().join("in.txt");
    fs::write(&input, "hello\nworld\n").unwrap();
    let status = af().args(["write", file.to_str().unwrap(), "--from", input.to_str().unwrap(), "--force"]).status().unwrap();
    assert!(status.success());
    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("hello"));

    let meta = af().args(["--json", "read", file.to_str().unwrap(), "--meta"]).output().unwrap();
    assert!(meta.status.success());
    let v: serde_json::Value = serde_json::from_slice(&meta.stdout).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "read");
    assert_eq!(v["schemaVersion"], "1");

    let app = dir.path().join("app.txt");
    fs::write(&app, "!\n").unwrap();
    let st = af().args(["append", file.to_str().unwrap(), "--from", app.to_str().unwrap()]).status().unwrap();
    assert!(st.success());

    let spec = dir.path().join("spec.json");
    fs::write(&spec, r#"{"version":1,"operations":[{"type":"replace","old":"world","new":"AF","expectedMatches":1}]}"#).unwrap();
    let st = af().args(["patch", file.to_str().unwrap(), "--spec", spec.to_str().unwrap()]).status().unwrap();
    assert!(st.success());
    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("AF"));
}

#[test]
fn search_finds_text() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("x.rs");
    fs::write(&f, "fn main() { let useState = 1; }\n").unwrap();
    let out = af().args(["--json", "search", "useState", dir.path().to_str().unwrap()]).output().unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["command"], "search");
    assert!(v["matches"].as_array().unwrap().len() >= 1);
}

#[test]
fn write_rejects_without_force() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.txt");
    fs::write(&file, "old").unwrap();
    let input = dir.path().join("in.txt");
    fs::write(&input, "new").unwrap();
    let out = af().args(["--json", "write", file.to_str().unwrap(), "--from", input.to_str().unwrap()]).output().unwrap();
    assert!(!out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "TARGET_EXISTS");
}

#[test]
fn hash_conflict_detected() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.txt");
    fs::write(&file, "abc").unwrap();
    let input = dir.path().join("in.txt");
    fs::write(&input, "def").unwrap();
    let out = af().args(["--json", "write", file.to_str().unwrap(), "--from", input.to_str().unwrap(), "--force", "--expected-hash", "deadbeef"]).output().unwrap();
    assert!(!out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["error"]["code"], "HASH_CONFLICT");
}
