//!
//! Individual backlog chunk Handler.
//!
//! The layout of such file is as follow; TBD
//!
use crate::Frame;
use crate::Header;

use crate::InitError;
use crate::ReadError;
use crate::WriteError;

use crate::Serialize;
use crate::Deserialize;

use std::fs::File;
use std::fs::OpenOptions;

use std::io::Write;
use std::io::ErrorKind;

use std::path::Path;
use std::path::PathBuf;


/// Single data chunk handled by [Backlog]. It contains
#[derive(Debug)]
pub struct Chunk
{
    /// Path to the file this chunk is stored in.
    path: PathBuf,

    /// Position of the chunk in the backlog chain of chunks. Also the suffix in the name extension.
    position: u32,

    /// Maximum size this chunk is allowed to reach.
    size: u32,

    /// File handle to the chunk. This is what we operate on.
    file: File,

    /// Header of the file. It contains the metadata of the chunk.
    header: Header,
}


impl Chunk
{
    pub(crate) fn path(&self) -> &Path
    {
        &self.path
    }

    pub(crate) fn capacity(&self) -> u64
    {
        self.size as u64 - self.header.write_cursor()
    }

    /// Create a chunk from a provided path and specify its size limits. If the file already exists,
    /// this operation errors out. The file should not be suffixed, since creation only happens at
    /// the start of a backlog. In other words; the first file with extension .bkl. Suffixes are
    /// appended as it gets rotated.
    pub(crate) fn create(path: &Path, size: u32) -> Result<Self, InitError>
    {
        let mut file = OpenOptions::new()
            .append(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| {
                match e.kind()
                {
                    ErrorKind::AlreadyExists    => InitError::AlreadyExists      { path: path.to_owned(), source: e },
                    ErrorKind::PermissionDenied => InitError::InsufficientRights { path: path.to_owned(), source: e },

                    _ => panic!("Unknown IO error has ocurred while creating {path:?} due to {e}")
                }
            })?;

        file.set_len(size as u64)
            .map_err(|e| InitError::InsufficientSpace { path: path.to_owned(), source: e })?;

        let header = Header::new();

        header.write_into(&mut file)
            .map_err(|e| InitError::HeaderWriteError { path: path.to_owned(), source: e })?;

        Ok(Chunk {
            path: path.to_owned(),
            position: 0, size, file,
            header
        })
    }

    /// Exclusively open a chunk from a provided path and specify its size limits. For that the
    /// chunk is required to exist, otherwise throwing an error.
    pub(crate) fn open(path: &Path, size: u32) -> Result<Self, InitError>
    {
        let position = extract_suffix(&path)?;

        let mut file = OpenOptions::new()
            .append(true)
            .write(true)
            .create(false)
            .open(&path)
            .map_err(|e| {
                match e.kind() {
                    ErrorKind::NotFound         => InitError::DoesNotExist { path: path.to_owned(), source: e },
                    ErrorKind::PermissionDenied => InitError::InsufficientRights { path: path.to_owned(), source: e },

                    _ => panic!("Unknown IO error has ocurred while opening {path:?} due to {e}")
                }
            })?;

        let header = Header::read_from(&mut file)
            .map_err(|e| InitError::HeaderReadError {path: path.to_owned(), source: e})?;

        Ok(Chunk {
            path: path.to_owned(),
            position, size, file,
            header,
        })
    }

    pub(crate) fn read<T>(&mut self) -> Result<T, ReadError>
        where T: Deserialize
    {
        let frame = Frame::from_file_at(&mut self.file, self.header.read_cursor())
            .map_err(|e| { ReadError::ReadError { path: self.path.to_owned(), source: e}})?;

        frame.verify_checksum()
            .map_err(|(expected, actual)| ReadError::InvalidChecksum {
                path:   self.path.to_owned(),
                offset: self.header.read_cursor(),
                data:   frame.data().to_owned(),
                expected, actual
            })?;

        frame.deserialize()
            .map_err(|e| ReadError::DeserializeError { path: self.path.to_owned(), offset: self.header.read_cursor(), source: e})
    }

    /// Advances read cursor by a count of entries. This marks them as read and consumed.
    pub(crate) fn advance(&mut self, count: usize) -> Result<(), ReadError>
    {
        for _ in 0..count
        {
            // read the frame to get its length to move forward
            let frame = Frame::from_file_at(&mut self.file, self.header.read_cursor())
                .map_err(|e| { ReadError::ReadError { path: self.path.to_owned(), source: e}})?;

            self.header.advance_read_cursor(frame.len());
        }

        self.header.write_into(&mut self.file)
            .map_err(|e| ReadError::AdvanceError { path: self.path.to_owned(), source: e})
    }

    /// Write a slice of bytes to the chunk. If the chunk is full, this operation errors out.
    pub(crate) fn write_entry<T>(&mut self, entry: &T) -> Result<(), WriteError>
        where T: Serialize
    {
        let frame = Frame::from_entry(entry);

        self.write_frame(frame)
    }

    pub(crate) fn write_frame(&mut self, frame: Frame) -> Result<(), WriteError>
    {
        if self.capacity() >= frame.len()
        {
            frame.write_at(&mut self.file, self.header.write_cursor())
                .map_err(|e| WriteError::IoError {path: self.path.to_owned(), source: e})?;

            self.header.advance_write_cursor(frame.len());

            self.header.write_into(&mut self.file)
                .map_err(|e| WriteError::IoError {path: self.path.to_owned(), source: e})?;

            self.flush_and_sync()?;

            Ok(())
        }
        else
        {
            Err(WriteError::ChunkFull {
                path:     self.path.to_owned(),
                size:     frame.len() as usize,
                max_size: self.size as usize,
                frame,
            })
        }
    }

    /// Flush chunk data to the underlying storage and send a sync operation to the OS.
    pub(crate) fn flush_and_sync(&mut self) -> Result<(), WriteError>
    {
        self.file.flush()
            .map_err(|e| WriteError::IoError {path: self.path.to_owned(), source: e})?;

        self.file.sync_all()
            .map_err(|e| WriteError::IoError {path: self.path.to_owned(), source: e})?;

        Ok(())
    }

    /// Renames file, suffixing it with 1 in case of being the main .bkl, or n + 1 in case of
    /// already being a suffixed chunk.
    pub(crate) fn rotate(&mut self) -> Result<(), std::io::Error>
    {
        let new_path = self.path.with_extension(format!("{}.bkl", self.position + 1));

        std::fs::rename(&self.path, new_path)
    }
}


/// Extracts the integer suffix in the extension of the file name. If there is no numeric suffix,
/// return 0
fn extract_suffix(path: &Path) -> Result<u32, InitError>
{
    let ext = path.extension()
        .expect("At this point the extension is known and this error caught. This is just an assertion");

    let ext_str = ext.to_string_lossy()
        .to_owned();

    let suffix = ext_str.trim_start_matches(|c| {
            match c {
                '0'..='9' => false,
                _         => true,
            }
        });

    if suffix.is_empty() {
        Ok(0)
    } else {
        suffix.parse::<u32>()
            .map_err(|_| InitError::InvalidSuffix {path: path.to_owned(), suffix: suffix.to_owned()})
    }
}
