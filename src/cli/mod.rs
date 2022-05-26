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
        match self.args.get(&'c').unwrap_or(&String::from("")).as_str() {
            "search" => {

            },
            _ => {}
        };
        match self.args.get(&'r').unwrap_or(&String::from("")).as_str() {
            "output" => {
                app.lock().unwrap();
            },
            "server" => {
                let listener: TcpListener = std::net::TcpListener::bind("127.0.0.1:8046").unwrap();
                for stream in listener.incoming(){
                    let _app = Arc::clone(&app);
                    thread::spawn(move ||{
                        _app.lock().unwrap().handler.run(stream.unwrap());
                    });
                }
            },
            _ => {}
        };
    }
}