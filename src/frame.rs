//!
//! Frame for one data entry.
//!
//! TBD description of the format
//!
use crate::Serialize;
use crate::Deserialize;

use crate::BincodeError;
use crate::BincodeBuilder;
use crate::BincodeOptions;

use crate::CRC32;

use std::fs::File;

use std::os::unix::fs::FileExt;


/// The frame consists of two u32's, the first is the size of the entry, the last is the checksum.
/// In the case of the size of the entry, it is seen as the size of the entry's data, including both
/// the length and the checksum. This way simple addition moves the pointer past the entry, ready to
/// continue writing the next one.
#[derive(Debug)]
pub struct Frame
{
    length:   u32,
    data:     Vec<u8>,
    checksum: u32,
}


impl Frame
{
    pub(crate) fn from_entry<T>(entry: &T) -> Self
        where T: Serialize
    {
        let data = bincode()
            .serialize(entry)
            .expect("Bincode serialization of known type can only fail on OOM, which is not recoverable in this case");

        let length = data.len() as u32 + 8;  // [length]:4 + [checksum]:4

        let mut digester = CRC32.digest();

        digester.update(&length.to_ne_bytes());
        digester.update(&data);

        let checksum = digester.finalize();

        Self {length, data, checksum}
    }

    /// Take a file handle and read the length, data and checksum, then verify the checksum. It does
    /// not serialize to the entry type. That you have to do in a separate step with
    /// [Frame::deserialize].
    pub(crate) fn from_file_at(file: &mut File, offset: u64) -> Result<Self, std::io::Error>
    {
        // Read data from buffer and split it into its semantic parts; length, data and checksum
        let mut length_buffer   = [0u8; 4];
        let mut checksum_buffer = [0u8; 4];

        file.read_exact_at(&mut length_buffer, offset)?;

        let length = u32::from_ne_bytes(length_buffer);

        let offset_data     = offset                 + 4;  // skip [length]:4 field
        let offset_checksum = offset + length as u64 - 4;  // skip [length]:4 and [data]:length fields

        file.read_exact_at(&mut checksum_buffer, offset_checksum)?;

        let checksum = u32::from_ne_bytes(checksum_buffer);

        let mut data_buffer = vec!(0; length as usize - 8);  // [data] is the frame length - 8 bytes for [length] and [checksum]

        file.read_exact_at(&mut data_buffer, offset_data)?;

        Ok(Self {length, data: data_buffer, checksum})
    }

    /// Size of the whole frame including contents; [length]:4 + [data]:n + [checksum]:4
    pub(crate) fn len(&self) -> u64
    {
        self.length as u64
    }

    /// Provides a view into the data within this frame.
    pub(crate) fn data(&self) -> &[u8]
    {
        &self.data
    }

    /// Returns Ok(()) in case of a valid checksum, or Err((expected, actual)) in case of a mismatch.
    pub(crate) fn verify_checksum(&self) -> Result<(), (u32, u32)>
    {
        let mut digester = CRC32.digest();

        digester.update(&self.length.to_ne_bytes());
        digester.update(&self.data);

        let newcheck = digester.finalize();

        if self.checksum == newcheck {
            Ok(())
        } else {
            Err((self.checksum, newcheck))
        }
    }

    /// Writes the frame to the file at the given offset.
    pub(crate) fn write_at(&self, file: &mut File, offset: u64) -> Result<(), std::io::Error>
    {
        let offset_length   = offset;                                   // 0                         --> [length]:4
        let offset_data     = offset + 4 + self.data.len() as u64;      // 0 + [length]:4            --> [data]:n
        let offset_checksum = offset + 4 + self.data.len() as u64 + 4;  // 0 + [length]:4 + [data]:n --> [checksum]:4

        file.write_all_at(&self.length.to_ne_bytes(),   offset_length)?;
        file.write_all_at(&self.data,                   offset_data)?;
        file.write_all_at(&self.checksum.to_ne_bytes(), offset_checksum)
    }

    pub(crate) fn deserialize<T>(self) -> Result<T, BincodeError>
        where T: Deserialize
    {
        bincode()
            .deserialize(&self.data)
    }
}


fn bincode() -> impl BincodeOptions
{
    BincodeBuilder::new()
        .reject_trailing_bytes()
        .with_native_endian()
        .with_fixint_encoding()
}


mod test
{
    use super::Serialize;

    #[derive(Serialize)]
    struct Test
    {
        a: u32,
        b: u32,
    }

    #[test]
    fn test_from_entry()
    {
        use super::Frame;
        use super::CRC32;

        let test  = Test {a: 1, b: 2};
        let frame = Frame::from_entry(&test);

        let len = frame.length.to_ne_bytes();
        let a   = test.a.to_ne_bytes();
        let b   = test.b.to_ne_bytes();

        let checksum = CRC32.checksum(&[len, a, b].concat());

        assert_eq!(frame.length,   16);
        assert_eq!(frame.data,     [a, b].concat());
        assert_eq!(frame.checksum, checksum);
    }

    #[test]
    fn test_from_bytes()
    {
        use super::Frame;
        use super::CRC32;

        use std::io::Write;

        let len   = 16u32.to_ne_bytes();
        let a     = 1u32.to_ne_bytes();
        let b     = 2u32.to_ne_bytes();

        let checksum = CRC32.checksum(&[len, a, b].concat());
        let checkbuf = checksum.to_ne_bytes();

        let buffer   = [len, a, b, checkbuf].concat();
        let mut file = tempfile::tempfile().unwrap();

        file.write_all(&buffer)
            .expect("Write to temporary file should not have failed");

        let frame = Frame::from_file_at(&mut file, 0)
            .expect("Given the data, it should have deserialized without issues at this point");

        assert_eq!(frame.length,   16);
        assert_eq!(frame.data,     [a, b].concat());
        assert_eq!(frame.checksum, checksum);
    }
}
