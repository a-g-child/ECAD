//! Functional overview and nested wiring built as one undoable document operation.

use ecad_adapters::{JsonFile, RandomIds};
use ecad_app::{Command, Document};
use ecad_domain::{
    Cable, CableEndpoint, Conductor, Device, End, InterfacePort, Project, Subsystem, Terminal,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conveyor = RandomIds::subsystem()?;
    let sensor_assembly = RandomIds::subsystem()?;
    let controller = RandomIds::device()?;
    let junction = RandomIds::device()?;
    let sensor = RandomIds::device()?;
    let controller_pin = RandomIds::terminal()?;
    let junction_pin = RandomIds::terminal()?;
    let sensor_pin = RandomIds::terminal()?;
    let input_port = RandomIds::interface_port()?;
    let external_wire = RandomIds::conductor()?;
    let internal_wire = RandomIds::conductor()?;
    let cable = RandomIds::cable()?;
    let project = Project::new(RandomIds::project()?);
    let directory = tempfile::tempdir()?;
    let store = JsonFile::new(directory.path().join("conveyor.json"));
    let mut document = Document::new(project.clone(), store.clone());
    let commands = vec![
        Command::InsertSubsystem(Subsystem::new(conveyor, "Conveyor", None)?),
        Command::InsertSubsystem(Subsystem::new(
            sensor_assembly,
            "Sensor assembly",
            Some(conveyor),
        )?),
        Command::InsertDevice(Device::new(
            controller,
            "-Controller",
            [Terminal::new(controller_pin, "1")?],
        )?),
        Command::InsertDevice(
            Device::new(junction, "-Junction", [Terminal::new(junction_pin, "1")?])?
                .in_subsystem(conveyor),
        ),
        Command::InsertDevice(
            Device::new(sensor, "-Sensor", [Terminal::new(sensor_pin, "1")?])?
                .in_subsystem(sensor_assembly),
        ),
        Command::InsertPort(InterfacePort::new(
            input_port,
            conveyor,
            "Sensor input",
            junction_pin,
        )?),
        Command::InsertPort(InterfacePort::new(
            RandomIds::interface_port()?,
            sensor_assembly,
            "Signal",
            sensor_pin,
        )?),
        Command::InsertConductor(Conductor::new(
            external_wire,
            Some(controller_pin),
            Some(junction_pin),
        )),
        Command::InsertConductor(Conductor::new(
            internal_wire,
            Some(junction_pin),
            Some(sensor_pin),
        )),
        Command::InsertCable(Cable::new(
            cable,
            "-C1",
            CableEndpoint::Device(controller),
            CableEndpoint::Subsystem(conveyor),
            [external_wire],
        )?),
    ];
    document.execute_batch(document.session().revision(), commands)?;
    document.undo(document.session().revision())?;
    if document.session().project() != &project {
        return Err("construction undo did not restore the empty project".into());
    }
    document.redo(document.session().revision())?;
    document.save()?;
    let reopened = Document::open(store)?;
    let model = reopened.session().project();
    if model != document.session().project() {
        return Err("hierarchy changed during save/reopen".into());
    }
    let core = model
        .conductor(external_wire)
        .ok_or("cable core disappeared")?;
    let port = model.port(input_port).ok_or("interface disappeared")?;
    if core.endpoint(End::B) != Some(port.terminal()) {
        return Err("overview and detailed wiring diverged".into());
    }
    println!("Controller -- cable -C1 --> Conveyor --> Junction --> Sensor assembly / Sensor");
    println!(
        "Verified {} blocks, {} devices, {} conductors, {} interfaces and {} cable; one-step undo/redo and guarded save/reopen passed.",
        model.subsystems().count(),
        model.devices().count(),
        model.conductors().count(),
        model.ports().count(),
        model.cables().count()
    );
    Ok(())
}
