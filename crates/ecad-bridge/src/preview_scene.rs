//! Presentation of closed layout geometry, shared by movement and orthogonal transforms.
use crate::dto::*;
use ecad_app::*;
use ecad_domain::*;
use std::collections::BTreeMap;
pub(crate) fn capture(
    session: &Session,
    context: &ActionContext,
    scene: Vec<SceneRow>,
) -> Result<BTreeMap<ObjectRef, SceneRow>, BridgeError> {
    // Capture touching routes even when the seed proposal leaves their geometry unchanged.
    let mut targets = context.selection.clone();
    let anchors = context.selection.iter().filter_map(|target| match target {
        ObjectRef::Anchor(id) => Some(*id),
        _ => None,
    });
    let touching = session
        .project()
        .drawing()
        .route_layout(anchors)
        .map_err(|e| BridgeError::new("move", e))?;
    targets.extend(touching.routes.iter().map(|r| ObjectRef::Route(r.id)));
    let scene = scene
        .into_iter()
        .filter_map(|row| {
            let target = row.target.object().ok()?;
            targets.contains(&target).then_some((target, row))
        })
        .collect();
    Ok(scene)
}
pub(crate) fn project(
    preview: &LayoutPreview,
    scene_rows: &BTreeMap<ObjectRef, SceneRow>,
) -> Result<Vec<SceneRow>, BridgeError> {
    let mut scene = Vec::new();
    for object in preview.objects() {
        let (target, points) = match object {
            PreviewObject::Label(v) => (ObjectRef::Label(v.id), vec![v.position]),
            PreviewObject::Polyline(v) => (ObjectRef::Polyline(v.id), v.points.clone()),
            PreviewObject::Anchor(v) => (ObjectRef::Anchor(v.id), vec![v.position]),
            PreviewObject::Route(v) => (ObjectRef::Route(v.id), v.points.clone()),
        };
        let mut row = scene_rows
            .get(&target)
            .cloned()
            .ok_or_else(|| BridgeError::new("move", "Preview object is missing from the scene"))?;
        row.points = points
            .iter()
            .map(|p| [p.x as f64 / 1000.0, p.y as f64 / 1000.0])
            .collect();
        scene.push(row);
    }
    Ok(scene)
}
