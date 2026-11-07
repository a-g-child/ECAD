//! Cooperative writer exclusion and coherent single-file library publication.

use crate::{FileVersion, JsonFile, LibraryFileError, decode_library, encode_library};
use ecad_app::{LibrarySnapshot, VersionedLibraryStore};
use ecad_domain::SubsystemLibrary;
use std::{
    fs::{self, File, OpenOptions},
    io,
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
        let name = self.path.file_name().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "library path needs a file name",
            )
        })?;
        let parent = self
            .path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let path = parent.canonicalize()?.join(name);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "library destination symlinks are unsupported",
                )
                .into());
            }
            Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error.into()),
            _ => {}
        }
        Ok(path)
    }
    /// Locate the reserved writer lock for diagnostics; never remove an active lock.
    /// # Errors
    /// Reports an invalid destination or inaccessible parent directory.
    pub fn lock_path(&self) -> Result<PathBuf, LibraryFileError> {
        Ok(lock_path(&self.canonical_path()?))
    }
}

fn lock_path(path: &Path) -> PathBuf {
    let mut name = std::ffi::OsString::from(".");
    name.push(
        path.file_name()
            .expect("canonical library path has a file name"),
    );
    name.push(".ecad-library-lock");
    path.with_file_name(name)
}

struct WriterLock {
    path: PathBuf,
    file: Option<File>,
}
impl WriterLock {
    fn acquire(path: PathBuf) -> Result<Self, LibraryFileError> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| {
                if error.kind() == io::ErrorKind::AlreadyExists {
                    LibraryFileError::Busy(path.clone())
                } else {
                    error.into()
                }
            })?;
        Ok(Self {
            path,
            file: Some(file),
        })
    }
}
impl Drop for WriterLock {
    fn drop(&mut self) {
        // Close before unlinking for Windows. Cleanup failure leaves a visible lock
        // and blocks future writes; it never grants another writer unsafe access.
        drop(self.file.take());
        let _ = fs::remove_file(&self.path);
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
        let _lock = WriterLock::acquire(lock_path(&path))?;
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
