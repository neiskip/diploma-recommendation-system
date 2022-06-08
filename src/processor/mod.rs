// use alloc::alloc;

pub mod learning;
pub mod request;
pub mod result;
pub mod products;
pub mod recommender;

use std::io::{Read, Write};
use std::net::TcpStream;

use polars::prelude::{DataFrame, PrimitiveChunkedBuilder, Utf8ChunkedBuilder, Int32Type, ChunkedBuilder, Series, Float64Type, Int64Type};
use polars::prelude::{Result as PolarResult};
use smartcore::linalg::naive::dense_matrix::DenseMatrix;
use smartcore::linear::linear_regression::LinearRegression;

use crate::app::App;
use crate::processor::request::Request;

macro_rules! response_fmt {
    () => {
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}"
    }
}

pub struct Processor{
    pub(crate) data: Vec<products::Product>,
    pub(crate) recommender: recommender::Recommender
}

impl Processor{
    pub async fn new(mut db: sqlx::MySqlConnection) -> Self{
        let data = Processor::get_data(&mut db).await.unwrap();
        println!("{:#?}", data);
        Processor{ data: data, recommender: recommender::Recommender::new() }
    }
    pub async fn get_data(db: &mut sqlx::MySqlConnection) -> Result<Vec<products::Product>, (i32, String)> {
        let product_view = App::get_config().database.product_data_view.unwrap_or("products".to_string());
        let rating_view = App::get_config().database.rating_view.unwrap_or("user_ratings".to_string());
        let q_s =
            format!("SELECT r.user_id, p.item_id, CAST(r.rating as FLOAT) as rating FROM ")
                + rating_view.as_str() + " as r JOIN " + product_view.as_str() + " p ON r.item_id = p.item_id";
        println!("{}", q_s);
        match sqlx::query_as(&q_s).fetch_all(db).await{
            Ok(r) => Ok(r),
            Err(e) => Err((-50, e.to_string()))
        }
    }
    pub async fn run(&mut self, mut stream: TcpStream) -> Result<result::Result, (i32, String)>{
        let mut buffer = vec![0; 1024];
        // stream.read(&mut buffer).unwrap();
        let mut result: Result<result::Result, (i32, String)>;
        let mut request: Result<Request, (i32, String)> = Ok(Request::default());
        request = stream.read(&mut buffer).map_or_else(|e| Err((-1, e.to_string())), |_|{
            String::from_utf8(buffer)
                .map_or_else(|e| Err((-2, e.to_string())),
                |request_str|{
                    let start = request_str.find("{").unwrap_or(0 as usize);
                    let end = request_str.find("}").unwrap_or(0 as usize);
                    println!("{}", request_str);
                    println!("{}", &request_str[start..end+1]);
                    serde_json::from_str::<Request>(&request_str[start..end+1]).map_err(|e| (-3, e.to_string()))
            })
        }).and_then(|req|{
            if let Err(e) = req.validate(){
                Err(e)
            } else {
                Ok(req)
            }
        });
        if let Err(e) = &request {
            let output = serde_json::to_string(&result::Result{
                result: None,
                error: Some(e.1.clone())
            }).unwrap_or(format!("{{\"error\": \"{}\", \"result\": null }}", e.1.clone()));
            let output = format!(response_fmt!(), "400 Bad Request",output.len(), output);
            match stream.write(output.as_bytes()){
                Ok(_) => return Err(e.to_owned()),
                Err(e2) => return Err((-4, e2.to_string()))
            };
        }
        let request = request.unwrap();
        match request.method.as_str() {
            "by_user" => { self.user_recommend(&request) },
            "by_product" => { self.product_recommend(&request) }
            _ => Err((-5, "Method not found".to_string()))
        }.and_then(|result| {
            let response = serde_json::to_string(&result).unwrap_or(
                format!("{{\"error\": \"{}\", \"result\": null }}", "Could not convert to JSON")
            );
            println!("{}", response);
            let output = format!(response_fmt!(),"200 OK", response.len(), response);
            stream.write(output.as_bytes()).map_or_else(|e|{ Err((-5, e.to_string())) }, |_| Ok(result))
        }).map_err(|e|{
            let result = result::Result{
                result: None,
                error: Some(e.1.clone())
            };
            let response = serde_json::to_string(&result).unwrap_or(
                format!("{{\"error\": \"{}\", \"result\": null }}", "Could not convert to JSON")
            );
            let output = format!(response_fmt!(),"500 Internal Server Error", response.len(), response);
            stream.write(output.as_bytes()).map_or_else(|e|{ (-5, e.to_string()) }, |_| e)
        })
    }

    pub fn product_recommend(&mut self, request: &Request) -> Result<result::Result, (i32, String)> {

        if request.product_id.is_none() {
            return Err((-60, "Product Id is not set".to_string()));
        }
        println!("{:#?}", request);
         self.recommender.target_item_recs(&self.data, request.product_id.clone().unwrap() as u32,
                                          if request.limit.is_some(){ request.limit.clone().unwrap() as usize }
                                                else { 100_usize }, 20, 10).map_or_else(
             |e| Err(e),
             |r| Ok(result::Result{
                 result: Some(r),
                 error: None
             })
         )
    }
    pub fn user_recommend(&mut self, request: &Request) -> Result<result::Result, (i32, String)> {
        if request.user_id.is_none() {
            return Err((-60, "User Id is not set".to_string()));
        }
        self.recommender.target_user_recs(&self.data, request.user_id.clone().unwrap() as u32,
                                          if request.limit.is_some(){ request.limit.clone().unwrap() as usize }
                                          else { 100_usize }, 20, 10).map_or_else(
            |e| Err(e),
            |r| Ok(result::Result{
                result: Some(r),
                error: None
            })
        )
    }

    pub fn test(&mut self) -> Result<(), (i32, String)>{
        self.recommender.complex_train(&self.data);
        Ok(())
    }
}
/*
pub fn convert_to_series(values: &[products::Product]) -> (Series, Series, Series){
    let mut item_id_builder = PrimitiveChunkedBuilder::<Int64Type>::new("item_id", values.len());
    let mut title_builder = Utf8ChunkedBuilder::new("title", values.len(), values.len()*5);
    let mut description_builder = Utf8ChunkedBuilder::new("description", values.len(), values.len()*5);
    let mut category_id_builder = PrimitiveChunkedBuilder::<Int32Type>::new("category_id", values.len());

    values.iter().for_each(|v|{
        item_id_builder.append_value(v.item_id as i64);
        title_builder.append_value(v.title.clone());
        description_builder.append_option::<String>(v.description.clone());
    });
    (
        item_id_builder.finish().into(),
        title_builder.finish().into(),
        description_builder.finish().into(),
    )
}
*/