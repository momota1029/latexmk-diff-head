use std::{
    fmt::Debug,
    io,
    path::{Path, PathBuf},
};

pub type Result<T, E = Error> = std::result::Result<T, E>;
#[derive(Debug)]
pub enum Error {
    CurrentDirFailed(io::Error),
    CreateDirFailed { path: PathBuf, source: io::Error },
    CanonicalizeFailed { path: PathBuf, source: io::Error },
    FileCopyFailed { from: PathBuf, to: PathBuf, source: io::Error },
    FileRenameFailed { from: PathBuf, to: PathBuf, source: io::Error },
    AlreadySaid,
    StdIoError(io::Error),
    CommandFailed(io::Error),
    StdErr(Vec<u8>),
    EnvError(io::Error),
}
pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    std::fs::copy(&from, &to).map_err(|e| Error::FileCopyFailed { from: from.as_ref().to_owned(), to: to.as_ref().to_owned(), source: e })
}
pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    std::fs::rename(&from, &to).map_err(|e| Error::FileRenameFailed {
        from: from.as_ref().to_owned(),
        to: to.as_ref().to_owned(),
        source: e,
    })
}
pub fn create_dir_all(path: impl AsRef<Path>) -> Result<()> {
    std::fs::create_dir_all(&path).map_err(|e| Error::CreateDirFailed { path: path.as_ref().to_owned(), source: e })
}
pub fn canonicalize(path: impl AsRef<Path>) -> Result<PathBuf> {
    std::fs::canonicalize(&path).map_err(|e| Error::CanonicalizeFailed { path: path.as_ref().to_owned(), source: e })
}

impl Error {
    pub fn print_and_exit<T>(self) -> T {
        todo!()
    }
}
