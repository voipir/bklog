#![doc = include_str!("../README.md")]
#![deny(clippy::all)]
#![deny(missing_docs)]

// Imports
#[macro_use] extern crate tracing;

use crc::Crc;
use crc::CRC_32_ISCSI;

const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

use serde::Serialize;
use serde::de::DeserializeOwned as Deserialize;

use bincode::Error          as BincodeError;
use bincode::DefaultOptions as BincodeBuilder;
use bincode::Options        as BincodeOptions;

use thiserror::Error as ThisError;

// Internals and Exports
mod glob;
mod chunk;
mod error;
mod frame;
mod header;
mod backlog;

use chunk::Chunk;

use frame::Frame;
use header::Header;

pub use error::InitError;
pub use error::ReadError;
pub use error::WriteError;

pub use error::GlobError;
pub use error::OpenError;
pub use error::CreateError;
pub use error::CursorError;
pub use error::RotationError;

pub use backlog::Backlog;
