//! Electrical edits and exact localized inverses; no full-document snapshots.

use ecad_domain::{Conductor, ConductorId, Device, DeviceId, End, ModelError, Project, TerminalId};

/// A committed document edit, independent of UI tool activation or transport DTOs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
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
