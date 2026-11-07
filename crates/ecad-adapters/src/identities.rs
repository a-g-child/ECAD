//! Fallible OS-random UUID-v4 generation outside the dependency-free domain.

use ecad_domain::{
    CableId, ConductorId, DefinitionId, DefinitionRevisionId, DeviceId, InterfacePortId, ProjectId,
    SubsystemId, TerminalId,
};

/// Generate typed persistent IDs from OS entropy, without timestamps or counters.
///
/// UUID-v4 collisions remain probabilistically possible; project insertion still
/// checks uniqueness. Fixed domain constructors remain available for imports/tests.
pub struct RandomIds;

impl RandomIds {
    /// Generate a reusable definition lineage identity.
    /// # Errors
    /// Returns the OS entropy error.
    pub fn definition() -> Result<DefinitionId, getrandom::Error> {
        Ok(DefinitionId::new(random_value()?).expect("UUID-v4 identity is nonzero"))
    }
    /// Generate an immutable definition snapshot identity.
    /// # Errors
    /// Returns the OS entropy error.
    pub fn definition_revision() -> Result<DefinitionRevisionId, getrandom::Error> {
        Ok(DefinitionRevisionId::new(random_value()?).expect("UUID-v4 identity is nonzero"))
    }
    /// Generate a placed subsystem occurrence ID, not a reusable definition ID.
    /// # Errors
    /// Returns the OS entropy error.
    pub fn subsystem() -> Result<SubsystemId, getrandom::Error> {
        Ok(SubsystemId::new(random_value()?).expect("UUID-v4 identity is nonzero"))
    }
    /// Generate an external interface identity.
    /// # Errors
    /// Returns the OS entropy error.
    pub fn interface_port() -> Result<InterfacePortId, getrandom::Error> {
        Ok(InterfacePortId::new(random_value()?).expect("UUID-v4 identity is nonzero"))
    }
    /// Generate a cable assembly identity.
    /// # Errors
    /// Returns the OS entropy error.
    pub fn cable() -> Result<CableId, getrandom::Error> {
        Ok(CableId::new(random_value()?).expect("UUID-v4 identity is nonzero"))
    }
    /// Generate a project lineage ID.
    /// # Errors
    /// Returns the OS entropy error; no fallback to a weak or fixed source is used.
    pub fn project() -> Result<ProjectId, getrandom::Error> {
        Ok(ProjectId::new(random_value()?)
            .expect("UUID-v4 version bits guarantee a nonzero identity"))
    }

    /// Generate a device occurrence ID.
    /// # Errors
    /// Returns the OS entropy error.
    pub fn device() -> Result<DeviceId, getrandom::Error> {
        Ok(DeviceId::new(random_value()?)
            .expect("UUID-v4 version bits guarantee a nonzero identity"))
    }

    /// Generate a terminal ID.
    /// # Errors
    /// Returns the OS entropy error.
    pub fn terminal() -> Result<TerminalId, getrandom::Error> {
        Ok(TerminalId::new(random_value()?)
            .expect("UUID-v4 version bits guarantee a nonzero identity"))
    }

    /// Generate a conductor ID.
    /// # Errors
    /// Returns the OS entropy error.
    pub fn conductor() -> Result<ConductorId, getrandom::Error> {
        Ok(ConductorId::new(random_value()?)
            .expect("UUID-v4 version bits guarantee a nonzero identity"))
    }
}

fn random_value() -> Result<u128, getrandom::Error> {
    value_with(getrandom::fill)
}

fn value_with(
    fill: impl FnOnce(&mut [u8]) -> Result<(), getrandom::Error>,
) -> Result<u128, getrandom::Error> {
    let mut bytes = [0; 16];
    fill(&mut bytes)?;
    Ok(uuid::Builder::from_random_bytes(bytes)
        .into_uuid()
        .as_u128())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatter_sets_version_and_variant_without_discarding_supplied_randomness() {
        let value = value_with(|bytes| {
            bytes.fill(0xab);
            Ok(())
        })
        .unwrap();
        let id = uuid::Uuid::from_u128(value);
        assert_eq!(id.get_version_num(), 4);
        assert_eq!(id.get_variant(), uuid::Variant::RFC4122);
        assert_eq!(id.as_bytes()[0], 0xab);
        assert!(ProjectId::new(value).is_ok());
    }

    #[test]
    fn entropy_failure_is_returned_without_generating_an_identity() {
        assert_eq!(
            value_with(|_| Err(getrandom::Error::UNSUPPORTED)),
            Err(getrandom::Error::UNSUPPORTED)
        );
    }
}
