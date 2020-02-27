use serde::{Deserialize, Serialize};

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
    pub fn new(key: String, val: String) -> Self {
        KVPair { key, val }
    }
}
