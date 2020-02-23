use sled::Db;
use std::path::Path;

use crate::engines::KVEngine;
use crate::error::{KVError, Result};

pub struct SledKVEngine(Db);

impl SledKVEngine {
    pub fn open(dir_path: &Path) -> Result<Self> {
        match dir_path.to_str() {
            Some(dir_str) => {
                let kvs_path = format!("{}{}", dir_str, "/log_file.txt");
                if Path::new(&kvs_path).exists() {
                    return Err(KVError::WrongEngine);
                }

                let db_path = format!("{}{}", dir_str, "/my_old_db");
                let db = sled::open(db_path)?;
                Ok(SledKVEngine(db))
            }
            None => Err(KVError::None)
        }
    }
}

impl KVEngine for SledKVEngine {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.0.insert(key, value.as_bytes()).map(|_| ())?;
        self.0.flush()?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        Ok(self
            .0
            .get(key)?
            .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
            .map(String::from_utf8)
            .transpose()?)
    }

    fn remove(&mut self, key: String) -> Result<()> {
        self.0
            .remove(key.to_owned())?
            .ok_or(KVError::FailGet(key))?;
        self.0.flush()?;
        Ok(())
    }
}
