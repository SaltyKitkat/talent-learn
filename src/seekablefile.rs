#![deny(missing_docs)]
use crate::error::Result;
use std::{
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    ops::{Deref, DerefMut},
};

pub(crate) struct PosBufReader<T: Read> {
    inner: BufReader<T>,
}

impl<T: Read + Seek> PosBufReader<T> {
    pub(crate) fn new(f: T) -> Self {
        Self {
            inner: BufReader::new(f),
        }
    }

    pub(crate) fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> Result<()> {
        self.inner.seek(SeekFrom::Start(offset))?;
        Ok(self.inner.read_exact(buf)?)
    }
}

impl<T: Read> Deref for PosBufReader<T> {
    type Target = BufReader<T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T: Read> DerefMut for PosBufReader<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub(crate) struct PosBufWriter<T: Write + Seek> {
    inner: BufWriter<T>,
    pos: u64,
}

impl<T: Write + Seek> PosBufWriter<T> {
    pub(crate) fn new(f: T) -> Result<Self> {
        let mut inner = BufWriter::new(f);
        let pos = inner.seek(SeekFrom::End(0))?;
        Ok(Self { inner, pos })
    }

    pub(crate) fn append(&mut self, buf: &[u8]) -> Result<u64> {
        let Self { inner, pos } = self;
        let old_pos = *pos;
        inner.write_all(buf)?;
        *pos = inner.stream_position()?;
        Ok(old_pos)
    }
    pub(crate) fn flush(&mut self) -> Result<()> {
        Ok(self.inner.flush()?)
    }
}

impl<T: Write + Seek> Drop for PosBufWriter<T> {
    fn drop(&mut self) {
        self.flush();
    }
}
