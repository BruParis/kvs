
use crate::error::{Result};

pub use self::kvstore::KVStore;

mod kvstore;
mod sled_kvs;

pub trait KVEngine {
    fn set(&mut self, key: String, value: String) -> Result<()>;

    fn get(&mut self, key: String) -> Result<Option<String>>;

    fn remove(&mut self, key: String) -> Result<()>;
}

