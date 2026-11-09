//! Exact transient previews, snap feedback, guarded commit, and coherent document history.
use ecad_adapters::{JsonFile, decode_project, encode_project};
use ecad_app::*;
use ecad_domain::*;
use std::collections::BTreeSet;

#[path = "placement/actions.rs"]
mod actions;
fn point(x: i64, y: i64) -> DrawingPoint {
    DrawingPoint { x, y }
}
fn anchor(id: u128) -> AnchorId {
    AnchorId::new(id).unwrap()
}
fn route(id: u128) -> RouteId {
    RouteId::new(id).unwrap()
}
fn label() -> LabelId {
    LabelId::new(1).unwrap()
}
fn line() -> PolylineId {
    PolylineId::new(1).unwrap()
}
fn view() -> ViewId {
    ViewId::new(1).unwrap()
}
fn setup() -> Project {
    let mut project = decode_project(include_str!("fixtures/routed-conveyor-v6.json")).unwrap();
    project
        .insert_drawing_label(DrawingLabel {
            id: label(),
            view: view(),
            text: "Preview me".into(),
            position: point(0, 1000),
            appearance: Appearance::default(),
        })
        .unwrap();
    project
        .insert_drawing_polyline(DrawingPolyline {
            id: line(),
            view: view(),
            points: vec![point(9000, 0), point(11000, 0)],
            appearance: Appearance::default(),
        })
        .unwrap();
    project
        .insert_terminal_anchor(TerminalAnchor {
            id: anchor(9),
            view: view(),
            terminal: TerminalId::new(301).unwrap(),
            position: point(40000, 5000),
        })
        .unwrap();
    project
}
fn group() -> BTreeSet<ObjectRef> {
    [
        ObjectRef::Anchor(anchor(1)),
        ObjectRef::Anchor(anchor(2)),
        ObjectRef::Anchor(anchor(3)),
        ObjectRef::Label(label()),
        ObjectRef::Polyline(line()),
    ]
    .into_iter()
    .collect()
}
fn pointer(position: DrawingPoint, choice: SnapChoice) -> MovePlacement {
    MovePlacement::Pointer {
        position,
        choice,
        settings: SnapSettings {
            tolerance_um: 200,
            terminals: true,
            endpoints: true,
            midpoints: true,
            grid: Some(DrawingGrid {
                origin: point(0, 0),
                spacing: DrawingLength::new(1000).unwrap(),
            }),
        },
    }
}
fn assert_projection(project: &Project, preview: &LayoutPreview) {
    for object in preview.objects() {
        match object {
            PreviewObject::Anchor(a) => {
                assert_eq!(project.drawing().terminal_anchor(a.id), Some(a))
            }
            PreviewObject::Route(r) => assert_eq!(project.drawing().route(r.id), Some(r)),
            PreviewObject::Picture(p) => assert_eq!(project.drawing().picture(p.id), Some(p)),
            PreviewObject::Label(l) => assert_eq!(project.drawing().label(l.id), Some(l)),
            PreviewObject::Polyline(l) => assert_eq!(project.drawing().polyline(l.id), Some(l)),
        }
    }
}

#[test]
fn snapped_group_preview_is_non_mutating_and_commit_matches_every_projected_record_exactly() {
    let before = setup();
    let mut session = Session::new(before.clone());
    let revision = session.revision();
    let preview = preview_move(
        &session,
        group(),
        ObjectRef::Anchor(anchor(1)),
        pointer(point(39900, 4990), SnapChoice::Best),
    )
    .unwrap();
    assert_eq!(
        preview.chosen().unwrap().target,
        SnapTarget::Terminal(anchor(9))
    );
    assert_eq!(preview.destination(), point(40000, 5000));
    assert_eq!(preview.layout().selection(), &group());
    assert_eq!(preview.layout().objects().len(), 7);
    assert_eq!(
        preview.layout().affected_routes(),
        &BTreeSet::from([route(1), route(2)])
    );
    assert_eq!(session.revision(), revision);
    assert_eq!(session.undo_len(), 0);
    assert_eq!(session.project(), &before);
    let projection = preview.layout().clone();
    session.apply_layout_preview(preview.into_layout()).unwrap();
    assert_projection(session.project(), &projection);
    assert_eq!(session.undo_len(), 1);
    assert_eq!(
        session
            .project()
            .conductor(ConductorId::new(1).unwrap())
            .unwrap()
            .endpoint(End::A),
        Some(TerminalId::new(101).unwrap())
    );
    let after = session.project().clone();
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
    session.redo(session.revision()).unwrap();
    assert_eq!(session.project(), &after);
}

