//! One registry of the currently implemented electrical field semantics.

use crate::{
    ObjectRef, PropertyDescriptor, PropertyId, PropertyOwnership, PropertyReferenceKind,
    PropertyValueKind,
};

/// All currently registered fields, in stable inspector order.
pub const PROPERTY_IDS: [PropertyId; 15] = [
    PropertyId::Identity,
    PropertyId::Designation,
    PropertyId::Name,
    PropertyId::TerminalNumber,
    PropertyId::DeviceTerminals,
    PropertyId::TerminalDevice,
    PropertyId::SubsystemParent,
    PropertyId::DeviceSubsystem,
    PropertyId::PortSubsystem,
    PropertyId::PortTerminal,
    PropertyId::ConductorA,
    PropertyId::ConductorB,
    PropertyId::CableA,
    PropertyId::CableB,
    PropertyId::CableConductors,
];
impl PropertyId {
    /// Resolve metadata for this stable identity, independent of a selected object.
    pub fn descriptor(self) -> PropertyDescriptor {
        use PropertyOwnership as Owner;
        use PropertyValueKind as Kind;
        let (key, label, value_kind, ownership, editable) = match self {
            Self::Identity => ("identity", "Identity", Kind::Object, Owner::Identity, false),
            Self::Designation => (
                "designation",
                "Designation",
                Kind::Text,
                Owner::Electrical,
                true,
            ),
            Self::Name => ("name", "Name", Kind::Text, Owner::Electrical, true),
            Self::TerminalNumber => (
                "terminal_number",
                "Terminal number",
                Kind::Text,
                Owner::Electrical,
                true,
            ),
            Self::DeviceTerminals => (
                "device_terminals",
                "Terminals",
                Kind::Objects,
                Owner::Structure,
                false,
            ),
            Self::TerminalDevice => (
                "terminal_device",
                "Owning device",
                Kind::Object,
                Owner::Structure,
                false,
            ),
            Self::SubsystemParent => (
                "subsystem_parent",
                "Parent subsystem",
                Kind::OptionalObject,
                Owner::Containment,
                true,
            ),
            Self::DeviceSubsystem => (
                "device_subsystem",
                "Device subsystem",
                Kind::OptionalObject,
                Owner::Containment,
                true,
            ),
            Self::PortSubsystem => (
                "port_subsystem",
                "Interface subsystem",
                Kind::Object,
                Owner::Structure,
                false,
            ),
            Self::PortTerminal => (
                "port_terminal",
                "Exposed terminal",
                Kind::Object,
                Owner::Connection,
                false,
            ),
            Self::ConductorA => (
                "conductor_a",
                "Conductor end A",
                Kind::OptionalObject,
                Owner::Connection,
                true,
            ),
            Self::ConductorB => (
                "conductor_b",
                "Conductor end B",
                Kind::OptionalObject,
                Owner::Connection,
                true,
            ),
            Self::CableA => (
                "cable_a",
                "Cable end A",
                Kind::Object,
                Owner::Connection,
                false,
            ),
            Self::CableB => (
                "cable_b",
                "Cable end B",
                Kind::Object,
                Owner::Connection,
                false,
            ),
            Self::CableConductors => (
                "cable_conductors",
                "Cable cores",
                Kind::Objects,
                Owner::Structure,
                false,
            ),
        };
        let reference_kind = match self {
            Self::Identity => Some(PropertyReferenceKind::Any),
            Self::DeviceTerminals | Self::PortTerminal | Self::ConductorA | Self::ConductorB => {
                Some(PropertyReferenceKind::Terminal)
            }
            Self::TerminalDevice => Some(PropertyReferenceKind::Device),
            Self::SubsystemParent | Self::DeviceSubsystem | Self::PortSubsystem => {
                Some(PropertyReferenceKind::Subsystem)
            }
            Self::CableA | Self::CableB => Some(PropertyReferenceKind::CableEndpoint),
            Self::CableConductors => Some(PropertyReferenceKind::Conductor),
            Self::Designation | Self::Name | Self::TerminalNumber => None,
        };
        PropertyDescriptor {
            id: self,
            key,
            label,
            value_kind,
            reference_kind,
            ownership,
            editable,
        }
    }
    /// Test applicability by semantic object kind, never by displayed property names.
    pub fn applies_to(self, target: ObjectRef) -> bool {
        use ObjectRef as Object;
        match self {
            Self::Identity => true,
            Self::Designation => matches!(target, Object::Device(_) | Object::Cable(_)),
            Self::Name => matches!(target, Object::Subsystem(_) | Object::Port(_)),
            Self::TerminalNumber | Self::TerminalDevice => matches!(target, Object::Terminal(_)),
            Self::DeviceTerminals | Self::DeviceSubsystem => matches!(target, Object::Device(_)),
            Self::SubsystemParent => matches!(target, Object::Subsystem(_)),
            Self::PortSubsystem | Self::PortTerminal => matches!(target, Object::Port(_)),
            Self::ConductorA | Self::ConductorB => matches!(target, Object::Conductor(_)),
            Self::CableA | Self::CableB | Self::CableConductors => {
                matches!(target, Object::Cable(_))
            }
        }
    }
}
