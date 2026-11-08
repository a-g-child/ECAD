//! Cooperatively locked, observed-version publication of complete template catalogues.
use crate::library_lock::{LibraryWriterLock, canonical_library_path, library_lock_path};
use crate::{
    FileVersion, JsonFile, TemplateLibraryFileError, decode_template_library,
    encode_template_library,
};
use ecad_app::{TemplateLibrarySnapshot, VersionedTemplateLibraryStore};
use ecad_domain::TemplateLibrary;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// A single-file template catalogue with canonical-parent location/version tokens.
/// Cooperating writers hold an exclusive create-new sidecar through read/check/stage/replace.
/// Reads see complete old/new files. Destination symlinks are rejected; hard-link aliases,
/// uncooperative writers and filesystems lacking exclusive creation/atomic replacement are
/// outside this guarantee. Staged contents are synced; directory durability and permission
/// preservation are unspecified. Retained locks are reported, never automatically stolen.
#[derive(Clone, Debug)]
pub struct TemplateLibraryFile {
    path: PathBuf,
}
impl TemplateLibraryFile {
    /// Target an explicit path. The parent must exist when IO is performed; no IO occurs here.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
    /// Configured destination, independent of project paths.
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Reserved writer-lock path for diagnostics; this does not acquire/remove a lock.
    /// # Errors
    /// Rejects invalid destinations or missing/inaccessible parents.
    pub fn lock_path(&self) -> Result<PathBuf, TemplateLibraryFileError> {
        Ok(library_lock_path(&canonical_library_path(&self.path)?))
    }
}
fn decode_bytes(bytes: &[u8]) -> Result<TemplateLibrary, TemplateLibraryFileError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    decode_template_library(text)
}
impl VersionedTemplateLibraryStore for TemplateLibraryFile {
    type Version = FileVersion;
    type Error = TemplateLibraryFileError;
    fn load_snapshot(&self) -> Result<TemplateLibrarySnapshot<FileVersion>, Self::Error> {
        let path = canonical_library_path(&self.path)?;
        let bytes = fs::read(&path)?;
        Ok(TemplateLibrarySnapshot {
            catalog: decode_bytes(&bytes)?,
            version: FileVersion::for_path(&path, &bytes)?,
        })
    }
    fn save_if_unchanged(
        &self,
        catalog: &TemplateLibrary,
        expected: Option<&FileVersion>,
    ) -> Result<FileVersion, Self::Error> {
        let path = canonical_library_path(&self.path)?;
        let lock = library_lock_path(&path);
        let _lock = LibraryWriterLock::acquire(lock.clone()).map_err(|error| {
            if error.kind() == io::ErrorKind::AlreadyExists {
                TemplateLibraryFileError::Busy(lock)
            } else {
                error.into()
            }
        })?;
        let bytes = match fs::read(&path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        let current = bytes
            .as_ref()
            .map(|bytes| FileVersion::for_path(&path, bytes))
            .transpose()?;
        if current.as_ref() != expected {
            return Err(TemplateLibraryFileError::ExternalChange(path));
        }
        if let Some(bytes) = &bytes {
            for previous in decode_bytes(bytes)?.revisions() {
                if catalog.revision(previous.reference()) != Some(previous) {
                    return Err(TemplateLibraryFileError::HistoryRewrite(
                        previous.reference(),
                    ));
                }
            }
        }
        let text = encode_template_library(catalog)?;
        let version = FileVersion::for_path(&path, text.as_bytes())?;
        if bytes.as_deref() == Some(text.as_bytes()) {
            return Ok(version);
        }
        let staged = JsonFile::new(&path).stage(&text)?;
        if expected.is_some() {
            staged
                .persist(&path)
                .map_err(|error| TemplateLibraryFileError::Io(error.error))?;
        } else {
            staged.persist_noclobber(&path).map_err(|error| {
                if error.error.kind() == io::ErrorKind::AlreadyExists {
                    TemplateLibraryFileError::ExternalChange(path)
                } else {
                    TemplateLibraryFileError::Io(error.error)
                }
            })?;
        }
        Ok(version)
    }
}
