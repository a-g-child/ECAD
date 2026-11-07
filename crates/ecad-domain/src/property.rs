//! Typed property identities and metadata over real electrical fields, not string bags.

use crate::{CableId, ConductorId, DeviceId, InterfacePortId, SubsystemId, TerminalId};

/// An addressable existing electrical object; IDs of different kinds remain distinct.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectRef {
    /// Drawing view.
    View(crate::ViewId),
    /// Free-standing text annotation.
    Label(crate::LabelId),
    /// Graphical path without electrical connectivity.
    Polyline(crate::PolylineId),
    /// Functional subsystem occurrence.
    Subsystem(SubsystemId),
    /// Device occurrence.
    Device(DeviceId),
    /// Terminal owned by a device.
    Terminal(TerminalId),
    /// Explicit interface exposure.
    Port(InterfacePortId),
    /// Individual conductor.
    Conductor(ConductorId),
    /// Cable assembly.
    Cable(CableId),
}

/// Stable property identities; sharing a label alone never combines two properties.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PropertyId {
    /// Requested font family on drawing text.
    FontFamily,
    /// Graphical text height in micrometres.
    TextHeight,
    /// Graphical stroke width in micrometres, never conductor area.
    StrokeWidth,
    /// Output drawing colour across text and strokes.
    DrawingColour,
    /// Literal free-standing annotation content.
    DrawingText,
    /// Annotation location in document units.
    DrawingPosition,
    /// Ordered whole graphical path vertices.
    DrawingVertices,
    /// Drawing scope, independent of functional containment.
    DrawingView,
    /// Optional named style link, independent of explicit overrides.
    DrawingStyle,

    /// Stable typed object identity.
    Identity,
    /// Device/cable engineering designation.
    Designation,
    /// Subsystem/interface name.
    Name,
    /// Displayed terminal number, independent of its identity.
    TerminalNumber,
    /// Device's owned terminal identities.
    DeviceTerminals,
    /// Terminal's owning device.
    TerminalDevice,
    /// Subsystem's containing block.
    SubsystemParent,
    /// Device's functional scope.
    DeviceSubsystem,
    /// Interface's containing block.
    PortSubsystem,
    /// Actual terminal exposed by an interface.
    PortTerminal,
    /// First conductor attachment.
    ConductorA,
    /// Second conductor attachment.
    ConductorB,
    /// First cable overview endpoint.
    CableA,
    /// Second cable overview endpoint.
    CableB,
    /// Cable's existing core membership.
    CableConductors,
}

/// Runtime value type, shared by inspection and validated edits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyValueKind {
    /// Positive document distance in micrometres.
    Length,
    /// Output RGB colour.
    Colour,
    /// Signed document coordinates.
    Point,
    /// Ordered whole-path vertices.
    Points,
    /// Optional named style identity.
    Style,

    /// Human-readable text; field constructors enforce required labels and uniqueness.
    Text,
    /// One typed object reference.
    Object,
    /// Optional typed reference; absence is an explicit free end/root scope.
    OptionalObject,
    /// Canonically ordered object references.
    Objects,
}

/// Typed inspected/proposed value; absence and mixed-selection state are separate concepts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyValue {
    /// Positive document distance in micrometres.
    Length(crate::DrawingLength),
    /// Output RGB colour.
    Colour(crate::DrawingColour),
    /// Signed document coordinates.
    Point(crate::DrawingPoint),
    /// Ordered whole graphical path vertices.
    Points(Vec<crate::DrawingPoint>),
    /// Explicit optional named style link.
    Style(Option<crate::StyleId>),

    /// A displayed label or engineering identifier.
    Text(String),
    /// One existing typed identity/reference.
    Object(ObjectRef),
    /// Root scope or a free connection end is represented by `None`.
    OptionalObject(Option<ObjectRef>),
    /// Existing structural members in canonical order.
    Objects(Vec<ObjectRef>),
}
impl PropertyValue {
    /// Identify this value's runtime kind without coercing strings or reference types.
    pub fn kind(&self) -> PropertyValueKind {
        match self {
            Self::Length(_) => PropertyValueKind::Length,
            Self::Colour(_) => PropertyValueKind::Colour,
            Self::Point(_) => PropertyValueKind::Point,
            Self::Points(_) => PropertyValueKind::Points,
            Self::Style(_) => PropertyValueKind::Style,
            Self::Text(_) => PropertyValueKind::Text,
            Self::Object(_) => PropertyValueKind::Object,
            Self::OptionalObject(_) => PropertyValueKind::OptionalObject,
            Self::Objects(_) => PropertyValueKind::Objects,
        }
    }
}

