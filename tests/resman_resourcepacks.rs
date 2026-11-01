#![cfg(feature = "resman-resourcepacks")]

mod common;

use std::fs;
use std::io::Write;
use std::path::Path;

use common::{HomeGuard, TempDir};
use rmlib::extra::resman::resourcepacks::{
    FallbackPolicy, Field, Format, Priority, RpError, info_file_as, info_from_file, parse_fields,
    resourcepacks_dir, scan, scan_dir,
};

fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let f = fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(f);
    let opts: zip::write::SimpleFileOptions = Default::default();
    for (name, data) in entries {
        zip.start_file(*name, opts).unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap();
}

fn rp(path: &Path, name: &str, pack_format: u32) {
    let meta = format!(r#"{{"pack":{{"pack_format":{pack_format},"description":"desc {name}"}}}}"#);
    write_zip(path, &[("pack.mcmeta", meta.as_bytes())]);
}

#[test]
fn scan_finds_packs() {
    let home = TempDir::new("rp_scan");
    let _guard = HomeGuard::new(home.path());
    let game = home.join("game");
    rp(&game.join("resourcepacks").join("a.zip"), "A", 34);
    rp(&game.join("resourcepacks").join("b.zip.disabled"), "B", 34);

    let list = scan(&game).unwrap();
    assert_eq!(list.len(), 2);
    assert!(list.iter().any(|r| r.enabled));
    assert!(list.iter().any(|r| !r.enabled));
}

#[test]
fn info_parsed_and_mc_mapped() {
    let home = TempDir::new("rp_info");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("faithful.zip");
    rp(&p, "Faithful", 34);

    let info = info_from_file(&p).unwrap();
    assert_eq!(info.description.as_deref(), Some("desc Faithful"));
    assert_eq!(info.pack_format, Some(34));
}

#[test]
fn missing_file_not_found() {
    let home = TempDir::new("rp_missing");
    let _guard = HomeGuard::new(home.path());
    assert!(matches!(
        info_from_file(&home.join("nope.zip")),
        Err(RpError::NotFound(_))
    ));
}

#[test]
fn non_pack_zip_errors() {
    let home = TempDir::new("rp_bad");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("bad.zip");
    write_zip(&p, &[("foo.txt", b"x")]);
    assert!(matches!(
        info_from_file(&p),
        Err(RpError::InvalidArgument(_))
    ));
}

#[test]
fn priority_parse_local() {
    let p = Priority::parse("local").unwrap();
    assert!(p.includes(rmlib::extra::resman::resourcepacks::Source::Local));
}

#[test]
fn priority_parse_rejects_unknown() {
    assert!(matches!(
        Priority::parse("bogus"),
        Err(RpError::InvalidArgument(_))
    ));
}

#[test]
fn fallback_skip_and_placeholder() {
    let home = TempDir::new("rp_fallback");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("broken.zip");
    fs::write(&p, b"not a zip").unwrap();

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let priority = Priority::parse("local").unwrap();

    let skipped = rt
        .block_on(rmlib::extra::resman::resourcepacks::info_priority(
            &p,
            &priority,
            &FallbackPolicy::Skip,
        ))
        .unwrap();
    assert!(skipped.is_none());

    let ph = rt
        .block_on(rmlib::extra::resman::resourcepacks::info_priority(
            &p,
            &priority,
            &FallbackPolicy::Placeholder,
        ))
        .unwrap()
        .unwrap();
    assert_eq!(ph.filename, "broken.zip");
}

#[test]
fn fields_and_format() {
    let home = TempDir::new("rp_fmt");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("f.zip");
    rp(&p, "F", 34);

    let fields = parse_fields("description,pack_format").unwrap();
    let out = info_file_as(&p, Some(&fields), Format::Json).unwrap();
    assert!(out.contains("\"pack_format\": 34"));

    let plain = info_file_as(&p, Some(&[Field::Description]), Format::Plain).unwrap();
    assert!(plain.starts_with("description:"));
}

#[test]
fn scan_dir_custom() {
    let home = TempDir::new("rp_custom_dir");
    let _guard = HomeGuard::new(home.path());
    let dir = home.join("custom_rp");
    rp(&dir.join("x.zip"), "X", 34);
    assert_eq!(scan_dir(&dir).unwrap().len(), 1);
}

#[test]
fn game_path_dir_layout() {
    assert_eq!(
        resourcepacks_dir(std::path::Path::new("/game")),
        std::path::PathBuf::from("/game/resourcepacks")
    );
}

#[test]
fn info_supports_min_max_format() {
    let home = TempDir::new("rp_minmax");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("faithful.zip");
    let meta = r#"{"pack":{"description":"The go-to 32x resource pack.","min_format":84,"max_format":97}}"#;
    write_zip(&p, &[("pack.mcmeta", meta.as_bytes())]);

    let info = info_from_file(&p).unwrap();
    assert_eq!(
        info.description.as_deref(),
        Some("The go-to 32x resource pack.")
    );
    assert_eq!(info.pack_format, Some(84));
    let sf = info.supported_formats.as_ref().unwrap();
    assert_eq!(sf["min_inclusive"], 84);
    assert_eq!(sf["max_inclusive"], 97);
}

#[test]
fn local_info_has_no_id() {
    let home = TempDir::new("rp_no_id");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("f.zip");
    rp(&p, "F", 34);
    let info = info_from_file(&p).unwrap();
    assert!(info.id.is_none());
}

#[test]
fn id_is_a_selectable_field() {
    let fields = parse_fields("id,filename,pack_format").unwrap();
    assert!(matches!(fields[0], Field::Id));
}
