#![deny(missing_docs)]
use crate::error::Result;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    path::Path,
};
pub(crate) struct SeekableLSFile {
    inner: File,
}

impl SeekableLSFile {
    /// give me a path and I will open a file for you.
    pub(crate) fn new(p: impl AsRef<Path>) -> Result<Self> {
        let inner = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(p)?;
        Ok(Self { inner })
    }
    /// read a line from offset, store them into buf and return the lenth
    pub(crate) fn read_line_at(&mut self, offset: SeekFrom, buf: &mut String) -> Result<usize> {
        self.inner.seek(offset)?;
        let len = BufReader::new(&mut self.inner).read_line(buf)?;
        Ok(len)
    }

    pub(crate) fn append(&mut self, b: &[u8]) -> Result<(u64, usize)> {
        let s = self.inner.seek(SeekFrom::End(0))?;
        let len = self.inner.write(b)?;
        Ok((s, len))
    }
}
