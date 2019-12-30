use crate::engines::KVEngine;
use crate::error::{KVError, Result};

use sled::Db; 

pub struct SledKVEngine(Db);

impl SledKVEngine {
    pub fn new(db: Db) -> SledKVEngine {
        SledKVEngine(db)
    }
}