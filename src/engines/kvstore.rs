use crate::common_struct::KVPair;
use crate::engines::KVEngine;
use crate::error::{KVError, Result};

use serde::Deserialize;
use serde_json::Deserializer;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const COMPACTION_THRESHOLD: u64 = 1024;

//#[derive(Clone)]
pub struct KVStore {
    log_path: Arc<PathBuf>,
    index: Arc<HashMap<String, KVEntry>>,
    writer: Arc<Mutex<BufWriterPos>>,
    readers: BufReaderMap,
}

impl KVStore {
    pub fn open(dir_path: &Path) -> Result<KVStore> {
        match dir_path.to_str() {
            Some(dir_str) => {
                let sled_path = format!("{}{}", dir_str, "/my_old_db");
                if Path::new(&sled_path).exists() {
                    return Err(KVError::WrongEngine);
                }

                let log_path = Path::new(dir_str).join("log_file.txt");
                let readers = BufReaderMap::new(&log_path);
                let writer = BufWriterPos::new(&log_path)?;

                Ok(KVStore {
                    log_path: Arc::new(log_path),
                    index: Arc::new(HashMap::new()),
                    writer: Arc::new(Mutex::new(writer)),
                    readers: readers,
                })
            }
            None => Err(KVError::None),
        }
    }
}

impl KVEngine for KVStore {
    fn set(&self, key: String, value: String) -> Result<()> {
        self.writer.lock().unwrap().set(key, value)
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        match self.index.get(&key) {
            Some(entry) => {
                if entry.rm {
                    Ok(None)
                } else {
                    let val = self.readers.read_entry(&key, entry)?;
                    Ok(Some(val))
                }
            }
            None => Ok(None),
        }
    }

    fn remove(&self, key: String) -> Result<()> {
        self.writer.lock().unwrap().remove(key)
    }
}

impl Clone for KVStore {
    fn clone(&self) -> Self {
        KVStore {
            log_path: Arc::clone(&self.log_path),
            index: Arc::clone(&self.index),
            writer: Arc::clone(&self.writer),
            readers: BufReaderMap::new(&self.log_path),
        }
    }
}

struct BufReaderMap {
    path: Arc<PathBuf>,
    readers: RefCell<BTreeMap<String, BufReaderPos<File>>>,
}

impl BufReaderMap {
    fn new(file_path: &PathBuf) -> Self {
        BufReaderMap {
            path: Arc::new(file_path.to_path_buf()),
            readers: RefCell::new(BTreeMap::new()),
        }
    }

    fn read_entry(&self, key: &String, entry: &KVEntry) -> Result<String> {
        let mut readers = self.readers.borrow_mut();
        if !readers.contains_key(key) {
            let reader = BufReaderPos::new(File::open(self.path.as_path())?);
            readers.insert(key.to_string(), reader);
        }
        let reader = readers.get_mut(key).unwrap();
        reader.read_entry(&entry)
    }

    fn generate_index(&self) -> HashMap<String, KVEntry> {
        let index = HashMap::new();
        index
    }
}

//#[derive(Clone)]
struct BufWriterPos {
    reader: BufReaderMap,
    index: Arc<HashMap<String, KVEntry>>,
    writer: Arc<BufWriter<File>>,
    pos: usize,
}

impl BufWriterPos {
    fn new(file_path: &PathBuf) -> Result<Self> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(file_path)?;

        let writer = BufWriter::new(file);
        let reader = BufReaderMap::new(file_path);
        let index = reader.generate_index();

        Ok(BufWriterPos {
            reader: reader,
            index: Arc::new(index),
            writer: Arc::new(writer),
            pos: 0,
        })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let (len, pos) = self.append_log_file(&key, &value)?;
        let entry = KVEntry {
            rm: false,
            len: len,
            pos: pos,
        };
        Arc::get_mut(&mut self.index).unwrap().insert(key, entry);

        // self.compaction()?;

        Ok(())
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        self.append_log_file(&key, "rm")?;
        //let index = Arc::get_mut(&mut self.index).unwrap();
        if let Some(entry) = Arc::get_mut(&mut self.index).unwrap().get_mut(&key) {
            entry.rm = true;
            //self.compaction()?;

            Ok(())
        } else {
            Err(KVError::FailRemove)
        }
    }

    fn append_log_file(&mut self, key: &str, val: &str) -> Result<(usize, usize)> {
        // println!("                  self.pos: {}", self.pos);

        let kv_pair: KVPair = KVPair::new(key.to_owned(), val.to_owned());
        let mut buf = vec![];
        serde_json::to_writer(&mut buf, &kv_pair)?;
        let len = buf.len();
        self.write_all(&buf)?;
        self.flush()?;

        // println!("                  start_pos -> self.pos: {} -> {}", start_pos, self.pos);

        Ok((len, self.pos))
    }

    pub fn compaction(&mut self) -> Result<()> {
        /*self.readers.reader.seek(SeekFrom::Start(0))?;
        let stream = Deserializer::from_reader(self.readers.reader).into_iter::<KVPair>();

        //let index = &mut self.index;
        let mut kv_map = HashMap::new();
        stream.for_each(|res_kv| {
            if let Ok(kv) = res_kv {
                if let Some(entry) = self.index.get(&kv.key) {
                    if !(entry.rm) {
                        kv_map.insert(kv.key, kv.val);
                    }
                }
            };
        });

        std::fs::remove_file(self.log_path)?;
        // let writer = BufWriterPos::new(self.log_path.to_string())?;
        // self.writer = Arc::new(writer);

        // println!("              compaction - append log file");
        for (k, v) in kv_map.iter() {
            // self.writer.append_log_file(k, v)?;
        }

        let file = File::open(self.log_path)?;
        self.readers = BufReaderPos::new(file);*/

        Ok(())
    }
}

impl Write for BufWriterPos {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let writer = Arc::get_mut(&mut self.writer).unwrap();
        self.pos += writer.write(buf)?;
        Ok(self.pos)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let writer = Arc::get_mut(&mut self.writer).unwrap();
        writer.flush()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct KVEntry {
    pub rm: bool,
    len: usize,
    pos: usize,
}

struct BufReaderPos<T: Seek + Read> {
    pub reader: BufReader<T>,
    pos: usize,
}

impl<T: Seek + Read> BufReaderPos<T> {
    fn new(inner: T) -> Self {
        BufReaderPos {
            reader: BufReader::new(inner),
            pos: 0,
        }
    }

    fn generate_index(&mut self) -> HashMap<String, KVEntry> {
        //let reader = Arc::get_mut(&mut self.reader).unwrap();
        let mut stream = Deserializer::from_reader(&mut self.reader).into_iter::<KVPair>();
        let mut index = HashMap::new();

        while let Some(kv_iter) = stream.next() {
            let len = stream.byte_offset() - self.pos;
            match kv_iter {
                Ok(kv) => {
                    if kv.val != "rm" {
                        let entry = KVEntry {
                            rm: false,
                            len: len,
                            pos: self.pos,
                        };
                        index.insert(kv.key, entry)
                    } else {
                        None
                    }
                }
                Err(_) => None,
            };
            self.pos += len;
        }

        index
    }

    fn read_entry(&mut self, entry: &KVEntry) -> Result<String> {
        //let reader = Arc::get_mut(&mut self.reader).unwrap();
        //reader.seek(SeekFrom::Start(entry.pos as u64))?;
        self.reader.seek(SeekFrom::Start(entry.pos as u64))?;
        let mut deserializer = Deserializer::from_reader(&mut self.reader);
        let KVPair { val, .. } = KVPair::deserialize(&mut deserializer)?;
        Ok(val)
    }
}
