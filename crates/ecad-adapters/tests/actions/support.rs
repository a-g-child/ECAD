//! Real document-controller fixtures and shared action invocation builders.

use ecad_adapters::JsonFile;
use ecad_app::*;
use ecad_domain::*;
use std::path::Path;

pub fn id(value: &str) -> ActionId {
    ActionId::new(value).unwrap()
}
pub fn document(path: &Path) -> Document<JsonFile> {
    let mut project = Project::new(ProjectId::new(1).unwrap());
    project
        .insert_drawing_view(DrawingView {
            id: ViewId::new(1).unwrap(),
            name: "Overview".into(),
            overrides: StylePatch::default(),
        })
        .unwrap();
    let mut document = Document::new(project, JsonFile::new(path));
    document.save().unwrap();
    document
}
pub fn context(document: &Document<JsonFile>) -> ActionContext {
    ActionContext {
        view: Some(ViewId::new(1).unwrap()),
        anchor: Some(DrawingPoint { x: 100, y: 200 }),
        ..ActionContext::for_session(document.session())
    }
}
pub fn invocation(
    number: u64,
    action: &str,
    context: ActionContext,
    parameters: ActionParameters,
) -> ActionInvocation {
    ActionInvocation {
        input_id: InputId::new(number).unwrap(),
        action: id(action),
        source: InvocationSource::Menu,
        repeat: false,
        context,
        parameters,
    }
}
pub fn label(number: u128) -> ActionParameters {
    ActionParameters::Label {
        id: LabelId::new(number).unwrap(),
        text: format!("Annotation {number}"),
        position: DrawingPoint { x: 100, y: 200 },
    }
}
