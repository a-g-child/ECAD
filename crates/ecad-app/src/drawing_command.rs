//! Localized reversible drawing operations; creation captures current project presets.

use ecad_domain::*;

/// One committed drawing operation, independent of interactive tools or rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrawingCommand {
    /// Set portable project fallbacks and future-object creation presets.
    Settings(DrawingSettings),
    /// Insert a named style.
    InsertStyle(DrawingStyle),
    /// Update a named style without rewriting its dependents.
    ReplaceStyle(DrawingStyle),
    /// Remove an unused named style.
    RemoveStyle(StyleId),
    /// Insert one drawing view.
    InsertView(DrawingView),
    /// Update view name/overrides without rewriting objects.
    ReplaceView(DrawingView),
    /// Remove an empty drawing view.
    RemoveView(ViewId),
    /// Insert exact annotation state, preserving imported/undo appearance intent.
    InsertLabel(DrawingLabel),
    /// Update exact annotation state with aggregate validation.
    ReplaceLabel(DrawingLabel),
    /// Remove one annotation.
    RemoveLabel(LabelId),
    /// Create an annotation using the current text creation preset.
    CreateLabel {
        /// Fresh annotation identity.
        id: LabelId,
        /// Existing drawing view.
        view: ViewId,
        /// Literal nonblank annotation text.
        text: String,
        /// Location in document units.
        position: DrawingPoint,
    },
    /// Insert exact whole-path state.
    InsertPolyline(DrawingPolyline),
    /// Update whole-path state while preserving identity.
    ReplacePolyline(DrawingPolyline),
    /// Remove a whole graphical path.
    RemovePolyline(PolylineId),
    /// Create a graphical path using the current stroke creation preset.
    CreatePolyline {
        /// Fresh path identity.
        id: PolylineId,
        /// Existing drawing view.
        view: ViewId,
        /// Ordered whole-path geometry in document units.
        points: Vec<DrawingPoint>,
    },
}
impl DrawingCommand {
    pub(crate) fn apply(&self, project: &mut Project) -> Result<Option<Self>, ModelError> {
        let inverse = match self {
            Self::Settings(settings) => {
                let old = project.set_drawing_settings(settings.clone())?;
                if old == *settings {
                    return Ok(None);
                }
                Self::Settings(old)
            }
            Self::InsertStyle(style) => {
                project.insert_drawing_style(style.clone())?;
                Self::RemoveStyle(style.id())
            }
            Self::ReplaceStyle(style) => {
                let old = project.replace_drawing_style(style.clone())?;
                if old == *style {
                    return Ok(None);
                }
                Self::ReplaceStyle(old)
            }
            Self::RemoveStyle(id) => Self::InsertStyle(project.remove_drawing_style(*id)?),
            Self::InsertView(view) => {
                project.insert_drawing_view(view.clone())?;
                Self::RemoveView(view.id)
            }
            Self::ReplaceView(view) => {
                let old = project.replace_drawing_view(view.clone())?;
                if old == *view {
                    return Ok(None);
                }
                Self::ReplaceView(old)
            }
            Self::RemoveView(id) => Self::InsertView(project.remove_drawing_view(*id)?),
            Self::InsertLabel(label) => {
                project.insert_drawing_label(label.clone())?;
                Self::RemoveLabel(label.id)
            }
            Self::ReplaceLabel(label) => {
                let old = project.replace_drawing_label(label.clone())?;
                if old == *label {
                    return Ok(None);
                }
                Self::ReplaceLabel(old)
            }
            Self::RemoveLabel(id) => Self::InsertLabel(project.remove_drawing_label(*id)?),
            Self::CreateLabel {
                id,
                view,
                text,
                position,
            } => {
                let label = DrawingLabel {
                    id: *id,
                    view: *view,
                    text: text.clone(),
                    position: *position,
                    appearance: project.drawing().settings().text_creation.clone(),
                };
                project.insert_drawing_label(label)?;
                Self::RemoveLabel(*id)
            }
            Self::InsertPolyline(line) => {
                project.insert_drawing_polyline(line.clone())?;
                Self::RemovePolyline(line.id)
            }
            Self::ReplacePolyline(line) => {
                let old = project.replace_drawing_polyline(line.clone())?;
                if old == *line {
                    return Ok(None);
                }
                Self::ReplacePolyline(old)
            }
            Self::RemovePolyline(id) => Self::InsertPolyline(project.remove_drawing_polyline(*id)?),
            Self::CreatePolyline { id, view, points } => {
                let line = DrawingPolyline {
                    id: *id,
                    view: *view,
                    points: points.clone(),
                    appearance: project.drawing().settings().stroke_creation.clone(),
                };
                project.insert_drawing_polyline(line)?;
                Self::RemovePolyline(*id)
            }
        };
        Ok(Some(inverse))
    }
}
