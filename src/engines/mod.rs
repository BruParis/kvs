use crate::error::Result;

pub use self::kvstore::KVStore;
pub use self::sled_kvs::SledKVEngine;

mod kvstore;
mod sled_kvs;

pub trait KVEngine: Clone {
    fn set(&self, key: String, value: String) -> Result<()>;

    fn get(&self, key: String) -> Result<Option<String>>;

    fn remove(&self, key: String) -> Result<()>;
}
