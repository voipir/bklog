//!
//! Header of a Backlog chunk file.
//!
use std::fs::File;

use std::os::unix::fs::FileExt;


#[derive(Debug)]
pub struct Header
{
    /// Position of the read cursor within the file. This gets updated after each consumption of an
    /// entry.
    read_cursor: u32,

    /// Position of the write cursor within the file. This gets updated after each write of an entry.
    write_cursor: u32,
}


impl Header
{
    pub(crate) fn new() -> Self
    {
        Self {read_cursor: 0, write_cursor: 0}
    }

    pub(crate) fn read_cursor(&self) -> u64
    {
        self.read_cursor as u64
    }

    pub(crate) fn advance_read_cursor(&mut self, offset: u64)
    {
        self.read_cursor += offset as u32
    }

    pub(crate) fn write_cursor(&self) -> u64
    {
        self.write_cursor as u64
    }

    pub(crate) fn advance_write_cursor(&mut self, offset: u64)
    {
        self.write_cursor += offset as u32
    }

    pub(crate) fn read_from(file: &mut File) -> Result<Self, std::io::Error>
    {
        let mut header = [0u8; 8];  // [read_cursor]:4 + [write_cursor]:4

        file.read_exact_at(&mut header, 0)?;

        let header_read:  [u8; 4] = header[0..3].try_into().unwrap();  // [read_cursor]:4
        let header_write: [u8; 4] = header[4..7].try_into().unwrap();  // [write_cursor]:4

        let read_cursor  = u32::from_ne_bytes(header_read);
        let write_cursor = u32::from_ne_bytes(header_write);

        Ok(Self {read_cursor, write_cursor})
    }

    pub(crate) fn write_into(&self, file: &mut File) -> Result<(), std::io::Error>
    {
        let data = &[
            self.read_cursor.to_ne_bytes(),
            self.write_cursor.to_ne_bytes()
        ].concat();

        file.write_all_at(data, 0)?;

        Ok(())
    }
}


#[test]
fn test_header_layout()
{
    todo!(); // TODO
}