/// Semantic ownership/category; future graphical properties have separate identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyOwnership {
    /// Display identification shared across electrical and drawing scopes.
    Identification,
    /// Document presentation on a drawing representation.
    Presentation,
    /// Immutable electrical identity.
    Identity,
    /// Engineering labels on the addressed electrical object.
    Electrical,
    /// Functional containment, independent of drawing layout.
    Containment,
    /// Actual electrical attachments and cable relationships.
    Connection,
    /// Ownership/membership established through structural operations.
    Structure,
}

/// Why a property is currently read-only, distinct from not being applicable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyRestriction {
    /// Stable identity is never edited as an ordinary property.
    Identity,
    /// A dedicated structural operation is required for this field.
    StructuralOperation,
    /// Make the owning pinned instance local before changing source-derived fields.
    PinnedInstance(SubsystemId),
}

/// Effective access for one property at one existing object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyAccess {
    /// The field can be edited through validated commands.
    Editable,
    /// Inspection is available, with a concrete reason editing is restricted.
    ReadOnly(PropertyRestriction),
}

/// Registry metadata used by inspection and editing; no UI or serialization dependency.
pub struct PropertyDescriptor {
    /// Stable typed identity.
    pub id: PropertyId,
    /// Stable machine-readable name, independent of translated display labels.
    pub key: &'static str,
    /// Initial human-readable label.
    pub label: &'static str,
    /// Required runtime value kind; reference target kinds are checked by the model.
    pub value_kind: PropertyValueKind,
    /// Allowed referenced object kinds, including list members; text fields have none.
    pub reference_kind: Option<PropertyReferenceKind>,
    /// Semantic ownership of the real model field.
    pub ownership: PropertyOwnership,
    /// Whether the property system supports editing this field in principle.
    pub editable: bool,
}

/// Allowed target kinds for a reference-valued field, independent of displayed labels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyReferenceKind {
    /// Drawing scope, independent of functional containment.
    View,
    /// An object's own identity may be of any supported kind.
    Any,
    /// A functional subsystem.
    Subsystem,
    /// A device occurrence.
    Device,
    /// A real connection terminal.
    Terminal,
    /// A conductor used as a cable core.
    Conductor,
    /// An overview endpoint is a device or subsystem.
    CableEndpoint,
}
impl PropertyReferenceKind {
    /// Check target kind only; existence and electrical compatibility require model validation.
    pub fn accepts(self, target: ObjectRef) -> bool {
        match self {
            Self::Any => true,
            Self::View => matches!(target, ObjectRef::View(_)),
            Self::Subsystem => matches!(target, ObjectRef::Subsystem(_)),
            Self::Device => matches!(target, ObjectRef::Device(_)),
            Self::Terminal => matches!(target, ObjectRef::Terminal(_)),
            Self::Conductor => matches!(target, ObjectRef::Conductor(_)),
            Self::CableEndpoint => matches!(target, ObjectRef::Device(_) | ObjectRef::Subsystem(_)),
        }
    }
}
impl PropertyDescriptor {
    /// Check runtime value and reference kinds, without checking labels, existence, or topology.
    pub fn accepts_value(&self, value: &PropertyValue) -> bool {
        if value.kind() != self.value_kind {
            return false;
        }
        let accepts = |target| self.reference_kind.is_some_and(|kind| kind.accepts(target));
        match value {
            PropertyValue::Length(_)
            | PropertyValue::Colour(_)
            | PropertyValue::Point(_)
            | PropertyValue::Points(_)
            | PropertyValue::Style(_)
            | PropertyValue::Text(_)
            | PropertyValue::OptionalObject(None) => true,
            PropertyValue::Object(target) | PropertyValue::OptionalObject(Some(target)) => {
                accepts(*target)
            }
            PropertyValue::Objects(targets) => targets.iter().all(|target| accepts(*target)),
        }
    }
}

/// Inspected value and effective access; mixed states are computed over selections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyObservation {
    /// Current real field value.
    pub value: PropertyValue,
    /// Current restrictions, including reusable-instance ownership.
    pub access: PropertyAccess,
    /// Effective style source per property; electrical and literal fields have none.
    pub origin: Option<PropertyOrigin>,
}

/// Effective style source for one resolved property.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyOrigin {
    /// Complete project role fallback.
    Project,
    /// Sparse value from a linked named style.
    NamedStyle(crate::StyleId),
    /// Override from the containing drawing view.
    View(crate::ViewId),
    /// Explicit object override, including values equal to inheritance.
    Object,
}
/// Exact editable state; style override intent is distinct from resolved appearance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyState {
    /// Literal electrical/geometry field or named style link.
    Field(PropertyValue),
    /// An explicit per-property value or absence restoring inheritance.
    Override(Option<PropertyValue>),
}
