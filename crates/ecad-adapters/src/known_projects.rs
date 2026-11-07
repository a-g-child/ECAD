//! Explicit registered-file discovery; read-only IO and an optional in-memory cache.

use crate::{FileVersion, JsonFile, ProjectFileError};
use ecad_app::{KnownUseIndex, UsageRefreshError, UsageStatus};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// One failed registered-file operation; other registrations remain discoverable.
pub struct UsageScanFailure {
    /// Canonical registered location that could not be inspected.
    pub path: PathBuf,
    /// Original typed load/check failure, with preserved source information.
    pub error: UsageRefreshError<ProjectFileError>,
}

/// Explicit scan coverage; callers should show failures rather than implying completeness.
pub struct UsageScanReport {
    /// Number of registered locations attempted by this operation.
    pub attempted: usize,
    /// Per-location failures; successful locations still update their summaries/statuses.
    pub failures: Vec<UsageScanFailure>,
}

/// A rebuildable index of explicitly registered local project files.
///
/// Registration resolves existing file aliases to one canonical path; missing files
/// resolve through their existing canonical parent. Copies/worktrees remain separate
/// locations even when their project IDs match. Hard-link aliases are not deduplicated.
/// Stored paths track resolved locations, not symlink retargets or moved files.
/// No directory recursion, watchers, automatic saves, or durable registry is implied.
#[derive(Default)]
pub struct KnownProjects {
    index: KnownUseIndex<PathBuf, FileVersion>,
}
impl KnownProjects {
    /// Register one file without reading its project contents; return its stable lookup key.
    /// Re-registering a canonical alias preserves the prior observation.
    /// # Errors
    /// Rejects invalid paths, directories, dangling symlinks, or inaccessible/missing parents.
    pub fn register(&mut self, path: impl AsRef<Path>) -> Result<PathBuf, io::Error> {
        let path = path.as_ref();
        let key = match fs::metadata(path) {
            Ok(metadata) if metadata.is_file() => path.canonicalize()?,
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "register a project file, not a directory",
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if fs::symlink_metadata(path)
                    .is_ok_and(|metadata| metadata.file_type().is_symlink())
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "register a resolved project location, not a dangling symlink",
                    ));
                }
                let name = path.file_name().ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "project path needs a file name",
                    )
                })?;
                let parent = path
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or(Path::new("."));
                parent.canonicalize()?.join(name)
            }
            Err(error) => return Err(error),
        };
        self.index.register(key.clone());
        Ok(key)
    }
    /// Forget a previously returned registration key without touching its file.
    pub fn unregister(&mut self, key: &Path) -> bool {
        self.index.unregister(&key.to_path_buf())
    }
    /// Inspect registration coverage, statuses, and last-known usage without performing IO.
    pub fn index(&self) -> &KnownUseIndex<PathBuf, FileVersion> {
        &self.index
    }
    /// Refresh exactly one registration key through validated, read-only snapshot loading.
    /// # Errors
    /// Unknown keys perform no IO. Failed loads retain labelled historical usage.
    pub fn refresh(&mut self, key: &Path) -> Result<(), UsageRefreshError<ProjectFileError>> {
        self.index.refresh(&key.to_path_buf(), &JsonFile::new(key))
    }
    /// Check observed content freshness for one registration key without changing any file.
    /// # Errors
    /// Unknown keys perform no IO; IO failures retain labelled historical usage.
    pub fn check_freshness(
        &mut self,
        key: &Path,
    ) -> Result<UsageStatus, UsageRefreshError<ProjectFileError>> {
        self.index
            .check_freshness(&key.to_path_buf(), &JsonFile::new(key))
    }
    /// Rescan only registered files; a failed location never aborts successful observations.
    pub fn rescan(&mut self) -> UsageScanReport {
        self.scan(false)
    }
    /// Compare content versions of all registrations; changes no cached usage bodies.
    pub fn check_all(&mut self) -> UsageScanReport {
        self.scan(true)
    }
    fn scan(&mut self, check_only: bool) -> UsageScanReport {
        let keys: Vec<_> = self.index.entries().map(|(key, _)| key.clone()).collect();
        let mut report = UsageScanReport {
            attempted: keys.len(),
            failures: Vec::new(),
        };
        for key in keys {
            let result = if check_only {
                self.check_freshness(&key).map(|_| ())
            } else {
                self.refresh(&key)
            };
            if let Err(error) = result {
                report.failures.push(UsageScanFailure { path: key, error });
            }
        }
        report
    }
}
