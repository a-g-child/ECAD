//! Generated identities, a compound edit, and guarded save/reopen without a desktop.

use ecad_adapters::{JsonFile, RandomIds, encode_project};
use ecad_app::{Command, Document};
use ecad_domain::{Conductor, Device, Project, Terminal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let store = JsonFile::new(directory.path().join("example.json"));
    let project = Project::new(RandomIds::project()?);
    let mut document = Document::new(project, store.clone());
    let terminals = [RandomIds::terminal()?, RandomIds::terminal()?];
    let mut commands = Vec::new();
    for (terminal_id, name) in terminals.into_iter().zip(["-Controller", "-Lamp"]) {
        let terminal = Terminal::new(terminal_id, "1")?;
        let device = Device::new(RandomIds::device()?, name, [terminal])?;
        commands.push(Command::InsertDevice(device));
    }
    let conductor = Conductor::new(
        RandomIds::conductor()?,
        Some(terminals[0]),
        Some(terminals[1]),
    );
    commands.push(Command::InsertConductor(conductor));
    document.execute_batch(document.session().revision(), commands)?;
    document.undo(document.session().revision())?;
    document.redo(document.session().revision())?;
    document.save()?;
    let reopened = Document::open(store)?;
    if reopened.session().project() != document.session().project()
        || document.session().is_dirty()
        || reopened.session().is_dirty()
    {
        return Err("saved model changed during reopening".into());
    }
    println!(
        "Generated project/object IDs; created one transaction, undid/redid it, and reopened the guarded save cleanly.\n"
    );
    print!("{}", encode_project(reopened.session().project())?);
    Ok(())
}
