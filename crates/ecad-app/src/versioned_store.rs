//! Optimistic IO contracts; store versions are adapter-owned, not document data.

use ecad_domain::Project;

/// A validated project and the exact storage version from which it was read.
pub struct ProjectSnapshot<V> {
    /// Completely loaded electrical model.
    pub project: Project,
    /// Opaque observed storage revision, including location where appropriate.
    pub version: V,
}

/// Persistence supporting explicit observed-version checks before replacement.
///
/// Unlike low-level [`crate::ProjectStore`], this contract lets the document
/// controller reject unobserved destinations and externally modified files.
/// Implementations must document their race/atomicity limits.
pub trait VersionedProjectStore {
    /// Opaque token identifying the observed bytes and destination.
    type Version: Clone + Eq;
    /// Concrete IO, conflict, schema, or model failure.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Read and validate one snapshot, deriving its token from the same data.
    /// # Errors
    /// Reports missing/unreadable files or invalid project content.
    fn load_snapshot(&self) -> Result<ProjectSnapshot<Self::Version>, Self::Error>;

    /// Inspect the stored revision without requiring valid project content.
    /// `None` means the destination does not exist.
    /// # Errors
    /// Reports failures other than an absent destination.
    fn current_version(&self) -> Result<Option<Self::Version>, Self::Error>;

    /// Save only when the destination still matches `expected`.
    /// `None` permits creating a missing file, never overwriting an existing one.
    /// # Errors
    /// Reports conflicts, encoding failures, or IO errors without claiming a save.
    fn save_if_unchanged(
        &self,
        project: &Project,
        expected: Option<&Self::Version>,
    ) -> Result<Self::Version, Self::Error>;
}
