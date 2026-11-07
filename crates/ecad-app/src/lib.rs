//! Editing sessions, reversible commands, and persistence contracts.
//!
//! Callers submit commands against the session's current revision. A rejected or
//! unchanged edit does not consume history or invalidate redo. The document is
//! exposed read-only; desktop integration is not required.
//!
//! ```
//! use ecad_app::{Command, Session};
//! use ecad_domain::{Device, DeviceId, Project, ProjectId};
//! let mut session = Session::new(Project::new(ProjectId::new(1)?));
//! let device = Device::new(DeviceId::new(1)?, "-X1", [])?;
//! session.execute(session.revision(), Command::InsertDevice(device))?;
//! session.undo(session.revision())?;
//! assert_eq!(session.project().devices().count(), 0);
//! session.redo(session.revision())?;
//! assert_eq!(session.project().devices().count(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod action_defaults;
mod action_dispatch;
mod action_registry;
mod action_types;
mod adoption;
mod command;
mod document;
mod document_error;
mod drawing_command;
mod error;
mod input_router;
mod instantiation;
mod keymap;
mod library;
mod project_usage;
mod property_edits;
mod property_selection;
mod revision;
mod route_command;
mod session;
mod shortcut;
mod transaction;
mod usage_index;
mod versioned_store;

pub use action_dispatch::{
    ActionDispatchError, ActionDispatcher, ActionInvocation, ActionOutcome, ActionParameters,
    ActionRequest, ActionRequestKind, InputId, InvocationSource,
};
pub use action_registry::{
    ActionDefinition, ActionHandler, ActionInputRequirement, ActionListing, ActionRegistry,
    ActionRepeat, ActionState, DefaultBindings,
};
pub use action_types::{
    ActionConfigError, ActionContext, ActionDisabled, ActionFocus, ActionId, BindingScope,
};
pub use adoption::{AdoptionPlan, adoption_plan};
pub use command::Command;
pub use document::{Document, ReloadPolicy};
pub use document_error::DocumentError;
pub use drawing_command::DrawingCommand;
pub use error::EditError;
pub use input_router::{ActionInputRouter, ActionKeyEvent, InputSuppression, KeyRoute};
pub use instantiation::instance_commands;
pub use keymap::{
    EffectiveBinding, EffectiveKeymap, KeymapIssue, KeymapOverrides, ScopedShortcut, compile_keymap,
};
pub use library::{
    LibraryPublisher, LibrarySnapshot, Recommendation, VersionedLibraryStore, library_import,
};
pub use project_usage::{ProjectUsage, RevisionUse};
pub use property_edits::{
    PropertyChange, PropertyEdit, PropertyEditPlan, property_edit_plan, property_patch_plan,
};
pub use property_selection::{
    PropertySelection, SelectionProperty, SelectionValue, inspect_properties,
};
pub use revision::{Revision, SessionEpoch};
pub use route_command::RouteCommand;
pub use session::Session;
pub use shortcut::{
    KeySequence, NamedKey, PhysicalModifiers, ResolvedChord, ShortcutChord, ShortcutKey,
    ShortcutModifiers, ShortcutPlatform,
};
pub use usage_index::{KnownUse, KnownUseIndex, RegisteredUsage, UsageRefreshError, UsageStatus};
pub use versioned_store::{ProjectSnapshot, VersionedProjectStore};

/// Low-level, unconditional IO boundary for fixtures/imports and standalone sessions.
/// Use [`VersionedProjectStore`] through [`Document`] for normal editing with
/// external-change checks. This contract does not track observed file versions.
pub trait ProjectStore {
    /// Concrete adapter failure, preserving structured source information.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Load a fully validated project; no partially loaded document escapes.
    /// # Errors
    /// Reports IO, format, schema, or model validation failures.
    fn load(&self) -> Result<ecad_domain::Project, Self::Error>;

    /// Save the complete document, without altering the editing session.
    /// # Errors
    /// Reports IO or encoding failures. Durability guarantees are adapter-specific.
    fn save(&self, project: &ecad_domain::Project) -> Result<(), Self::Error>;
}
