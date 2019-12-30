#[macro_use]
extern crate clap;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use clap::App;
use kvs::{KVRequest, KVServer, Result};
use serde_json::Deserializer;
use slog::{Drain, Logger};
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process;

// use hello_web_server::ThreadPool;

fn main() -> Result<()> {
    let decorator = slog_term::PlainDecorator::new(std::io::stdout());
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());

    info!(log, "kvs-server {}", env!("CARGO_PKG_VERSION"));

    let yaml = load_yaml!("cli-server.yml");
    let m = App::from_yaml(yaml).get_matches();

    let mut ipAddr = "";
    let mut port = "";
    let mut engine = "";
    if let Some(arg) = m.value_of("addr") {
        let splitVec: Vec<&str> = arg.split(":").collect();
        if splitVec.len() != 2 {
            process::exit(1);
        }
        ipAddr = splitVec[0];
        port = splitVec[1];
    }

    if let Some(arg) = m.value_of("engine") {
        if arg != "kvs" && arg != "sled" {
            process::exit(1);
        }
        engine = arg;
    }

    let tcpAddr = format!("{}:{}", ipAddr, port);
    info!(log, "Storage engine: {}", engine);
    info!(log, "Listening on port: {}", tcpAddr);

    start_server(tcpAddr.to_owned(), &log)?;

    Ok(())
}

fn start_server(addr: String, log: &Logger) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    let mut kvServer = KVServer::new()?;

    for stream in listener.incoming() {
        let stream = stream?;
        handle_connection(&stream, &mut kvServer, &log);
    }

    Ok(())

    /*let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(2) {
        let stream = stream?;

        pool.execute(|| {
            handle_connection(stream);
        });
    }*/
}

fn handle_connection(stream: &TcpStream, kvServer: &mut KVServer, log: &Logger) -> Result<()> {
    let _peer_addr = stream.peer_addr()?;
    let mut reader = BufReader::new(stream);

    let mut buffer = [0; 512];
    reader.read(&mut buffer)?;

    let executed = executeCmd(buffer, kvServer, log);
    let resp: String;
    match executed {
        Ok(val) => resp = format!("val {}", val),
        Err(error) => resp = format!("error {}", error),
    }

    let mut writer = BufWriter::new(stream);
    writer.write_all(resp.as_bytes())?;

    Ok(())
}

fn executeCmd(buffer: [u8; 512], kvServer: &mut KVServer, log: &Logger) -> Result<String> {
    let serialized: String = buffer.into_iter().map(|c| *c as char).collect();
    let mut deserialized = Deserializer::from_str(&serialized).into_iter::<KVRequest>();
    let mut resp = String::from("");

    if let Some(Ok(req_iter)) = deserialized.next() {
        info!(log, "  -> received request: {:#?}", req_iter);

        match req_iter {
            KVRequest::Get { key } => resp = kvServer.executeGetCmd(key)?,
            KVRequest::Set { key, val } => resp = kvServer.executeSetCmd(key, val)?,
            KVRequest::Rm { key } => resp = kvServer.executeRmCmd(key)?,
        }
    }

    Ok(resp)
}
