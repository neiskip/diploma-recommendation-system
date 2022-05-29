// use alloc::alloc;
use std::any::TypeId;
use std::io::{Read, Write};
use std::net::TcpStream;
pub mod learning;
mod request;
pub mod result;
pub mod products;
use polars::series::Series;
use crate::app::App;
use crate::processor::request::Request;

pub struct Processor{
    pub(crate) db_connect: sqlx::any::AnyConnection
}

impl Processor{
    pub fn new(db: sqlx::any::AnyConnection) -> Self{
        Processor{ db_connect: db }
    }
    pub async fn run(&mut self, mut stream: TcpStream) -> Result<result::Result, Box<dyn std::error::Error>>{
        let mut buffer = vec![0; 1024];
        // stream.read(&mut buffer).unwrap();
        let mut request: Request;
        stream.read(&mut buffer).and_then(|_|{
            String::from_utf8(buffer).and_then(|request_str|{
                let start = request_str.find("{").unwrap_or(0 as usize);
                let end = request_str.find("}").unwrap_or(0 as usize);
                match serde_json::from_str(&request_str[start..end]){
                    Ok(r) => request = r,
                    Err(_) => {}
                };
                Ok(request_str)
            }).unwrap();
            Ok(0)
        });
        // buffer = buffer.iter().filter(|&i| { *i != 0_u8 }).map(|&i| { i }).collect();
        // let payload = String::from_utf8(buffer).unwrap();
        // let start_json_i = payload.find("{").unwrap();
        // let end_json_i = payload.find("}").unwrap();
        // println!("{}", &payload[start_json_i..end_json_i+1]);
        // let request: Request = serde_json::from_str(&payload[start_json_i..end_json_i+1]).unwrap();
        match request.validate() {
            Ok(_) => (),
            Err(e) => return Err(e)
        };
        let result = match request.method.as_str() {
            "product_recommend" => { self.product_recommend(&request).await },
            "by_category" => { self.recommend_by_category(&request).await }
            _ => Err(MethodNotFound::new("Method not found!").into())
        }.unwrap();
        println!("{:#?}", result);
        let response = serde_json::to_string(&result).unwrap();
        let output = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", response.len(), response);
        stream.write(output.as_bytes());
        // serde_json::to_writer(stream, &result);
        Ok(result)
    }

    pub async fn product_recommend(&mut self, request: &Request) -> Result<result::Result, Box<dyn std::error::Error>> {
        
        let q_s = r#"SELECT * FROM "#.to_owned() + App::get_config().database.product_data_view.unwrap().as_str() + r#";"#;
        let raw_data: Vec<products::Product> = sqlx::query_as(&q_s).fetch_all(&mut self.db_connect).await?;
        let collection = convert_to_series(&raw_data);
        println!("{:#?}\n{:#?}", raw_data, collection);
        let df = polars::prelude::DataFrame::new(vec![collection.0, collection.1, collection.2, collection.3]).unwrap();
        println!("{:#?}", df);
        Ok(result::Result{ error: None, result: None })
    }
    pub async fn recommend_by_category(&mut self, request: &Request) -> Result<result::Result, Box<dyn std::error::Error>> {
        
        Ok(result::Result{ error: None, result: None })

    }
}

#[derive(Debug)]
pub struct MethodNotFound{ message: String }

impl MethodNotFound{
    fn new(msg: &str) -> MethodNotFound {
        MethodNotFound { message: msg.to_string() }
    }
}

impl std::fmt::Display for MethodNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MethodNotFound {
    fn description(&self) -> &str {
        &self.message
    }
}

pub fn convert_to_series(values: &[products::Product]) -> (Series, Series, Series, Series){
    use polars::prelude::*;
    let mut item_id_builder = PrimitiveChunkedBuilder::<Int32Type>::new("item_id", values.len());
    let mut title_builder = Utf8ChunkedBuilder::new("title", values.len(), values.len()*5);
    let mut description_builder = Utf8ChunkedBuilder::new("description", values.len(), values.len()*5);
    let mut category_id_builder = PrimitiveChunkedBuilder::<Int32Type>::new("category_id", values.len());

    values.iter().for_each(|v|{
        item_id_builder.append_value(v.item_id);
        title_builder.append_value(&v.title);
        description_builder.append_option::<String>(v.description.clone());
        category_id_builder.append_option(v.category_id);
    });
    (
        item_id_builder.finish().into(),
        title_builder.finish().into(),
        description_builder.finish().into(),
        category_id_builder.finish().into()
    )
}