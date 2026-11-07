//! Strict portable user keymap schema; preferences stay outside shared project data.

use ecad_app::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, error::Error, fmt};

/// Keymap import/export failed; imported unknown action IDs remain valid data until compilation.
#[derive(Debug)]
pub enum KeymapFileError {
    /// Syntax, field shape, unknown fields, keys, scopes, or modifiers are invalid.
    Json(serde_json::Error),
    /// Schema must not be silently downgraded.
    UnsupportedVersion(u32),
    /// Typed action/key/sequence construction failed or an override/modifier was repeated.
    Configuration(ActionConfigError),
}
impl fmt::Display for KeymapFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(f, "invalid keymap JSON: {error}"),
            Self::UnsupportedVersion(version) => write!(f, "unsupported keymap schema {version}"),
            Self::Configuration(error) => error.fmt(f),
        }
    }
}
impl Error for KeymapFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::Configuration(error) => Some(error),
            _ => None,
        }
    }
}
impl From<serde_json::Error> for KeymapFileError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
impl From<ActionConfigError> for KeymapFileError {
    fn from(error: ActionConfigError) -> Self {
        Self::Configuration(error)
    }
}

#[derive(Serialize, Deserialize)]
enum Kind {
    #[serde(rename = "ecad-keymap")]
    Keymap,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct KeymapData {
    kind: Kind,
    schema_version: u32,
    overrides: Vec<OverrideData>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OverrideData {
    action: String,
    bindings: Vec<BindingData>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingData {
    scope: ScopeData,
    sequence: Vec<ChordData>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ScopeData {
    Application,
    Document,
    Canvas,
    Tool,
    ProjectTree,
    Inspector,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ChordData {
    key: KeyData,
    modifiers: Vec<ModifierData>,
}
#[derive(Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum KeyData {
    Character(String),
    Named(NamedData),
    Function(u8),
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NamedData {
    Escape,
    Enter,
    Delete,
    Backspace,
    Tab,
    Space,
    Left,
    Right,
    Up,
    Down,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ModifierData {
    Primary,
    Control,
    Meta,
    Alt,
    Shift,
}

impl From<BindingScope> for ScopeData {
    fn from(scope: BindingScope) -> Self {
        match scope {
            BindingScope::Application => Self::Application,
            BindingScope::Document => Self::Document,
            BindingScope::Canvas => Self::Canvas,
            BindingScope::Tool => Self::Tool,
            BindingScope::ProjectTree => Self::ProjectTree,
            BindingScope::Inspector => Self::Inspector,
        }
    }
}
impl From<ScopeData> for BindingScope {
    fn from(scope: ScopeData) -> Self {
        match scope {
            ScopeData::Application => Self::Application,
            ScopeData::Document => Self::Document,
            ScopeData::Canvas => Self::Canvas,
            ScopeData::Tool => Self::Tool,
            ScopeData::ProjectTree => Self::ProjectTree,
            ScopeData::Inspector => Self::Inspector,
        }
    }
}
impl From<NamedKey> for NamedData {
    fn from(key: NamedKey) -> Self {
        match key {
            NamedKey::Escape => Self::Escape,
            NamedKey::Enter => Self::Enter,
            NamedKey::Delete => Self::Delete,
            NamedKey::Backspace => Self::Backspace,
            NamedKey::Tab => Self::Tab,
            NamedKey::Space => Self::Space,
            NamedKey::Left => Self::Left,
            NamedKey::Right => Self::Right,
            NamedKey::Up => Self::Up,
            NamedKey::Down => Self::Down,
        }
    }
}
impl From<NamedData> for NamedKey {
    fn from(key: NamedData) -> Self {
        match key {
            NamedData::Escape => Self::Escape,
            NamedData::Enter => Self::Enter,
            NamedData::Delete => Self::Delete,
            NamedData::Backspace => Self::Backspace,
            NamedData::Tab => Self::Tab,
            NamedData::Space => Self::Space,
            NamedData::Left => Self::Left,
            NamedData::Right => Self::Right,
            NamedData::Up => Self::Up,
            NamedData::Down => Self::Down,
        }
    }
}
impl From<&ShortcutChord> for ChordData {
    fn from(chord: &ShortcutChord) -> Self {
        let key = if let Some(character) = chord.key.as_character() {
            KeyData::Character(character.to_string())
        } else if let Some(number) = chord.key.as_function() {
            KeyData::Function(number)
        } else {
            KeyData::Named(
                chord
                    .key
                    .as_named()
                    .expect("key is a character or named key")
                    .into(),
            )
        };
        let flags = chord.modifiers;
        let modifiers = [
            (flags.primary, ModifierData::Primary),
            (flags.control, ModifierData::Control),
            (flags.meta, ModifierData::Meta),
            (flags.alt, ModifierData::Alt),
            (flags.shift, ModifierData::Shift),
        ]
        .into_iter()
        .filter_map(|(present, value)| present.then_some(value))
        .collect();
        Self { key, modifiers }
    }
}
impl TryFrom<ChordData> for ShortcutChord {
    type Error = KeymapFileError;
    fn try_from(chord: ChordData) -> Result<Self, Self::Error> {
        let key = match chord.key {
            KeyData::Character(value) => {
                let mut characters = value.chars();
                let first = characters.next().ok_or(ActionConfigError::InvalidKey)?;
                if characters.next().is_some() {
                    return Err(ActionConfigError::InvalidKey.into());
                }
                ShortcutKey::character(first)?
            }
            KeyData::Named(key) => ShortcutKey::named(key.into()),
            KeyData::Function(number) => ShortcutKey::function(number)?,
        };
        let mut modifiers = ShortcutModifiers::default();
        for modifier in chord.modifiers {
            let flag = match modifier {
                ModifierData::Primary => &mut modifiers.primary,
                ModifierData::Control => &mut modifiers.control,
                ModifierData::Meta => &mut modifiers.meta,
                ModifierData::Alt => &mut modifiers.alt,
                ModifierData::Shift => &mut modifiers.shift,
            };
            if *flag {
                return Err(ActionConfigError::DuplicateModifier.into());
            }
            *flag = true;
        }
        Ok(Self { key, modifiers })
    }
}
/// Export canonical schema v1 with stable action order, explicit empty unbindings, and final LF.
/// # Errors
/// Returns JSON encoding failures; does not modify any project or user preference file.
pub fn encode_keymap(overrides: &KeymapOverrides) -> Result<String, KeymapFileError> {
    let data = KeymapData {
        kind: Kind::Keymap,
        schema_version: 1,
        overrides: overrides
            .actions
            .iter()
            .map(|(action, bindings)| OverrideData {
                action: action.to_string(),
                bindings: bindings
                    .iter()
                    .map(|binding| BindingData {
                        scope: binding.scope.into(),
                        sequence: binding
                            .sequence
                            .chords()
                            .iter()
                            .map(ChordData::from)
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
    };
    Ok(serde_json::to_string_pretty(&data)? + "\n")
}
/// Import typed overrides without discarding unknown/retired actions or silently resolving conflicts.
/// Call `compile_keymap` for the selected registry/platform and install only after success.
/// # Errors
/// Rejects future schemas, unknown fields, invalid identities/keys/sequences, repeated actions/modifiers.
pub fn decode_keymap(text: &str) -> Result<KeymapOverrides, KeymapFileError> {
    #[derive(Deserialize)]
    struct Version {
        schema_version: u32,
    }
    let version: Version = serde_json::from_str(text)?;
    if version.schema_version != 1 {
        return Err(KeymapFileError::UnsupportedVersion(version.schema_version));
    }
    let data: KeymapData = serde_json::from_str(text)?;
    let mut actions = BTreeMap::new();
    for entry in data.overrides {
        let action = ActionId::new(entry.action)?;
        if actions.contains_key(&action) {
            return Err(ActionConfigError::DuplicateOverride(action).into());
        }
        let bindings = entry
            .bindings
            .into_iter()
            .map(|binding| {
                Ok(ScopedShortcut {
                    scope: binding.scope.into(),
                    sequence: KeySequence::new(
                        binding
                            .sequence
                            .into_iter()
                            .map(ShortcutChord::try_from)
                            .collect::<Result<Vec<_>, KeymapFileError>>()?,
                    )?,
                })
            })
            .collect::<Result<Vec<_>, KeymapFileError>>()?;
        actions.insert(action, bindings);
    }
    Ok(KeymapOverrides { actions })
}
