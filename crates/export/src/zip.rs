//! A ZIP archive with stored (uncompressed) entries: the container of ODS and XLSX.
//!
//! ```text
//! [local header + name + data] ... [central directory] [end record]
//! ```

use crate::png_write::crc32;
use crate::ExportError;

const LOCAL_SIGNATURE: u32 = 0x0403_4B50;
const CENTRAL_SIGNATURE: u32 = 0x0201_4B50;
const END_SIGNATURE: u32 = 0x0605_4B50;
/// Version 2.0, the least that knows UTF-8 names.
const VERSION: u16 = 20;
/// Bit 11: names are UTF-8.
const FLAG_UTF8: u16 = 1 << 11;
const METHOD_STORED: u16 = 0;
/// 1980-01-01 00:00: the format's earliest date, so the file is the same every time.
const DOS_TIME: u16 = 0;
const DOS_DATE: u16 = 0x0021;
/// Bytes of a local header before the name.
const LOCAL_HEADER: usize = 30;
/// Bytes of the end record.
const END_RECORD: usize = 22;

/// Entries in the order they are added; `finish` writes the archive.
#[derive(Default)]
pub(crate) struct Zip {
    out: Vec<u8>,
    central: Vec<u8>,
    entries: u16,
}

impl Zip {
    /// Refused (`ZipTooBig`) when the file, its offset, its name or the entry count passes what
    /// the format's plain fields hold; the archive stays as it was.
    pub(crate) fn add(&mut self, name: &str, data: &[u8]) -> Result<(), ExportError> {
        let crc = crc32(data);
        let offset = to_u32(self.out.len())?;
        let size = to_u32(data.len())?;
        let name = name.as_bytes();
        let name_len = u16::try_from(name.len()).map_err(|_| ExportError::ZipTooBig)?;
        let entries = self.entries.checked_add(1).ok_or(ExportError::ZipTooBig)?;
        // The data and both headers must still end below 4 GiB.
        let end = self.out.len() + LOCAL_HEADER + name.len() + data.len();
        to_u32(end)?;

        // Local header, then the data.
        put32(&mut self.out, LOCAL_SIGNATURE);
        self.out.extend(common(crc, size, name_len));
        self.out.extend_from_slice(name);
        self.out.extend_from_slice(data);

        // Central directory entry for the same file.
        put32(&mut self.central, CENTRAL_SIGNATURE);
        put16(&mut self.central, VERSION);
        self.central.extend(common(crc, size, name_len));
        self.central.extend_from_slice(&[0; 2 + 2 + 2 + 4]); // comment, disk, attributes
        put32(&mut self.central, offset);
        self.central.extend_from_slice(name);

        self.entries = entries;

        Ok(())
    }

    pub(crate) fn finish(mut self) -> Result<Vec<u8>, ExportError> {
        let directory_at = to_u32(self.out.len())?;
        let directory_size = to_u32(self.central.len())?;
        // The directory and the end record must end below 4 GiB too.
        to_u32(self.out.len() + self.central.len() + END_RECORD)?;
        self.out.append(&mut self.central);

        put32(&mut self.out, END_SIGNATURE);
        put16(&mut self.out, 0); // this disk
        put16(&mut self.out, 0); // disk of the directory
        put16(&mut self.out, self.entries);
        put16(&mut self.out, self.entries);
        put32(&mut self.out, directory_size);
        put32(&mut self.out, directory_at);
        put16(&mut self.out, 0); // comment length

        Ok(self.out)
    }
}

/// The fields a local header and a central entry share, from "version needed" to the name
/// length (the central entry's extra length and the rest follow in `add`).
fn common(crc: u32, size: u32, name_len: u16) -> Vec<u8> {
    let mut out = Vec::new();
    put16(&mut out, VERSION);
    put16(&mut out, FLAG_UTF8);
    put16(&mut out, METHOD_STORED);
    put16(&mut out, DOS_TIME);
    put16(&mut out, DOS_DATE);
    put32(&mut out, crc);
    put32(&mut out, size); // compressed
    put32(&mut out, size); // uncompressed
    put16(&mut out, name_len);
    put16(&mut out, 0); // extra length

    out
}

/// Whether `entries` files of `bytes` in all stay within the format's plain limits.
pub(crate) fn fits(entries: usize, bytes: u64) -> Result<(), ExportError> {
    if entries > usize::from(u16::MAX) || bytes > u64::from(u32::MAX) {
        return Err(ExportError::ZipTooBig);
    }

    Ok(())
}

/// A size or offset as the format's 32 bits.
fn to_u32(len: usize) -> Result<u32, ExportError> {
    u32::try_from(len).map_err(|_| ExportError::ZipTooBig)
}

fn put16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn put32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{zip_files, zip_fits, ExportError};

    const GIB4: u64 = 1 << 32;

    #[test]
    fn the_65536th_entry_is_refused() {
        let names: Vec<String> = (0..=u16::MAX).map(|n| n.to_string()).collect();
        let files: Vec<(&str, &[u8])> = names.iter().map(|n| (n.as_str(), &[][..])).collect();

        assert!(matches!(zip_files(&files), Err(ExportError::ZipTooBig)));
        assert!(zip_files(&files[..usize::from(u16::MAX)]).is_ok());
    }

    #[test]
    fn a_size_of_4_gib_is_refused_before_anything_is_read() {
        assert!(matches!(zip_fits(1, GIB4), Err(ExportError::ZipTooBig)));
        assert!(matches!(zip_fits(1, u64::MAX), Err(ExportError::ZipTooBig)));
        assert!(matches!(
            zip_fits(usize::from(u16::MAX) + 1, 0),
            Err(ExportError::ZipTooBig)
        ));
        assert!(zip_fits(usize::from(u16::MAX), GIB4 - 1).is_ok());
    }

    #[test]
    fn an_offset_past_4_gib_is_refused() {
        assert!(matches!(to_u32(GIB4 as usize), Err(ExportError::ZipTooBig)));
        assert!(to_u32(1).is_ok());
    }
}
