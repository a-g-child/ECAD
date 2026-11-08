//! Revisioned tree/scene projections. Millimetre floats are presentation-only, never edit coordinates.
use crate::{dto::*, properties};
use ecad_app::*;
use ecad_domain::*;
fn tree(
    target: ObjectRef,
    label: String,
    parent: Option<ObjectRef>,
    view: Option<ViewId>,
) -> TreeRow {
    TreeRow {
        target: target.into(),
        label,
        parent: parent.map(Into::into),
        view: view.map(|id| id.to_string()),
    }
}
fn mm(p: DrawingPoint) -> [f64; 2] {
    [p.x as f64 / 1000.0, p.y as f64 / 1000.0]
}
pub(crate) fn snapshot(
    document: &Document<ecad_adapters::JsonFile>,
    path: Option<&std::path::Path>,
    registry: &ActionRegistry,
    keymap: &EffectiveKeymap,
) -> Snapshot {
    let session = document.session();
    let project = session.project();
    let drawing = project.drawing();
    let mut objects = vec![];
    let mut scene = vec![];
    for s in project.subsystems() {
        objects.push(tree(
            ObjectRef::Subsystem(s.id()),
            s.name().into(),
            s.parent().map(ObjectRef::Subsystem),
            None,
        ));
    }
    for d in project.devices() {
        objects.push(tree(
            ObjectRef::Device(d.id()),
            d.designation().into(),
            d.subsystem().map(ObjectRef::Subsystem),
            None,
        ));
        for t in d.terminals() {
            objects.push(tree(
                ObjectRef::Terminal(t.id()),
                format!("Terminal {}", t.number()),
                Some(ObjectRef::Device(d.id())),
                None,
            ));
        }
    }
    for c in project.cables() {
        objects.push(tree(
            ObjectRef::Cable(c.id()),
            c.designation().into(),
            None,
            None,
        ));
    }
    for c in project.conductors() {
        objects.push(tree(
            ObjectRef::Conductor(c.id()),
            format!("Conductor {}", c.id()),
            project.conductor_cable(c.id()).map(ObjectRef::Cable),
            None,
        ));
    }
    for p in project.ports() {
        objects.push(tree(
            ObjectRef::Port(p.id()),
            p.name().into(),
            Some(ObjectRef::Subsystem(p.subsystem())),
            None,
        ));
    }
    for v in drawing.views() {
        objects.push(tree(
            ObjectRef::View(v.id),
            v.name.clone(),
            None,
            Some(v.id),
        ));
    }
    for l in drawing.labels() {
        let target = ObjectRef::Label(l.id);
        objects.push(tree(
            target,
            l.text.clone(),
            Some(ObjectRef::View(l.view)),
            Some(l.view),
        ));
        scene.push(label(project, l));
    }
    for p in drawing.polylines() {
        let target = ObjectRef::Polyline(p.id);
        objects.push(tree(
            target,
            "Graphical path".into(),
            Some(ObjectRef::View(p.view)),
            Some(p.view),
        ));
        scene.push(polyline(project, p));
    }
    for r in drawing.routes() {
        let target = ObjectRef::Route(r.id);
        let label = format!("Route · conductor {}", r.conductor);
        objects.push(tree(
            target,
            label.clone(),
            Some(ObjectRef::View(r.view)),
            Some(r.view),
        ));
        let (colour, width) = properties::stroke(project, target);
        scene.push(SceneRow {
            target: target.into(),
            view: r.view.to_string(),
            label,
            shape: "path".into(),
            points: r.points.iter().copied().map(mm).collect(),
            colour,
            width,
            text_height: 0.0,
            font: String::new(),
        });
    }
    for a in drawing.terminal_anchors() {
        let device = project
            .device(
                project
                    .terminal_owner(a.terminal)
                    .expect("validated terminal"),
            )
            .expect("validated owner");
        let terminal = device
            .terminals()
            .find(|t| t.id() == a.terminal)
            .expect("validated terminal");
        let label = format!("{} : {}", device.designation(), terminal.number());
        let target = ObjectRef::Anchor(a.id);
        objects.push(tree(
            target,
            label.clone(),
            Some(ObjectRef::View(a.view)),
            Some(a.view),
        ));
        scene.push(SceneRow {
            target: target.into(),
            view: a.view.to_string(),
            label,
            shape: "anchor".into(),
            points: vec![mm(a.position)],
            colour: "#344454".into(),
            width: 0.25,
            text_height: 2.3,
            font: "sans-serif".into(),
        });
    }
    let mut context = ActionContext::for_session(session);
    context.view = drawing.views().next().map(|v| v.id);
    let actions = registry
        .definitions()
        .map(|d| {
            let supported = matches!(
                d.handler,
                ActionHandler::Sheets
                    | ActionHandler::OpenProject
                    | ActionHandler::CompanyProfiles
                    | ActionHandler::ProjectMetadata
                    | ActionHandler::DrawingSettings
                    | ActionHandler::CreateLabel
                    | ActionHandler::CreatePolyline
                    | ActionHandler::FitDrawing
                    | ActionHandler::MoveSelection
                    | ActionHandler::RotateSelection
                    | ActionHandler::MirrorSelection
                    | ActionHandler::AlignSelection
                    | ActionHandler::DistributeSelection
                    | ActionHandler::Undo
                    | ActionHandler::Redo
                    | ActionHandler::Save
                    | ActionHandler::Properties
                    | ActionHandler::Palette
                    | ActionHandler::ConnectionReport
            );
            let mut target = context.clone();
            if d.handler == ActionHandler::FitDrawing
                && let Some(sheet) = project.sheets().first()
            {
                target.sheet = Some(sheet.id);
                target.view = sheet.placement.as_ref().map(|p| p.view);
            }
            let state = registry.state(&d.id, &target, Some(session));
            ActionRow {
                id: d.id.to_string(),
                label: d.label.clone(),
                enabled: supported
                    && (state.enabled()
                        || matches!(
                            d.handler,
                            ActionHandler::Properties
                                | ActionHandler::MoveSelection
                                | ActionHandler::RotateSelection
                                | ActionHandler::MirrorSelection
                                | ActionHandler::AlignSelection
                                | ActionHandler::DistributeSelection
                        )),
                reason: if !supported {
                    Some("Tool UI is planned".into())
                } else {
                    state.disabled.map(|r| format!("{r:?}"))
                },
                shortcuts: keymap.shortcuts(&d.id),
            }
        })
        .collect();
    Snapshot {
        drawing_documents: crate::document_rows::project(project),
        sheets: project.sheets().iter().map(Into::into).collect(),
        sheet_frames: project
            .sheets()
            .iter()
            .map(|s| {
                crate::frame_rows::FrameRow::new(
                    s.id,
                    &project.sheet_frame(s.id).expect("validated sheet owner"),
                )
            })
            .collect(),
        bridge_version: 1,
        next_input: "1".into(),
        revision: session.revision().into(),
        name: path.and_then(|p| p.file_name()).map_or_else(
            || "Conveyor demonstration".into(),
            |n| n.to_string_lossy().into_owned(),
        ),
        path: path.map(|p| p.to_string_lossy().into_owned()),
        dirty: session.is_dirty(),
        views: drawing
            .views()
            .map(|v| ViewRow {
                id: v.id.to_string(),
                name: v.name.clone(),
            })
            .collect(),
        objects,
        scene,
        actions,
    }
}
pub(crate) fn demo() -> Result<Project, BridgeError> {
    let mut project = ecad_adapters::decode_project(include_str!(
        "../../ecad-adapters/tests/fixtures/controller-conveyor-v6.json"
    ))
    .map_err(|e| BridgeError::new("sample", e))?;
    let view = ViewId::new(1).map_err(|e| BridgeError::new("sample", e))?;
    let result = (|| -> Result<(), ModelError> {
        project.insert_drawing_view(DrawingView {
            id: view,
            name: "Conveyor · detailed wiring".into(),
            overrides: StylePatch::default(),
        })?;
        for (id, terminal, x) in [(1, 101, 0), (2, 201, 50000), (3, 301, 100000)] {
            project.insert_terminal_anchor(TerminalAnchor {
                id: AnchorId::new(id).expect("constant"),
                view,
                terminal: TerminalId::new(terminal).expect("constant"),
                position: DrawingPoint { x, y: 0 },
            })?;
        }
        for (id, points) in [
            (
                1,
                vec![
                    (0, 0),
                    (20000, 0),
                    (20000, 15000),
                    (50000, 15000),
                    (50000, 0),
                ],
            ),
            (
                2,
                vec![
                    (50000, 0),
                    (75000, 0),
                    (75000, -15000),
                    (100000, -15000),
                    (100000, 0),
                ],
            ),
        ] {
            project.insert_drawing_route(DrawingRoute {
                id: RouteId::new(id).expect("constant"),
                view,
                conductor: ConductorId::new(id).expect("constant"),
                anchors: [
                    Some(AnchorId::new(id).expect("constant")),
                    Some(AnchorId::new(id + 1).expect("constant")),
                ],
                points: points
                    .into_iter()
                    .map(|(x, y)| DrawingPoint { x, y })
                    .collect(),
                mode: RouteMode::Orthogonal,
                appearance: Appearance {
                    style: None,
                    overrides: StylePatch {
                        stroke_width: Some(DrawingLength::new(500)?),
                        colour: Some(DrawingColour {
                            red: 43,
                            green: 108,
                            blue: 94,
                        }),
                        ..StylePatch::default()
                    },
                },
            })?;
        }
        project.insert_drawing_label(DrawingLabel {
            id: LabelId::new(1).expect("constant"),
            view,
            text: "CONVEYOR / SIGNAL HARNESS".into(),
            position: DrawingPoint { x: 0, y: -33000 },
            appearance: Appearance {
                style: None,
                overrides: StylePatch {
                    text_height: Some(DrawingLength::new(3500)?),
                    ..StylePatch::default()
                },
            },
        })?;
        project.insert_sheet(
            DrawingSheet {
                id: SheetId::new(1).expect("constant"),
                number: "DEMO".into(),
                title: "Conveyor · detailed wiring".into(),
                paper: SheetPaper::default(),
                placement: Some(SheetViewPlacement {
                    view,
                    paper_origin: DrawingPoint {
                        x: 20_000,
                        y: 60_000,
                    },
                    view_origin: DrawingPoint::default(),
                    numerator: 1,
                    denominator: 1,
                }),
            },
            0,
        )?;
        project.validate()
    })();
    result.map_err(|e| BridgeError::new("sample", e))?;
    Ok(project)
}

