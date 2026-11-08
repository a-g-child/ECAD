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
mod drawing_document;
mod drawing_document_info;
mod drawing_error;
mod drawing_list;
mod drawing_objects;
mod drawing_style;
mod drawing_transform;
mod drawing_values;
mod error;
mod hierarchy;
mod identity;
mod library;
mod objects;
mod project;
mod project_documents;
mod project_metadata;
mod project_sheets;
mod property;
mod property_error;
mod property_registry;
mod reuse;
mod reuse_error;
mod route;
mod route_error;
mod route_geometry;
mod sheet;
mod sheet_defaults;
mod sheet_frame;
mod snapping;

pub use adoption_changes::{ObjectChanges, RevisionChanges};
pub use cable::{Cable, CableEndpoint};
pub use drawing_document::DrawingDocument;
pub use drawing_error::DrawingError;
pub use drawing_objects::{DrawingLabel, DrawingPolyline, DrawingView};
pub use drawing_style::{
    Appearance, DrawingRole, DrawingSettings, DrawingStyle, DrawingValues, StylePatch,
};
pub use drawing_values::{DrawingColour, DrawingLength, DrawingPoint, FontFamily};
pub use error::ModelError;
pub use hierarchy::{InterfacePort, MAX_SUBSYSTEM_DEPTH, Subsystem};
pub use identity::{AnchorId, DrawingDocumentId, ProfileId, RouteId, SheetId};
pub use identity::{
    CableId, ConductorId, DefinitionId, DefinitionRevisionId, DeviceId, IdError, InterfacePortId,
    ProjectId, SubsystemId, TerminalId,
};
pub use identity::{LabelId, PolylineId, StyleId, ViewId};
pub use library::{LibraryError, SubsystemLibrary};
pub use objects::{Conductor, Device, End, Terminal};
pub use project::Project;
pub use property::{
    ObjectRef, PropertyAccess, PropertyDescriptor, PropertyId, PropertyObservation, PropertyOrigin,
    PropertyOwnership, PropertyReferenceKind, PropertyRestriction, PropertyState, PropertyValue,
    PropertyValueKind,
};
pub use property_error::PropertyError;
pub use property_registry::PROPERTY_IDS;
pub use reuse::{
    DefinitionRef, InstanceBinding, InstanceIds, InstanceLimits, InstancePlan, InstanceState,
    MAX_INSTANCE_OBJECTS, SubsystemRevision,
};
pub use reuse_error::ReuseError;
pub use route::{AnchorLayout, DrawingRoute, RouteEdit, RouteLayout, RouteMode, TerminalAnchor};
pub use route_error::RouteError;

pub use snapping::{DrawingGrid, SnapCandidate, SnapEnd, SnapQuery, SnapSettings, SnapTarget};

pub use drawing_transform::{DrawingTransform, MirrorAxis, QuarterTurn};

pub use project_metadata::{MetadataField, ProjectMetadata};

pub use drawing_list::{DrawingListEntry, DrawingListPage};
pub use sheet::{DrawingSheet, SheetError, SheetPaper, SheetViewPlacement};
pub use sheet_defaults::{DrawingTemplateDefaults, SheetKind};
pub use sheet_frame::{FrameCell, FrameField, FrameRect, SheetFrame, SheetTemplate};

pub use drawing_document_info::{
    DrawingDocumentError, DrawingDocumentLayout, ProjectDrawingDocument,
};

mod template_border_snap;
mod template_graphic;
mod template_layout;
pub use identity::TemplateGraphicId;
pub use template_graphic::{TemplateGraphic, TemplateGraphicShape};
mod template_margin_layout;
pub use template_border_snap::{
    MarginEdge, TableBorderDrag, TableBorderPlacement, table_border_placement,
};
mod template_table;
mod template_table_edit;
mod template_table_insert;
pub use identity::{
    TemplateCellId, TemplateDefinitionId, TemplateRevisionId, TemplateTableId, TemplateVariantId,
};
pub use template_layout::{SavedTemplateVariant, TemplateLayout, TemplateVariantRef};
pub use template_table::{
    TemplateCell, TemplateError, TemplateTable, TemplateText, TemplateTextRun,
    resolve_template_text,
};
pub use template_table_edit::{MergeContents, TableAxis, TableEditError};
pub use template_table_insert::TableGrowthAnchor;

mod legacy_template_layout;

mod template_library;
pub use template_library::{
    TemplateLibrary, TemplateLibraryError, TemplateRevision, TemplateRevisionRef,
};

mod project_templates;

mod template_assignment;
pub use template_assignment::{
    SheetTemplateAssignment, TemplateAssignmentError, TemplateAssignments,
};
