//! Validated persistence pin DTO, distinct from domain and bridge identity representations.
use ecad_domain::{IdError, TemplateVariantRef};
use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "RawPin", into = "RawPin")]
pub(crate) struct PinData(pub(crate) TemplateVariantRef);
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPin {
    definition: String,
    revision: String,
    variant: String,
}
impl TryFrom<RawPin> for PinData {
    type Error = IdError;
    fn try_from(p: RawPin) -> Result<Self, Self::Error> {
        Ok(Self(TemplateVariantRef {
            definition: p.definition.parse()?,
            revision: p.revision.parse()?,
            variant: p.variant.parse()?,
        }))
    }
}
impl From<PinData> for RawPin {
    fn from(p: PinData) -> Self {
        Self {
            definition: p.0.definition.to_string(),
            revision: p.0.revision.to_string(),
            variant: p.0.variant.to_string(),
        }
    }
}