pub(crate) fn label(project: &Project, l: &DrawingLabel) -> SceneRow {
    let target = ObjectRef::Label(l.id);
    let (colour, width) = properties::stroke(project, target);
    let height = match project
        .property(target, PropertyId::TextHeight)
        .map(|v| v.value)
    {
        Ok(PropertyValue::Length(v)) => f64::from(v.micrometres()) / 1000.0,
        _ => 2.5,
    };
    let font = match project
        .property(target, PropertyId::FontFamily)
        .map(|v| v.value)
    {
        Ok(PropertyValue::Text(v)) => v,
        _ => "sans-serif".into(),
    };
    SceneRow {
        target: target.into(),
        view: l.view.to_string(),
        label: l.text.clone(),
        shape: "text".into(),
        points: vec![mm(l.position)],
        colour,
        width,
        text_height: height,
        font,
    }
}

pub(crate) fn polyline(project: &Project, p: &DrawingPolyline) -> SceneRow {
    let target = ObjectRef::Polyline(p.id);
    let (colour, width) = properties::stroke(project, target);
    SceneRow {
        target: target.into(),
        view: p.view.to_string(),
        label: "Graphical path".into(),
        shape: "path".into(),
        points: p.points.iter().copied().map(mm).collect(),
        colour,
        width,
        text_height: 0.0,
        font: String::new(),
    }
}