#[test]
fn exact_and_suppressed_input_bypass_snapping_and_unavailable_targets_leave_pointer_exact() {
    let mut session = Session::new(setup());
    let selection = [ObjectRef::Label(label())];
    let target = point(39900, 4990);
    for placement in [
        MovePlacement::Exact(target),
        pointer(target, SnapChoice::Suppressed),
        pointer(point(12345, 6789), SnapChoice::Best),
    ] {
        let expected = match placement {
            MovePlacement::Exact(p) | MovePlacement::Pointer { position: p, .. } => p,
        };
        let preview = preview_move(&session, selection, selection[0], placement).unwrap();
        assert_eq!(preview.destination(), expected);
        assert!(preview.chosen().is_none());
        assert!(preview.candidates().is_empty());
        session.apply_layout_preview(preview.into_layout()).unwrap();
        assert_eq!(
            session.project().drawing().label(label()).unwrap().position,
            expected
        );
        session.undo(session.revision()).unwrap();
    }
}

#[test]
fn moving_selection_and_touching_routes_are_excluded_from_snap_targets_including_midpoints() {
    let session = Session::new(setup());
    let preview = preview_move(
        &session,
        group(),
        ObjectRef::Anchor(anchor(1)),
        pointer(point(100, 100), SnapChoice::Best),
    )
    .unwrap();
    assert!(
        preview
            .candidates()
            .iter()
            .all(|candidate| candidate.target == SnapTarget::Grid)
    );
    assert_eq!(preview.destination(), point(0, 0));
    assert!(preview.layout().changes().is_empty());
    assert_eq!(preview.layout().unchanged(), &group());
}

#[test]
fn candidate_cycling_is_explicit_and_invalid_winners_never_silently_fall_back() {
    let session = Session::new(setup());
    let selection = [ObjectRef::Anchor(anchor(1))];
    let placement = |choice| MovePlacement::Pointer {
        position: point(10000, 0),
        choice,
        settings: SnapSettings {
            tolerance_um: 1500,
            terminals: true,
            endpoints: true,
            ..SnapSettings::default()
        },
    };
    assert!(matches!(
        preview_move(
            &session,
            selection,
            selection[0],
            placement(SnapChoice::Best)
        ),
        Err(MovePreviewError::Layout(LayoutError::Model(
            ModelError::Route(_)
        )))
    ));
    // Terminal 2 and route 2's first endpoint rank before graphical endpoints.
    let preview = preview_move(
        &session,
        selection,
        selection[0],
        placement(SnapChoice::Candidate(2)),
    )
    .unwrap();
    assert_eq!(
        preview.chosen().unwrap().target,
        SnapTarget::PolylineEndpoint {
            polyline: line(),
            end: SnapEnd::First
        }
    );
    assert_eq!(preview.destination(), point(9000, 0));
    assert!(matches!(
        preview_move(
            &session,
            selection,
            selection[0],
            placement(SnapChoice::Candidate(999))
        ),
        Err(MovePreviewError::CandidateOutOfRange { index: 999, .. })
    ));
    assert_eq!(session.undo_len(), 0);
}

#[test]
fn missing_reference_bad_selection_and_large_translation_offsets_are_typed_failures() {
    let session = Session::new(setup());
    assert!(matches!(
        preview_move(
            &session,
            [ObjectRef::Label(label())],
            ObjectRef::Anchor(anchor(1)),
            MovePlacement::Exact(point(0, 0))
        ),
        Err(MovePreviewError::ReferenceNotSelected(_))
    ));
    assert!(matches!(
        preview_move(
            &session,
            [ObjectRef::Route(route(1))],
            ObjectRef::Route(route(1)),
            MovePlacement::Exact(point(0, 0))
        ),
        Err(MovePreviewError::Layout(LayoutError::UnsupportedTarget(_)))
    ));
    let mut project = setup();
    let mut item = project.drawing().label(label()).unwrap().clone();
    item.position.x = i64::MIN;
    project.replace_drawing_label(item).unwrap();
    let session = Session::new(project);
    assert!(matches!(
        preview_move(
            &session,
            [ObjectRef::Label(label())],
            ObjectRef::Label(label()),
            MovePlacement::Exact(point(i64::MAX, 0))
        ),
        Err(MovePreviewError::Layout(LayoutError::Model(
            ModelError::Drawing(DrawingError::CoordinateOverflow)
        )))
    ));
}

