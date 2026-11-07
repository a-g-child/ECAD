//! Applicability, type, ownership, uniqueness, and failure rollback contracts.

use super::*;

#[test]
fn missing_selection_wrong_types_and_readonly_fields_are_never_silently_skipped() {
    let mut project = fixture();
    let before = project.clone();
    let missing = ObjectRef::Device(DeviceId::new(999).unwrap());
    assert!(matches!(
        inspect_properties(&project, [missing]),
        Err(ModelError::MissingDevice(_))
    ));
    assert!(matches!(
        property_edit_plan(
            &project,
            [missing],
            BTreeMap::from([(PropertyId::Name, PropertyValue::Text("unused".into()))])
        ),
        Err(ModelError::MissingDevice(_))
    ));
    let target = ObjectRef::Device(DeviceId::new(1).unwrap());
    assert!(matches!(
        project.set_property(
            target,
            PropertyId::Designation,
            PropertyValue::OptionalObject(None)
        ),
        Err(ModelError::Property(PropertyError::WrongType { .. }))
    ));
    assert!(matches!(
        project.set_property(
            target,
            PropertyId::Identity,
            PropertyValue::Object(ObjectRef::Device(DeviceId::new(2).unwrap()))
        ),
        Err(ModelError::Property(PropertyError::ReadOnly { .. }))
    ));
    let wire = ObjectRef::Conductor(ConductorId::new(1).unwrap());
    assert!(matches!(
        project.set_property(
            wire,
            PropertyId::ConductorA,
            PropertyValue::OptionalObject(Some(target))
        ),
        Err(ModelError::Property(PropertyError::WrongType { .. }))
    ));
    assert_eq!(project, before);
}

