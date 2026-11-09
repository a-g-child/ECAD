//! Cell block defaults, exact override intent, immutable storage and independent undo.
use ecad_adapters::*;
use ecad_app::*;
use ecad_domain::*;
fn body(layout: TemplateLayout) -> SavedTemplateVariant {
    SavedTemplateVariant::new(
        TemplateVariantRef {
            definition: TemplateDefinitionId::new(91).unwrap(),
            revision: TemplateRevisionId::new(91).unwrap(),
            variant: TemplateVariantId::new(91).unwrap(),
        },
        "Cell layout".into(),
        layout,
    )
    .unwrap()
}
#[test]
fn block_layout_resolves_per_property_roundtrips_and_restores_exact_defaults_and_overrides() {
    let old = SheetTemplate::StandardV1
        .layout(&SheetPaper::default())
        .unwrap();
    let legacy = encode_template_variant(&body(old.clone())).unwrap();
    let defaults = TemplateTextStyles {
        layout: TemplateCellTextLayout {
            caption_position: Some("inline".into()),
            separator: Some(" - ".into()),
            horizontal: Some("right".into()),
            vertical: Some("middle".into()),
        },
        ..Default::default()
    };
    let mut cells = old.tables()[0].cells().to_vec();
    cells[0].text_styles.layout.separator = Some(String::new());
    cells[0].text_styles.layout.horizontal = Some("right".into());
    let original = cells[0].clone();
    let table = TemplateTable::new(old.tables()[0].id(), old.tables()[0].bounds(), cells).unwrap();
    let split = table
        .split_cell(original.id, 2, 1, &[TemplateCellId::new(999).unwrap()])
        .unwrap();
    assert_eq!(
        split.cells()[1].text_styles.layout,
        original.text_styles.layout
    );
    let layout = TemplateLayout::new(old.paper().clone(), old.content(), vec![table])
        .unwrap()
        .with_text_defaults(defaults.clone())
        .unwrap();
    let saved = body(layout.clone());
    let encoded = encode_template_variant(&saved).unwrap();
    assert!(encoded.contains("\"schema_version\": 7"));
    assert_eq!(decode_template_variant(&encoded).unwrap(), saved);
    assert!(
        decode_template_variant(&encoded.replace("\"schema_version\": 7", "\"schema_version\": 6"))
            .is_err()
    );
    assert_eq!(
        encode_template_variant(&decode_template_variant(&legacy).unwrap()).unwrap(),
        legacy
    );
    let family =
        TemplateRevision::new(saved.reference().revision_ref(), vec![saved.clone()]).unwrap();
    let mut project = Project::new(ProjectId::new(1).unwrap());
    project
        .insert_template_revision(std::sync::Arc::new(family.clone()))
        .unwrap();
    let sheet = SheetId::new(1).unwrap();
    project
        .insert_sheet(
            DrawingSheet {
                id: sheet,
                kind: SheetKind::Cover,
                template: Some(SheetTemplate::Saved(saved.reference())),
                inherited_variant: None,
                number: "1".into(),
                title: "Layout sheet".into(),
                paper: old.paper().clone(),
                placement: None,
            },
            0,
        )
        .unwrap();
    let frame = project.sheet_frame(sheet).unwrap();
    let resolved = &frame.cells[0].text_styles.layout;
    assert_eq!(resolved.caption_position.as_deref(), Some("inline"));
    assert_eq!(resolved.separator.as_deref(), Some(""));
    assert_eq!(resolved.vertical.as_deref(), Some("middle"));
    let encoded = encode_project(&project).unwrap();
    assert_eq!(
        decode_project(&encoded)
            .unwrap()
            .sheet_frame(sheet)
            .unwrap(),
        frame
    );
    let mut doc = TemplateDocument::open(&family);
    let reset = old.with_text_defaults(defaults).unwrap();
    doc.replace_variant(
        doc.revision(),
        saved.reference().variant,
        "Cell layout".into(),
        reset.clone(),
    )
    .unwrap();
    doc.undo(doc.revision()).unwrap();
    assert_eq!(
        doc.variant(saved.reference().variant).unwrap().layout(),
        &layout
    );
    doc.redo(doc.revision()).unwrap();
    assert_eq!(
        doc.variant(saved.reference().variant).unwrap().layout(),
        &reset
    );
}
#[test]
fn invalid_layout_values_are_rejected_without_changing_the_draft() {
    let base = SheetTemplate::StandardV1
        .layout(&SheetPaper::default())
        .unwrap();
    for invalid in [
        TemplateCellTextLayout {
            caption_position: Some("below".into()),
            ..Default::default()
        },
        TemplateCellTextLayout {
            horizontal: Some("justify".into()),
            ..Default::default()
        },
        TemplateCellTextLayout {
            vertical: Some("centre".into()),
            ..Default::default()
        },
        TemplateCellTextLayout {
            separator: Some("x\ny".into()),
            ..Default::default()
        },
        TemplateCellTextLayout {
            separator: Some("x".repeat(65)),
            ..Default::default()
        },
    ] {
        assert!(invalid.validate().is_err());
        let styles = TemplateTextStyles {
            layout: invalid,
            ..Default::default()
        };
        assert!(base.clone().with_text_defaults(styles).is_err());
    }
}
