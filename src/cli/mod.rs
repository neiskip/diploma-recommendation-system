use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use serde::de::Unexpected::Str;
use crate::app::App;
#[derive(Default, Debug)]
pub struct CLI{
    package_name: String,
    args: HashMap<char, String>
}
impl CLI{
    pub fn get_arguments(&mut self) -> &mut Self{
        if !self.package_name.is_empty(){
            return self;
        }
        let args: Vec<String> = std::env::args().collect();
        self.package_name = args[0].clone();
        for i in 1..args.len()-1{
            let re = regex::Regex::new(r"^-[a-z]{1}$").unwrap();
            if re.is_match(&args[i]) {
                let mut flag = args[i].clone();
                flag.remove(0);
                let val = if !re.is_match(&args[i+1]){
                    args[i+1].clone()
                } else {
                    String::from("")
                };
                self.args.insert(flag.chars().nth(0).unwrap(), val);
            }
        }
        println!("{:#?}", self.args);
        self
    }
    pub fn run(&self, app: Arc<Mutex<App>>){
        match self.args.get(&'r').unwrap_or(&String::from("")).as_str() {
            "test" => {
                app.lock().unwrap().handler.test().unwrap();
            },
            "terminal" => {
                let mut _app = app.lock().unwrap();
                let result = match self.args.get(&'m').unwrap_or(&String::from("")).as_str() {
                    "by_product" => _app.handler.product_recommend(&crate::processor::request::Request{
                        user_id: None,
                        product_id: match self.args.get(&'i').expect("Option `i` is not set").trim().parse(){
                            Ok(id) if id > 0 => Some(id),
                            Err(_) => panic!("Not numeric ID"),
                            Ok(id) => panic!("Incorrect ID")
                        },
                        method: "by_product".to_string(),
                        limit: None
                    }),
                    _ => panic!("Method not found!")
                };
            },
            "server" => {
                let listener: TcpListener = std::net::TcpListener::bind("127.0.0.1:8047").unwrap();

                for stream in listener.incoming(){
                    let _app = Arc::clone(&app);
                    thread::spawn(move ||{
                        futures::executor::block_on(_app.lock().unwrap().handler.run(stream.unwrap())).unwrap();
                    });
                }
            },
            _ => {}
        };
    }
}