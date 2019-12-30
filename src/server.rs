use crate::engines::{KVEngine, KVStore};
use crate::error::{KVError, Result};

pub struct KVServer {
    kvs: KVStore,
}

impl KVServer {
    pub fn new() -> Result<KVServer> {
        let current_path = std::env::current_dir()?;
        let kvs = KVStore::open(&current_path)?;
        Ok(KVServer { kvs })
    }

    pub fn sendBackRequest(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn executeGetCmd(&mut self, key: String) -> Result<String> {
        if let Some(value) = self.kvs.get(key.to_owned())? {
            println!("{}", value);
            Ok(value)
        } else {
            println!("Key not found");
            Err(KVError::FailGet(format!(
                "Failed to get value from key: {}",
                key
            )))
        }
    }

    pub fn executeSetCmd(&mut self, key: String, val: String) -> Result<String> {
        self.kvs.set(key.to_owned(), val.to_owned())?;
        self.kvs.compaction()?;
        Ok(format!(
            "set key: {} value: {} succesffully done !",
            key, val
        ))
    }

    pub fn executeRmCmd(&mut self, key: String) -> Result<String> {
        match self.kvs.remove(key.to_owned()) {
            Ok(()) => {}
            Err(_) => {
                println!("Key not found");
                return Err(KVError::FailGet(format!(
                    "Failed to get value from key: {}",
                    key
                )));
            }
        }
        self.kvs.compaction()?;
        Ok(format!("rm key: {} succesffully done !", key))
    }
}
