use crate::common_struct::KVRequest;
use crate::error::Result;
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::TcpStream;

pub struct KVClient {
    req: KVRequest,
    writer: BufWriter<TcpStream>,
    reader: BufReader<TcpStream>,
}

impl KVClient {
    pub fn new(addr: String, req: KVRequest) -> Result<KVClient> {
        let stream_w = TcpStream::connect(&addr)?;
        let stream_r = stream_w.try_clone()?;
        let writer = BufWriter::new(stream_w);
        let reader = BufReader::new(stream_r);

        println!("request: {:#?}", req);
        Ok(KVClient {
            req,
            writer,
            reader,
        })
    }

    pub fn connect(&mut self) -> Result<Option<String>> {
        let mut buf = vec![];
        serde_json::to_writer(&mut buf, &self.req)?;
        let _len = buf.len() as u64;
        self.writer.write_all(&buf)?;
        self.writer.flush()?;

        let mut buffer = [0; 512];
        self.reader.read(&mut buffer)?;
        let res = std::str::from_utf8(&buffer)?;
        Ok(Some(res.to_owned()))
    }
}
