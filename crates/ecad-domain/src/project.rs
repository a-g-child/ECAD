//! Project ownership and atomic, invariant-preserving single-object operations.

use crate::reuse::OwnedObject;
use crate::{
    Cable, CableId, Conductor, ConductorId, Device, DeviceId, End, InterfacePort, InterfacePortId,
    ModelError, ProjectId, Subsystem, SubsystemId, TerminalId,
};
use crate::{DefinitionRef, InstanceBinding, SubsystemRevision};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[path = "project_adoption.rs"]
mod adoption;
#[path = "project_cables.rs"]
mod cables;
#[path = "project_drawing.rs"]
mod drawing;
#[path = "project_hierarchy.rs"]
mod hierarchy;
#[path = "project_instance_checks.rs"]
mod instance_checks;
#[path = "project_instance_states.rs"]
mod instance_states;
#[path = "project_instances.rs"]
mod instances;
#[path = "project_interfaces.rs"]
mod interfaces;
#[path = "project_labels.rs"]
mod labels;
#[path = "project_properties.rs"]
mod properties;
#[path = "project_routes.rs"]
mod routes;

/// A validated project with electrical identity and separate view-local drawing representations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    id: ProjectId,
    pub(crate) drawing_documents: Vec<crate::ProjectDrawingDocument>,
    pub(crate) sheet_owners: BTreeMap<crate::SheetId, crate::DrawingDocumentId>,
    pub(crate) sheets: Vec<crate::DrawingSheet>,
    pub(crate) metadata: crate::ProjectMetadata,
    pub(crate) template_defaults: crate::DrawingTemplateDefaults,
    pub(crate) template_revisions:
        BTreeMap<crate::TemplateRevisionRef, Arc<crate::TemplateRevision>>,
    drawing: crate::DrawingDocument,
    devices: BTreeMap<DeviceId, Device>,
    terminal_owners: BTreeMap<TerminalId, DeviceId>,
    conductors: BTreeMap<ConductorId, Conductor>,
    subsystems: BTreeMap<SubsystemId, Subsystem>,
    ports: BTreeMap<InterfacePortId, InterfacePort>,
    exports: BTreeMap<(SubsystemId, TerminalId), BTreeSet<InterfacePortId>>,
    cables: BTreeMap<CableId, Cable>,
    conductor_cables: BTreeMap<ConductorId, CableId>,
    definitions: BTreeMap<DefinitionRef, Arc<SubsystemRevision>>,
    instances: BTreeMap<SubsystemId, InstanceBinding>,
    local_origins: BTreeMap<SubsystemId, InstanceBinding>,
    instance_owners: BTreeMap<OwnedObject, SubsystemId>,
}

impl Project {
    /// Create an empty document with an explicit persistent identity.
    /// Production callers should supply a generated ID; fixtures may use fixed IDs.
    pub fn new(id: ProjectId) -> Self {
        Self {
            id,
            drawing_documents: vec![crate::ProjectDrawingDocument::initial(id)],
            sheet_owners: BTreeMap::new(),
            sheets: Vec::new(),
            metadata: crate::ProjectMetadata::default(),
            template_defaults: crate::DrawingTemplateDefaults::default(),
            template_revisions: BTreeMap::new(),
            drawing: crate::DrawingDocument::default(),
            devices: BTreeMap::new(),
            terminal_owners: BTreeMap::new(),
            conductors: BTreeMap::new(),
            subsystems: BTreeMap::new(),
            ports: BTreeMap::new(),
            exports: BTreeMap::new(),
            cables: BTreeMap::new(),
            conductor_cables: BTreeMap::new(),
            definitions: BTreeMap::new(),
            instances: BTreeMap::new(),
            local_origins: BTreeMap::new(),
            instance_owners: BTreeMap::new(),
        }
    }

    /// Return the project lineage identity, preserved by saves and file copies.
    pub fn id(&self) -> ProjectId {
        self.id
    }

    /// Iterate devices in canonical identity order.
    pub fn devices(&self) -> impl ExactSizeIterator<Item = &Device> {
        self.devices.values()
    }

    /// Iterate conductors in canonical identity order.
    pub fn conductors(&self) -> impl ExactSizeIterator<Item = &Conductor> {
        self.conductors.values()
    }

    /// Find an existing device without exposing mutable model storage.
    pub fn device(&self, id: DeviceId) -> Option<&Device> {
        self.devices.get(&id)
    }

    /// Find a conductor without exposing mutable model storage.
    pub fn conductor(&self, id: ConductorId) -> Option<&Conductor> {
        self.conductors.get(&id)
    }

    /// Resolve a project-wide terminal identity to its owning device.
    pub fn terminal_owner(&self, id: TerminalId) -> Option<DeviceId> {
        self.terminal_owners.get(&id).copied()
    }

    /// Insert a device and all its terminals as one operation.
    /// # Errors
    /// Duplicate device or terminal identities reject the entire insertion.
    pub fn insert_device(&mut self, device: Device) -> Result<(), ModelError> {
        self.ensure_local_parent(device.subsystem())?;
        self.require_subsystem(device.subsystem())?;
        if self.devices.contains_key(&device.id()) {
            return Err(ModelError::DuplicateDevice(device.id()));
        }
        for terminal in device.terminals() {
            if self.terminal_owners.contains_key(&terminal.id()) {
                return Err(ModelError::DuplicateTerminal(terminal.id()));
            }
        }
        for terminal in device.terminals() {
            self.terminal_owners.insert(terminal.id(), device.id());
        }
        self.devices.insert(device.id(), device);
        Ok(())
    }