#[test]
fn stale_different_session_and_noop_previews_cannot_commit_against_a_new_revision() {
    let mut session = Session::new(setup());
    let selection = [ObjectRef::Label(label())];
    let preview =
        preview_layout(&session, selection, LayoutOperation::Translate(point(0, 0))).unwrap();
    let mut other = Session::new(session.project().clone());
    assert!(matches!(
        other.apply_layout_preview(preview.clone()),
        Err(EditError::StaleRevision { .. })
    ));
    session
        .execute(
            session.revision(),
            Command::Drawing(DrawingCommand::ReplaceLabel(DrawingLabel {
                text: "Changed".into(),
                ..session.project().drawing().label(label()).unwrap().clone()
            })),
        )
        .unwrap();
    let before = session.project().clone();
    let revision = session.revision();
    assert!(matches!(
        session.apply_layout_preview(preview),
        Err(EditError::StaleRevision { .. })
    ));
    assert_eq!(session.project(), &before);
    assert_eq!(session.revision(), revision);
}

#[test]
fn dropping_preview_cancels_and_fresh_noops_preserve_redo_checkpoint_and_revision() {
    let directory = tempfile::tempdir().unwrap();
    let mut document = Document::new(
        setup(),
        JsonFile::new(directory.path().join("preview.json")),
    );
    document.save().unwrap();
    let before = document.session().project().clone();
    let revision = document.session().revision();
    let preview = preview_move(
        document.session(),
        group(),
        ObjectRef::Anchor(anchor(1)),
        pointer(point(39900, 4990), SnapChoice::Best),
    )
    .unwrap();
    drop(preview);
    assert_eq!(document.session().project(), &before);
    assert_eq!(document.session().revision(), revision);
    assert!(!document.session().is_dirty());
    let preview = preview_layout(
        document.session(),
        group(),
        LayoutOperation::Translate(point(10, 20)),
    )
    .unwrap();
    document.apply_layout_preview(preview).unwrap();
    document.undo(document.session().revision()).unwrap();
    let revision = document.session().revision();
    let redo = document.session().redo_len();
    let preview = preview_layout(
        document.session(),
        group(),
        LayoutOperation::Translate(point(0, 0)),
    )
    .unwrap();
    assert!(!document.apply_layout_preview(preview).unwrap());
    assert_eq!(document.session().revision(), revision);
    assert_eq!(document.session().redo_len(), redo);
    assert!(!document.session().is_dirty());
}

#[test]
fn saved_snapped_move_reopens_as_exact_v6_geometry_with_no_persisted_preview_state() {
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("preview.json"));
    let mut document = Document::new(setup(), store.clone());
    document.save().unwrap();
    let preview = preview_move(
        document.session(),
        group(),
        ObjectRef::Anchor(anchor(1)),
        pointer(point(39900, 4990), SnapChoice::Best),
    )
    .unwrap();
    let projection = preview.layout().clone();
    document
        .apply_layout_preview(preview.into_layout())
        .unwrap();
    document.save().unwrap();
    let reopened = Document::open(store).unwrap();
    assert_projection(reopened.session().project(), &projection);
    let text = encode_project(reopened.session().project()).unwrap();
    assert!(!text.contains("preview"));
    assert!(!text.contains("snap"));
    document.undo(document.session().revision()).unwrap();
    assert!(document.session().is_dirty());
    document.redo(document.session().revision()).unwrap();
    assert!(!document.session().is_dirty());
    assert_eq!(document.session().project(), reopened.session().project());
}

#[test]
fn alignment_and_distribution_previews_share_validated_geometry_with_their_actual_commits() {
    for operation in [
        LayoutOperation::Align {
            axis: LayoutAxis::Y,
            coordinate_um: 17,
        },
        LayoutOperation::Distribute(LayoutAxis::X),
    ] {
        let mut session = Session::new(setup());
        let before = session.project().clone();
        let preview = preview_layout(&session, group(), operation).unwrap();
        assert_eq!(preview.operation(), operation);
        assert_eq!(session.project(), &before);
        session.apply_layout_preview(preview.clone()).unwrap();
        assert_projection(session.project(), &preview);
        session.undo(session.revision()).unwrap();
        assert_eq!(session.project(), &before);
    }
}
