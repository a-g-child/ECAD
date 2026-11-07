//! Typed reads/access and validated edits over authoritative model fields.

use super::Project;
use crate::reuse::OwnedObject;
use crate::{
    CableEndpoint, End, ModelError, ObjectRef, PropertyAccess, PropertyError, PropertyId,
    PropertyObservation, PropertyRestriction, PropertyValue,
};

impl ObjectRef {
    fn owned(self) -> OwnedObject {
        match self {
            Self::Subsystem(id) => OwnedObject::Subsystem(id),
            Self::Device(id) => OwnedObject::Device(id),
            Self::Terminal(id) => OwnedObject::Terminal(id),
            Self::Port(id) => OwnedObject::Port(id),
            Self::Conductor(id) => OwnedObject::Conductor(id),
            Self::Cable(id) => OwnedObject::Cable(id),
        }
    }
}
impl Project {
    /// Require an existing object before inspection/planning; missing selections are errors.
    /// # Errors
    /// Returns the matching typed missing-object error, never an unsupported-property result.
    pub fn require_object(&self, target: ObjectRef) -> Result<(), ModelError> {
        let error = match target {
            ObjectRef::Subsystem(id) if self.subsystem(id).is_none() => {
                Some(ModelError::MissingSubsystem(id))
            }
            ObjectRef::Device(id) if self.device(id).is_none() => {
                Some(ModelError::MissingDevice(id))
            }
            ObjectRef::Terminal(id) if self.terminal_owner(id).is_none() => {
                Some(ModelError::MissingTerminal(id))
            }
            ObjectRef::Port(id) if self.port(id).is_none() => Some(ModelError::MissingPort(id)),
            ObjectRef::Conductor(id) if self.conductor(id).is_none() => {
                Some(ModelError::MissingConductor(id))
            }
            ObjectRef::Cable(id) if self.cable(id).is_none() => Some(ModelError::MissingCable(id)),
            _ => None,
        };
        error.map_or(Ok(()), Err)
    }
    /// Inspect a real field and its effective access, including pinned ownership.
    /// # Errors
    /// Missing targets and inapplicable properties return distinct typed failures.
    pub fn property(
        &self,
        target: ObjectRef,
        property: PropertyId,
    ) -> Result<PropertyObservation, ModelError> {
        self.require_object(target)?;
        let value = self.property_value(target, property)?;
        let descriptor = property.descriptor();
        let access = if property == PropertyId::Identity {
            PropertyAccess::ReadOnly(PropertyRestriction::Identity)
        } else if !descriptor.editable {
            PropertyAccess::ReadOnly(PropertyRestriction::StructuralOperation)
        } else if let Some(root) = self.instance_owners.get(&target.owned()) {
            let parameter = target == ObjectRef::Subsystem(*root)
                && matches!(property, PropertyId::Name | PropertyId::SubsystemParent);
            if parameter {
                PropertyAccess::Editable
            } else {
                PropertyAccess::ReadOnly(PropertyRestriction::PinnedInstance(*root))
            }
        } else {
            PropertyAccess::Editable
        };
        Ok(PropertyObservation { value, access })
    }
    fn property_value(
        &self,
        target: ObjectRef,
        property: PropertyId,
    ) -> Result<PropertyValue, ModelError> {
        use ObjectRef as Object;
        use PropertyId as Field;
        use PropertyValue as Value;
        let endpoint = |end| match end {
            CableEndpoint::Device(id) => Object::Device(id),
            CableEndpoint::Subsystem(id) => Object::Subsystem(id),
        };
        let value = match (target, property) {
            (_, Field::Identity) => Value::Object(target),
            (Object::Device(id), Field::Designation) => {
                Value::Text(self.devices[&id].designation().to_owned())
            }
            (Object::Cable(id), Field::Designation) => {
                Value::Text(self.cables[&id].designation().to_owned())
            }
            (Object::Subsystem(id), Field::Name) => {
                Value::Text(self.subsystems[&id].name().to_owned())
            }
            (Object::Port(id), Field::Name) => Value::Text(self.ports[&id].name().to_owned()),
            (Object::Terminal(id), Field::TerminalNumber) => {
                let owner = self
                    .terminal_owner(id)
                    .expect("required terminal has owner");
                let terminal = self.devices[&owner]
                    .terminals()
                    .find(|terminal| terminal.id() == id)
                    .expect("index resolves an owned terminal");
                Value::Text(terminal.number().to_owned())
            }
            (Object::Device(id), Field::DeviceTerminals) => Value::Objects(
                self.devices[&id]
                    .terminals()
                    .map(|t| Object::Terminal(t.id()))
                    .collect(),
            ),
            (Object::Terminal(id), Field::TerminalDevice) => Value::Object(Object::Device(
                self.terminal_owner(id)
                    .expect("required terminal has owner"),
            )),
            (Object::Subsystem(id), Field::SubsystemParent) => {
                Value::OptionalObject(self.subsystems[&id].parent().map(Object::Subsystem))
            }
            (Object::Device(id), Field::DeviceSubsystem) => {
                Value::OptionalObject(self.devices[&id].subsystem().map(Object::Subsystem))
            }
            (Object::Port(id), Field::PortSubsystem) => {
                Value::Object(Object::Subsystem(self.ports[&id].subsystem()))
            }
            (Object::Port(id), Field::PortTerminal) => {
                Value::Object(Object::Terminal(self.ports[&id].terminal()))
            }
            (Object::Conductor(id), Field::ConductorA | Field::ConductorB) => {
                Value::OptionalObject(
                    self.conductors[&id]
                        .endpoint(if property == Field::ConductorA {
                            End::A
                        } else {
                            End::B
                        })
                        .map(Object::Terminal),
                )
            }
            (Object::Cable(id), Field::CableA | Field::CableB) => Value::Object(endpoint(
                self.cables[&id].endpoint(if property == Field::CableA {
                    End::A
                } else {
                    End::B
                }),
            )),
            (Object::Cable(id), Field::CableConductors) => Value::Objects(
                self.cables[&id]
                    .conductors()
                    .map(Object::Conductor)
                    .collect(),
            ),
            _ => return Err(PropertyError::NotApplicable { target, property }.into()),
        };
        Ok(value)
    }
    /// Change a supported real field atomically and return its previous value.
    /// Exact no-ops leave even read-only fields untouched. Ordinary application edits
    /// must use commands so validation, revision guards, and undo remain coherent.
    /// # Errors
    /// Rejects missing/inapplicable targets, type errors, access restrictions, invalid
    /// labels, and topology violations without changing the preceding model.
    pub fn set_property(
        &mut self,
        target: ObjectRef,
        property: PropertyId,
        value: PropertyValue,
    ) -> Result<PropertyValue, ModelError> {
        let observation = self.property(target, property)?;
        let wrong_type = || {
            ModelError::from(PropertyError::WrongType {
                property,
                expected: property.descriptor().value_kind,
            })
        };
        if !property.descriptor().accepts_value(&value) {
            return Err(wrong_type());
        }
        if observation.value == value {
            return Ok(observation.value);
        }
        if let PropertyAccess::ReadOnly(reason) = observation.access {
            return Err(PropertyError::ReadOnly {
                target,
                property,
                reason,
            }
            .into());
        }
        match (target, property, value) {
            (
                _,
                PropertyId::Name | PropertyId::Designation | PropertyId::TerminalNumber,
                PropertyValue::Text(text),
            ) => self.set_property_label(target, text)?,
            (
                ObjectRef::Subsystem(id),
                PropertyId::SubsystemParent,
                PropertyValue::OptionalObject(parent),
            ) => {
                let parent = match parent {
                    None => None,
                    Some(ObjectRef::Subsystem(id)) => Some(id),
                    _ => return Err(wrong_type()),
                };
                self.set_subsystem_parent(id, parent)?;
            }
            (
                ObjectRef::Device(id),
                PropertyId::DeviceSubsystem,
                PropertyValue::OptionalObject(scope),
            ) => {
                let scope = match scope {
                    None => None,
                    Some(ObjectRef::Subsystem(id)) => Some(id),
                    _ => return Err(wrong_type()),
                };
                self.set_device_subsystem(id, scope)?;
            }
            (
                ObjectRef::Conductor(id),
                PropertyId::ConductorA | PropertyId::ConductorB,
                PropertyValue::OptionalObject(terminal),
            ) => {
                let terminal = match terminal {
                    None => None,
                    Some(ObjectRef::Terminal(id)) => Some(id),
                    _ => return Err(wrong_type()),
                };
                self.set_termination(
                    id,
                    if property == PropertyId::ConductorA {
                        End::A
                    } else {
                        End::B
                    },
                    terminal,
                )?;
            }
            _ => return Err(wrong_type()),
        }
        Ok(observation.value)
    }
}
