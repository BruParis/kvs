#[macro_use]
extern crate clap;

#[macro_use]
extern crate dotenv_codegen;
use clap::App;
// use std::env;
use kvs::{KVClient, KVRequest, Result};

use std::process;

fn main() -> Result<()> {
    /*println!("{}", dotenv!("PORT"));
    let key = "PATH";
    match env::var_os(key) {
        Some(val) => println!("{}: {:?}", key, val),
        None => println!("{} is not defined in the environment", key),
    }*/

    let yaml = load_yaml!("cli-client.yml");
    let m = App::from_yaml(yaml).get_matches();

    if let (cmd, Some(sub_input)) = m.subcommand() {
        let addr = addr_subcmd_arg(sub_input);
        let key = sub_input.value_of("KEY").unwrap();
        // println!("{:#?}", sub_input);

        match cmd {
            "get" => {
                let req = KVRequest::Get {
                    key: key.to_owned(),
                };

                let mut kvClient = KVClient::new(addr, req)?;
                if let Some(resp) = kvClient.connect()? {
                    println!("{}", resp);
                } else {
                    println!("Key not found");
                }
            }
            "set" => {
                let val = sub_input.value_of("VAL").unwrap();
                let req = KVRequest::Set {
                    key: key.to_owned(),
                    val: val.to_owned(),
                };
                let mut kvClient = KVClient::new(addr, req)?;
                kvClient.connect()?;
            }
            "rm" => {
                let req = KVRequest::Rm {
                    key: key.to_owned(),
                };
                let mut kvClient = KVClient::new(addr, req)?;
                match kvClient.connect() {
                    Ok(_) => {}
                    Err(_) => {
                        eprintln!("Key not found");
                        process::exit(1)
                    }
                }
            }
            _ => process::exit(1),
        }

        process::exit(0);
    } else {
        process::exit(1);
    }
}

fn addr_subcmd_arg(sub_input: &clap::ArgMatches) -> String {
    let mut ipAddr = "";
    let mut port = "";
    if let Some(arg) = sub_input.value_of("addr") {
        let splitVec: Vec<&str> = arg.split(":").collect();
        if splitVec.len() != 2 {
            process::exit(1);
        }
        ipAddr = splitVec[0];
        port = splitVec[1];
    }
    let tcpAddr = format!("{}:{}", ipAddr, port);
    // println!("IPAdress: {}", tcpAddr);

    tcpAddr
}
