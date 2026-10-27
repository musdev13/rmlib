use rmlib::{CLIENT_ID, CLIENT_TOKEN, NAME, VERSION, available_modules};

#[test]
fn version_matches_cargo_pkg_version() {
    assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
}

#[test]
fn name_matches_cargo_pkg_name() {
    assert_eq!(NAME, "rmlib");
}

#[test]
fn client_token_is_stable() {
    assert_eq!(CLIENT_TOKEN, "93301754-cd11-4a00-bd50-f69dffaf4b0b");
}

#[test]
fn client_id_is_stable() {
    assert_eq!(CLIENT_ID, "00000000402b5328");
}

#[test]
fn available_modules_contains_core() {
    let mods = available_modules();
    assert!(mods.iter().any(|m| m.name == "core"));
}

#[test]
fn available_modules_reflect_features() {
    let mods = available_modules();

    let has = |name: &str| mods.iter().any(|m| m.name == name);

    assert_eq!(has("core"), cfg!(feature = "core"));
    assert_eq!(has("ely"), cfg!(feature = "ely"));
    assert_eq!(has("fabric"), cfg!(feature = "fabric"));
    assert_eq!(has("neoforge"), cfg!(feature = "neoforge"));
}

#[test]
fn available_modules_has_no_duplicates() {
    let mods = available_modules();
    for (i, a) in mods.iter().enumerate() {
        for b in mods.iter().skip(i + 1) {
            assert_ne!(a.name, b.name, "duplicate module {}", a.name);
        }
    }
}

#[test]
fn module_info_is_debug_eq_copy() {
    let a = rmlib::ModuleInfo { name: "core" };
    let b = a;
    assert_eq!(a, b);
    let _ = format!("{:?}", a);
}
