//! Profile storage is independent, deterministic and guarded against unobserved replacement.
use ecad_adapters::*;
use ecad_app::CompanyProfile;
use ecad_domain::*;
use std::collections::BTreeMap;
fn profile(id: u128, name: &str) -> CompanyProfile {
    CompanyProfile::new(
        ProfileId::new(id).unwrap(),
        name.into(),
        ProjectMetadata::new(BTreeMap::from([(
            MetadataField::CompanyAddress,
            " First\n\nLast\n".into(),
        )])),
    )
    .unwrap()
}
#[test]
fn missing_profiles_load_without_writes_and_staged_saves_preserve_literal_values_and_observed_versions()
 {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config/profiles.json");
    let file = ProfileFile::new(&path);
    let missing = file.load().unwrap();
    assert!(missing.profiles.is_empty());
    assert!(!path.parent().unwrap().exists());
    let profiles = BTreeMap::from([
        (ProfileId::new(2).unwrap(), profile(2, "Second")),
        (ProfileId::new(1).unwrap(), profile(1, "First")),
    ]);
    let version = file.save(&profiles, None).unwrap();
    let first = std::fs::read(&path).unwrap();
    let loaded = file.load().unwrap();
    assert_eq!(loaded.profiles, profiles);
    assert_eq!(loaded.version, Some(version.clone()));
    file.save(&profiles, Some(&version)).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), first);
    assert!(file.save(&profiles, None).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), first);
    std::fs::write(&path, b"external change").unwrap();
    assert!(file.save(&profiles, Some(&version)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"external change");
    let other = ProfileFile::new(directory.path().join("other.json"));
    assert!(other.save(&profiles, Some(&version)).is_err());
    assert!(!other.path().exists());
}
#[test]
fn malformed_unknown_duplicate_and_future_profile_files_never_load_as_an_empty_catalog() {
    let directory = tempfile::tempdir().unwrap();
    let file = ProfileFile::new(directory.path().join("profiles.json"));
    let profiles = BTreeMap::from([(ProfileId::new(1).unwrap(), profile(1, "First"))]);
    file.save(&profiles, None).unwrap();
    let text = std::fs::read_to_string(file.path()).unwrap();
    let data: serde_json::Value = serde_json::from_str(&text).unwrap();
    let mut duplicate = data.clone();
    duplicate["profiles"]
        .as_array_mut()
        .unwrap()
        .push(data["profiles"][0].clone());
    let mut same_name = duplicate.clone();
    same_name["profiles"][1]["id"] = serde_json::json!(format!("{:032x}", 2));
    for bad in [
        text.replace("\"schema_version\": 1", "\"schema_version\": 2"),
        text.replace("\"company.name\":", "\"project.title\":"),
        text.replace(
            "\"company.name\":",
            "\"company.name\": \"duplicate\", \"company.name\":",
        ),
        duplicate.to_string(),
        same_name.to_string(),
        "broken".into(),
    ] {
        std::fs::write(file.path(), bad).unwrap();
        assert!(file.load().is_err());
    }
    let mismatched = BTreeMap::from([(ProfileId::new(2).unwrap(), profile(1, "First"))]);
    assert!(file.save(&mismatched, None).is_err());
}
