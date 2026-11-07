//! Explicit revision review/adoption, external conflicts, and exact lifecycle history.

#[path = "adoption/identity_contract.rs"]
mod identity_contract;
#[path = "adoption/support.rs"]
mod support;
use ecad_adapters::{JsonFile, RandomIds, decode_project, encode_project};
use ecad_app::{Command, Session, adoption_plan};
use ecad_domain::*;
use serde_json::json;
use support::*;

#[test]
fn adoption_preserves_connections_and_other_instances_with_exact_save_undo_redo() {
    let target = revision(2, |j| {
        j["conductors"][0]["b"] = json!(null);
        j["devices"][0]["designation"] = json!("-Updated");
        j["devices"][0]["terminals"][0]["number"] = json!("A");
        j["ports"][0]["name"] = json!("Renamed interface");
    });
    let (mut session, a, b) = setup(target);
    let wire = external_wire(&mut session, &a);
    let cable = Cable::new(
        CableId::new(900).unwrap(),
        "External",
        CableEndpoint::Subsystem(a.root()),
        CableEndpoint::Subsystem(b.root()),
        [wire],
    )
    .unwrap();
    session
        .execute(session.revision(), Command::InsertCable(cable))
        .unwrap();
    let before = session.project().clone();
    let preview = plan(&session, &a).unwrap();
    assert!(
        preview
            .changes
            .terminals
            .modified
            .contains(&TerminalId::new(201).unwrap())
    );
    assert!(
        preview
            .changes
            .ports
            .modified
            .contains(&InterfacePortId::new(1).unwrap())
    );
    assert!(
        preview
            .changes
            .conductors
            .modified
            .contains(&ConductorId::new(2).unwrap())
    );
    assert_eq!(session.project(), &before); // Review is read-only.
    session
        .execute(session.revision(), preview.command)
        .unwrap();
    let adopted = session.project().clone();
    assert_eq!(adopted.instance(a.root()).unwrap().ids(), a.ids());
    assert_eq!(adopted.instance(a.root()).unwrap().source(), source(2));
    assert_eq!(adopted.instance(b.root()), Some(&b));
    assert_eq!(
        adopted.conductor(wire).unwrap().endpoint(End::A),
        Some(terminal(&a))
    );
    assert_eq!(adopted.subsystem(a.root()).unwrap().name(), "Left");
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("adopted.json"));
    session.save(&store).unwrap();
    assert_eq!(Session::open(&store).unwrap().project(), &adopted);
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
    assert!(session.is_dirty());
    session.redo(session.revision()).unwrap();
    assert_eq!(session.project(), &adopted);
    assert!(!session.is_dirty());
}

#[test]
fn removed_connected_terminal_is_never_matched_by_number_and_requires_explicit_disconnect() {
    let target = revision(2, |j| {
        let replacement = json!("00000000000000000000000000000191");
        j["devices"][0]["terminals"][0]["id"] = replacement.clone();
        j["ports"][0]["terminal"] = replacement.clone();
        j["conductors"][0]["a"] = replacement;
    });
    let (mut session, a, _) = setup(target);
    let wire = external_wire(&mut session, &a);
    let before = session.project().clone();
    let token = session.revision();
    assert!(matches!(
        plan(&session, &a),
        Err(ModelError::MissingTerminal(_))
    ));
    assert_eq!(session.project(), &before);
    assert_eq!(session.revision(), token);
    session
        .execute(
            token,
            Command::SetTermination {
                conductor: wire,
                end: End::A,
                terminal: None,
            },
        )
        .unwrap();
    let preview = plan(&session, &a).unwrap();
    assert!(
        preview
            .changes
            .terminals
            .removed
            .contains(&TerminalId::new(201).unwrap())
    );
    assert!(
        preview
            .changes
            .terminals
            .added
            .contains(&TerminalId::new(401).unwrap())
    );
    session
        .execute(session.revision(), preview.command)
        .unwrap();
    session.undo(session.revision()).unwrap();
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}

#[test]
fn removing_exposure_rejects_even_a_spare_external_wire() {
    let target = revision(2, |j| {
        j["ports"].as_array_mut().unwrap().remove(0);
    });
    let (mut session, a, _) = setup(target);
    external_wire(&mut session, &a);
    let before = session.project().clone();
    assert!(matches!(
        plan(&session, &a),
        Err(ModelError::Reuse(ReuseError::UnexposedInstanceTerminal(_)))
    ));
    assert_eq!(session.project(), &before);
}

