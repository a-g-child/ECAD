//! Immutable reusable content and instance-local identity mappings.

use crate::{
    Cable, CableId, Conductor, ConductorId, DefinitionId, DefinitionRevisionId, Device, DeviceId,
    InterfacePort, InterfacePortId, ModelError, Project, Subsystem, SubsystemId, TerminalId,
};
use std::{collections::BTreeMap, fmt, num::NonZeroUsize, sync::Arc};

#[path = "reuse_mapping.rs"]
mod mapping;
pub use mapping::MAX_INSTANCE_OBJECTS;

/// A pinned definition lineage/revision pair, distinct from Git or session revisions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DefinitionRef {
    definition: DefinitionId,
    revision: DefinitionRevisionId,
}
impl DefinitionRef {
    /// Identify one immutable revision.
    pub fn new(definition: DefinitionId, revision: DefinitionRevisionId) -> Self {
        Self {
            definition,
            revision,
        }
    }
    /// Return the reusable lineage.
    pub fn definition(self) -> DefinitionId {
        self.definition
    }
    /// Return the pinned snapshot identity.
    pub fn revision(self) -> DefinitionRevisionId {
        self.revision
    }
}
impl fmt::Display for DefinitionRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.definition, self.revision)
    }
}

/// An immutable, self-contained local hierarchy reusable as an electrical assembly.
///
/// Content must be one rooted subtree with all devices inside it. Nested local
/// blocks are supported; nested reusable-definition dependencies are not yet supported.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubsystemRevision {
    source: DefinitionRef,
    root: SubsystemId,
    content: Arc<Project>,
}
impl SubsystemRevision {
    /// Freeze a validated standalone design as one immutable revision.
    /// # Errors
    /// Rejects invalid content, foreign/root-level devices, extra roots, and nested libraries/instances.
    pub fn new(
        source: DefinitionRef,
        root: SubsystemId,
        content: Project,
    ) -> Result<Self, ModelError> {
        content.validate()?;
        if content.has_custom_drawing_documents() {
            return Err(crate::DrawingDocumentError::ReusableDocumentsUnsupported.into());
        }
        if !content.sheets().is_empty() {
            return Err(crate::SheetError::ReusableSheetsUnsupported.into());
        }
        if !content.metadata().values().is_empty() {
            return Err(crate::ReuseError::ProjectMetadataUnsupported.into());
        }
        if content.template_defaults() != &crate::DrawingTemplateDefaults::default()
            || content.drawing() != &crate::DrawingDocument::default()
        {
            return Err(crate::DrawingError::ReusableDrawingsUnsupported.into());
        }
        if content.definition_count() != 0 || content.instances().next().is_some() {
            return Err(crate::ReuseError::NestedDefinitionsUnsupported.into());
        }
        let block = content
            .subsystem(root)
            .ok_or(crate::ReuseError::InvalidDefinitionRoot)?;
        if block.parent().is_some() {
            return Err(crate::ReuseError::InvalidDefinitionRoot.into());
        }
        for block in content.subsystems() {
            if content.subsystem_path(block.id())?.first() != Some(&root) {
                return Err(crate::ReuseError::InvalidDefinitionRoot.into());
            }
        }
        if content.devices().any(|device| device.subsystem().is_none()) {
            return Err(crate::ReuseError::InvalidDefinitionRoot.into());
        }
        Ok(Self {
            source,
            root,
            content: Arc::new(content),
        })
    }
    /// Return the immutable source identity.
    pub fn source(&self) -> DefinitionRef {
        self.source
    }
    /// Return the definition-local root ID, not a placed occurrence ID.
    pub fn root(&self) -> SubsystemId {
        self.root
    }
    /// Borrow the immutable definition-local electrical design.
    pub fn content(&self) -> &Project {
        &self.content
    }
    /// Count all objects that must receive occurrence identities, including terminals.
    pub fn object_count(&self) -> usize {
        self.content.subsystems().len()
            + self.content.devices().len()
            + self
                .content
                .devices()
                .map(|d| d.terminals().len())
                .sum::<usize>()
            + self.content.ports().len()
            + self.content.conductors().len()
            + self.content.cables().len()
    }
}

