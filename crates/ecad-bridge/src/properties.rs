//! Full union inspector projection and the bounded set of typed editors shipped by the UI.
use crate::dto::*;
use ecad_app::*;
use ecad_domain::*;
use std::collections::{BTreeMap, BTreeSet};
fn colour(c: DrawingColour) -> String {
    format!("#{:02x}{:02x}{:02x}", c.red, c.green, c.blue)
}
fn format_value(v: &PropertyValue) -> String {
    match v {
        PropertyValue::Text(s) => s.clone(),
        PropertyValue::Length(v) => v.micrometres().to_string(),
        PropertyValue::Colour(c) => colour(*c),
        PropertyValue::Point(p) => format!("{}, {}", p.x, p.y),
        PropertyValue::Points(p) => p
            .iter()
            .map(|p| format!("({}, {})", p.x, p.y))
            .collect::<Vec<_>>()
            .join(" → "),
        PropertyValue::Object(o) => format!("{} {}", Target::from(*o).kind, Target::from(*o).id),
        PropertyValue::OptionalObject(o) => o.map_or_else(
            || "None".into(),
            |o| format_value(&PropertyValue::Object(o)),
        ),
        PropertyValue::Objects(o) => o
            .iter()
            .map(|o| format_value(&PropertyValue::Object(*o)))
            .collect::<Vec<_>>()
            .join(", "),
        PropertyValue::Style(id) => id.map_or_else(|| "Default".into(), |id| id.to_string()),
        PropertyValue::RouteMode(mode) => format!("{mode:?}"),
    }
}
pub(crate) fn inspect(
    project: &Project,
    selection: impl IntoIterator<Item = ObjectRef>,
) -> Result<Vec<PropertyRow>, BridgeError> {
    let inspection =
        inspect_properties(project, selection).map_err(|e| BridgeError::new("model", e))?;
    Ok(inspection
        .properties
        .into_iter()
        .map(|(id, p)| {
            let descriptor = id.descriptor();
            let mixed = p.value == SelectionValue::Mixed;
            let value = match &p.value {
                SelectionValue::Uniform(v) => Some(format_value(v)),
                SelectionValue::Mixed => None,
            };
            let (kind, editor) = match descriptor.value_kind {
                PropertyValueKind::Text => ("text", true),
                PropertyValueKind::Length => ("length", true),
                PropertyValueKind::Colour => ("colour", true),
                PropertyValueKind::Point => ("point", true),
                _ => ("reference", false),
            };
            let editable = editor && !p.editable.is_empty() && p.read_only.is_empty();
            let origin = p
                .origins
                .values()
                .map(|o| match o {
                    PropertyOrigin::Project => "Project defaults".into(),
                    PropertyOrigin::NamedStyle(id) => project
                        .drawing()
                        .style(*id)
                        .map(|s| format!("Style: {}", s.name()))
                        .unwrap_or_else(|| "Named style".into()),
                    PropertyOrigin::View(_) => "View override".into(),
                    PropertyOrigin::Object => "Object override".into(),
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ");
            let reason = if !p.read_only.is_empty() {
                p.read_only
                    .values()
                    .map(|r| {
                        match r {
                            PropertyRestriction::Identity => "Stable identity",
                            PropertyRestriction::StructuralOperation => {
                                "Requires a structural operation"
                            }
                            PropertyRestriction::PinnedInstance(_) => {
                                "Pinned subsystem; make local before editing"
                            }
                        }
                        .to_string()
                    })
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ")
            } else if !editor {
                "Use a structural or geometry operation".into()
            } else {
                String::new()
            };
            PropertyRow {
                key: descriptor.key.into(),
                label: descriptor.label.into(),
                kind: kind.into(),
                display: value.clone().unwrap_or_else(|| "Mixed".into()),
                value,
                mixed,
                editable,
                applicable: p.applicable.len(),
                unsupported: p.inapplicable.len(),
                restricted: p.read_only.len(),
                origin,
                reason,
            }
        })
        .collect())
}
pub(crate) fn edit(
    project: &Project,
    selection: impl IntoIterator<Item = ObjectRef>,
    key: &str,
    text: &str,
) -> Result<Vec<Command>, BridgeError> {
    let id = PROPERTY_IDS
        .into_iter()
        .find(|id| id.descriptor().key == key)
        .ok_or_else(|| BridgeError::new("property", "Unknown property"))?;
    let bad = || {
        BridgeError::new(
            "value",
            "Use a valid value in integer micrometres, or #rrggbb for colour",
        )
    };
    let value = match id.descriptor().value_kind {
        PropertyValueKind::Text => PropertyValue::Text(text.into()),
        PropertyValueKind::Length => PropertyValue::Length(
            DrawingLength::new(text.parse().map_err(|_| bad())?)
                .map_err(|e| BridgeError::new("value", e))?,
        ),
        PropertyValueKind::Point => {
            let (x, y) = text.split_once(',').ok_or_else(bad)?;
            PropertyValue::Point(DrawingPoint {
                x: x.trim().parse().map_err(|_| bad())?,
                y: y.trim().parse().map_err(|_| bad())?,
            })
        }
        PropertyValueKind::Colour => {
            if text.len() != 7 || !text.starts_with('#') || !text[1..].is_ascii() {
                return Err(bad());
            }
            let n = u32::from_str_radix(&text[1..], 16).map_err(|_| bad())?;
            let [_, red, green, blue] = n.to_be_bytes();
            PropertyValue::Colour(DrawingColour { red, green, blue })
        }
        _ => {
            return Err(BridgeError::new(
                "unsupported_editor",
                "This field requires a structural or geometry operation",
            ));
        }
    };
    property_edit_plan(project, selection, BTreeMap::from([(id, value)]))
        .map(|plan| plan.commands)
        .map_err(|e| BridgeError::new("model", e))
}
/// Compound style assignments/defaults retain the normal union applicability and exact history.
pub(crate) fn appearance(
    project: &Project,
    selection: impl IntoIterator<Item = ObjectRef>,
    changes: BTreeMap<String, Option<String>>,
) -> Result<Vec<Command>, BridgeError> {
    let mut proposals = BTreeMap::new();
    for (key, value) in changes {
        let id = match key.as_str() {
            "font_family" => PropertyId::FontFamily,
            "text_height" => PropertyId::TextHeight,
            _ => return Err(BridgeError::new("property", "Unsupported font property")),
        };
        let edit =
            match value {
                None => PropertyEdit::Reset,
                Some(value) => PropertyEdit::Set(match id {
                    PropertyId::FontFamily => PropertyValue::Text(value),
                    _ => PropertyValue::Length(
                        DrawingLength::new(value.parse().map_err(|_| {
                            BridgeError::new("value", "Use a positive text height")
                        })?)
                        .map_err(|e| BridgeError::new("value", e))?,
                    ),
                }),
            };
        proposals.insert(id, edit);
    }
    property_patch_plan(project, selection, proposals)
        .map(|p| p.commands)
        .map_err(|e| BridgeError::new("model", e))
}
pub(crate) fn stroke(project: &Project, target: ObjectRef) -> (String, f64) {
    let colour = match project
        .property(target, PropertyId::DrawingColour)
        .map(|v| v.value)
    {
        Ok(PropertyValue::Colour(c)) => colour(c),
        _ => "#343c43".into(),
    };
    let width = match project
        .property(target, PropertyId::StrokeWidth)
        .map(|v| v.value)
    {
        Ok(PropertyValue::Length(v)) => f64::from(v.micrometres()) / 1000.0,
        _ => 0.25,
    };
    (colour, width)
}
