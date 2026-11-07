//! Shared electrical fixtures for versioned-document lifecycle tests.

use ecad_adapters::{JsonFile, decode_project};
use ecad_app::{Command, ProjectStore};
use ecad_domain::{Device, DeviceId, Project};
use std::path::Path;

pub fn fixture() -> Project {
    decode_project(include_str!("../fixtures/minimal-v4.json")).unwrap()
}

pub fn seed(path: &Path) -> JsonFile {
    let store = JsonFile::new(path);
    store.save(&fixture()).unwrap();
    store
}

pub fn device(value: u128) -> Command {
    Command::InsertDevice(
        Device::new(DeviceId::new(value).unwrap(), format!("-X{value}"), []).unwrap(),
    )
}
