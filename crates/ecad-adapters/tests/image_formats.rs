//! Format validation, portable version gates and exclusive-cell edits preserve historical data.
use ecad_adapters::*;
use ecad_app::*;
use ecad_domain::*;
const JPEG: &[u8] = include_bytes!("../../../apps/desktop/tests/fixtures/picture.jpg");
const SVG: &[u8] = include_bytes!("../../../apps/desktop/tests/fixtures/picture.svg");
fn picture(bytes: &[u8]) -> Picture {
    Picture {
        asset: import_picture(bytes.to_vec()).unwrap(),
        fit: PictureFit::Contain,
    }
}
fn layout(picture: Picture, text: bool) -> TemplateLayout {
    let base = SheetTemplate::StandardV1
        .layout(&SheetPaper::default())
        .unwrap();
    let mut cells = base.tables()[0].cells().to_vec();
    cells[0].picture = Some(picture);
    if !text {
        cells[0].caption.clear();
        cells[0].contents.clear();
    }
    let table =
        TemplateTable::new(base.tables()[0].id(), base.tables()[0].bounds(), cells).unwrap();
    TemplateLayout::new(base.paper().clone(), base.content(), vec![table]).unwrap()
}
fn body(layout: TemplateLayout) -> SavedTemplateVariant {
    SavedTemplateVariant::new(
        TemplateVariantRef {
            definition: TemplateDefinitionId::new(1).unwrap(),
            revision: TemplateRevisionId::new(1).unwrap(),
            variant: TemplateVariantId::new(1).unwrap(),
        },
        "Images".into(),
        layout,
    )
    .unwrap()
}
#[test]
fn jpeg_and_svg_remain_original_portable_assets_with_explicit_schema_gates() {
    for (bytes, format, dimensions) in [
        (JPEG, PictureFormat::Jpeg, (2, 1)),
        (SVG, PictureFormat::Svg, (200, 100)),
    ] {
        let image = picture(bytes);
        assert_eq!(image.asset.bytes(), bytes);
        assert_eq!(image.asset.format(), format);
        assert_eq!((image.asset.width(), image.asset.height()), dimensions);
        let body = body(layout(image.clone(), false));
        let encoded = encode_template_variant(&body).unwrap();
        assert_eq!(decode_template_variant(&encoded).unwrap(), body);
        let mut data: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(data["schema_version"], 6);
        data["schema_version"] = 5.into();
        assert!(decode_template_variant(&data.to_string()).is_err());
        let mut project = Project::new(ProjectId::new(1).unwrap());
        let view = ViewId::new(1).unwrap();
        project
            .insert_drawing_view(DrawingView {
                id: view,
                name: "Pictures".into(),
                overrides: Default::default(),
            })
            .unwrap();
        project
            .insert_drawing_picture(DrawingPicture {
                id: PictureId::new(1).unwrap(),
                view,
                position: DrawingPoint { x: 0, y: 0 },
                width: 40000,
                height: 20000,
                picture: image,
            })
            .unwrap();
        project
            .insert_template_revision(std::sync::Arc::new(
                TemplateRevision::new(body.reference().revision_ref(), vec![body.clone()]).unwrap(),
            ))
            .unwrap();
        let encoded = encode_project(&project).unwrap();
        let reopened = decode_project(&encoded).unwrap();
        assert_eq!(encode_project(&reopened).unwrap(), encoded);
        assert_eq!(reopened.saved_template(body.reference()).unwrap(), &body);
        let mut data: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(data["schema_version"], 15);
        data["schema_version"] = 14.into();
        assert!(decode_project(&data.to_string()).is_err());
    }
}
#[test]
fn svg_rejects_active_external_malformed_and_excessive_content_without_resolution() {
    for content in [
        "<script>alert(1)</script>",
        "<foreignObject/>",
        "<image href='file:///tmp/logo.png'/>",
        "<use href='https://example.test/logo.svg#x'/>",
        "<g onload='alert(1)'/>",
        "<style>@import 'https://example.test/x.css';</style>",
        "<style>.x {fill:url(https://example.test/x.svg)}</style>",
        r#"<path style='fill:u\72l("https://example.test/x.svg")'/>"#,
        "<g xml:base='https://example.test/'/>",
        "<animate attributeName='href'/>",
        "<rect width='10' height='10'></g>",
    ] {
        let svg = format!(
            "<svg xmlns='http://www.w3.org/2000/svg' width='20' height='10'>{content}</svg>"
        );
        assert!(
            import_picture(svg.into_bytes()).is_err(),
            "accepted {content}"
        );
    }
    for text in [
        "<svg/>",
        "<!DOCTYPE svg [<!ENTITY x SYSTEM 'file:///tmp/x'>]><svg xmlns='http://www.w3.org/2000/svg'>&x;</svg>",
        "<svg xmlns='http://www.w3.org/2000/svg' width='90000' height='90000'/>",
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 NaN 10'/>",
    ] {
        assert!(import_picture(text.as_bytes().to_vec()).is_err());
    }
    let svg = b"<svg xmlns='http://www.w3.org/2000/svg' width='25.4mm' height='12.7mm'><defs><path id='a'/></defs><use href='#a' style='fill:url(\"#a\")'/></svg>";
    let image = import_picture(svg.to_vec()).unwrap();
    assert_eq!((image.width(), image.height()), (96, 48));
    assert!(import_picture(JPEG[..40].to_vec()).is_err());
}
#[test]
fn exif_orientation_preserves_original_jpeg_and_swaps_display_dimensions() {
    let tiff = b"II\x2a\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x06\0\0\0\0\0\0\0";
    let mut bytes = vec![0xff, 0xd8, 0xff, 0xe1];
    bytes.extend_from_slice(&u16::try_from(tiff.len() + 8).unwrap().to_be_bytes());
    bytes.extend_from_slice(b"Exif\0\0");
    bytes.extend_from_slice(tiff);
    bytes.extend_from_slice(&JPEG[2..]);
    let asset = import_picture(bytes.clone()).unwrap();
    assert_eq!(asset.bytes(), bytes);
    assert_eq!((asset.width(), asset.height()), (1, 2));
}
#[test]
fn new_mixed_cells_reject_atomically_while_legacy_cells_resolve_with_exact_undo() {
    let png = include_bytes!("../../../apps/desktop/tests/fixtures/picture.png");
    let mixed = layout(picture(png), true);
    let plain = layout(picture(png), false);
    let variant = TemplateVariantId::new(1).unwrap();
    assert!(
        TemplateDocument::new(
            TemplateDefinitionId::new(2).unwrap(),
            variant,
            "New".into(),
            mixed.clone()
        )
        .is_err()
    );
    let mut doc = TemplateDocument::new(
        TemplateDefinitionId::new(2).unwrap(),
        variant,
        "New".into(),
        plain.clone(),
    )
    .unwrap();
    let revision = doc.revision();
    assert!(
        doc.replace_variant(revision, variant, "New".into(), mixed.clone())
            .is_err()
    );
    assert_eq!(doc.revision(), revision);
    let old = body(mixed.clone());
    let encoded = encode_template_variant(&old).unwrap();
    assert_eq!(
        encode_template_variant(&decode_template_variant(&encoded).unwrap()).unwrap(),
        encoded
    );
    let saved = TemplateRevision::new(old.reference().revision_ref(), vec![old]).unwrap();
    let mut doc = TemplateDocument::open(&saved);
    doc.replace_variant(doc.revision(), variant, "Images".into(), plain.clone())
        .unwrap();
    doc.undo(doc.revision()).unwrap();
    assert_eq!(doc.variant(variant).unwrap().layout(), &mixed);
    doc.redo(doc.revision()).unwrap();
    assert_eq!(doc.variant(variant).unwrap().layout(), &plain);
}