#[test]
fn stale_or_detached_plans_reject_and_failed_batches_restore_adoption_and_redo() {
    let target = revision(2, |j| {
        j["conductors"][0]["b"] = json!(null);
    });
    let (mut session, a, _) = setup(target);
    let command = plan(&session, &a).unwrap().command;
    let before = session.project().clone();
    let token = session.revision();
    let history = session.undo_len();
    assert!(
        session
            .execute_batch(
                token,
                vec![
                    command.clone(),
                    Command::RemoveSubsystem(SubsystemId::new(999).unwrap())
                ]
            )
            .is_err()
    );
    assert_eq!(session.project(), &before);
    assert_eq!(session.revision(), token);
    assert_eq!(session.undo_len(), history);
    session.execute(token, command.clone()).unwrap();
    assert!(
        session
            .execute(session.revision(), command.clone())
            .is_err()
    );
    session.undo(session.revision()).unwrap();
    let redo = session.redo_len();
    assert!(
        session
            .execute_batch(
                session.revision(),
                vec![
                    command.clone(),
                    Command::RemoveSubsystem(SubsystemId::new(999).unwrap())
                ]
            )
            .is_err()
    );
    assert_eq!(session.redo_len(), redo);
    assert_eq!(session.project(), &before);
    session
        .execute(session.revision(), Command::DetachInstance(a.root()))
        .unwrap();
    let local = session.project().clone();
    assert!(session.execute(session.revision(), command).is_err());
    assert!(plan(&session, &a).is_err());
    assert_eq!(session.project(), &local);
}

#[test]
fn adoption_of_current_revision_is_noop_and_preserves_redo() {
    let (mut session, a, _) = setup(revision(2, |_| {}));
    session
        .execute(
            session.revision(),
            Command::InsertConductor(Conductor::new(ConductorId::new(900).unwrap(), None, None)),
        )
        .unwrap();
    session.undo(session.revision()).unwrap();
    let token = session.revision();
    let redo = session.redo_len();
    let ids = RandomIds::instance_ids(
        session.project().definition(source(1)).unwrap(),
        InstanceLimits::default(),
    )
    .unwrap();
    let preview = adoption_plan(
        session.project(),
        a.root(),
        source(1),
        ids,
        InstanceLimits::default(),
    )
    .unwrap();
    assert!(!session.execute(token, preview.command).unwrap());
    assert_eq!(session.revision(), token);
    assert_eq!(session.redo_len(), redo);
    assert_eq!(
        decode_project(&encode_project(session.project()).unwrap()).unwrap(),
        *session.project()
    );
}

#[test]
fn structural_additions_and_removals_get_fresh_ids_and_restore_exactly() {
    let target = revision(2, |j| {
        j["subsystems"].as_array_mut().unwrap().pop();
        j["devices"].as_array_mut().unwrap().pop();
        j["ports"].as_array_mut().unwrap().pop();
        j["conductors"] = json!([]);
        j["cables"] = json!([]);
        j["subsystems"].as_array_mut().unwrap().push(json!({
            "id": "0000000000000000000000000000001e", "name": "Replacement", "parent": "0000000000000000000000000000000a"
        }));
        j["devices"].as_array_mut().unwrap().push(json!({
            "id": "00000000000000000000000000000004", "designation": "-New", "subsystem": "0000000000000000000000000000001e",
            "terminals": [{"id": "00000000000000000000000000000191", "number": "1"}]
        }));
        j["ports"].as_array_mut().unwrap().push(json!({
            "id": "00000000000000000000000000000003", "name": "New exposure", "subsystem": "0000000000000000000000000000001e", "terminal": "00000000000000000000000000000191"
        }));
        j["conductors"] = json!([{"id": "00000000000000000000000000000003", "a": "000000000000000000000000000000c9", "b": "00000000000000000000000000000191"}]);
        j["cables"] = json!([{"id": "00000000000000000000000000000003", "designation": "New cable", "a": {"kind":"device", "id":"00000000000000000000000000000002"}, "b": {"kind":"subsystem", "id":"0000000000000000000000000000001e"}, "conductors": ["00000000000000000000000000000003"]}]);
    });
    let (mut session, a, _) = setup(target);
    external_wire(&mut session, &a);
    let before = session.project().clone();
    let preview = plan(&session, &a).unwrap();
    assert_eq!(
        preview.changes.subsystems.removed,
        [SubsystemId::new(20).unwrap()].into()
    );
    assert_eq!(
        preview.changes.subsystems.added,
        [SubsystemId::new(30).unwrap()].into()
    );
    assert_eq!(
        preview.changes.devices.removed,
        [DeviceId::new(3).unwrap()].into()
    );
    assert_eq!(
        preview.changes.devices.added,
        [DeviceId::new(4).unwrap()].into()
    );
    assert_eq!(
        preview.changes.terminals.added,
        [TerminalId::new(401).unwrap()].into()
    );
    assert_eq!(
        preview.changes.ports.added,
        [InterfacePortId::new(3).unwrap()].into()
    );
    assert_eq!(
        preview.changes.conductors.added,
        [ConductorId::new(3).unwrap()].into()
    );
    assert_eq!(
        preview.changes.cables.added,
        [CableId::new(3).unwrap()].into()
    );
    session
        .execute(session.revision(), preview.command)
        .unwrap();
    let next = session.project().instance(a.root()).unwrap();
    assert!(
        !a.ids()
            .devices
            .values()
            .any(|id| next.ids().devices[&DeviceId::new(4).unwrap()] == *id)
    );
    assert!(
        session
            .project()
            .device(a.ids().devices[&DeviceId::new(3).unwrap()])
            .is_none()
    );
    session.project().validate().unwrap();
    let after = session.project().clone();
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
    session.redo(session.revision()).unwrap();
    assert_eq!(session.project(), &after);
}

