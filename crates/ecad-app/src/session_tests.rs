//! Internal boundary conditions unavailable through the public session API.

use super::{EditError, Session};
use crate::Command;
use ecad_domain::{Device, DeviceId, Project};

#[test]
fn exhausted_revision_rejects_before_document_or_history_mutation() {
    let mut session = Session::new(Project::new(ecad_domain::ProjectId::new(1).unwrap()));
    session.revision = session.revision.exhausted();
    let device = Device::new(DeviceId::new(1).unwrap(), "-X1", []).unwrap();
    assert_eq!(
        session.execute(session.revision(), Command::InsertDevice(device)),
        Err(EditError::RevisionExhausted)
    );
    assert_eq!(session.project().devices().count(), 0);
    assert_eq!(session.undo_len(), 0);
    assert_eq!(session.revision().get(), u64::MAX);
}
