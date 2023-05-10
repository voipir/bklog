//!
//! An attempt at a backlog handler with basic corruption detection and effects mitigation.
//!
//! # Motivation
//!
//! This library is concieved for the purpose of saving data on local storage, should a remote main
//! storage system be temporarily unavailable. Once it returns, the backlog is then read and
//! consumed as it is being written to said main storage.
//!
//! Its main use is aimed at applications that would otherwise loose data. For example IoT devices
//! that can be potentialy and temporarily disconnected from the storage they are supposed to log
//! data to.
//!
//! # Warning
//!
//! Current approach does not avoid data loss. Unfortunatelly, there is no way to guarantee that
//! other than battery with powered underlying storage hardware or a UPS. This library is meant to
//! merely keep the log from corrupting beyond the ability of an automated recovery, all the while
//! being explicit about what was lost, so it can be reported as an error.
#![deny(clippy::all)]
#![deny(missing_docs)]

// Imports
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

use error::InitError;
use error::PeekError;
use error::ReadError;
use error::WriteError;

pub use backlog::Backlog;
