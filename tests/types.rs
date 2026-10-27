#![cfg(feature = "core")]

use rmlib::core::types::{VersionType, build_maven_path};

#[test]
fn version_type_as_str_all_variants() {
    assert_eq!(VersionType::Release.as_str(), "release");
    assert_eq!(VersionType::Snapshot.as_str(), "snapshot");
    assert_eq!(VersionType::OldBeta.as_str(), "old_beta");
    assert_eq!(VersionType::OldAlpha.as_str(), "old_alpha");
}

#[test]
fn version_type_is_copy_eq() {
    let a = VersionType::Release;
    let b = a;
    assert_eq!(a, b);
    assert_ne!(VersionType::Release, VersionType::Snapshot);
}

#[test]
fn maven_basic() {
    assert_eq!(
        build_maven_path("net.minecraft:client:1.20.1", None).unwrap(),
        "net/minecraft/client/1.20.1/client-1.20.1.jar"
    );
}

#[test]
fn maven_with_classifier_argument() {
    assert_eq!(
        build_maven_path("net.minecraft:client:1.20.1", Some("natives-linux")).unwrap(),
        "net/minecraft/client/1.20.1/client-1.20.1-natives-linux.jar"
    );
}

#[test]
fn maven_with_classifier_in_name() {
    assert_eq!(
        build_maven_path("net.minecraft:client:1.20.1:natives-linux", None).unwrap(),
        "net/minecraft/client/1.20.1/client-1.20.1-natives-linux.jar"
    );
}

#[test]
fn maven_classifier_argument_wins_over_name_segment() {
    assert_eq!(
        build_maven_path(
            "net.minecraft:client:1.20.1:natives-windows",
            Some("natives-linux")
        )
        .unwrap(),
        "net/minecraft/client/1.20.1/client-1.20.1-natives-linux.jar"
    );
}

#[test]
fn maven_strips_jar_suffix() {
    assert_eq!(
        build_maven_path("net.minecraft:client:1.20.1@jar", None).unwrap(),
        "net/minecraft/client/1.20.1/client-1.20.1.jar"
    );
}

#[test]
fn maven_strips_jar_suffix_with_classifier() {
    assert_eq!(
        build_maven_path("net.minecraft:client:1.20.1:natives-linux@jar", None).unwrap(),
        "net/minecraft/client/1.20.1/client-1.20.1-natives-linux.jar"
    );
}

#[test]
fn maven_dots_become_slashes() {
    assert_eq!(
        build_maven_path("com.example.deep.group:artifact:1.0", None).unwrap(),
        "com/example/deep/group/artifact/1.0/artifact-1.0.jar"
    );
}

#[test]
fn maven_too_few_parts_returns_none() {
    assert!(build_maven_path("net.minecraft:client", None).is_none());
    assert!(build_maven_path("onlyone", None).is_none());
    assert!(build_maven_path("", None).is_none());
}

#[test]
fn maven_empty_classifier_argument_is_used() {
    assert_eq!(
        build_maven_path("net.minecraft:client:1.20.1", Some("")).unwrap(),
        "net/minecraft/client/1.20.1/client-1.20.1-.jar"
    );
}
