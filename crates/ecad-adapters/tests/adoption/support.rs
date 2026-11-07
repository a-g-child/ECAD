//! Deterministic source fixtures and production-ID occurrence setup.

use ecad_adapters::{RandomIds, decode_project};
use ecad_app::{AdoptionPlan, Session, adoption_plan, instance_commands};
use ecad_domain::*;
use serde_json::Value;
use std::sync::Arc;

pub fn source(number: u128) -> DefinitionRef {
    DefinitionRef::new(
        DefinitionId::new(1).unwrap(),
        DefinitionRevisionId::new(number).unwrap(),
    )
}
pub fn revision(number: u128, edit: impl FnOnce(&mut Value)) -> Arc<SubsystemRevision> {
    let mut json: Value =
        serde_json::from_str(include_str!("../fixtures/sensor-module-v3.json")).unwrap();
    edit(&mut json);
    let body = decode_project(&json.to_string()).unwrap();
    Arc::new(SubsystemRevision::new(source(number), SubsystemId::new(10).unwrap(), body).unwrap())
}
pub fn setup(target: Arc<SubsystemRevision>) -> (Session, InstanceBinding, InstanceBinding) {
    let mut project = Project::new(ProjectId::new(77).unwrap());
    project.insert_definition(revision(1, |_| {})).unwrap();
    project.insert_definition(target).unwrap();
    let mut session = Session::new(project);
    let a = place(&mut session, "Left");
    let b = place(&mut session, "Right");
    (session, a, b)
}
pub fn place(session: &mut Session, name: &str) -> InstanceBinding {
    let ids = RandomIds::instance_ids(
        session.project().definition(source(1)).unwrap(),
        InstanceLimits::default(),
    )
    .unwrap();
    let root = ids.subsystems[&SubsystemId::new(10).unwrap()];
    let commands = instance_commands(
        session.project(),
        source(1),
        ids,
        None,
        name,
        InstanceLimits::default(),
    )
    .unwrap();
    session.execute_batch(session.revision(), commands).unwrap();
    session.project().instance(root).unwrap().clone()
}
pub fn plan(session: &Session, binding: &InstanceBinding) -> Result<AdoptionPlan, ModelError> {
    let ids = RandomIds::instance_ids(
        session.project().definition(source(2)).unwrap(),
        InstanceLimits::default(),
    )
    .unwrap();
    adoption_plan(
        session.project(),
        binding.root(),
        source(2),
        ids,
        InstanceLimits::default(),
    )
}
pub fn terminal(binding: &InstanceBinding) -> TerminalId {
    binding.ids().terminals[&TerminalId::new(201).unwrap()]
}
pub fn external_wire(session: &mut Session, binding: &InstanceBinding) -> ConductorId {
    let id = ConductorId::new(900).unwrap();
    session
        .execute(
            session.revision(),
            ecad_app::Command::InsertConductor(Conductor::new(id, Some(terminal(binding)), None)),
        )
        .unwrap();
    id
}