#[test]
fn two_projects_adopt_independently_and_reopen_offline_at_their_chosen_revision() {
    let target = revision(2, |j| {
        j["conductors"][0]["b"] = json!(null);
    });
    let (mut first, a, _) = setup(std::sync::Arc::clone(&target));
    let mut project = Project::new(ProjectId::new(88).unwrap());
    project.insert_definition(revision(1, |_| {})).unwrap();
    project.insert_definition(target).unwrap();
    let mut second = Session::new(project);
    let c = place(&mut second, "Independent module");
    assert_ne!(first.project().id(), second.project().id());
    assert_ne!(a.ids(), c.ids());
    let baseline = second.project().clone();
    let preview = plan(&first, &a).unwrap();
    first.execute(first.revision(), preview.command).unwrap();
    assert_eq!(second.project(), &baseline);
    assert_eq!(
        second.project().instance(c.root()).unwrap().source(),
        source(1)
    );
    let directory = tempfile::tempdir().unwrap();
    let store_a = JsonFile::new(directory.path().join("a.json"));
    let store_b = JsonFile::new(directory.path().join("b.json"));
    first.save(&store_a).unwrap();
    second.save(&store_b).unwrap();
    assert_eq!(
        Session::open(&store_a)
            .unwrap()
            .project()
            .instance(a.root())
            .unwrap()
            .source(),
        source(2)
    );
    assert_eq!(Session::open(&store_b).unwrap().project(), &baseline);
    let preview = plan(&second, &c).unwrap();
    second.execute(second.revision(), preview.command).unwrap();
    assert_eq!(
        second.project().instance(c.root()).unwrap().source(),
        source(2)
    );
    assert_eq!(second.project().instance(c.root()).unwrap().ids(), c.ids());
    second.undo(second.revision()).unwrap();
    assert_eq!(second.project(), &baseline);
}

#[test]
fn parent_parameters_and_outer_interfaces_survive_only_compatible_adoption() {
    let target = revision(2, |j| {
        j["ports"].as_array_mut().unwrap().remove(0);
    });
    let (mut session, a, _) = setup(target);
    let parent = SubsystemId::new(800).unwrap();
    session
        .execute_batch(
            session.revision(),
            vec![
                Command::InsertSubsystem(Subsystem::new(parent, "Outer", None).unwrap()),
                Command::SetSubsystemParent {
                    subsystem: a.root(),
                    parent: Some(parent),
                },
            ],
        )
        .unwrap();
    // Reparenting is an occurrence parameter and is retained by adoption/undo.
    let before = session.project().clone();
    let preview = plan(&session, &a).unwrap();
    session
        .execute(session.revision(), preview.command)
        .unwrap();
    assert_eq!(
        session.project().subsystem(a.root()).unwrap().parent(),
        Some(parent)
    );
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
    session
        .execute(
            session.revision(),
            Command::InsertPort(
                InterfacePort::new(
                    InterfacePortId::new(800).unwrap(),
                    parent,
                    "Outer input",
                    terminal(&a),
                )
                .unwrap(),
            ),
        )
        .unwrap();
    let before = session.project().clone();
    assert!(matches!(
        plan(&session, &a),
        Err(ModelError::InterfaceRequired(_, _))
    ));
    assert_eq!(session.project(), &before);
}

#[test]
fn commit_rechecks_external_connections_added_after_a_successful_preview() {
    let (mut session, a, _) = setup(revision(2, |j| {
        j["ports"].as_array_mut().unwrap().remove(0);
    }));
    let preview = plan(&session, &a).unwrap();
    external_wire(&mut session, &a);
    let before = session.project().clone();
    let token = session.revision();
    let history = session.undo_len();
    assert!(session.execute(token, preview.command).is_err());
    assert_eq!(session.project(), &before);
    assert_eq!(session.revision(), token);
    assert_eq!(session.undo_len(), history);
}

#[test]
fn selected_occurrences_adopt_as_one_undoable_batch() {
    let (mut session, a, b) = setup(revision(2, |j| {
        j["conductors"][0]["b"] = json!(null);
    }));
    let before = session.project().clone();
    let left = plan(&session, &a).unwrap().command;
    let right = plan(&session, &b).unwrap().command;
    let history = session.undo_len();
    session
        .execute_batch(session.revision(), vec![left, right])
        .unwrap();
    assert_eq!(session.undo_len(), history + 1);
    assert!(
        session
            .project()
            .instances()
            .all(|pin| pin.source() == source(2))
    );
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
    session.redo(session.revision()).unwrap();
    assert!(
        session
            .project()
            .instances()
            .all(|pin| pin.source() == source(2))
    );
}
