//! Small subsystem library fixtures shared by persistence and workflow contracts.

use ecad_adapters::decode_project;
use ecad_domain::*;
use std::sync::Arc;

pub fn source(number: u128) -> DefinitionRef {
    DefinitionRef::new(
        DefinitionId::new(1).unwrap(),
        DefinitionRevisionId::new(number).unwrap(),
    )
}
pub fn revision(number: u128) -> Arc<SubsystemRevision> {
    let mut body = decode_project(include_str!("../fixtures/sensor-module-v3.json")).unwrap();
    if number.is_multiple_of(2) {
        body.set_termination(ConductorId::new(2).unwrap(), End::B, None)
            .unwrap();
    }
    Arc::new(SubsystemRevision::new(source(number), SubsystemId::new(10).unwrap(), body).unwrap())
}
pub fn catalog() -> SubsystemLibrary {
    let mut catalog = SubsystemLibrary::default();
    catalog.publish(revision(1)).unwrap();
    catalog
        .set_recommended(source(1).definition(), Some(source(1)))
        .unwrap();
    catalog
}
