#![cfg(feature = "core")]

mod common;

use std::panic::{AssertUnwindSafe, catch_unwind};

use common::TempDir;
use rmlib::core::types::VersionType;
use rmlib::core::version::install::resolve_version_data;
use rmlib::core::version::{sort_by_type, strip_version};
use serde_json::{Value, json};

fn sample_versions() -> Vec<Value> {
    vec![
        json!({"id": "1.20.1", "type": "release",  "url": "u1", "releaseTime": "r1", "time": "t1"}),
        json!({"id": "1.20.2", "type": "release",  "url": "u2", "releaseTime": "r2", "time": "t2"}),
        json!({"id": "23w45a", "type": "snapshot", "url": "u3", "releaseTime": "r3", "time": "t3"}),
        json!({"id": "b1.7.3", "type": "old_beta", "url": "u4", "releaseTime": "r4", "time": "t4"}),
        json!({"id": "a1.2.6", "type": "old_alpha","url": "u5", "releaseTime": "r5", "time": "t5"}),
    ]
}

#[test]
fn sort_by_type_release() {
    let v = sample_versions();
    let out = sort_by_type(&v, VersionType::Release);
    assert_eq!(out.len(), 2);
    assert_eq!(out[0]["id"], json!("1.20.1"));
    assert_eq!(out[1]["id"], json!("1.20.2"));
}

#[test]
fn sort_by_type_snapshot() {
    let v = sample_versions();
    let out = sort_by_type(&v, VersionType::Snapshot);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0]["id"], json!("23w45a"));
}

#[test]
fn sort_by_type_old_beta() {
    let v = sample_versions();
    let out = sort_by_type(&v, VersionType::OldBeta);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0]["id"], json!("b1.7.3"));
}

#[test]
fn sort_by_type_old_alpha() {
    let v = sample_versions();
    let out = sort_by_type(&v, VersionType::OldAlpha);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0]["id"], json!("a1.2.6"));
}

#[test]
fn sort_by_type_empty_input() {
    let out = sort_by_type(&[], VersionType::Release);
    assert!(out.is_empty());
}

#[test]
fn sort_by_type_missing_type_field() {
    let v = vec![json!({"id": "x"}), json!({"id": "y", "type": "release"})];
    let out = sort_by_type(&v, VersionType::Release);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0]["id"], json!("y"));
}

#[test]
fn sort_by_type_unknown_type_not_matched() {
    let v = vec![json!({"id": "x", "type": "weird"})];
    for t in [
        VersionType::Release,
        VersionType::Snapshot,
        VersionType::OldBeta,
        VersionType::OldAlpha,
    ] {
        assert!(sort_by_type(&v, t).is_empty());
    }
}

#[test]
fn strip_version_keep_all() {
    let v = sample_versions();
    let out = strip_version(&v, true, true, true, true, true);
    assert_eq!(out.len(), v.len());
    assert_eq!(out[0]["id"], json!("1.20.1"));
    assert_eq!(out[0]["type"], json!("release"));
    assert_eq!(out[0]["url"], json!("u1"));
    assert_eq!(out[0]["releaseTime"], json!("r1"));
    assert_eq!(out[0]["time"], json!("t1"));
}

#[test]
fn strip_version_keep_nothing() {
    let v = sample_versions();
    let out = strip_version(&v, false, false, false, false, false);
    assert_eq!(out.len(), v.len());
    for item in &out {
        assert_eq!(item, &json!({}));
    }
}

#[test]
fn strip_version_keep_only_id() {
    let v = sample_versions();
    let out = strip_version(&v, true, false, false, false, false);
    assert_eq!(out[0], json!({"id": "1.20.1"}));
}

#[test]
fn strip_version_keep_only_url() {
    let v = sample_versions();
    let out = strip_version(&v, false, false, false, false, true);
    assert_eq!(out[0], json!({"url": "u1"}));
}

#[test]
fn strip_version_keep_only_type() {
    let v = sample_versions();
    let out = strip_version(&v, false, false, false, true, false);
    assert_eq!(out[0], json!({"type": "release"}));
}

#[test]
fn strip_version_keep_only_release_time() {
    let v = sample_versions();
    let out = strip_version(&v, false, true, false, false, false);
    assert_eq!(out[0], json!({"releaseTime": "r1"}));
}

#[test]
fn strip_version_keep_only_time() {
    let v = sample_versions();
    let out = strip_version(&v, false, false, true, false, false);
    assert_eq!(out[0], json!({"time": "t1"}));
}

#[test]
fn strip_version_non_object_input_becomes_empty_object() {
    let v = vec![json!("scalar"), json!(42), json!(null)];
    let out = strip_version(&v, true, true, true, true, true);
    assert_eq!(out.len(), 3);
    for item in &out {
        assert_eq!(item, &json!({}));
    }
}

#[test]
fn strip_version_empty_input() {
    let out = strip_version(&[], true, true, true, true, true);
    assert!(out.is_empty());
}

#[test]
fn strip_version_missing_field_is_skipped() {
    let v = vec![json!({"id": "x"})];
    let out = strip_version(&v, true, true, true, true, true);
    assert_eq!(out[0], json!({"id": "x"}));
}

#[tokio::test]
async fn resolve_version_data_from_json_path() {
    let dir = TempDir::new("resolve_data");
    let path = dir.write(
        "1.20.1.json",
        br#"{"id":"1.20.1","mainClass":"net.minecraft.client.main.Main"}"#,
    );

    let (id, json_str) = resolve_version_data(None, Some(path)).await;
    assert_eq!(id, "1.20.1");
    assert!(json_str.contains("\"mainClass\""));
}

#[test]
fn resolve_version_data_from_json_path_invalid_json_panics() {
    let dir = TempDir::new("resolve_bad");
    let path = dir.write("bad.json", b"not json");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(resolve_version_data(None, Some(path)))
    }));
    assert!(result.is_err());
}

#[test]
fn resolve_version_data_from_json_path_missing_id_panics() {
    let dir = TempDir::new("resolve_noid");
    let path = dir.write("x.json", br#"{"mainClass":"Main"}"#);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(resolve_version_data(None, Some(path)))
    }));
    assert!(result.is_err());
}

#[test]
fn resolve_version_data_from_json_path_missing_file_panics() {
    let dir = TempDir::new("resolve_missing");
    let path = dir.join("does_not_exist.json");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(resolve_version_data(None, Some(path)))
    }));
    assert!(result.is_err());
}
