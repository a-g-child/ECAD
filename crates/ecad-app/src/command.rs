//! Electrical edits and exact localized inverses; no full-document snapshots.

use ecad_domain::{
    Cable, CableId, Conductor, ConductorId, Device, DeviceId, End, InterfacePort, InterfacePortId,
    ModelError, Project, Subsystem, SubsystemId, TerminalId,
};
use ecad_domain::{DefinitionRef, InstanceBinding, InstanceState, SubsystemRevision};
use std::sync::Arc;

/// A committed document edit, independent of UI tool activation or transport DTOs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Replace complete template/default assignments with exact localized undo.
    TemplateAssignments(ecad_domain::TemplateAssignments),
    /// Bundle an explicitly selected complete template revision; does not assign sheets.
    InsertTemplateRevision(Arc<ecad_domain::TemplateRevision>),
    /// Remove a bundled template body, with its exact body retained by undo.
    RemoveTemplateRevision(ecad_domain::TemplateRevisionRef),
    /// Replace only project creation-template choices; existing sheets keep their frame.
    TemplateDefaults(ecad_domain::DrawingTemplateDefaults),
    /// Logical drawing package metadata and sheet ownership with exact inverses.
    DrawingDocument(crate::DrawingDocumentCommand),
    /// Paper-sheet layout with exact localized history.
    Sheet(crate::SheetCommand),
    /// Replace portable project metadata with an exact localized inverse.
    ProjectMetadata(ecad_domain::ProjectMetadata),
    /// Whole-route geometry and placed-terminal layout with exact local inverses.
    Route(crate::RouteCommand),
    /// Drawing content, shared styles, and creation settings with localized inverses.
    Drawing(crate::DrawingCommand),
    /// Restore exact literal/override state, used for inheritance-aware undo and reset.
    SetPropertyState {
        /// Existing addressed object.
        target: ecad_domain::ObjectRef,
        /// Stable field identity.
        property: ecad_domain::PropertyId,
        /// Literal value or exact override/inheritance state.
        state: ecad_domain::PropertyState,
    },
    /// Change one supported real field with validated access and an exact localized inverse.
    SetProperty {
        /// Existing electrical object whose field changes.
        target: ecad_domain::ObjectRef,
        /// Stable property identity, independent of display names.
        property: ecad_domain::PropertyId,
        /// Typed proposed value.
        value: ecad_domain::PropertyValue,
    },
    /// Adopt one reviewed revision; inverse restores the previous binding and content.
    AdoptInstance {
        /// Pin observed during planning; rejects stale or locally detached content.
        expected: InstanceBinding,
        /// Same-lineage target with preserved surviving identities.
        target: InstanceBinding,
    },
    /// Bundle one immutable reusable revision into the project.
    InsertDefinition(Arc<SubsystemRevision>),
    /// Remove a bundled revision only when no pin or local origin references it.
    RemoveDefinition(DefinitionRef),
    /// Pin an already-materialized occurrence after exact correspondence checks.
    BindInstance(InstanceBinding),
    /// Make one occurrence project-local while retaining its objects and wiring.
    DetachInstance(SubsystemId),
    /// Deliberately forget a local variant's source ancestry without changing objects.
    ForgetLocalOrigin(SubsystemId),
    /// Reversible validated link/provenance transition, used by lifecycle undo.
    SetInstanceState {
        /// Placed root whose state changes.
        root: SubsystemId,
        /// Desired pin, local ancestry, or deliberate unlinking.
        state: InstanceState,
    },
    /// Insert a project-local functional block.
    InsertSubsystem(Subsystem),
    /// Remove an empty unreferenced block.
    RemoveSubsystem(SubsystemId),
    /// Change containment without altering identity or electrical connections.
    SetSubsystemParent {
        /// Occurrence to reparent.
        subsystem: SubsystemId,
        /// New containing block or project root.
        parent: Option<SubsystemId>,
    },
    /// Move a device between functional scopes.
    SetDeviceSubsystem {
        /// Device occurrence being moved.
        device: DeviceId,
        /// New block or project root.
        subsystem: Option<SubsystemId>,
    },
    /// Expose an internal terminal through a functional interface.
    InsertPort(InterfacePort),
    /// Remove an unused interface exposure.
    RemovePort(InterfacePortId),
    /// Group existing conductors into an overview cable.
    InsertCable(Cable),
    /// Remove a cable grouping while retaining the conductors.
    RemoveCable(CableId),
    /// Insert a generic device and its terminals.
    InsertDevice(Device),
    /// Remove an unconnected device.
    RemoveDevice(DeviceId),
    /// Insert one conductor, allowing unassigned ends.
    InsertConductor(Conductor),
    /// Remove one conductor without removing its devices.
    RemoveConductor(ConductorId),
    /// Change one attachment without changing conductor identity.
    SetTermination {
        /// Conductor being edited.
        conductor: ConductorId,
        /// Endpoint being edited.
        end: End,
        /// Existing terminal or `None` for a free end.
        terminal: Option<TerminalId>,
    },
}

