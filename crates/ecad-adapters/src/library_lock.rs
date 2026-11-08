//! Shared file coordination for the concrete subsystem and drawing-template libraries.
use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

/// Canonicalize the parent while rejecting destination symlinks. Hard-link aliases and
/// races with uncooperative filesystem actors remain outside the cooperative contract.
pub(crate) fn canonical_library_path(path: &Path) -> io::Result<PathBuf> {
    let name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "library path needs a file name",
        )
    })?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let path = parent.canonicalize()?.join(name);
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "library destination symlinks are unsupported",
            ));
        }
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
        _ => {}
    }
    Ok(path)
}
/// Both library kinds reserve the same sidecar name when targeting the same location.
pub(crate) fn library_lock_path(path: &Path) -> PathBuf {
    let mut name = std::ffi::OsString::from(".");
    name.push(
        path.file_name()
            .expect("canonical library path has a file name"),
    );
    name.push(".ecad-library-lock");
    path.with_file_name(name)
}
/// Exclusive create-new guard, held through read/check/stage/replace. An existing lock
/// is never stolen; callers map AlreadyExists to their own typed Busy error.
pub(crate) struct LibraryWriterLock {
    path: PathBuf,
    file: Option<File>,
}
impl LibraryWriterLock {
    pub(crate) fn acquire(path: PathBuf) -> io::Result<Self> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        Ok(Self {
            path,
            file: Some(file),
        })
    }
}
impl Drop for LibraryWriterLock {
    fn drop(&mut self) {
        // Windows requires close before unlink. Failed cleanup leaves a visible lock.
        drop(self.file.take());
        let _ = fs::remove_file(&self.path);
    }
}
