#![deny(missing_docs)]
use crate::{
    engine::kvstore::KvsFileId,
    error::{KvsError, KvsResult},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    ops::{Deref, DerefMut},
};
#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum Log {
    Set(String, String),
    Rm(String),
}

pub(crate) struct LogMeta {
    file_id: KvsFileId,
    offset: u64,
    len: usize,
}

impl LogMeta {
    pub(crate) fn new(file_id: KvsFileId, offset: u64, len: usize) -> Self {
        Self {
            file_id,
            offset,
            len,
        }
    }
    pub(crate) fn len(&self) -> usize {
        self.len
    }
}

pub(crate) struct LogReader<T: Read> {
    inner: BufReader<T>,
}

impl<T: Read + Seek> LogReader<T> {
    pub(crate) fn new(f: T) -> Self {
        Self {
            inner: BufReader::new(f),
        }
    }

    pub(crate) fn read_log(&mut self, meta: &LogMeta) -> KvsResult<Log> {
        self.inner.seek(SeekFrom::Start(meta.offset))?;
        let mut buf = vec![0; meta.len];
        self.inner.read_exact(&mut buf)?;
        Ok(serde_json::de::from_slice(&buf)?)
    }
}

impl<T: Read> AsRef<BufReader<T>> for LogReader<T> {
    fn as_ref(&self) -> &BufReader<T> {
        &self.inner
    }
}
impl<T: Read> AsMut<BufReader<T>> for LogReader<T> {
    fn as_mut(&mut self) -> &mut BufReader<T> {
        &mut self.inner
    }
}

pub(crate) struct LogReaders {
    readers: BTreeMap<KvsFileId, LogReader<File>>,
}
impl LogReaders {
    pub(crate) fn new() -> Self {
        Self {
            readers: BTreeMap::new(),
        }
    }

    pub(crate) fn read_log(&mut self, meta: &LogMeta) -> KvsResult<Log> {
        if let Some(reader) = self.readers.get_mut(&meta.file_id) {
            Ok(reader.read_log(meta)?)
        } else {
            Err(KvsError::Inner(format!(
                "failed to find file id {} in readers index.",
                meta.file_id
            )))
        }
    }
}
impl Deref for LogReaders {
    type Target = BTreeMap<KvsFileId, LogReader<File>>;

    fn deref(&self) -> &Self::Target {
        &self.readers
    }
}
impl DerefMut for LogReaders {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.readers
    }
}

pub(crate) struct LogWriter<T: Write + Seek> {
    file_id: KvsFileId,
    inner: BufWriter<T>,
}

impl<T: Write + Seek> LogWriter<T> {
    pub(crate) fn new(file_id: KvsFileId, f: T) -> Self {
        let inner = BufWriter::new(f);
        Self { file_id, inner }
    }

    #[inline(always)]
    pub(crate) fn id(&self) -> KvsFileId {
        self.file_id
    }

    fn append(&mut self, buf: &[u8]) -> KvsResult<u64> {
        let old_pos = self.inner.seek(SeekFrom::End(0))?;
        self.inner.write_all(buf)?;
        Ok(old_pos)
    }
    fn flush(&mut self) -> KvsResult<()> {
        Ok(self.inner.flush()?)
    }

    pub(crate) fn append_log(&mut self, log: Log) -> KvsResult<(String, LogMeta)> {
        let buf = serde_json::ser::to_vec(&log)?;
        let offset = self.append(&buf)?;
        self.flush()?;
        Ok((
            match log {
                Log::Set(k, _) => k,
                Log::Rm(k) => k,
            },
            LogMeta::new(self.file_id, offset, buf.len()),
        ))
    }
}
