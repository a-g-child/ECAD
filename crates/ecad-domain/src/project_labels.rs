//! Identity-preserving label replacement; constructors retain existing validation rules.

use super::Project;
use crate::{Cable, Device, End, InterfacePort, ModelError, ObjectRef, Subsystem, Terminal};

impl Project {
    pub(super) fn set_property_label(
        &mut self,
        target: ObjectRef,
        text: String,
    ) -> Result<(), ModelError> {
        // Caller has checked target existence, property applicability, and pinned ownership.
        match target {
            ObjectRef::Device(id) => {
                let old = &self.devices[&id];
                let mut replacement = Device::new(id, text, old.terminals().cloned())?;
                if let Some(scope) = old.subsystem() {
                    replacement = replacement.in_subsystem(scope);
                }
                self.devices.insert(id, replacement);
            }
            ObjectRef::Terminal(id) => {
                let owner = self
                    .terminal_owner(id)
                    .expect("required terminal has owner");
                let old = &self.devices[&owner];
                let terminals = old
                    .terminals()
                    .map(|terminal| {
                        if terminal.id() == id {
                            Terminal::new(id, text.clone())
                        } else {
                            Ok(terminal.clone())
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let mut replacement = Device::new(owner, old.designation(), terminals)?;
                if let Some(scope) = old.subsystem() {
                    replacement = replacement.in_subsystem(scope);
                }
                self.devices.insert(owner, replacement);
            }
            ObjectRef::Subsystem(id) => {
                let old = &self.subsystems[&id];
                let replacement = Subsystem::new(id, text, old.parent())?;
                self.subsystems.insert(id, replacement);
            }
            ObjectRef::Port(id) => {
                let old = &self.ports[&id];
                let replacement = InterfacePort::new(id, old.subsystem(), text, old.terminal())?;
                if self.ports.values().any(|port| {
                    port.id() != id
                        && port.subsystem() == replacement.subsystem()
                        && port.name() == replacement.name()
                }) {
                    return Err(ModelError::PortNameInUse(
                        replacement.subsystem(),
                        replacement.name().to_owned(),
                    ));
                }
                self.ports.insert(id, replacement);
            }
            ObjectRef::Cable(id) => {
                let old = &self.cables[&id];
                let replacement = Cable::new(
                    id,
                    text,
                    old.endpoint(End::A),
                    old.endpoint(End::B),
                    old.conductors(),
                )?;
                self.cables.insert(id, replacement);
            }
            ObjectRef::View(_)
            | ObjectRef::Label(_)
            | ObjectRef::Polyline(_)
            | ObjectRef::Conductor(_) => unreachable!("conductors have no registered label field"),
        }
        Ok(())
    }
}
