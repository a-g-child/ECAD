//! Process-local session epochs distinguish reopened documents at equal revisions.

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_EPOCH: AtomicU64 = AtomicU64::new(1);

/// Unique identity for an editing session within the current process.
///
/// This is not a persistent project ID, reusable-definition revision, or token for
/// communication across application restarts. Never persist it as document data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionEpoch(u64);

impl SessionEpoch {
    /// Return the process-local epoch for diagnostics and transport conversion.
    pub fn get(self) -> u64 {
        self.0
    }
}

/// A session epoch and monotonic sequence; both must match when editing.
///
/// Sequence numbers advance through edits, undo, and redo. A new session receives
/// a different epoch even if it opens the same project at sequence zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Revision {
    epoch: SessionEpoch,
    sequence: u64,
}

impl Revision {
    /// Return the monotonic sequence, meaningful only with [`Self::epoch`].
    pub fn get(self) -> u64 {
        self.sequence
    }

    /// Return the editing-session identity accompanying this sequence.
    pub fn epoch(self) -> SessionEpoch {
        self.epoch
    }

    /// Allocate an epoch; exhaustion is a process-internal invariant failure.
    pub(crate) fn fresh() -> Self {
        let epoch = NEXT_EPOCH
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .expect("a process cannot create u64::MAX editing sessions");
        Self {
            epoch: SessionEpoch(epoch),
            sequence: 0,
        }
    }

    pub(crate) fn next(self) -> Option<Self> {
        self.sequence
            .checked_add(1)
            .map(|sequence| Self { sequence, ..self })
    }

    #[cfg(test)]
    pub(crate) fn exhausted(self) -> Self {
        Self {
            sequence: u64::MAX,
            ..self
        }
    }
}
