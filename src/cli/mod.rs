use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
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
                let (item_id, user_id) = {
                    let parsed = self.args.get(&'i').unwrap_or(&String::from(""))
                                        .split(',').filter_map(|x| x.parse::<u32>().ok()).collect::<Vec<u32>>();
                    (
                        parsed.get(0).expect("Option `i`: item ID is not set").to_owned(),
                        parsed.get(1).expect("Option `i`: user ID is not set").to_owned()
                    )
                };
                app.lock().unwrap().handler.test(item_id, user_id).unwrap();
            },
            "terminal" => {
                let mut _app = app.lock().unwrap();
                match self.args.get(&'m').unwrap_or(&String::from("")).as_str() {
                    "by_product" => _app.handler.product_recommend(&crate::processor::request::Request{
                        user_id: None,
                        product_id: match self.args.get(&'i').expect("Option `i` is not set").trim().parse(){
                            Ok(id) if id > 0 => Some(id),
                            Err(_) => panic!("Not numeric ID"),
                            Ok(_) => panic!("Incorrect ID")
                        },
                        method: "by_product".to_string(),
                        limit: match self.args.get(&'l'){
                            Some(count) => {
                                Some(count.parse::<u32>().unwrap())
                            },
                            _ => None
                        }
                    }),
                    "by_user" => _app.handler.user_recommend(&crate::processor::request::Request{
                        user_id: match self.args.get(&'i').expect("Option `i` is not set").trim().parse(){
                            Ok(id) if id > 0 => Some(id),
                            Err(_) => panic!("Not numeric ID"),
                            Ok(_) => panic!("Incorrect ID")
                        },
                        product_id: None,
                        method: "by_user".to_string(),
                        limit: match self.args.get(&'l'){
                            Some(count) => {
                                Some(count.parse::<u32>().unwrap())
                            },
                            _ => None
                        }
                    }),
                    _ => panic!("Method not found!")
                }.and_then(|r|{
                    println!("{}", serde_json::to_string(&r).unwrap_or(format!("{{ \"error\": \"{}\", \"result\": null }}", "Unable to serialize the result")));
                    Ok(r)
                }).map_err(|e|{
                    println!("{{ \"error\": {}, \"result\": null }}", e.1);
                    e
                }).ok();
            },
            "server" => {
                let listener: TcpListener = std::net::TcpListener::bind("127.0.0.1:8047").unwrap();

                for stream in listener.incoming(){
                    let _app = Arc::clone(&app);
                    thread::spawn(move ||{
                        futures::executor::block_on(_app.lock().unwrap().handler.run(stream.unwrap()));
                    });
                }
            },
            _ => {}
        };
    }
}