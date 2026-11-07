//! Logical-character key sequences with explicit portable primary-modifier resolution.

use crate::{ActionConfigError, ActionContext};
use std::fmt;

/// Target modifier convention; selecting this does not claim native platform validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutPlatform {
    /// Primary resolves to Control.
    Linux,
    /// Primary resolves to Control.
    Windows,
    /// Primary resolves to Meta/Command.
    MacOS,
}
/// Named logical keys used alongside printable character mnemonics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NamedKey {
    /// Escape/cancel.
    Escape,
    /// Enter/confirm.
    Enter,
    /// Forward delete.
    Delete,
    /// Backward delete.
    Backspace,
    /// Focus traversal.
    Tab,
    /// Space character as an explicit named key.
    Space,
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
}
/// Logical keys; ASCII letters normalize to lowercase, while Shift stays explicit.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShortcutKey(KeyValue);
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum KeyValue {
    Character(char),
    Named(NamedKey),
    Function(u8),
}
impl ShortcutKey {
    /// Use a logical printable character. Non-ASCII scalars retain exact spelling.
    /// # Errors
    /// Rejects whitespace/control characters; Space has a named representation.
    pub fn character(value: char) -> Result<Self, ActionConfigError> {
        if value.is_control() || value.is_whitespace() {
            return Err(ActionConfigError::InvalidKey);
        }
        Ok(Self(KeyValue::Character(value.to_ascii_lowercase())))
    }
    /// Use a named non-character key.
    pub fn named(value: NamedKey) -> Self {
        Self(KeyValue::Named(value))
    }
    /// Bind a logical function key F1 through F24.
    /// # Errors
    /// Rejects zero and unsupported function-key numbers.
    pub fn function(number: u8) -> Result<Self, ActionConfigError> {
        if !(1..=24).contains(&number) {
            return Err(ActionConfigError::InvalidKey);
        }
        Ok(Self(KeyValue::Function(number)))
    }
    /// Inspect a logical function key, if present.
    pub fn as_function(&self) -> Option<u8> {
        match self.0 {
            KeyValue::Function(number) => Some(number),
            _ => None,
        }
    }
    /// Inspect the logical character, if present.
    pub fn as_character(&self) -> Option<char> {
        match self.0 {
            KeyValue::Character(c) => Some(c),
            _ => None,
        }
    }
    /// Inspect the named key, if present.
    pub fn as_named(&self) -> Option<NamedKey> {
        match self.0 {
            KeyValue::Named(k) => Some(k),
            _ => None,
        }
    }
}
impl fmt::Display for ShortcutKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            KeyValue::Character(c) => write!(f, "{}", c.to_ascii_uppercase()),
            KeyValue::Named(key) => write!(f, "{key:?}"),
            KeyValue::Function(number) => write!(f, "F{number}"),
        }
    }
}
/// Declarative modifier intent; Primary is resolved once against the selected platform.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShortcutModifiers {
    /// Control on Linux/Windows; Meta on macOS.
    pub primary: bool,
    /// Explicit Control.
    pub control: bool,
    /// Explicit Meta/Command.
    pub meta: bool,
    /// Alt/Option.
    pub alt: bool,
    /// Shift, separate from character normalization.
    pub shift: bool,
}
/// Normalized host modifiers; this value contains no platform-relative Primary flag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalModifiers {
    /// Control is pressed.
    pub control: bool,
    /// Meta/Command is pressed.
    pub meta: bool,
    /// Alt/Option is pressed.
    pub alt: bool,
    /// Shift is pressed.
    pub shift: bool,
}
/// One declared simultaneous chord; a sequence is an ordered list of these values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutChord {
    /// Logical key, independent of physical keyboard position.
    pub key: ShortcutKey,
    /// Portable or explicit modifier intent.
    pub modifiers: ShortcutModifiers,
}
/// A normalized event/effective-keymap chord.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResolvedChord {
    /// Normalized logical key.
    pub key: ShortcutKey,
    /// Actual host modifier state.
    pub modifiers: PhysicalModifiers,
}
impl ShortcutChord {
    /// Resolve portable modifier intent to concrete host flags.
    pub fn resolve(&self, platform: ShortcutPlatform) -> ResolvedChord {
        let primary_meta = platform == ShortcutPlatform::MacOS;
        ResolvedChord {
            key: self.key.clone(),
            modifiers: PhysicalModifiers {
                control: self.modifiers.control || (self.modifiers.primary && !primary_meta),
                meta: self.modifiers.meta || (self.modifiers.primary && primary_meta),
                alt: self.modifiers.alt,
                shift: self.modifiers.shift,
            },
        }
    }
}
/// Validated nonempty ordered key sequence, with no arbitrary action/sequence-count cap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeySequence(Vec<ShortcutChord>);
impl KeySequence {
    /// Construct a nonempty ordered sequence.
    /// # Errors
    /// Rejects an empty sequence.
    pub fn new(chords: Vec<ShortcutChord>) -> Result<Self, ActionConfigError> {
        if chords.is_empty() {
            return Err(ActionConfigError::EmptySequence);
        }
        Ok(Self(chords))
    }
    /// Inspect declarative chords in input order.
    pub fn chords(&self) -> &[ShortcutChord] {
        &self.0
    }
    /// Resolve each chord under one modifier convention.
    pub fn resolve(&self, platform: ShortcutPlatform) -> Vec<ResolvedChord> {
        self.0.iter().map(|chord| chord.resolve(platform)).collect()
    }
}
impl ResolvedChord {
    /// Produce shortcut text for menus/help from the effective, normalized keymap.
    pub fn display(&self, platform: ShortcutPlatform) -> String {
        let mut parts = Vec::new();
        if self.modifiers.control {
            parts.push("Ctrl".to_owned());
        }
        if self.modifiers.meta {
            parts.push(
                if platform == ShortcutPlatform::MacOS {
                    "Cmd"
                } else {
                    "Meta"
                }
                .to_owned(),
            );
        }
        if self.modifiers.alt {
            parts.push(
                if platform == ShortcutPlatform::MacOS {
                    "Option"
                } else {
                    "Alt"
                }
                .to_owned(),
            );
        }
        if self.modifiers.shift {
            parts.push("Shift".to_owned());
        }
        parts.push(self.key.to_string());
        parts.join("+")
    }
}
pub(crate) fn protected_focus(context: &ActionContext) -> bool {
    context.composing
        || context.control_owns_key
        || matches!(
            context.focus,
            crate::ActionFocus::TextInput | crate::ActionFocus::Modal
        )
}
