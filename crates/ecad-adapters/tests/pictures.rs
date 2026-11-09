//! Portable PNG validation, content deduplication, schema gates and exact placement history.
use ecad_adapters::*;
use ecad_app::*;
use ecad_domain::*;
const PNG: &[u8] = include_bytes!("../../../apps/desktop/tests/fixtures/picture.png");
fn picture() -> Picture {
    Picture {
        asset: import_png(PNG.to_vec()).unwrap(),
        fit: PictureFit::Contain,
    }
}
#[test]
fn png_import_is_bounded_and_rejects_corruption_without_guessing_formats() {
    let p = picture();
    assert_eq!((p.asset.width(), p.asset.height()), (2, 1));
    assert_eq!(p.asset.bytes(), PNG);
    assert_eq!(
        picture_from_base64(&picture_base64(&p.asset)).unwrap(),
        p.asset
    );
    for bytes in [
        vec![],
        b"<svg/>".to_vec(),
        PNG[..PNG.len() - 8].to_vec(),
        vec![0; MAX_PICTURE_BYTES + 1],
    ] {
        assert!(import_png(bytes).is_err());
    }
    let mut bad = PNG.to_vec();
    bad[45] ^= 1;
    assert!(import_png(bad).is_err());
    assert!(picture_from_base64("not base64").is_err());
}
#[test]
fn template_pictures_are_deduplicated_and_survive_partitions_and_old_schema_rejection() {
    let base = SheetTemplate::StandardV1
        .layout(&SheetPaper::default())
        .unwrap();
    let mut cells = base.tables()[0].cells().to_vec();
    cells[0].picture = Some(picture());
    let cell_id = cells[0].id;
    let table =
        TemplateTable::new(base.tables()[0].id(), base.tables()[0].bounds(), cells).unwrap();
    let graphic = TemplateGraphic {
        id: TemplateGraphicId::new(9).unwrap(),
        bounds: FrameRect {
            x: 1000,
            y: 1000,
            width: 20000,
            height: 10000,
        },
        shape: TemplateGraphicShape::Picture(picture()),
        font: "sans-serif".into(),
        text_height: 1000,
        stroke_width: 250,
        colour: [0, 0, 0],
    };
    let layout = TemplateLayout::new(base.paper().clone(), base.content(), vec![table.clone()])
        .unwrap()
        .with_graphics(vec![graphic])
        .unwrap();
    let body = SavedTemplateVariant::new(
        TemplateVariantRef {
            definition: TemplateDefinitionId::new(1).unwrap(),
            revision: TemplateRevisionId::new(1).unwrap(),
            variant: TemplateVariantId::new(1).unwrap(),
        },
        "Pictures".into(),
        layout,
    )
    .unwrap();
    let encoded = encode_template_variant(&body).unwrap();
    let mut data: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(data["schema_version"], 5);
    assert_eq!(data["assets"].as_object().unwrap().len(), 1);
    assert_eq!(decode_template_variant(&encoded).unwrap(), body);
    let mut bundled = Project::new(ProjectId::new(10).unwrap());
    bundled
        .insert_template_revision(std::sync::Arc::new(
            TemplateRevision::new(body.reference().revision_ref(), vec![body.clone()]).unwrap(),
        ))
        .unwrap();
    let reopened = decode_project(&encode_project(&bundled).unwrap()).unwrap();
    assert_eq!(reopened.saved_template(body.reference()).unwrap(), &body);

    let key = picture_key(&picture().asset);
    let duplicate = encoded.replacen(
        &format!("\"{key}\":"),
        &format!("\"{key}\": \"bad\", \"{key}\":"),
        1,
    );
    assert_ne!(duplicate, encoded);
    assert!(decode_template_variant(&duplicate).is_err());

    let split = table
        .split_cell(cell_id, 2, 1, &[TemplateCellId::new(99).unwrap()])
        .unwrap();
    assert_eq!(
        split.cells().iter().filter(|c| c.picture.is_some()).count(),
        1
    );
    assert_eq!(
        split
            .cells()
            .iter()
            .find(|c| c.id == cell_id)
            .unwrap()
            .picture,
        table.cells()[0].picture
    );

    let extra = TemplateCellId::new(99).unwrap();
    let mut cells = split.cells().to_vec();
    cells.iter_mut().find(|c| c.id == extra).unwrap().picture = Some(picture());
    let both = TemplateTable::new(split.id(), split.bounds(), cells).unwrap();
    assert!(matches!(
        both.merge_cells(&[cell_id, extra], cell_id, MergeContents::Combine),
        Err(TableEditError::Pictures)
    ));
    let merged = both
        .merge_cells(&[cell_id, extra], cell_id, MergeContents::KeepPrimary)
        .unwrap();
    assert_eq!(
        merged
            .cells()
            .iter()
            .find(|c| c.id == cell_id)
            .unwrap()
            .picture,
        table.cells()[0].picture
    );
    data["schema_version"] = serde_json::json!(4);
    assert!(decode_template_variant(&data.to_string()).is_err());
    data["schema_version"] = serde_json::json!(5);
    data["assets"] = serde_json::json!({});
    assert!(decode_template_variant(&data.to_string()).is_err());
}
#[test]
fn drawing_pictures_roundtrip_share_assets_move_and_restore_exactly() {
    let mut project = Project::new(ProjectId::new(1).unwrap());
    let view = ViewId::new(1).unwrap();
    project
        .insert_drawing_view(DrawingView {
            id: view,
            name: "Schematic".into(),
            overrides: Default::default(),
        })
        .unwrap();
    let placement = DrawingPicture {
        id: PictureId::new(1).unwrap(),
        view,
        position: DrawingPoint { x: 5, y: -9 },
        width: 20000,
        height: 10000,
        picture: picture(),
    };
    project.insert_drawing_picture(placement.clone()).unwrap();
    let mut second = placement.clone();
    second.id = PictureId::new(2).unwrap();
    second.position.x = 30000;
    project.insert_drawing_picture(second).unwrap();
    let encoded = encode_project(&project).unwrap();
    let mut data: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(data["schema_version"], 14);
    assert_eq!(data["pictures"]["assets"].as_object().unwrap().len(), 1);
    assert_eq!(
        encode_project(&decode_project(&encoded).unwrap()).unwrap(),
        encoded
    );
    data["schema_version"] = serde_json::json!(13);
    assert!(decode_project(&data.to_string()).is_err());
    assert!(project.remove_drawing_view(view).is_err());
    let mut session = Session::new(project);
    let before = session
        .project()
        .drawing()
        .picture(placement.id)
        .unwrap()
        .clone();
    let preview = preview_layout(
        &session,
        [ObjectRef::Picture(placement.id)],
        LayoutOperation::Translate(DrawingPoint { x: 1234, y: -4321 }),
    )
    .unwrap();
    session.apply_layout_preview(preview).unwrap();
    let moved = session.project().drawing().picture(placement.id).unwrap();
    assert_eq!(moved.position, DrawingPoint { x: 1239, y: -4330 });
    assert_eq!(moved.picture, before.picture);
    session.undo(session.revision()).unwrap();
    assert_eq!(
        session.project().drawing().picture(placement.id),
        Some(&before)
    );
}
