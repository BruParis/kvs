use sled::Db;
use std::path::Path;

use crate::engines::KVEngine;
use crate::error::{KVError, Result};

pub struct SledKVEngine(Db);

impl SledKVEngine {
    pub fn open(dir_path: &Path) -> Result<Self> {
        match dir_path.to_str() {
            Some(dir_str) => {
                let db_path = format!("{}{}", dir_str, "/my_old_db");
                let db = sled::open(db_path)?;
                Ok(SledKVEngine(db))
            }
            None => return Err(KVError::None),
        }
    }
}

impl KVEngine for SledKVEngine {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        Ok(None)
    }

    fn remove(&mut self, key: String) -> Result<()> {
        let rm_val = "rm".to_owned();
        Ok(())
    }
}
