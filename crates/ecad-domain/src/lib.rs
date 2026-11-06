//! Headless electrical objects and invariant-preserving edits.
//!
//! This crate has no IO, serialization, desktop, or database dependencies.
//! Devices own terminals; conductors reference terminals independently of any
//! drawing geometry. Unassigned conductor ends are valid unfinished engineering.
//!
//! ```
//! use ecad_domain::{Device, DeviceId, Project};
//! let mut project = Project::default();
//! let device = Device::new(DeviceId::new(1)?, "-X1", [])?;
//! project.insert_device(device)?;
//! assert_eq!(project.devices().count(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod error;
mod identity;
mod objects;
mod project;

pub use error::ModelError;
pub use identity::{ConductorId, DeviceId, IdError, TerminalId};
pub use objects::{Conductor, Device, End, Terminal};
pub use project::Project;