    /// Remove an unconnected device, returning its exact value for undo.
    /// # Errors
    /// Missing, connected, placed-anchor, or interface/cable-referenced devices are rejected without mutation.
    /// Currently scans conductors; the scale harness should inform a reverse index.
    pub fn remove_device(&mut self, id: DeviceId) -> Result<Device, ModelError> {
        if self
            .drawing
            .terminal_anchors()
            .any(|anchor| self.terminal_owner(anchor.terminal) == Some(id))
        {
            return Err(crate::RouteError::DeviceAnchored(id).into());
        }
        self.ensure_editable(OwnedObject::Device(id))?;
        let device = self.devices.get(&id).ok_or(ModelError::MissingDevice(id))?;
        for conductor in self.conductors.values() {
            for end in [End::A, End::B] {
                if conductor
                    .endpoint(end)
                    .is_some_and(|t| self.terminal_owner(t) == Some(id))
                {
                    return Err(ModelError::DeviceConnected(id));
                }
            }
        }
        if self
            .ports
            .values()
            .any(|port| self.terminal_owner(port.terminal()) == Some(id))
            || self.cables.values().any(|cable| {
                [End::A, End::B]
                    .into_iter()
                    .any(|end| cable.endpoint(end) == crate::CableEndpoint::Device(id))
            })
        {
            return Err(ModelError::DeviceReferenced(id));
        }
        for terminal in device.terminals() {
            self.terminal_owners.remove(&terminal.id());
        }
        self.devices
            .remove(&id)
            .ok_or(ModelError::MissingDevice(id))
    }

    /// Insert a conductor; free ends are allowed, dangling references are not.
    /// # Errors
    /// Rejects duplicate IDs, nonexistent terminals, or unexposed crossed boundaries atomically.
    pub fn insert_conductor(&mut self, conductor: Conductor) -> Result<(), ModelError> {
        if self.conductors.contains_key(&conductor.id()) {
            return Err(ModelError::DuplicateConductor(conductor.id()));
        }
        self.check_conductor(&conductor)?;
        self.conductors.insert(conductor.id(), conductor);
        Ok(())
    }

    /// Remove a conductor, retaining its identity and endpoints in the return value.
    /// # Errors
    /// Returns a missing-conductor, drawing-route, ownership, or cable-membership error without mutation.
    pub fn remove_conductor(&mut self, id: ConductorId) -> Result<Conductor, ModelError> {
        if self.drawing.conductor_routes(id).next().is_some() {
            return Err(crate::RouteError::ConductorInUse(id).into());
        }
        self.ensure_editable(OwnedObject::Conductor(id))?;
        if self.conductor_cables.contains_key(&id) {
            return Err(ModelError::ConductorInCable(id));
        }
        self.conductors
            .remove(&id)
            .ok_or(ModelError::MissingConductor(id))
    }

    /// Set one termination, returning the previous attachment for undo.
    /// # Errors
    /// Unknown references, unexposed boundaries, or cable endpoint mismatches are
    /// rejected before mutation.
    pub fn set_termination(
        &mut self,
        id: ConductorId,
        end: End,
        terminal: Option<TerminalId>,
    ) -> Result<Option<TerminalId>, ModelError> {
        let current = self
            .conductors
            .get(&id)
            .ok_or(ModelError::MissingConductor(id))?;
        if current.endpoint(end) != terminal {
            self.ensure_editable(OwnedObject::Conductor(id))?;
        }
        let mut candidate = current.clone();
        candidate.set_endpoint(end, terminal);
        self.check_conductor(&candidate)?;
        let conductor = self
            .conductors
            .get_mut(&id)
            .ok_or(ModelError::MissingConductor(id))?;
        let previous = conductor.endpoint(end);
        conductor.set_endpoint(end, terminal);
        Ok(previous)
    }

    /// Recheck complete hierarchy, interfaces, wiring, and cable endpoint consistency.
    /// # Errors
    /// Returns the first structural/boundary error in deterministic validation order.
    pub fn validate(&self) -> Result<(), ModelError> {
        self.validate_hierarchy()?;
        for conductor in self.conductors.values() {
            self.check_conductor(conductor)?;
        }
        for cable in self.cables.values() {
            self.check_cable(cable)?;
        }
        self.validate_instances()?;
        self.drawing.validate()?;
        self.validate_sheets()?;
        self.validate_drawing_documents()?;
        self.check_drawing_references(&self.drawing)?;
        Ok(())
    }

    fn check_terminal(&self, terminal: Option<TerminalId>) -> Result<(), ModelError> {
        if let Some(id) = terminal
            && !self.terminal_owners.contains_key(&id)
        {
            return Err(ModelError::MissingTerminal(id));
        }
        Ok(())
    }

    fn check_conductor(&self, conductor: &Conductor) -> Result<(), ModelError> {
        for end in [End::A, End::B] {
            self.check_terminal(conductor.endpoint(end))?;
        }
        self.check_boundaries(conductor.endpoint(End::A), conductor.endpoint(End::B))?;
        self.check_instance_wire(conductor)?;
        self.check_conductor_routes(conductor)?;
        if let Some(cable_id) = self.conductor_cables.get(&conductor.id()) {
            let cable = self
                .cables
                .get(cable_id)
                .ok_or(ModelError::MissingCable(*cable_id))?;
            self.check_core_endpoints(cable, conductor)?;
        }
        Ok(())
    }
}
