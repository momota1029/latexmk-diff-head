use std::{
    fmt::Debug,
    io::{self, Write as _},
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
        match self {
            Error::AlreadySaid => {
                // Already printed, nothing to do
            }
            Error::StdErr(stderr) => {
                if let Err(e) = std::io::stderr().write_all(&stderr) {
                    eprintln!("Failed to write error output to stderr: {}", e);
                }
            }
            Error::CurrentDirFailed(e) => {
                eprintln!("Failed to get current directory: {}", e);
            }
            Error::CreateDirFailed { path, source } => {
                eprintln!("Failed to create directory {}: {}", path.display(), source);
            }
            Error::CanonicalizeFailed { path, source } => {
                eprintln!("Failed to canonicalize path {}: {}", path.display(), source);
            }
            Error::FileCopyFailed { from, to, source } => {
                eprintln!("Failed to copy file {} -> {}: {}", from.display(), to.display(), source);
            }
            Error::FileRenameFailed { from, to, source } => {
                eprintln!("Failed to rename file {} -> {}: {}", from.display(), to.display(), source);
            }
            Error::StdIoError(e) => {
                eprintln!("I/O error occurred: {}", e);
            }
            Error::CommandFailed(e) => {
                eprintln!("Command execution failed: {}", e);
            }
            Error::EnvError(e) => {
                eprintln!("Failed to get environment information: {}", e);
            }
        }
        std::process::exit(1);
    }
}
