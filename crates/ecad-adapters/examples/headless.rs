//! Run the first complete edit/save/reopen workflow without a desktop or remote service.

use ecad_adapters::{JsonFile, encode_project};
use ecad_app::{Command, Session};
use ecad_domain::{Conductor, ConductorId, Device, DeviceId, Project, Terminal, TerminalId};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = Session::new(Project::default());
    let mut commands = Vec::new();
    for (value, name) in [(1, "-Controller"), (2, "-Lamp")] {
        let terminal = Terminal::new(TerminalId::new(value)?, "1")?;
        let device = Device::new(DeviceId::new(value)?, name, [terminal])?;
        commands.push(Command::InsertDevice(device));
    }
    let conductor = Conductor::new(
        ConductorId::new(1)?,
        Some(TerminalId::new(1)?),
        Some(TerminalId::new(2)?),
    );
    commands.push(Command::InsertConductor(conductor));
    session.execute_batch(session.revision(), commands)?;
    session.undo(session.revision())?;
    session.redo(session.revision())?;

    let directory = tempfile::tempdir()?;
    let store = JsonFile::new(directory.path().join("example.json"));
    session.save(&store)?;
    let reopened = Session::open(&store)?;
    if reopened.project() != session.project() || session.is_dirty() || reopened.is_dirty() {
        return Err("saved model changed during reopening".into());
    }
    println!(
        "Two devices and one conductor created as one transaction, undone/redone, saved and reopened cleanly.\n"
    );
    print!("{}", encode_project(reopened.project())?);
    Ok(())
}
