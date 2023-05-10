//!
//!
//!
#![allow(missing_docs)]  // docs are found in the #[error(...)] attributes

use crate::Frame;

use crate::ThisError;
use crate::BincodeError;

use std::path::PathBuf;


#[derive(Debug, ThisError)]
pub enum InitError
{
    #[error(transparent)]
    FindError {#[from] source: GlobError},

    #[error(transparent)]
    OpenError {#[from] source: OpenError},

    #[error(transparent)]
    CreateError {#[from] source: CreateError},
}


#[derive(Debug, ThisError)]
pub enum ReadError
{
    #[error("Failed to read from backlog file at {path} due to {source}")]
    ReadError {path: PathBuf, source: std::io::Error},

    #[error("Invalid checksum in {path} at byte {offset} over data {data:?}, expected {expected}, got {actual}")]
    InvalidChecksum {path: PathBuf, offset: u64, data: Vec<u8>, expected: u32, actual: u32},

    #[error("Failed to deserialize data from backlog file at {path}, offset {offset} due to {source}")]
    DeserializeError {path: PathBuf, offset: u64, source: BincodeError},

    #[error("Failed to advance read pointer in backlog file at {path} due to {source}")]
    AdvanceError {path: PathBuf, source: std::io::Error},

    #[error("Failed to seek/read from backlog file due to {source}")]
    IoError {#[from] source: std::io::Error},
}


#[derive(Debug, ThisError)]
pub enum WriteError
{
    #[error("Attempt to write to backlog failed. Chunk is already full at {path}. Attempted to write {size} bytes, but maximum size is {max_size}")]
    ChunkFull {path: PathBuf, size: usize, max_size: usize, frame: Frame},

    #[error("Could not seek/write/flush to backlog at {path}, due to I/O errors or EOF being reached: {source}")]
    IoError {path: PathBuf, source: std::io::Error},

    #[error(transparent)]
    RotationError {#[from] source: RotationError},
}


//
// Secondary / Internal Errors
//
#[derive(Debug, ThisError)]
pub enum GlobError
{
    #[error("Provided path to backlog does not have a stem {path}. It is required for the location and names of the backlog, as well as its chunks.")]
    NoStem {path: PathBuf},

    #[error("Provided path to backlog does not have a containing directory {path}. It is required for the location and naming of the backlog, as well as its chunks.")]
    NoParent {path: PathBuf},

    #[error("Could not open directory at {path} to look for backlog files: {source}")]
    DirReadError {path: PathBuf, source: std::io::Error},

    #[error("Could not open backlog file at {path} due to an unexpected error: {source}")]
    Unknown {path: PathBuf, source: std::io::Error},
}


#[derive(Debug, ThisError)]
pub enum OpenError
{
    #[error("Could not open backlog file at {path}, as it does not exists")]
    DoesNotExist {path: PathBuf, source: std::io::Error},

    #[error("Could not open backlog file at {path}, due to insufficient rights")]
    InsufficientRights {path: PathBuf, source: std::io::Error},

    #[error("Could not read header from backlog file at {path}, due to {source}")]
    HeaderReadError {path: PathBuf, source: std::io::Error},

    #[error("Backlog suffix in {path} is not a valid backlog suffix. It should be a number, instead got {suffix}")]
    InvalidSuffix {path: PathBuf, suffix: String},
}


#[derive(Debug, ThisError)]
pub enum CreateError
{
    #[error("Could not create new backlog file at {path}, as it already exists")]
    AlreadyExists {path: PathBuf, source: std::io::Error},

    #[error("Could not create new backlog file at {path}, due to insufficient rights")]
    InsufficientRights {path: PathBuf, source: std::io::Error},

    #[error("Could not create new backlog file at {path}, due to insufficient space")]
    InsufficientSpace {path: PathBuf, source: std::io::Error},

    #[error("Could not write initial header to backlog file at {path}, due to {source}")]
    HeaderWriteError {path: PathBuf, source: std::io::Error},
}


#[derive(Debug, ThisError)]
pub enum RotationError
{
    #[error("Failed to rotate backlog chunks at {path} due to {source}")]
    RotationError {path: PathBuf, source: std::io::Error},

    #[error(transparent)]
    CreateError {#[from] source: CreateError},
}
