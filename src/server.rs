use serde_json::Deserializer;
use slog::{Drain, Logger};
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::common_struct::KVRequest;
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
        reader.read(&mut buffer)?;

        let executed = self.executeCmd(buffer, log);
        let resp: String;
        match executed {
            Ok(val) => resp = format!("val {}", val),
            Err(error) => resp = format!("error {}", error),
        }

        let mut writer = BufWriter::new(stream);
        writer.write_all(resp.as_bytes())?;

        Ok(())
    }

    fn executeCmd(&mut self, buffer: [u8; 512], log: &Logger) -> Result<String> {

        let serialized: String = buffer.into_iter().map(|c| *c as char).collect();
        let mut deserialized = Deserializer::from_str(&serialized).into_iter::<KVRequest>();
        let mut resp = String::from("");

        if let Some(Ok(req_iter)) = deserialized.next() {
            info!(log, "  -> received request: {:#?}", req_iter);

            match req_iter {
                KVRequest::Get { key } => resp = self.executeGetCmd(key)?,
                KVRequest::Set { key, val } => resp = self.executeSetCmd(key, val)?,
                KVRequest::Rm { key } => resp = self.executeRmCmd(key)?,
            }
        }

        Ok(resp)
    }

    fn executeGetCmd(&mut self, key: String) -> Result<String> {
        if let Some(value) = self.engine.get(key.to_owned())? {
            println!("{}", value);
            Ok(value)
        } else {
            println!("Key not found");
            Ok(format!("Failed to get value from key: {}", key))
        }
    }

    fn executeSetCmd(&mut self, key: String, val: String) -> Result<String> {

        self.engine.set(key.to_owned(), val.to_owned())?;
        Ok(format!(
            "set key: {} value: {} succesffully done !",
            key, val
        ))
    }

    fn executeRmCmd(&mut self, key: String) -> Result<String> {
        match self.engine.remove(key.to_owned()) {
            Ok(()) => Ok(format!("rm key: {} succesffully done !", key)),
            Err(_) => {
                println!("Key not found");
                return Err(KVError::FailGet(format!(
                    "{}",
                    key
                )));
            }
        }
    }
}
