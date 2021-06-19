use crate::error::Result;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    path::Path,
};

pub(crate) struct SeekableLSFile {
    inner: File,
}

// todo: impl sth for this and come up with ideas to impl the read method.
impl SeekableLSFile {
    pub fn new(p: impl AsRef<Path>) -> Result<Self> {
        let inner = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(p)?;
        Ok(Self { inner })
    }

    pub fn read_line_from(&mut self, offset: SeekFrom, buf: &mut String) -> Result<usize> {
        self.inner.seek(offset)?;
        let len = BufReader::new(&mut self.inner).read_line(buf)?;
        Ok(len)
    }

    pub fn append(&mut self, b: &[u8]) -> Result<(u64, usize)> {
        let s = self.inner.seek(SeekFrom::End(0))?;
        let len = self.inner.write(b)?;
        Ok((s, len))
    }
}

// impl Seek for SeekableLSFile {
//     fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
//     }
// }
