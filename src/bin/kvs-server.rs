#[macro_use]
extern crate clap;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use clap::App;
use kvs::{KVServer, KVStore, Result, SledKVEngine};
use slog::{Drain, Logger};
use std::process;

// use hello_web_server::ThreadPool;

enum Engine {
    kvs,
    sled,
}

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
    // info!(log, "Storage engine: {}", engine);
    // info!(log, "Listening on port: {}", tcpAddr);

    eprintln!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    eprintln!("addr {}", tcpAddr);

    start_server(tcpAddr.to_owned(), engine.to_owned(), &log)?;

    Ok(())
}

fn start_server(addr: String, engine: String, log: &Logger) -> Result<()> {
    let current_path = std::env::current_dir()?;
    let engine = current_engine(engine, log);

    match engine {
        Some(Engine::kvs) => {
            let mut server = KVServer::new(KVStore::open(&current_path)?);
            server.run(addr, log)?;
        }
        Some(Engine::sled) => {
            let mut server = KVServer::new(SledKVEngine::open(&current_path)?);
            server.run(addr, log)?;
        }
        None => {}
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

fn current_engine(engine: String, log: &Logger) -> Option<Engine> {
    match engine.as_ref() {
        "kvs" => Some(Engine::kvs),
        "sled" => Some(Engine::sled),
        _ => {
            warn!(log, "Error -> engine {} found not found", engine);
            None
        }
    }
}
