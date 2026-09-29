use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use tokn_ingest::{JsonlConfig, read_jsonl};

fn temp_file(name: &str, content: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("tokn-{name}-{stamp}.jsonl"));
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn reads_complete_records() {
    let path = temp_file("complete", "{\"type\":\"a\"}\n{\"type\":\"b\"}\n");
    let mut count = 0;
    let stats = read_jsonl(&path, &JsonlConfig::default(), |_| count += 1).unwrap();
    let _ = fs::remove_file(path);
    assert_eq!(count, 2);
    assert_eq!(stats.records_valid, 2);
}

#[test]
fn tolerates_truncated_tail() {
    let path = temp_file("tail", "{\"type\":\"a\"}\n{\"type\":");
    let stats = read_jsonl(&path, &JsonlConfig::default(), |_| {}).unwrap();
    let _ = fs::remove_file(path);
    assert_eq!(stats.records_valid, 1);
    assert_eq!(stats.truncated_tail, 1);
}

#[test]
fn snapshot_hash_is_stable_for_same_bytes() {
    let content = r#"{"type":"a"}\n{"type":"b"}\n"#;
    let first = temp_file("hash-a", content);
    let second = temp_file("hash-b", content);
    let a = read_jsonl(&first, &JsonlConfig::default(), |_| {}).unwrap();
    let b = read_jsonl(&second, &JsonlConfig::default(), |_| {}).unwrap();
    let _ = fs::remove_file(first);
    let _ = fs::remove_file(second);
    assert!(!a.snapshot_hash.is_empty());
    assert_eq!(a.snapshot_hash, b.snapshot_hash);
}
