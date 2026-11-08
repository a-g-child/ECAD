//! Cooperative writer exclusion and coherent single-file library publication.

use crate::library_lock::{LibraryWriterLock, canonical_library_path, library_lock_path};
use crate::{FileVersion, JsonFile, LibraryFileError, decode_library, encode_library};
use ecad_app::{LibrarySnapshot, VersionedLibraryStore};
use ecad_domain::SubsystemLibrary;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// A canonical-parent single-file shared subsystem library.
///
/// Writers using this adapter acquire an exclusive `create_new` sidecar lock around
/// read/check/stage/replace. Readers see a complete old/new catalog. Uncooperative
/// writers, hard-link aliases, and filesystems without reliable exclusive creation
/// or atomic replacement are outside this guarantee. Destination symlinks are rejected.
/// File contents are synced before rename; directory durability and metadata preservation
/// remain unspecified. A crashed writer can leave a lock: report it rather than steal it.
#[derive(Clone, Debug)]
pub struct LibraryFile {
    path: PathBuf,
}
impl LibraryFile {
    /// Target a library path; its parent must exist when IO is performed.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
    /// Inspect the configured destination.
    pub fn path(&self) -> &Path {
        &self.path
    }
    fn canonical_path(&self) -> Result<PathBuf, LibraryFileError> {
        Ok(canonical_library_path(&self.path)?)
    }
    /// Locate the reserved writer lock for diagnostics; never remove an active lock.
    /// # Errors
    /// Reports an invalid destination or inaccessible parent directory.
    pub fn lock_path(&self) -> Result<PathBuf, LibraryFileError> {
        Ok(library_lock_path(&self.canonical_path()?))
    }
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, LibraryFileError> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}
fn decode_bytes(bytes: &[u8]) -> Result<SubsystemLibrary, LibraryFileError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    decode_library(text)
}

impl VersionedLibraryStore for LibraryFile {
    type Version = FileVersion;
    type Error = LibraryFileError;

    fn load_snapshot(&self) -> Result<LibrarySnapshot<FileVersion>, Self::Error> {
        let path = self.canonical_path()?;
        let bytes = fs::read(&path)?;
        Ok(LibrarySnapshot {
            catalog: decode_bytes(&bytes)?,
            version: FileVersion::for_path(&path, &bytes)?,
        })
    }

    fn save_if_unchanged(
        &self,
        catalog: &SubsystemLibrary,
        expected: Option<&FileVersion>,
    ) -> Result<FileVersion, Self::Error> {
        let path = self.canonical_path()?;
        let lock = library_lock_path(&path);
        let _lock = LibraryWriterLock::acquire(lock.clone()).map_err(|error| {
            if error.kind() == io::ErrorKind::AlreadyExists {
                LibraryFileError::Busy(lock)
            } else {
                error.into()
            }
        })?;
        let bytes = read_optional(&path)?;
        let current = bytes
            .as_ref()
            .map(|bytes| FileVersion::for_path(&path, bytes))
            .transpose()?;
        if current.as_ref() != expected {
            return Err(LibraryFileError::ExternalChange(path));
        }
        if let Some(bytes) = &bytes {
            for previous in decode_bytes(bytes)?.revisions() {
                if catalog.revision(previous.source()) != Some(previous) {
                    return Err(LibraryFileError::HistoryRewrite(previous.source()));
                }
            }
        }
        let text = encode_library(catalog)?;
        let version = FileVersion::for_path(&path, text.as_bytes())?;
        if bytes.as_deref() == Some(text.as_bytes()) {
            return Ok(version);
        }
        let staged = JsonFile::new(&path).stage(&text)?;
        if expected.is_some() {
            staged
                .persist(&path)
                .map_err(|error| LibraryFileError::Io(error.error))?;
        } else {
            staged.persist_noclobber(&path).map_err(|error| {
                if error.error.kind() == io::ErrorKind::AlreadyExists {
                    LibraryFileError::ExternalChange(path)
                } else {
                    LibraryFileError::Io(error.error)
                }
            })?;
        }
        Ok(version)
    }
}
