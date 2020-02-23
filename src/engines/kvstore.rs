use serde_json::Deserializer;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Seek, SeekFrom};

use crate::common_struct::{BufReaderPos, BufWriterPos, KVEntry, KVPair};
use crate::engines::KVEngine;
use crate::error::{KVError, Result};

const COMPACTION_THRESHOLD: u64 = 1024;

use std::path::Path;

pub struct KVStore {
    log_path: String,
    writer: BufWriterPos,
    reader: BufReaderPos,
    index: HashMap<String, KVEntry>,
}

impl KVStore {
    pub fn open(dir_path: &Path) -> Result<KVStore> {
        match dir_path.to_str() {
            Some(dir_str) => {
                let sled_path = format!("{}{}", dir_str, "/my_old_db");
                if Path::new(&sled_path).exists() {
                    return Err(KVError::WrongEngine);
                }

                let log_path = format!("{}{}", dir_str, "/log_file.txt");
                let writer = writer_log_file(log_path.to_owned())?;
                let mut reader = reader_log_file(log_path.to_owned())?;
                let index = generate_index(&mut reader);
                Ok(KVStore {
                    log_path,
                    writer,
                    reader,
                    index,
                })
            }
            None => Err(KVError::None)
        }
    }

    pub fn compaction(&mut self) -> Result<()> {
        self.reader.seek(SeekFrom::Start(0))?;
        let stream = Deserializer::from_reader(&mut self.reader.reader).into_iter::<KVPair>();

        let index = &mut self.index;
        let mut kv_map = HashMap::new();
        stream.for_each(|res_kv| {
            if let Ok(kv) = res_kv {
                if let Some(entry) = index.get(&kv.key) {
                    if !(entry.rm) {
                        kv_map.insert(kv.key, kv.val);
                    }
                }
            };
        });

        std::fs::remove_file(self.log_path.to_owned())?;
        let writer = writer_log_file(self.log_path.to_owned())?;
        self.writer = writer;

        // println!("              compaction - append log file");
        for (k, v) in kv_map.iter() {
            self.writer.append_log_file(k, v)?;
        }

        let mut reader = reader_log_file(self.log_path.to_owned())?;
        self.index = generate_index(&mut reader);
        self.reader = reader;

        Ok(())
    }
}

fn reader_log_file(file_path_str: String) -> Result<BufReaderPos> {
    let file = File::open(file_path_str)?;
    let reader = BufReader::new(file);
    Ok(BufReaderPos::new(reader))
}

fn writer_log_file(file_path_str: String) -> Result<BufWriterPos> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(file_path_str)?;
    let pos = file.seek(SeekFrom::End(0))?;
    let writer = BufWriter::new(file);
    // println!("              writer log file - self.pos: {}", pos);
    Ok(BufWriterPos::new(writer, pos))
}

fn generate_index(reader: &mut BufReaderPos) -> HashMap<String, KVEntry> {
    let mut stream = Deserializer::from_reader(&mut reader.reader).into_iter::<KVPair>();
    let mut res = HashMap::new();
    let mut pos = 0;
    while let Some(kv_iter) = stream.next() {
        let len = stream.byte_offset() as u64 - pos;
        match kv_iter {
            Ok(kv) => {
                if kv.val != "rm" {
                    let entry = KVEntry::new(false, len, pos);
                    res.insert(kv.key, entry)
                } else {
                    None
                }
            }
            Err(_) => None,
        };
        pos += len;
    }

    res
}

impl KVEngine for KVStore {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let (len, pos) = self.writer.append_log_file(&key, &value)?;
        let entry = KVEntry::new(false, len, pos);
        self.index.insert(key, entry);
        self.compaction()?;

        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.index.get(&key) {
            Some(entry) => {
                if entry.rm {
                    Ok(None)
                } else {
                    // println!("         get - check log_file");
                    let val = self.reader.read_entry(entry)?;
                    Ok(Some(val))
                }
            }
            None => Ok(None),
        }
    }

    fn remove(&mut self, key: String) -> Result<()> {
        let rm_val = "rm".to_owned();
        self.writer.append_log_file(&key, &rm_val)?;
        if let Some(entry) = self.index.get_mut(&key) {
            (*entry).rm = true;
            self.compaction()?;

            Ok(())
        } else {
            Err(KVError::FailRemove)
        }
    }
}
