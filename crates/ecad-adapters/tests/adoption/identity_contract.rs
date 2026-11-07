//! Adoption identity compatibility, resource limits, and collision contracts.

use super::*;

#[test]
fn surviving_terminal_cannot_silently_move_to_a_different_device() {
    let target = revision(2, |j| {
        j["devices"][0]["terminals"] = json!([]);
        j["devices"][1]["terminals"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":"000000000000000000000000000000c9", "number":"Extra"}));
        // Expose the moved anchor through the child so the body itself is valid.
        j["ports"].as_array_mut().unwrap().push(json!({"id":"00000000000000000000000000000003", "name":"Extra", "subsystem":"00000000000000000000000000000014", "terminal":"000000000000000000000000000000c9"}));
        j["cables"] = json!([]);
    });
    let (session, a, _) = setup(target);
    assert!(matches!(
        plan(&session, &a),
        Err(ModelError::Reuse(ReuseError::IncompatibleAdoption))
    ));
}

#[test]
fn changing_the_definition_root_is_an_explicit_incompatibility() {
    let target = revision(2, |_| {});
    let (session, a, _) = setup(target);
    let mut project = session.project().clone();
    let mut body = Project::new(ProjectId::new(1).unwrap());
    let new_root = SubsystemId::new(30).unwrap();
    body.insert_subsystem(Subsystem::new(new_root, "New root", None).unwrap())
        .unwrap();
    let source = DefinitionRef::new(
        source(1).definition(),
        DefinitionRevisionId::new(3).unwrap(),
    );
    let revision = std::sync::Arc::new(SubsystemRevision::new(source, new_root, body).unwrap());
    let ids = RandomIds::instance_ids(&revision, InstanceLimits::default()).unwrap();
    project.insert_definition(revision).unwrap();
    assert!(matches!(
        adoption_plan(&project, a.root(), source, ids, InstanceLimits::default()),
        Err(ModelError::Reuse(ReuseError::IncompatibleAdoption))
    ));
}

#[test]
fn forged_mapping_cannot_reassign_existing_ids_or_recycle_removed_ids() {
    let (session, a, _) = setup(revision(2, |_| {}));
    let mut project = session.project().clone();
    let before = project.clone();
    let mut ids = a.ids().clone();
    ids.terminals
        .insert(TerminalId::new(201).unwrap(), TerminalId::new(900).unwrap());
    let target = InstanceBinding::new(a.root(), source(2), ids);
    assert_eq!(
        project.adopt_instance(&a, &target),
        Err(ReuseError::IncompatibleAdoption.into())
    );
    assert_eq!(project, before);
    let target = revision(2, |j| {
        j["devices"][0]["terminals"][0]["id"] = json!("00000000000000000000000000000191");
        j["ports"][0]["terminal"] = json!("00000000000000000000000000000191");
        j["conductors"][0]["a"] = json!("00000000000000000000000000000191");
    });
    let (session, a, _) = setup(target);
    let mut ids = a.ids().clone();
    let old = ids
        .terminals
        .remove(&TerminalId::new(201).unwrap())
        .unwrap();
    ids.terminals.insert(TerminalId::new(401).unwrap(), old);
    assert!(matches!(
        adoption_plan(
            session.project(),
            a.root(),
            source(2),
            ids,
            InstanceLimits::default()
        ),
        Err(ModelError::Reuse(ReuseError::IncompatibleAdoption))
    ));
}

#[test]
fn planning_rejects_foreign_lineages_missing_maps_limits_and_new_object_collisions() {
    let target = revision(2, |j| {
        j["devices"][0]["terminals"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":"00000000000000000000000000000191", "number":"Extra"}));
    });
    let (session, a, b) = setup(target);
    let make_ids = || {
        RandomIds::instance_ids(
            session.project().definition(source(2)).unwrap(),
            InstanceLimits::default(),
        )
        .unwrap()
    };
    let mut ids = make_ids();
    ids.terminals
        .insert(TerminalId::new(401).unwrap(), terminal(&b));
    assert!(matches!(
        adoption_plan(
            session.project(),
            a.root(),
            source(2),
            ids,
            InstanceLimits::default()
        ),
        Err(ModelError::DuplicateTerminal(_))
    ));
    assert!(
        adoption_plan(
            session.project(),
            a.root(),
            source(2),
            InstanceIds::default(),
            InstanceLimits::default()
        )
        .is_err()
    );
    let limits = InstanceLimits {
        max_objects: std::num::NonZeroUsize::new(1).unwrap(),
    };
    assert!(matches!(
        adoption_plan(session.project(), a.root(), source(2), make_ids(), limits),
        Err(ModelError::Reuse(ReuseError::ExpansionLimit))
    ));
    let mut project = session.project().clone();
    let original = project.definition(source(2)).unwrap();
    let foreign = SubsystemRevision::new(
        DefinitionRef::new(DefinitionId::new(2).unwrap(), source(2).revision()),
        original.root(),
        original.content().clone(),
    )
    .unwrap();
    let reference = foreign.source();
    project
        .insert_definition(std::sync::Arc::new(foreign))
        .unwrap();
    assert!(matches!(
        adoption_plan(
            &project,
            a.root(),
            reference,
            make_ids(),
            InstanceLimits::default()
        ),
        Err(ModelError::Reuse(ReuseError::IncompatibleAdoption))
    ));
}
