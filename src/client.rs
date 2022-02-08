use crate::{
    cli::{Request, Response},
    error::Result,
};
use serde::Deserialize;
use serde_json::Deserializer;
use std::{
    io::{BufReader, BufWriter, Write},
    net::{TcpStream, ToSocketAddrs},
};

pub struct KvsClient {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl KvsClient {
    pub fn connect(socket: impl ToSocketAddrs) -> Result<Self> {
        let stream = TcpStream::connect(socket)?;
        let reader = BufReader::new(stream.try_clone()?);
        let writer = BufWriter::new(stream);
        Ok(Self { reader, writer })
    }
    pub fn send_request(&mut self, request: &Request) -> Result<Response> {
        serde_json::to_writer(&mut self.writer, &request)?;
        self.writer.flush()?;
        Ok(Response::deserialize(&mut Deserializer::from_reader(
            &mut self.reader,
        ))?)
    }
}
