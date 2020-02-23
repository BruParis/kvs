#[macro_use]
extern crate clap;
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

                let mut kv_client = KVClient::new(addr, req)?;
                if let Some(resp) = kv_client.connect()? {
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
                let mut kv_client = KVClient::new(addr, req)?;
                kv_client.connect()?;
            }
            "rm" => {
                let req = KVRequest::Rm {
                    key: key.to_owned(),
                };
                let mut kv_client = KVClient::new(addr, req)?;
                match kv_client.connect() {
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
    let mut ip_addr = "";
    let mut port = "";
    if let Some(arg) = sub_input.value_of("addr") {
        let split_vec: Vec<&str> = arg.split(':').collect();
        if split_vec.len() != 2 {
            process::exit(1);
        }
        ip_addr = split_vec[0];
        port = split_vec[1];
    }
    let tcp_addr = format!("{}:{}", ip_addr, port);
    // println!("IPAdress: {}", tcpAddr);

    tcp_addr
}
