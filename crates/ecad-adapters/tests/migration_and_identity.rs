//! Explicit legacy conversion and persistent lineage distinct from occurrence IDs.

use ecad_adapters::{ProjectFileError, RandomIds, decode_project, encode_project, migrate_v1};
use ecad_domain::ProjectId;

const LEGACY: &str = include_str!("fixtures/minimal-v1.json");
const CURRENT: &str = include_str!("fixtures/minimal-v6.json");

#[test]
fn v1_requires_explicit_migration_and_does_not_invent_a_project_identity() {
    assert!(matches!(
        decode_project(LEGACY),
        Err(ProjectFileError::MigrationRequired(1))
    ));
    let migrated = migrate_v1(LEGACY, ProjectId::new(1).unwrap()).unwrap();
    assert_eq!(migrated, decode_project(CURRENT).unwrap());
    assert_eq!(encode_project(&migrated).unwrap(), CURRENT);
    let different_lineage = migrate_v1(LEGACY, ProjectId::new(2).unwrap()).unwrap();
    assert_ne!(migrated.id(), different_lineage.id());
    assert_eq!(
        migrated.devices().collect::<Vec<_>>(),
        different_lineage.devices().collect::<Vec<_>>()
    );
    assert_eq!(
        migrated.conductors().collect::<Vec<_>>(),
        different_lineage.conductors().collect::<Vec<_>>()
    );
}

#[test]
fn migration_rejects_unknown_data_and_dangling_references() {
    let unknown = LEGACY.replace(
        "\"schema_version\": 1,",
        "\"schema_version\": 1, \"unsupported\": 1,",
    );
    assert!(matches!(
        migrate_v1(&unknown, ProjectId::new(1).unwrap()),
        Err(ProjectFileError::Json(_))
    ));
    let missing = LEGACY.replace(
        "\"b\": \"000000000000000000000000000000c9\"",
        "\"b\": \"000000000000000000000000000000ff\"",
    );
    assert!(matches!(
        migrate_v1(&missing, ProjectId::new(1).unwrap()),
        Err(ProjectFileError::Model(_))
    ));
    assert!(matches!(
        migrate_v1(CURRENT, ProjectId::new(1).unwrap()),
        Err(ProjectFileError::UnsupportedVersion(6))
    ));
}

#[test]
fn current_schema_requires_a_valid_lineage_identity() {
    let missing = CURRENT.replace(
        "  \"project_id\": \"00000000000000000000000000000001\",\n",
        "",
    );
    assert!(matches!(
        decode_project(&missing),
        Err(ProjectFileError::Json(_))
    ));
    let zero = CURRENT.replace(
        "\"project_id\": \"00000000000000000000000000000001\"",
        "\"project_id\": \"00000000000000000000000000000000\"",
    );
    assert!(matches!(
        decode_project(&zero),
        Err(ProjectFileError::Identity(_))
    ));
}

#[test]
fn production_generator_produces_canonical_uuid_v4_typed_identities() {
    let generated = [
        RandomIds::project().unwrap().to_string(),
        RandomIds::device().unwrap().to_string(),
        RandomIds::terminal().unwrap().to_string(),
        RandomIds::conductor().unwrap().to_string(),
        RandomIds::subsystem().unwrap().to_string(),
        RandomIds::interface_port().unwrap().to_string(),
        RandomIds::cable().unwrap().to_string(),
        RandomIds::definition().unwrap().to_string(),
        RandomIds::definition_revision().unwrap().to_string(),
        RandomIds::view().unwrap().to_string(),
        RandomIds::label().unwrap().to_string(),
        RandomIds::polyline().unwrap().to_string(),
        RandomIds::style().unwrap().to_string(),
        RandomIds::anchor().unwrap().to_string(),
        RandomIds::route().unwrap().to_string(),
    ];
    for text in &generated {
        assert_eq!(text.len(), 32);
        assert_eq!(&text[12..13], "4");
        let parsed = uuid::Uuid::parse_str(text).unwrap();
        assert_eq!(parsed.get_variant(), uuid::Variant::RFC4122);
        assert!(!parsed.is_nil());
    }
    let unique: std::collections::BTreeSet<_> = generated.iter().collect();
    assert_eq!(unique.len(), generated.len());
}
