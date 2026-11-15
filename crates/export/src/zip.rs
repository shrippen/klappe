//! A ZIP archive with stored (uncompressed) entries: the container of ODS and XLSX.
//!
//! ```text
//! [local header + name + data] ... [central directory] [end record]
//! ```

use crate::png_write::crc32;

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

/// Entries in the order they are added; `finish` writes the archive.
#[derive(Default)]
pub(crate) struct Zip {
    out: Vec<u8>,
    central: Vec<u8>,
    entries: u16,
}

impl Zip {
    pub(crate) fn add(&mut self, name: &str, data: &[u8]) {
        let crc = crc32(data);
        let offset = self.out.len() as u32;
        let size = data.len() as u32;
        let name = name.as_bytes();

        // Local header, then the data.
        put32(&mut self.out, LOCAL_SIGNATURE);
        self.out.extend(common(crc, size, name.len()));
        self.out.extend_from_slice(name);
        self.out.extend_from_slice(data);

        // Central directory entry for the same file.
        put32(&mut self.central, CENTRAL_SIGNATURE);
        put16(&mut self.central, VERSION);
        self.central.extend(common(crc, size, name.len()));
        self.central.extend_from_slice(&[0; 2 + 2 + 2 + 4]); // comment, disk, attributes
        put32(&mut self.central, offset);
        self.central.extend_from_slice(name);

        self.entries += 1;
    }

    pub(crate) fn finish(mut self) -> Vec<u8> {
        let directory_at = self.out.len() as u32;
        let directory_size = self.central.len() as u32;
        self.out.append(&mut self.central);

        put32(&mut self.out, END_SIGNATURE);
        put16(&mut self.out, 0); // this disk
        put16(&mut self.out, 0); // disk of the directory
        put16(&mut self.out, self.entries);
        put16(&mut self.out, self.entries);
        put32(&mut self.out, directory_size);
        put32(&mut self.out, directory_at);
        put16(&mut self.out, 0); // comment length

        self.out
    }
}

/// The fields a local header and a central entry share, from "version needed" to the name
/// length (the central entry's extra length and the rest follow in `add`).
fn common(crc: u32, size: u32, name_len: usize) -> Vec<u8> {
    let mut out = Vec::new();
    put16(&mut out, VERSION);
    put16(&mut out, FLAG_UTF8);
    put16(&mut out, METHOD_STORED);
    put16(&mut out, DOS_TIME);
    put16(&mut out, DOS_DATE);
    put32(&mut out, crc);
    put32(&mut out, size); // compressed
    put32(&mut out, size); // uncompressed
    put16(&mut out, name_len as u16);
    put16(&mut out, 0); // extra length

    out
}

fn put16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn put32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}
