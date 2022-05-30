// use alloc::alloc;

pub mod learning;
mod request;
pub mod result;
pub mod products;

use std::any::TypeId;
use std::io::{Read, Write};
use std::net::TcpStream;
use serde::Serialize;

use polars::prelude::{DataFrame, PrimitiveChunkedBuilder, Utf8ChunkedBuilder, Int32Type, ChunkedBuilder, Series};
use polars::prelude::{Result as PolarResult};
use smartcore::linalg::naive::dense_matrix::DenseMatrix;
use smartcore::linalg::BaseMatrix;
use smartcore::metrics::mean_squared_error;
use smartcore::linear::linear_regression::LinearRegression;

use crate::app::App;
use crate::processor::request::Request;

macro_rules! response_fmt {
    () => {
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}"
    }
}

pub struct Processor{
    pub(crate) db_connect: sqlx::any::AnyConnection
}

impl Processor{
    pub fn new(db: sqlx::any::AnyConnection) -> Self{
        Processor{ db_connect: db }
    }
    pub async fn run(&mut self, mut stream: TcpStream) -> Result<result::Result, Box<dyn std::error::Error>>{
        let mut is_exit = false;
        let mut buffer = vec![0; 1024];
        // stream.read(&mut buffer).unwrap();
        let mut result: Option<Result<result::Result, Box<dyn std::error::Error>>> = None;
        let mut request: Result<Request, Box<dyn std::error::Error>> = Ok(Request::default());
        match stream.read(&mut buffer).and_then(|i|{
            match String::from_utf8(buffer){
                Ok(request_str) => {
                    let start = request_str.find("{").unwrap_or(0 as usize);
                    let end = request_str.find("}").unwrap_or(0 as usize);
                    request = match serde_json::from_str::<Request>(&request_str[start..end]){
                        Ok(ref r) => Ok(r.clone()),
                        Err(e) => Err(e.into())
                    };
                },
                Err(e) => {
                    request = Err(e.clone().into());
                }
            };
            Ok(i)
        }){
            Ok(_) => {},
            Err(e) => { request = Err(Box::new(e)); }
        };
        match request{
            Ok(ref r) => {
                match r.validate() {
                    Ok(_) => {},
                    Err(e) => { result = Some(Err(e)); is_exit = true; }
                };
            }
            Err(ref e) => {
                result = Some(Err(e.to_string().into())); is_exit = true;
            }
        };
        if let Some(Err(e)) = result {
            let output = serde_json::to_string(&result::Result{
                result: None,
                error: Some(e.to_string())
            }).unwrap_or(format!("{{\"error\": \"{}\", \"result\": null }}", e.to_string()));
            let output = format!(response_fmt!(), "400 Bad Request",output.len(), output);
            match stream.write(output.as_bytes()){
                Ok(_) => return Err(e),
                Err(e2) => return Err(e2.into())
            };
        };
        let request = request.unwrap();
        // buffer = buffer.iter().filter(|&i| { *i != 0_u8 }).map(|&i| { i }).collect();
        // let payload = String::from_utf8(buffer).unwrap();
        // let start_json_i = payload.find("{").unwrap();
        // let end_json_i = payload.find("}").unwrap();
        // println!("{}", &payload[start_json_i..end_json_i+1]);
        // let request: Request = serde_json::from_str(&payload[start_json_i..end_json_i+1]).unwrap();
        result = Some(match request.method.as_str() {
            "product_recommend" => { self.product_recommend(&request).await },
            "by_category" => { self.recommend_by_category(&request).await }
            _ => Ok(result::Result{ result: None, error: Some("Method not found".to_string()) })
        });
        if let Some(r) = &result {
            match r {
                Ok(_) => {},
                Err(ref e) => {
                    let output = serde_json::to_string(&result::Result{
                        result: None,
                        error: Some(e.to_string())
                    }).unwrap_or(format!("{{\"error\": \"{}\", \"result\": null }}", e.to_string()));
                    let output = format!(response_fmt!(), "400 Bad Request",output.len(), output);
                    match stream.write(output.as_bytes()){
                        Ok(_) => return Err(e.to_string().into()),
                        Err(e2) => return Err(e2.into())
                    };
                }
            }
        }
        let result = result.unwrap();
        let response = serde_json::to_string(&result.as_ref().unwrap()).unwrap_or(
            format!("{{\"error\": \"{}\", \"result\": null }}", "Could not convert to JSON")
        );
        let output = format!(response_fmt!(),"200 OK", response.len(), response);
        stream.write(output.as_bytes());
        // serde_json::to_writer(stream, &result);
        result
    }

    pub async fn product_recommend(&mut self, request: &Request) -> Result<result::Result, Box<dyn std::error::Error>> {
        
        let q_s = r#"SELECT * FROM "#.to_owned() + App::get_config().database.product_data_view.unwrap().as_str() + r#";"#;
        let raw_data: Vec<products::Product> = sqlx::query_as(&q_s).fetch_all(&mut self.db_connect).await?;
        let collection = convert_to_series(&raw_data);
        println!("{:#?}\n{:#?}", raw_data, collection);
        let df = DataFrame::new(vec![collection.0, collection.1, collection.2, collection.3]).unwrap();
        let features = df.select(vec![
            "description"
            "title",
            "category_id"
        ]).unwrap();
        let target = df.select(["item_id"]).unwrap();
        let target_array = target.unwrap().to_ndarray::<Int32Type>().unwrap();
        let mut y: Vec<i32> = Vec::new();
        for val in target_array.iter(){
            y.push(*val);
        }
        let xmatrix = convert_features_to_matrix(&features.unwrap()).unwrap();
        use smartcore::model_selection::train_test_split;
        let (x_train, x_test, y_train, y_test) = train_test_split(&xmatrix, &y, 0.3, true);
        let linear_regression = LinearRegression::fit(&x_train, &y_train, Default::default()).unwrap();
        // predictions
        let preds = linear_regression.predict(&x_test).unwrap();
        // metrics
        let mse = mean_squared_error(&y_test, &preds);
        let cancer_data = smartcore::dataset::breast_cancer::load_dataset();
        // let (x_train, x_test, y_train, y_test) = train_test_split(DenseMatrix::from_array(df.c));
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




pub fn convert_features_to_matrix(in_df: &DataFrame) -> PolarResult<DenseMatrix<i32>>{

    /* function to convert feature dataframe to a DenseMatrix, readable by smartcore*/

    let nrows = in_df.height();
    let ncols = in_df.width();
    // convert to array
    let features_res = in_df.to_ndarray::<Int32Type>().unwrap();
    // create a zero matrix and populate with features
    let mut xmatrix: DenseMatrix<i32> = BaseMatrix::zeros(nrows, ncols);
    // populate the matrix
    // initialize row and column counters
    let mut col:  u32 = 0;
    let mut row:  u32 = 0;

    for val in features_res.iter(){

        // Debug
        //println!("{},{}", usize::try_from(row).unwrap(), usize::try_from(col).unwrap());
        // define the row and col in the final matrix as usize
        let m_row = usize::try_from(row).unwrap();
        let m_col = usize::try_from(col).unwrap();
        // NB we are dereferencing the borrow with *val otherwise we would have a &val type, which is
        // not what set wants
        xmatrix.set(m_row, m_col, *val);
        // check what we have to update
        if (m_col==ncols-1) {
            row+=1;
            col = 0;
        } else{
            col+=1;
        }
    }

    // Ok so we can return DenseMatrix, otherwise we'll have std::result::Result<Densematrix, PolarsError>
    Ok(xmatrix)
}
