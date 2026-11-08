//! Standalone immutable template bodies retain exact geometry/bindings without project values.
use ecad_adapters::*;
use ecad_domain::*;
fn variant(layout: TemplateLayout) -> SavedTemplateVariant {
    SavedTemplateVariant::new(
        TemplateVariantRef {
            definition: TemplateDefinitionId::new(1).unwrap(),
            revision: TemplateRevisionId::new(2).unwrap(),
            variant: TemplateVariantId::new(3).unwrap(),
        },
        "Custom\nTemplate".into(),
        layout,
    )
    .unwrap()
}
#[test]
fn all_legacy_layouts_roundtrip_without_project_or_library_state() {
    for template in SheetTemplate::ALL {
        for paper in [
            SheetPaper::default(),
            SheetPaper {
                width: 10,
                height: 20,
                left: 1,
                right: 2,
                top: 3,
                bottom: 4,
            },
        ] {
            let v = variant(template.layout(&paper).unwrap());
            let text = encode_template_variant(&v).unwrap();
            assert!(text.ends_with('\n'));
            let reopened = decode_template_variant(&text).unwrap();
            assert_eq!(reopened, v);
            assert_eq!(encode_template_variant(&reopened).unwrap(), text);
            assert!(!text.contains("project_id"));
        }
    }
}
#[test]
fn uneven_rows_and_literal_field_runs_roundtrip_with_stable_ids() {
    let mut json: serde_json::Value = serde_json::from_str(
        &encode_template_variant(&variant(
            SheetTemplate::StandardV1
                .layout(&SheetPaper::default())
                .unwrap(),
        ))
        .unwrap(),
    )
    .unwrap();
    let table = &mut json["tables"][0];
    // One full-width cell above two unequal cells, with a literal/field composition.
    table["cells"] = serde_json::json!([
        {"id": "0000000000000000000000000000000a", "bounds_um": [0, 0, 180000, 10000], "caption": "Company\naddress", "contents": [{"kind":"literal","value":"  prefix\n\n"},{"kind":"field","value":"company.address"}]},
        {"id": "0000000000000000000000000000000b", "bounds_um": [0, 10000, 60000, 30000], "caption": "", "contents": []},
        {"id": "0000000000000000000000000000000c", "bounds_um": [60000, 10000, 120000, 30000], "caption": "Title", "contents": [{"kind":"field","value":"sheet.title"}]}
    ]);
    let v = decode_template_variant(&json.to_string()).unwrap();
    let first = &v.layout().tables()[0].cells()[0];
    assert_eq!(
        first.resolve(|_| Some("A\nB".into())).value,
        "  prefix\n\nA\nB"
    );
    assert_eq!(
        decode_template_variant(&encode_template_variant(&v).unwrap()).unwrap(),
        v
    );
}
#[test]
fn strict_template_schema_rejects_future_data_invalid_geometry_and_unknown_fields() {
    let text = encode_template_variant(&variant(
        SheetTemplate::StandardV1
            .layout(&SheetPaper::default())
            .unwrap(),
    ))
    .unwrap();
    let base: serde_json::Value = serde_json::from_str(&text).unwrap();
    let mut json = base.clone();
    json["schema_version"] = 99.into();
    assert!(matches!(
        decode_template_variant(&json.to_string()),
        Err(TemplateFileError::Version(99))
    ));
    let mut json = base.clone();
    json["tables"][0]["cells"][0]["contents"][0]["value"] = "future.field".into();
    assert!(matches!(
        decode_template_variant(&json.to_string()),
        Err(TemplateFileError::Field(_))
    ));
    let mut json = base.clone();
    json["tables"][0]["cells"][0]["bounds_um"][2] = 1.into();
    assert!(matches!(
        decode_template_variant(&json.to_string()),
        Err(TemplateFileError::Layout(TemplateError::Gap))
    ));
    for path in ["root", "table", "cell", "run"] {
        let mut json = base.clone();
        let target = match path {
            "root" => &mut json,
            "table" => &mut json["tables"][0],
            "cell" => &mut json["tables"][0]["cells"][0],
            _ => &mut json["tables"][0]["cells"][0]["contents"][0],
        };
        target["future"] = true.into();
        assert!(
            matches!(
                decode_template_variant(&json.to_string()),
                Err(TemplateFileError::Json(_))
            ),
            "{path}"
        );
    }
    for id in [
        "0",
        "00000000000000000000000000000000",
        "000000000000000000000000000000AF",
    ] {
        let mut json = base.clone();
        json["revision"] = id.into();
        assert!(matches!(
            decode_template_variant(&json.to_string()),
            Err(TemplateFileError::Identity(_))
        ));
    }
    assert_eq!(
        encode_template_variant(&decode_template_variant(&text).unwrap()).unwrap(),
        text
    );
}
