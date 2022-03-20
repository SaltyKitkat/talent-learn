use crate::{
    cli::{Request, Response},
    Result,
};
use serde::Deserialize;
use serde_json::Deserializer;
use std::net::{TcpStream, ToSocketAddrs};

pub struct KvsClient {
    tcp_stream: TcpStream,
}

impl KvsClient {
    pub fn connect(socket: impl ToSocketAddrs) -> Result<Self> {
        let tcp_stream = TcpStream::connect(socket)?;
        Ok(Self { tcp_stream })
    }
    pub fn send_request(&mut self, request: &Request) -> Result<Response> {
        serde_json::to_writer(&mut self.tcp_stream, &request)?;
        Ok(Response::deserialize(&mut Deserializer::from_reader(
            &self.tcp_stream,
        ))?)
    }
}
