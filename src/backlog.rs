//!
//! TBD
//!
use crate::glob;

use crate::Chunk;

use crate::Serialize;
use crate::Deserialize;

use crate::InitError;
use crate::PeekError;
use crate::ReadError;
use crate::WriteError;

use std::path::Path;


/// Backlog to handle writes and reads. It wraps each read and write as a unit with a length
/// preamble and an appended checksum for integrity checking. It is to write sequentially to size
/// limited files, and read them back in the same order on consumption. Consumed entries are
/// deleted, for more details look at the [Chunk] documentation.
#[derive(Debug)]
pub struct Backlog<T>
    where T: Serialize + Deserialize
{
    /// Path to main backlog file. It is created with the ending .bkl. Any individual chunk is
    /// suffixed an index number, starting from 1. For example *.bkl.1, *.bkl.2, etc.
    path: std::path::PathBuf,

    /// Maximum size of each chunk in bytes. If the chunk exceeds this size, a file rotation takes
    /// place and a new chunk is created.
    chunk_size: u32,

    /// Handlers for all backlog files, each representing a chunk of the backlog. Limited by the
    chunks: Vec<Chunk>,

    /// Index of the chunk currently being read from.
    reading_chunk: usize,

    /// Index of the chunk currently being written to.
    writing_chunk: usize,

    _entry_ty: std::marker::PhantomData<T>,
}


impl<T> Backlog<T>
    where T: Serialize + Deserialize
{
    /// Opens the backlog at the specified path. If the backlog does not exist, it is created.
    pub fn new<P: AsRef<Path>>(path: P, size: u32) -> Result<Self, InitError>
    {
        let mut chunks = Vec::new();

        // Attempt to open an existing backlog
        for fname in glob::find_files(path.as_ref())?
        {
            chunks.push(
                Chunk::open(&fname, size)?
            );
        }

        // If no backlog exists, create a new one from scratch
        if chunks.is_empty()
        {
            chunks.push(
                Chunk::create(path.as_ref(), size)?
            );
        }

        let reading_chunk = 0;
        let writing_chunk = 0;

        Ok(Self {
            path: path.as_ref().to_owned(),
            chunk_size: size,
            chunks, reading_chunk, writing_chunk,

            _entry_ty: std::marker::PhantomData,
        })
    }

    /// Write a single entry to the backlog.
    pub fn write_entry(&mut self, entry: &T) -> Result<(), WriteError>
    {
        if let Err(e) = self.chunks[self.writing_chunk].write_entry(entry)
        {
            match e
            {
                WriteError::ChunkFull { .. } => {
                    self.rotate()?;

                    self.chunks[self.writing_chunk].write_entry(entry)?;

                    Ok(())
                },

                _ => Err(e),
            }
        } else {
            Ok(())
        }
    }

    /// Write a number of entries to the backlog.
    pub fn write_entries(&mut self, entries: &[T]) -> Result<(), WriteError>
    {
        for entry in entries {
            self.write_entry(entry)?;
        }

        Ok(())
    }

    /// Reads a single entry from the backlog without removing it. If you wish to read and remove
    /// use [Backlog::read_entry].
    pub fn peek_entry(&mut self) -> Result<T, PeekError>
    {
        Ok(
            self.chunks[self.reading_chunk]
                .read()?
        )
    }

    /// Reads a number of entries from the backlog without removing them. If you wish to read and
    /// remove them, use [Backlog::read_entries].
    pub fn peek_entries(&mut self, count: usize) -> Result<Vec<T>, PeekError>
    {
        let mut entries = Vec::with_capacity(count);

        for _ in 0..count
        {
            let entry = self.chunks[self.reading_chunk]
                .read()?;

            entries.push(entry);
        }

        Ok(entries)
    }

    /// Consumes `count` entries from the backlog. This results in the read entry to be removed from
    /// the backlog, which essentially moves forward the persisted read pointer.
    pub fn consume(&mut self, count: usize) -> Result<(), ReadError>
    {
        self.chunks[self.reading_chunk]
            .advance(count)
    }

    /// Read a single entry from the backlog. This results in the read entry to be removed from
    /// backlog. If you wish to read without removing, use [Backlog::peek_entry].
    pub fn read_entry(&mut self) -> Result<T, ReadError>
    {
        let entry = self.chunks[self.reading_chunk]
            .read()?;

        self.chunks[self.reading_chunk]
            .advance(1)?;

        Ok(entry)
    }

    /// Reads a number of entries from the backlog. This results in the read entries to be removed
    /// from backlog. If you wish to read without removing, use [Backlog::peek_entries].
    pub fn read_entries(&mut self, count: usize) -> Result<Vec<T>, ReadError>
    {
        let mut entries = Vec::with_capacity(count);

        for _ in 0..count
        {
            let entry = self.chunks[self.reading_chunk]
                .read()?;

            entries.push(entry);
        }

        self.chunks[self.reading_chunk]
            .advance(count)?;

        Ok(entries)
    }
}


/// Private interface
impl<T> Backlog<T>
    where T: Serialize + Deserialize
{
    fn rotate(&mut self) -> Result<(), WriteError>
    {
        // Rotate all chunks backwards, since we increment suffixes. This way we increment from top to bottom.
        for chunk in self.chunks.iter_mut()
        {
            chunk.rotate()
                .map_err(|e| WriteError::RotationError {path: chunk.path().to_owned(), source: e})?;
        }

        // Create a new chunk as main to write to.
        // self.chunks.insert(0, Chunk::create(&self.path, self.chunk_size)?);

        // Update internal indices
        self.reading_chunk += 1;  // this one moved by incrementing its suffix
        self.writing_chunk  = 0;  // the newly created one which stays at 0

        Ok(())
    }
}
