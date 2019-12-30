
use crate::error::{Result};


const COMPACTION_THRESHOLD: u64 = 20;

pub trait KVEngine {
    fn set(&mut self, key: String, value: String) -> Result<()>;

    fn get(&mut self, key: String) -> Result<Option<String>>;

    fn remove(&mut self, key: String) -> Result<()>;
}
