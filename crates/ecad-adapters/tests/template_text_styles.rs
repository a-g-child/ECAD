//! Appearance inheritance, structural edits, immutable bodies and exact draft history.
use ecad_adapters::*;
use ecad_app::*;
use ecad_domain::*;
fn variant(layout: TemplateLayout) -> SavedTemplateVariant {
    SavedTemplateVariant::new(
        TemplateVariantRef {
            definition: TemplateDefinitionId::new(710).unwrap(),
            revision: TemplateRevisionId::new(710).unwrap(),
            variant: TemplateVariantId::new(710).unwrap(),
        },
        "Styled table".into(),
        layout,
    )
    .unwrap()
}
#[test]
fn independent_channels_inherit_per_property_and_survive_structure_storage_and_history() {
    let old = SheetTemplate::StandardV1
        .layout(&SheetPaper::default())
        .unwrap();
    let legacy = encode_template_variant(&variant(old.clone())).unwrap();
    let defaults = TemplateTextStyles {
        layout: Default::default(),
        caption: TemplateTextStyle {
            font: Some("serif".into()),
            height: Some(1800),
            bold: Some(true),
            ..Default::default()
        },
        content: TemplateTextStyle {
            height: Some(3200),
            colour: Some([4, 5, 6]),
            ..Default::default()
        },
    };
    let table = &old.tables()[0];
    let mut cells = table.cells().to_vec();
    cells[0].text_styles.caption.bold = Some(false);
    cells[0].text_styles.content.height = Some(3200); // Equal-valued explicit override survives.
    let styles = cells[0].text_styles.clone();
    let resolved = styles.caption.over(&defaults.caption);
    assert_eq!(resolved.bold, Some(false));
    assert_eq!(resolved.font.as_deref(), Some("serif"));
    assert_eq!(styles.content.font, None);
    let table = TemplateTable::new(table.id(), table.bounds(), cells).unwrap();
    let fresh = TemplateCellId::new(9999).unwrap();
    let split = table
        .split_cell(table.cells()[0].id, 2, 1, &[fresh])
        .unwrap();
    assert_eq!(split.cells()[1].text_styles, styles);
    let merged = split
        .merge_cells(
            &[split.cells()[0].id, fresh],
            split.cells()[0].id,
            MergeContents::KeepPrimary,
        )
        .unwrap();
    assert_eq!(merged.cells()[0].text_styles, styles);
    let layout = TemplateLayout::new(old.paper().clone(), old.content(), vec![merged])
        .unwrap()
        .with_text_defaults(defaults)
        .unwrap();
    let body = variant(layout.clone());
    let encoded = encode_template_variant(&body).unwrap();
    assert!(encoded.contains("\"schema_version\": 4"));
    assert_eq!(decode_template_variant(&encoded).unwrap(), body);
    assert_eq!(
        encode_template_variant(&decode_template_variant(&legacy).unwrap()).unwrap(),
        legacy
    );
    assert!(
        decode_template_variant(&encoded.replace("\"schema_version\": 4", "\"schema_version\": 1"))
            .is_err()
    );
    // A pinned project sheet resolves both channels without changing its reusable body.
    let pin = body.reference();
    let mut project = Project::new(ProjectId::new(710).unwrap());
    project
        .insert_template_revision(std::sync::Arc::new(
            TemplateRevision::new(pin.revision_ref(), vec![body.clone()]).unwrap(),
        ))
        .unwrap();
    let sheet = SheetId::new(710).unwrap();
    project
        .insert_sheet(
            DrawingSheet {
                id: sheet,
                kind: SheetKind::Cover,
                template: Some(SheetTemplate::Saved(pin)),
                inherited_variant: None,
                number: "1".into(),
                title: "Styled cover".into(),
                paper: SheetPaper::default(),
                placement: None,
            },
            0,
        )
        .unwrap();
    let frame = project.sheet_frame(sheet).unwrap();
    assert_eq!(
        frame.cells[0].text_styles.caption.font.as_deref(),
        Some("serif")
    );
    assert_eq!(frame.cells[0].text_styles.caption.bold, Some(false));
    assert_eq!(frame.cells[0].text_styles.content.colour, Some([4, 5, 6]));
    let bytes = encode_project(&project).unwrap();
    let loaded = decode_project(&bytes).unwrap();
    assert_eq!(loaded.sheet_frame(sheet).unwrap(), frame);
    assert_eq!(loaded.saved_template(pin).unwrap(), &body);
    let revision =
        TemplateRevision::new(body.reference().revision_ref(), vec![variant(old.clone())]).unwrap();
    let mut doc = TemplateDocument::open(&revision);
    doc.replace_variant(
        doc.revision(),
        body.reference().variant,
        body.name().into(),
        layout.clone(),
    )
    .unwrap();
    doc.undo(doc.revision()).unwrap();
    assert_eq!(
        doc.variant(body.reference().variant).unwrap().layout(),
        &old
    );
    doc.redo(doc.revision()).unwrap();
    assert_eq!(
        doc.variant(body.reference().variant).unwrap().layout(),
        &layout
    );
}
#[test]
fn invalid_explicit_styles_are_rejected_before_a_layout_can_be_committed() {
    for invalid in [
        TemplateTextStyle {
            height: Some(0),
            ..Default::default()
        },
        TemplateTextStyle {
            font: Some(" \n".into()),
            ..Default::default()
        },
        TemplateTextStyle {
            horizontal: Some("justify".into()),
            ..Default::default()
        },
    ] {
        let old = SheetTemplate::StandardV1
            .layout(&SheetPaper::default())
            .unwrap();
        assert!(
            old.clone()
                .with_text_defaults(TemplateTextStyles {
                    layout: Default::default(),
                    caption: invalid.clone(),
                    content: Default::default()
                })
                .is_err()
        );
        let table = &old.tables()[0];
        let mut cells = table.cells().to_vec();
        cells[0].text_styles.content = invalid;
        assert!(TemplateTable::new(table.id(), table.bounds(), cells).is_err());
    }
}
