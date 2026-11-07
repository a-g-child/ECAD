//! Stable named styles and real drawing objects for defaults/style contracts.

use ecad_adapters::decode_project;
use ecad_app::{Command, DrawingCommand, Session};
use ecad_domain::*;

pub fn length(value: u32) -> DrawingLength {
    DrawingLength::new(value).unwrap()
}
pub fn view() -> ViewId {
    ViewId::new(1).unwrap()
}
pub fn text_style() -> StyleId {
    StyleId::new(1).unwrap()
}
pub fn stroke_style() -> StyleId {
    StyleId::new(2).unwrap()
}
pub fn label(value: u128) -> ObjectRef {
    ObjectRef::Label(LabelId::new(value).unwrap())
}
pub fn path() -> ObjectRef {
    ObjectRef::Polyline(PolylineId::new(1).unwrap())
}
pub fn points() -> Vec<DrawingPoint> {
    vec![
        DrawingPoint::default(),
        DrawingPoint { x: 10_000, y: 0 },
        DrawingPoint {
            x: 10_000,
            y: 5_000,
        },
    ]
}
pub fn run(session: &mut Session, command: DrawingCommand) {
    session
        .execute(session.revision(), Command::Drawing(command))
        .unwrap();
}
pub fn set(session: &mut Session, target: ObjectRef, property: PropertyId, value: PropertyValue) {
    session
        .execute(
            session.revision(),
            Command::SetProperty {
                target,
                property,
                value,
            },
        )
        .unwrap();
}
pub fn setup() -> Session {
    let mut project =
        decode_project(include_str!("../fixtures/controller-conveyor-v5.json")).unwrap();
    project
        .insert_drawing_view(DrawingView {
            id: view(),
            name: "Overview".into(),
            overrides: StylePatch::default(),
        })
        .unwrap();
    project
        .insert_drawing_style(
            DrawingStyle::new(
                text_style(),
                "Device labels",
                DrawingRole::Text,
                StylePatch {
                    font: Some(FontFamily::new("Named family").unwrap()),
                    text_height: Some(length(3_000)),
                    ..StylePatch::default()
                },
            )
            .unwrap(),
        )
        .unwrap();
    project
        .insert_drawing_style(
            DrawingStyle::new(
                stroke_style(),
                "Graphical strokes",
                DrawingRole::Stroke,
                StylePatch {
                    stroke_width: Some(length(400)),
                    ..StylePatch::default()
                },
            )
            .unwrap(),
        )
        .unwrap();
    let mut settings = project.drawing().settings().clone();
    settings.text_creation.style = Some(text_style());
    settings.stroke_creation.style = Some(stroke_style());
    project.set_drawing_settings(settings).unwrap();
    let mut session = Session::new(project);
    for id in [1, 2] {
        run(
            &mut session,
            DrawingCommand::CreateLabel {
                id: LabelId::new(id).unwrap(),
                view: view(),
                text: format!("Label {id}"),
                position: DrawingPoint {
                    x: -1_000,
                    y: 2_000,
                },
            },
        );
    }
    run(
        &mut session,
        DrawingCommand::CreatePolyline {
            id: PolylineId::new(1).unwrap(),
            view: view(),
            points: points(),
        },
    );
    session
}
