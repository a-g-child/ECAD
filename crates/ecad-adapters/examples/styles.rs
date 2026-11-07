//! Distinguish copied creation presets from live per-property style inheritance.

use ecad_adapters::{JsonFile, RandomIds};
use ecad_app::{
    Command, Document, DrawingCommand, PropertyEdit, property_edit_plan, property_patch_plan,
};
use ecad_domain::*;
use std::collections::BTreeMap;

fn draw(
    document: &mut Document<JsonFile>,
    command: DrawingCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    document.execute(document.session().revision(), Command::Drawing(command))?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let store = JsonFile::new(directory.path().join("styles.json"));
    let mut document = Document::new(Project::new(RandomIds::project()?), store.clone());
    let view = RandomIds::view()?;
    let style = RandomIds::style()?;
    draw(
        &mut document,
        DrawingCommand::InsertView(DrawingView {
            id: view,
            name: "Overview".into(),
            overrides: StylePatch::default(),
        }),
    )?;
    draw(
        &mut document,
        DrawingCommand::InsertStyle(DrawingStyle::new(
            style,
            "Annotations",
            DrawingRole::Text,
            StylePatch {
                text_height: Some(DrawingLength::new(3_000)?),
                ..StylePatch::default()
            },
        )?),
    )?;
    let mut ids = Vec::new();
    for (name, preset) in [
        ("Captured default", Some(4_000)),
        ("New default", Some(5_000)),
        ("Inherited", None),
        ("Explicit equal value", None),
    ] {
        let mut settings = document.session().project().drawing().settings().clone();
        settings.text_creation = Appearance {
            style: Some(style),
            overrides: StylePatch {
                text_height: preset.map(DrawingLength::new).transpose()?,
                ..StylePatch::default()
            },
        };
        draw(&mut document, DrawingCommand::Settings(settings))?;
        let id = RandomIds::label()?;
        draw(
            &mut document,
            DrawingCommand::CreateLabel {
                id,
                view,
                text: name.into(),
                position: DrawingPoint::default(),
            },
        )?;
        ids.push(id);
    }
    let target = ObjectRef::Label(ids[3]);
    let plan = property_edit_plan(
        document.session().project(),
        [target],
        BTreeMap::from([(
            PropertyId::TextHeight,
            PropertyValue::Length(DrawingLength::new(3_000)?),
        )]),
    )?;
    if plan.changes.len() != 1 {
        return Err("equal override intent was lost".into());
    }
    document.execute_batch(document.session().revision(), plan.commands)?;
    draw(
        &mut document,
        DrawingCommand::ReplaceStyle(DrawingStyle::new(
            style,
            "Annotations",
            DrawingRole::Text,
            StylePatch {
                text_height: Some(DrawingLength::new(6_000)?),
                ..StylePatch::default()
            },
        )?),
    )?;
    for (id, expected) in ids.iter().zip([4_000, 5_000, 6_000, 3_000]) {
        if document
            .session()
            .project()
            .property(ObjectRef::Label(*id), PropertyId::TextHeight)?
            .value
            != PropertyValue::Length(DrawingLength::new(expected)?)
        {
            return Err("creation/style precedence changed".into());
        }
    }
    document.save()?;
    let before = document.session().project().clone();
    let reset = property_patch_plan(
        &before,
        [target],
        BTreeMap::from([(PropertyId::TextHeight, PropertyEdit::Reset)]),
    )?;
    document.execute_batch(document.session().revision(), reset.commands)?;
    if document
        .session()
        .project()
        .property(target, PropertyId::TextHeight)?
        .origin
        != Some(PropertyOrigin::NamedStyle(style))
    {
        return Err("reset did not restore inheritance".into());
    }
    document.undo(document.session().revision())?;
    if document.session().project() != &before
        || document.session().is_dirty()
        || Document::open(store)?.session().project() != &before
    {
        return Err("intent/settings failed exact undo or save/reopen".into());
    }
    println!(
        "Text heights: 4/5 mm captured defaults, 6 mm inherited, 3 mm explicit override. Reset, exact undo, and portable save/reopen verified."
    );
    Ok(())
}