impl Command {
    /// Apply one validated edit, returning its inverse or `None` for a no-op.
    pub(crate) fn apply(&self, project: &mut Project) -> Result<Option<Self>, ModelError> {
        let inverse = match self {
            Self::TemplateAssignments(value) => {
                if &project.template_assignments() == value {
                    return Ok(None);
                }
                Self::TemplateAssignments(project.replace_template_assignments(value.clone())?)
            }
            Self::InsertTemplateRevision(revision) => {
                if !project.insert_template_revision(Arc::clone(revision))? {
                    return Ok(None);
                }
                Self::RemoveTemplateRevision(revision.reference())
            }
            Self::RemoveTemplateRevision(pin) => {
                Self::InsertTemplateRevision(project.remove_template_revision(*pin)?)
            }
            Self::DrawingDocument(command) => {
                return Ok(command.apply(project)?.map(Self::DrawingDocument));
            }
            Self::Sheet(command) => return Ok(command.apply(project)?.map(Self::Sheet)),
            Self::TemplateDefaults(value) => {
                if project.template_defaults() == value {
                    return Ok(None);
                }
                let previous = project.replace_template_defaults(value.clone())?;
                Self::TemplateDefaults(previous)
            }
            Self::ProjectMetadata(value) => {
                let previous = project.replace_metadata(value.clone());
                if previous == *value {
                    return Ok(None);
                }
                Self::ProjectMetadata(previous)
            }
            Self::Route(command) => return Ok(command.apply(project)?.map(Self::Route)),
            Self::Drawing(command) => return Ok(command.apply(project)?.map(Self::Drawing)),
            Self::SetPropertyState {
                target,
                property,
                state,
            } => {
                let previous = project.set_property_state(*target, *property, state.clone())?;
                if previous == *state {
                    return Ok(None);
                }
                Self::SetPropertyState {
                    target: *target,
                    property: *property,
                    state: previous,
                }
            }
            Self::SetProperty {
                target,
                property,
                value,
            } => {
                let previous = project.property_state(*target, *property)?;
                project.set_property(*target, *property, value.clone())?;
                if previous == project.property_state(*target, *property)? {
                    return Ok(None);
                }
                Self::SetPropertyState {
                    target: *target,
                    property: *property,
                    state: previous,
                }
            }
            Self::AdoptInstance { expected, target } => {
                project.adopt_instance(expected, target)?;
                if expected == target {
                    return Ok(None);
                }
                Self::AdoptInstance {
                    expected: target.clone(),
                    target: expected.clone(),
                }
            }
            Self::InsertDefinition(revision) => {
                if !project.insert_definition(Arc::clone(revision))? {
                    return Ok(None);
                }
                Self::RemoveDefinition(revision.source())
            }
            Self::RemoveDefinition(source) => {
                Self::InsertDefinition(project.remove_definition(*source)?)
            }
            Self::BindInstance(binding) => {
                let previous = project.instance_state(binding.root());
                project.bind_instance(binding.clone())?;
                Self::SetInstanceState {
                    root: binding.root(),
                    state: previous,
                }
            }
            Self::DetachInstance(root) => Self::SetInstanceState {
                root: *root,
                state: InstanceState::Pinned(project.detach_instance(*root)?),
            },
            Self::ForgetLocalOrigin(root) => Self::SetInstanceState {
                root: *root,
                state: InstanceState::Local(project.forget_local_origin(*root)?),
            },
            Self::SetInstanceState { root, state } => {
                let previous = project.set_instance_state(*root, state.clone())?;
                if previous == *state {
                    return Ok(None);
                }
                Self::SetInstanceState {
                    root: *root,
                    state: previous,
                }
            }
            Self::InsertSubsystem(subsystem) => {
                project.insert_subsystem(subsystem.clone())?;
                Self::RemoveSubsystem(subsystem.id())
            }
            Self::RemoveSubsystem(id) => Self::InsertSubsystem(project.remove_subsystem(*id)?),
            Self::SetSubsystemParent { subsystem, parent } => {
                let previous = project.set_subsystem_parent(*subsystem, *parent)?;
                if previous == *parent {
                    return Ok(None);
                }
                Self::SetSubsystemParent {
                    subsystem: *subsystem,
                    parent: previous,
                }
            }
            Self::SetDeviceSubsystem { device, subsystem } => {
                let previous = project.set_device_subsystem(*device, *subsystem)?;
                if previous == *subsystem {
                    return Ok(None);
                }
                Self::SetDeviceSubsystem {
                    device: *device,
                    subsystem: previous,
                }
            }
            Self::InsertPort(port) => {
                project.insert_port(port.clone())?;
                Self::RemovePort(port.id())
            }
            Self::RemovePort(id) => Self::InsertPort(project.remove_port(*id)?),
            Self::InsertCable(cable) => {
                project.insert_cable(cable.clone())?;
                Self::RemoveCable(cable.id())
            }
            Self::RemoveCable(id) => Self::InsertCable(project.remove_cable(*id)?),
            Self::InsertDevice(device) => {
                project.insert_device(device.clone())?;
                Self::RemoveDevice(device.id())
            }
            Self::RemoveDevice(id) => Self::InsertDevice(project.remove_device(*id)?),
            Self::InsertConductor(conductor) => {
                project.insert_conductor(conductor.clone())?;
                Self::RemoveConductor(conductor.id())
            }
            Self::RemoveConductor(id) => Self::InsertConductor(project.remove_conductor(*id)?),
            Self::SetTermination {
                conductor,
                end,
                terminal,
            } => {
                let previous = project.set_termination(*conductor, *end, *terminal)?;
                if previous == *terminal {
                    return Ok(None);
                }
                Self::SetTermination {
                    conductor: *conductor,
                    end: *end,
                    terminal: previous,
                }
            }
        };
        Ok(Some(inverse))
    }
}
