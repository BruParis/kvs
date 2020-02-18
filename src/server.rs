use serde::Deserialize;
use serde_json::Deserializer;
use slog::{Logger};
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::common_struct::{KVRequest, KVResponse};
use crate::engines::KVEngine;
use crate::error::{KVError, Result};

pub struct KVServer<E: KVEngine> {
    engine: E,
}

impl<E: KVEngine> KVServer<E> {
    pub fn new(engine: E) -> Self {
        // let current_path = std::env::current_dir()?;
        // let kvs = KVStore::open(&current_path)?;
        // Ok(KVServer { kvs })
        KVServer { engine }
    }

    pub fn run(&mut self, addr: String, log: &Logger) -> Result<()> {
        let listener = TcpListener::bind(addr)?;

        for stream in listener.incoming() {
            let stream = stream?;
            self.handle_connection(&stream, &log)?;
        }

        Ok(())
    }

    fn handle_connection(&mut self, stream: &TcpStream, log: &Logger) -> Result<()> {
        let _peer_addr = stream.peer_addr()?;
        let mut reader = BufReader::new(stream);

        let mut buffer = [0; 512];
        reader.read_exact(&mut buffer)?;

        let executed = self.execute_cmd(buffer, log);
        let resp: KVResponse;
        match executed {
            Ok(val) => resp = KVResponse::Ok(Some(format!("{}", val))),
            Err(error) => resp = KVResponse::Err(format!("error {}", error)),
        }

        let mut buf = vec![];
        serde_json::to_writer(&mut buf, &resp)?;

        let mut writer = BufWriter::new(stream);
        writer.write_all(&buf)?;

        Ok(())
    }

    fn execute_cmd(&mut self, mut buffer: [u8; 512], _log: &Logger) -> Result<String> {
        let mut deserializer = Deserializer::from_slice(&mut buffer);

        let req = KVRequest::deserialize(&mut deserializer)?;

        match req {
            KVRequest::Get { key } => self.execute_get_cmd(key),
            KVRequest::Set { key, val } => self.execute_set_cmd(key, val),
            KVRequest::Rm { key } => self.execute_rm_cmd(key),
        }
    }

    fn execute_get_cmd(&mut self, key: String) -> Result<String> {
        if let Some(value) = self.engine.get(key.to_owned())? {
            println!("{}", value);
            Ok(value)
        } else {
            println!("Key not found");
            Ok(String::from("Key not found"))
        }
    }

    fn execute_set_cmd(&mut self, key: String, val: String) -> Result<String> {
        self.engine.set(key.to_owned(), val.to_owned())?;
        Ok(format!(
            "set key: {} value: {} succesffully done !",
            key, val
        ))
    }

    fn execute_rm_cmd(&mut self, key: String) -> Result<String> {
        match self.engine.remove(key.to_owned()) {
            Ok(()) => Ok(format!("rm key: {} succesffully done !", key)),
            Err(_) => {
                println!("Key not found");
                return Err(KVError::FailGet(format!("{}", key)));
            }
        }
    }
}