/// Explicit total mappings from definition-local IDs to fresh occurrence IDs.
/// These are plan inputs, never mutable access to committed model storage.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstanceIds {
    /// Definition-local blocks to placed blocks.
    pub subsystems: BTreeMap<SubsystemId, SubsystemId>,
    /// Definition-local devices to placed devices.
    pub devices: BTreeMap<DeviceId, DeviceId>,
    /// Definition-local anchors to distinct placed anchors.
    pub terminals: BTreeMap<TerminalId, TerminalId>,
    /// Definition-local exposures to placed exposures.
    pub ports: BTreeMap<InterfacePortId, InterfacePortId>,
    /// Definition-local conductors to placed conductors.
    pub conductors: BTreeMap<ConductorId, ConductorId>,
    /// Definition-local cables to placed cables.
    pub cables: BTreeMap<CableId, CableId>,
}

/// Provenance and identity mapping of one materialized, pinned occurrence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceBinding {
    root: SubsystemId,
    source: DefinitionRef,
    ids: InstanceIds,
}

/// Reversible link/provenance state of a placed subsystem root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstanceState {
    /// Ordinary local content with no retained source ancestry.
    Unlinked,
    /// Exact source correspondence; definition-derived objects are locked.
    Pinned(InstanceBinding),
    /// Editable local variation retaining its original source and identity mapping.
    Local(InstanceBinding),
}
impl InstanceBinding {
    /// Construct a binding proposal; insertion checks the complete mapped content.
    pub fn new(root: SubsystemId, source: DefinitionRef, ids: InstanceIds) -> Self {
        Self { root, source, ids }
    }
    /// Return the placed root identity.
    pub fn root(&self) -> SubsystemId {
        self.root
    }
    /// Return the pinned source; it never follows a latest/default pointer.
    pub fn source(&self) -> DefinitionRef {
        self.source
    }
    /// Inspect stable definition-to-occurrence correspondence.
    pub fn ids(&self) -> &InstanceIds {
        &self.ids
    }
}

/// Provisional per-placement expansion budget, independent of containment depth.
#[derive(Clone, Copy, Debug)]
pub struct InstanceLimits {
    /// Maximum total newly materialized objects, capped by [`MAX_INSTANCE_OBJECTS`].
    pub max_objects: NonZeroUsize,
}
impl Default for InstanceLimits {
    fn default() -> Self {
        Self {
            max_objects: NonZeroUsize::new(MAX_INSTANCE_OBJECTS).expect("nonzero constant"),
        }
    }
}

/// Validated materialization values in dependency order, ready for one transaction.
pub struct InstancePlan {
    /// Parent-first placed blocks.
    pub subsystems: Vec<Subsystem>,
    /// Devices with remapped terminals and owners.
    pub devices: Vec<Device>,
    /// Child-first interface exposures.
    pub ports: Vec<InterfacePort>,
    /// Remapped point-to-point conductors.
    pub conductors: Vec<Conductor>,
    /// Assemblies referencing the placed conductors.
    pub cables: Vec<Cable>,
    /// Binding applied after all objects exist.
    pub binding: InstanceBinding,
}

/// Typed cache key for ownership checks without scanning every instance per edit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum OwnedObject {
    Subsystem(SubsystemId),
    Device(DeviceId),
    Terminal(TerminalId),
    Port(InterfacePortId),
    Conductor(ConductorId),
    Cable(CableId),
}
impl InstanceIds {
    pub(crate) fn owned(&self) -> impl Iterator<Item = OwnedObject> + '_ {
        self.subsystems
            .values()
            .copied()
            .map(OwnedObject::Subsystem)
            .chain(self.devices.values().copied().map(OwnedObject::Device))
            .chain(self.terminals.values().copied().map(OwnedObject::Terminal))
            .chain(self.ports.values().copied().map(OwnedObject::Port))
            .chain(
                self.conductors
                    .values()
                    .copied()
                    .map(OwnedObject::Conductor),
            )
            .chain(self.cables.values().copied().map(OwnedObject::Cable))
    }
}
