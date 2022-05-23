// use alloc::alloc;
use std::any::TypeId;
use std::io::Read;
use std::net::TcpStream;
pub mod learning;
mod request;

use crate::processor::request::Request;

#[derive(Default)]
pub struct Processor{
    pub(crate) db_connect: Option<sqlx::any::AnyConnection>
}

impl Processor{
    pub fn new(db: sqlx::any::AnyConnection) -> Self{
        Processor{ db_connect: Some(db) }
    }
    pub fn handle(&self, mut stream: TcpStream) -> Vec<u8>{
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();

        let payload = String::from_utf8_lossy(&buffer[..]);
        let request: Request = serde_json::from_str(payload.as_ref()).unwrap();

        vec![]
    }

    pub fn recommend(&self, request: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let request: Request = serde_json::from_str(request.as_ref()).unwrap();
        match request.validate() {
            Ok(_) => (),
            Err(e) => return Err(e)
        };
        Ok(vec![])
    }
}
