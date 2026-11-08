//! Artwork survives portable bodies and resolves against each sheet without rewriting bindings.
use ecad_adapters::*;
use ecad_domain::*;
fn graphic() -> TemplateGraphic {
    TemplateGraphic {
        id: TemplateGraphicId::new(1).unwrap(),
        bounds: FrameRect {
            x: 1000,
            y: 1000,
            width: 80000,
            height: 20000,
        },
        shape: TemplateGraphicShape::Text(vec![
            TemplateTextRun::Literal("Drawing\n".into()),
            TemplateTextRun::Field(FrameField::SheetTitle),
        ]),
        font: "sans-serif".into(),
        text_height: 3500,
        stroke_width: 250,
        colour: [10, 20, 30],
    }
}
fn body(graphics: Vec<TemplateGraphic>) -> SavedTemplateVariant {
    SavedTemplateVariant::new(
        TemplateVariantRef {
            definition: TemplateDefinitionId::new(1).unwrap(),
            revision: TemplateRevisionId::new(1).unwrap(),
            variant: TemplateVariantId::new(1).unwrap(),
        },
        "Artwork".into(),
        SheetTemplate::StandardV1
            .layout(&SheetPaper::default())
            .unwrap()
            .with_graphics(graphics)
            .unwrap(),
    )
    .unwrap()
}
#[test]
fn artwork_bodies_are_versioned_exact_and_strict() {
    let mut shapes = vec![graphic()];
    for shape in [
        TemplateGraphicShape::Rectangle,
        TemplateGraphicShape::Ellipse,
        TemplateGraphicShape::Polyline(vec![[0, 0], [80000, 0], [80000, 20000]]),
    ] {
        let mut g = graphic();
        g.id = TemplateGraphicId::new(shapes.len() as u128 + 1).unwrap();
        g.shape = shape;
        shapes.push(g);
    }
    let original = body(shapes);
    let encoded = encode_template_variant(&original).unwrap();
    assert_eq!(decode_template_variant(&encoded).unwrap(), original);
    let v: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(v["schema_version"], 3);
    for (key, value) in [
        ("schema_version", serde_json::json!(1)),
        ("schema_version", serde_json::json!(99)),
        ("graphics", serde_json::json!(null)),
    ] {
        let mut bad = v.clone();
        bad[key] = value;
        assert!(decode_template_variant(&bad.to_string()).is_err());
    }
    let mut bad = v.clone();
    bad["graphics"][0]["shape"]["value"][1]["value"] = "unknown.field".into();
    assert!(decode_template_variant(&bad.to_string()).is_err());
    let mut bad = v;
    bad["graphics"][0]["bounds_um"][0] = u32::MAX.into();
    assert!(decode_template_variant(&bad.to_string()).is_err());
    let plain = encode_template_variant(&body(vec![])).unwrap();
    assert!(!plain.contains("graphics"));
    assert_eq!(decode_template_variant(&plain).unwrap(), body(vec![]));
}
#[test]
fn artwork_validation_retains_margin_objects_and_rejects_invalid_geometry() {
    let base = SheetTemplate::StandardV1
        .layout(&SheetPaper::default())
        .unwrap();
    let good = graphic();
    assert!(base.clone().with_graphics(vec![good.clone()]).is_ok());
    assert!(
        base.clone()
            .with_graphics(vec![good.clone(), good.clone()])
            .is_err()
    );
    for shape in [
        TemplateGraphicShape::Polyline(vec![]),
        TemplateGraphicShape::Polyline(vec![[0, 0], [0, 0]]),
        TemplateGraphicShape::Polyline(vec![[0, 0], [80001, 0]]),
    ] {
        let mut g = good.clone();
        g.shape = shape;
        assert!(base.clone().with_graphics(vec![g]).is_err());
    }
    let mut g = good.clone();
    g.stroke_width = 0;
    assert!(base.clone().with_graphics(vec![g]).is_err());
    let mut line = good;
    line.bounds.height = 0;
    line.shape = TemplateGraphicShape::Polyline(vec![[0, 0], [80000, 0]]);
    assert!(base.with_graphics(vec![line]).is_ok());
}
#[test]
fn project_roundtrip_resolves_per_sheet_without_changing_library_bindings() {
    let body = body(vec![graphic()]);
    let pin = body.reference();
    let revision = TemplateRevision::new(pin.revision_ref(), vec![body.clone()]).unwrap();
    let mut project = Project::new(ProjectId::new(1).unwrap());
    project
        .insert_template_revision(std::sync::Arc::new(revision))
        .unwrap();
    for (index, title) in ["First\nSheet", "Second Sheet"].into_iter().enumerate() {
        let id = SheetId::new(index as u128 + 1).unwrap();
        project
            .insert_sheet(
                DrawingSheet {
                    id,
                    kind: SheetKind::Cover,
                    template: Some(SheetTemplate::Saved(pin)),
                    inherited_variant: None,
                    number: (index + 1).to_string(),
                    title: title.into(),
                    paper: SheetPaper::default(),
                    placement: None,
                },
                index,
            )
            .unwrap();
        let frame = project.sheet_frame(id).unwrap();
        assert_eq!(
            frame.graphics[0].shape,
            TemplateGraphicShape::Text(vec![TemplateTextRun::Literal(format!("Drawing\n{title}"))])
        );
    }
    let bytes = encode_project(&project).unwrap();
    let loaded = decode_project(&bytes).unwrap();
    assert_eq!(encode_project(&loaded).unwrap(), bytes);
    assert_eq!(loaded.saved_template(pin).unwrap(), &body);
    assert_eq!(
        loaded.sheet_frame(SheetId::new(1).unwrap()).unwrap(),
        project.sheet_frame(SheetId::new(1).unwrap()).unwrap()
    );
}