#[test]
fn uniqueness_and_blank_label_failures_leave_the_source_untouched_during_preflight() {
    let mut project = fixture();
    let root = SubsystemId::new(10).unwrap();
    project
        .insert_port(
            InterfacePort::new(
                InterfacePortId::new(3).unwrap(),
                root,
                "Other input",
                TerminalId::new(201).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let before = project.clone();
    let ports = [1, 3].map(|id| ObjectRef::Port(InterfacePortId::new(id).unwrap()));
    assert!(matches!(
        property_edit_plan(
            &project,
            ports,
            BTreeMap::from([(PropertyId::Name, PropertyValue::Text("Same".into()))])
        ),
        Err(ModelError::PortNameInUse(_, _))
    ));
    assert_eq!(project, before);
    assert!(matches!(
        property_edit_plan(
            &project,
            selection(),
            BTreeMap::from([(PropertyId::Designation, PropertyValue::Text("  ".into()))])
        ),
        Err(ModelError::EmptyLabel)
    ));
    let device = Device::new(
        DeviceId::new(9).unwrap(),
        "-X9",
        [
            Terminal::new(TerminalId::new(901).unwrap(), "1").unwrap(),
            Terminal::new(TerminalId::new(902).unwrap(), "2").unwrap(),
        ],
    )
    .unwrap();
    project.insert_device(device).unwrap();
    let before = project.clone();
    assert!(matches!(
        project.set_property(
            ObjectRef::Terminal(TerminalId::new(901).unwrap()),
            PropertyId::TerminalNumber,
            PropertyValue::Text("2".into())
        ),
        Err(ModelError::DuplicateTerminalNumber(_))
    ));
    assert_eq!(project, before);
}

#[test]
fn pinned_fields_report_restrictions_while_root_occurrence_parameters_stay_editable() {
    let mut session = Session::new(fixture());
    let binding = pinned(&mut session);
    let ids = binding.ids();
    let targets = [
        (
            ObjectRef::Device(ids.devices[&DeviceId::new(2).unwrap()]),
            PropertyId::Designation,
        ),
        (
            ObjectRef::Terminal(ids.terminals[&TerminalId::new(201).unwrap()]),
            PropertyId::TerminalNumber,
        ),
        (
            ObjectRef::Port(ids.ports[&InterfacePortId::new(1).unwrap()]),
            PropertyId::Name,
        ),
        (
            ObjectRef::Subsystem(ids.subsystems[&SubsystemId::new(20).unwrap()]),
            PropertyId::Name,
        ),
        (
            ObjectRef::Conductor(ids.conductors[&ConductorId::new(2).unwrap()]),
            PropertyId::ConductorB,
        ),
        (
            ObjectRef::Cable(ids.cables[&CableId::new(2).unwrap()]),
            PropertyId::Designation,
        ),
    ];
    for (target, property) in targets {
        assert_eq!(
            session.project().property(target, property).unwrap().access,
            PropertyAccess::ReadOnly(PropertyRestriction::PinnedInstance(binding.root()))
        );
    }
    let root = ObjectRef::Subsystem(binding.root());
    for property in [PropertyId::Name, PropertyId::SubsystemParent] {
        assert_eq!(
            session.project().property(root, property).unwrap().access,
            PropertyAccess::Editable
        );
    }
    let before = session.project().clone();
    let local = ObjectRef::Device(DeviceId::new(1).unwrap());
    assert!(matches!(
        property_edit_plan(
            session.project(),
            [local, targets[0].0],
            BTreeMap::from([(PropertyId::Designation, PropertyValue::Text("-New".into()))])
        ),
        Err(ModelError::Property(PropertyError::ReadOnly { .. }))
    ));
    assert_eq!(session.project(), &before);
    session
        .execute(
            session.revision(),
            Command::SetProperty {
                target: root,
                property: PropertyId::Name,
                value: PropertyValue::Text("Renamed occurrence".into()),
            },
        )
        .unwrap();
    session.project().validate().unwrap();
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}

#[test]
fn make_local_and_compounded_properties_undo_to_the_exact_original_pin() {
    let mut session = Session::new(fixture());
    let binding = pinned(&mut session);
    let before = session.project().clone();
    let mut local = before.clone();
    local.detach_instance(binding.root()).unwrap();
    let target = ObjectRef::Device(binding.ids().devices[&DeviceId::new(2).unwrap()]);
    let plan = property_edit_plan(
        &local,
        [target],
        BTreeMap::from([(
            PropertyId::Designation,
            PropertyValue::Text("-Local".into()),
        )]),
    )
    .unwrap();
    let mut commands = vec![Command::DetachInstance(binding.root())];
    commands.extend(plan.commands);
    session.execute_batch(session.revision(), commands).unwrap();
    assert!(session.project().local_origin(binding.root()).is_some());
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
    session.redo(session.revision()).unwrap();
    assert!(session.project().local_origin(binding.root()).is_some());
}

#[test]
fn late_failure_rolls_back_properties_and_preserves_redo_and_revision() {
    let mut session = Session::new(fixture());
    session
        .execute(
            session.revision(),
            Command::InsertConductor(Conductor::new(ConductorId::new(9).unwrap(), None, None)),
        )
        .unwrap();
    session.undo(session.revision()).unwrap();
    let before = session.project().clone();
    let revision = session.revision();
    let redo = session.redo_len();
    let mut plan = property_edit_plan(
        &before,
        selection(),
        BTreeMap::from([(
            PropertyId::Designation,
            PropertyValue::Text("-Change".into()),
        )]),
    )
    .unwrap();
    plan.commands
        .push(Command::RemoveDevice(DeviceId::new(999).unwrap()));
    assert!(session.execute_batch(revision, plan.commands).is_err());
    assert_eq!(session.project(), &before);
    assert_eq!(session.revision(), revision);
    assert_eq!(session.redo_len(), redo);
}

#[test]
fn stale_plans_recheck_revision_and_missing_targets_at_commit_instead_of_silently_omitting_them() {
    let mut session = Session::new(fixture());
    let extra = DeviceId::new(9).unwrap();
    session
        .execute(
            session.revision(),
            Command::InsertDevice(Device::new(extra, "-Extra", []).unwrap()),
        )
        .unwrap();
    let plan = property_edit_plan(
        session.project(),
        [
            ObjectRef::Device(DeviceId::new(1).unwrap()),
            ObjectRef::Device(extra),
        ],
        BTreeMap::from([(PropertyId::Designation, PropertyValue::Text("-Bulk".into()))]),
    )
    .unwrap();
    let observed = session.revision();
    session
        .execute(observed, Command::RemoveDevice(extra))
        .unwrap();
    let before = session.project().clone();
    assert!(matches!(
        session.execute_batch(observed, plan.commands.clone()),
        Err(ecad_app::EditError::StaleRevision { .. })
    ));
    assert!(matches!(
        session.execute_batch(session.revision(), plan.commands),
        Err(ecad_app::EditError::Model(ModelError::MissingDevice(_)))
    ));
    assert_eq!(session.project(), &before);
}
