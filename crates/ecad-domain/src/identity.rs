//! Opaque, typed identities with canonical text representations.

use std::{error::Error, fmt, str::FromStr};

/// An ID was zero or was not exactly 32 lowercase hexadecimal digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdError;

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("identity must be a nonzero 128-bit value encoded as 32 lowercase hex digits")
    }
}

impl Error for IdError {}

macro_rules! identity {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u128);

        impl $name {
            /// Wrap a nonzero identity supplied by the caller.
            ///
            /// Callers are responsible for uniqueness; this does not generate IDs.
            /// # Errors
            /// Returns [`IdError`] for zero.
            pub fn new(value: u128) -> Result<Self, IdError> {
                if value == 0 {
                    Err(IdError)
                } else {
                    Ok(Self(value))
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:032x}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                if value.len() != 32
                    || !value
                        .bytes()
                        .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
                {
                    return Err(IdError);
                }
                Self::new(u128::from_str_radix(value, 16).map_err(|_| IdError)?)
            }
        }
    };
}

identity!(
    ProjectId,
    "Persistent identity of one project lineage, independent of its file path."
);
identity!(
    DeviceId,
    "Identity of one device occurrence; independent of its designation."
);
identity!(
    TerminalId,
    "Project-wide identity of a device terminal; independent of its number."
);
identity!(
    ConductorId,
    "Identity of an individual conductor; independent of its drawing."
);
identity!(
    SubsystemId,
    "Identity of a placed subsystem occurrence, never a reusable definition ID."
);
identity!(
    InterfacePortId,
    "Identity of one subsystem interface exposure of an existing terminal."
);
identity!(
    CableId,
    "Identity of a cable assembly grouping existing conductors."
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identities_have_one_canonical_nonzero_encoding() {
        let id = DeviceId::new(0xabcdef).unwrap();
        assert_eq!(id.to_string(), "00000000000000000000000000abcdef");
        assert_eq!(id.to_string().parse(), Ok(id));
        for invalid in [
            "1",
            "00000000000000000000000000000000",
            "00000000000000000000000000ABCDEF",
        ] {
            assert_eq!(invalid.parse::<DeviceId>(), Err(IdError));
        }
    }
}
