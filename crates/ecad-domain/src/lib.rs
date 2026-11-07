//! Headless electrical objects and invariant-preserving edits.
//!
//! This crate has no IO, serialization, desktop, or database dependencies.
//! Devices own terminals; conductors reference terminals independently of any
//! drawing geometry. Unassigned conductor ends are valid unfinished engineering.
//!
//! ```
//! use ecad_domain::{Device, DeviceId, Project, ProjectId};
//! let mut project = Project::new(ProjectId::new(1)?);
//! let device = Device::new(DeviceId::new(1)?, "-X1", [])?;
//! project.insert_device(device)?;
//! assert_eq!(project.devices().count(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod adoption_changes;
mod cable;
mod error;
mod hierarchy;
mod identity;
mod library;
mod objects;
mod project;
mod reuse;
mod reuse_error;

pub use adoption_changes::{ObjectChanges, RevisionChanges};
pub use cable::{Cable, CableEndpoint};
pub use error::ModelError;
pub use hierarchy::{InterfacePort, MAX_SUBSYSTEM_DEPTH, Subsystem};
pub use identity::{
    CableId, ConductorId, DefinitionId, DefinitionRevisionId, DeviceId, IdError, InterfacePortId,
    ProjectId, SubsystemId, TerminalId,
};
pub use library::{LibraryError, SubsystemLibrary};
pub use objects::{Conductor, Device, End, Terminal};
pub use project::Project;
pub use reuse::{
    DefinitionRef, InstanceBinding, InstanceIds, InstanceLimits, InstancePlan, InstanceState,
    MAX_INSTANCE_OBJECTS, SubsystemRevision,
};
pub use reuse_error::ReuseError;
