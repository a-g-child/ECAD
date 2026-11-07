//! Connected conductors sharing a placed terminal, with independent view geometry.

use ecad_adapters::decode_project;
use ecad_app::{Command, RouteCommand, Session};
use ecad_domain::*;

pub fn point(x: i64, y: i64) -> DrawingPoint {
    DrawingPoint { x, y }
}
pub fn view() -> ViewId {
    ViewId::new(1).unwrap()
}
pub fn anchor(id: u128) -> AnchorId {
    AnchorId::new(id).unwrap()
}
pub fn route(id: u128) -> RouteId {
    RouteId::new(id).unwrap()
}
pub fn setup() -> Session {
    let mut project =
        decode_project(include_str!("../fixtures/controller-conveyor-v6.json")).unwrap();
    project
        .insert_drawing_view(DrawingView {
            id: view(),
            name: "Detailed wiring".into(),
            overrides: StylePatch::default(),
        })
        .unwrap();
    for (id, terminal, position) in [
        (1, 101, point(0, 0)),
        (2, 201, point(10_000, 0)),
        (3, 301, point(20_000, 0)),
    ] {
        project
            .insert_terminal_anchor(TerminalAnchor {
                id: anchor(id),
                view: view(),
                terminal: TerminalId::new(terminal).unwrap(),
                position,
            })
            .unwrap();
    }
    for id in [1, 2] {
        project
            .insert_drawing_route(DrawingRoute {
                id: route(id),
                view: view(),
                conductor: ConductorId::new(id).unwrap(),
                anchors: [Some(anchor(id)), Some(anchor(id + 1))],
                points: vec![
                    point(i64::try_from(id - 1).unwrap() * 10_000, 0),
                    point(i64::try_from(id).unwrap() * 10_000, 0),
                ],
                mode: RouteMode::Orthogonal,
                appearance: Appearance::default(),
            })
            .unwrap();
    }
    Session::new(project)
}
pub fn run(session: &mut Session, command: RouteCommand) {
    session
        .execute(session.revision(), Command::Route(command))
        .unwrap();
}
