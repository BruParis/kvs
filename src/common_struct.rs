use crate::error::{Result};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::fs::File;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};

#[derive(Serialize, Deserialize, Debug)]
pub enum KVRequest {
    Get { key: String },
    Set { key: String, val: String },
    Rm { key: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum KVResponse {
    Ok(Option<String>),
    Err(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KVPair {
    pub key: String,
    pub val: String,
}

impl KVPair {
    pub fn new(key: String, val: String) -> KVPair {
        KVPair { key, val }
    }
}

pub struct BufReaderPos {
    pub reader: BufReader<File>,
    pos: usize,
}

impl BufReaderPos {
    pub fn new(reader: BufReader<File>) -> BufReaderPos {
        BufReaderPos { reader, pos: 0 }
    }

    pub fn read_entry(&mut self, entry: &KVEntry) -> Result<String> {
        self.reader.seek(SeekFrom::Start(entry.pos))?;
        let mut deserializer = Deserializer::from_reader(&mut self.reader);
        let KVPair { key: _, val } = KVPair::deserialize(&mut deserializer)?;
        Ok(val)
    }
}

impl Seek for BufReaderPos {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}

pub struct BufWriterPos {
    writer: BufWriter<File>,
    pos: u64,
}

impl BufWriterPos {
    pub fn new(writer: BufWriter<File>, pos: u64) -> BufWriterPos {
        BufWriterPos { writer, pos }
    }

    pub fn append_log_file(&mut self, key: &String, val: &String) -> Result<(u64, u64)> {
        let start_pos = self.pos;
        // println!("                  self.pos: {}", self.pos);

        let kv_pair: KVPair = KVPair::new(key.to_owned(), val.to_owned());
        let mut buf = vec![];
        serde_json::to_writer(&mut buf, &kv_pair)?;
        let len = buf.len() as u64;
        self.write_all(&buf)?;
        self.flush()?;

        // println!("                  start_pos -> self.pos: {} -> {}", start_pos, self.pos);

        Ok((len, start_pos))
    }
}

impl Write for BufWriterPos {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let size = self.writer.write(buf)?;
        self.pos += size as u64;
        Ok(size)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct KVEntry {
    pub rm: bool,
    len: u64,
    pos: u64,
}

impl KVEntry {
    pub fn new(rm: bool, len: u64, pos: u64) -> KVEntry {
        KVEntry { rm, len, pos }
    }
}
