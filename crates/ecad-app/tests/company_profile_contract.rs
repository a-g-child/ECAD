//! Contact-default scope and closed, selective metadata-copy semantics.
use ecad_app::*;
use ecad_domain::*;
use std::collections::{BTreeMap, BTreeSet};
fn profile(values: ProjectMetadata) -> CompanyProfile {
    CompanyProfile::new(
        ProfileId::new(1).unwrap(),
        " Company\nProfile ".into(),
        values,
    )
    .unwrap()
}
#[test]
fn selective_import_preserves_unselected_values_and_only_explicit_empty_selection_clears() {
    let original = ProjectMetadata::new(BTreeMap::from([
        (MetadataField::CompanyName, "Old".into()),
        (MetadataField::CompanyAddress, "Keep address".into()),
        (MetadataField::ProjectTitle, "Keep title".into()),
    ]));
    let mut project = Project::new(ProjectId::new(1).unwrap());
    project.replace_metadata(original.clone());
    let session = Session::new(project);
    let profile = profile(ProjectMetadata::new(BTreeMap::from([(
        MetadataField::CompanyName,
        " New\nName ".into(),
    )])));
    let preview = preview_profile_import(
        &session,
        &profile,
        BTreeSet::from([MetadataField::CompanyName]),
    )
    .unwrap();
    assert_eq!(preview.revision(), session.revision());
    assert_eq!(
        preview.changes(),
        &[ProfileFieldChange {
            field: MetadataField::CompanyName,
            before: Some("Old".into()),
            after: Some(" New\nName ".into())
        }]
    );
    let clear = preview_profile_import(
        &session,
        &profile,
        BTreeSet::from([MetadataField::CompanyAddress]),
    )
    .unwrap();
    assert_eq!(clear.changes()[0].after, None);
    assert_eq!(session.project().metadata(), &original);
    assert_eq!(session.undo_len(), 0);
    assert!(
        preview_profile_import(&session, &profile, BTreeSet::new())
            .unwrap()
            .changes()
            .is_empty()
    );
}
#[test]
fn profiles_and_import_selections_reject_engineering_fields_without_conflating_responsibilities() {
    assert!(
        CompanyProfile::new(
            ProfileId::new(1).unwrap(),
            " \n ".into(),
            ProjectMetadata::default()
        )
        .is_err()
    );
    assert!(matches!(
        CompanyProfile::new(
            ProfileId::new(1).unwrap(),
            "Profile".into(),
            ProjectMetadata::new(BTreeMap::from([(MetadataField::Issue, "B".into())]))
        ),
        Err(ProfileError::UnsupportedField(MetadataField::Issue))
    ));
    let session = Session::new(Project::new(ProjectId::new(1).unwrap()));
    assert!(matches!(
        preview_profile_import(
            &session,
            &profile(ProjectMetadata::default()),
            BTreeSet::from([MetadataField::ProjectTitle])
        ),
        Err(ProfileError::UnsupportedField(MetadataField::ProjectTitle))
    ));
}
