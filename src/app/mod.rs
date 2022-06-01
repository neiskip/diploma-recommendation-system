use crate::config::Config;
use crate::processor::Processor;
use sqlx::any::{AnyColumn, AnyConnection, AnyKind, AnyRow};
use serde::Deserialize;
use sqlx::Connection;

pub struct App{
    pub(crate) config: Box<Config>,
    pub(crate) handler: Processor
}

impl App{
    pub async fn new() -> Self{
        let content = std::fs::read_to_string("Application.toml")
                                .expect("Unable to read application config file");
        let config: Config = toml::from_str(content.as_str()).unwrap();
        if config.database.driver.is_empty() {
            println!("Database driver is not set"); std::process::exit(1);
        }
        if config.database.login.is_none() {
            println!("Database login is not set"); std::process::exit(1);
        }
        if config.database.host.is_empty() {
            println!("Database host is not set"); std::process::exit(1);
        }
        if config.database.name.is_empty() {
            println!("Database name is not set"); std::process::exit(1);
        }
        if config.data_definition == "from_view"
            && (config.database.rating_view.is_none()
                || config.database.product_data_view.is_none()){
            println!("Data views' name is not set"); std::process::exit(1);
        }
        let db = sqlx::MySqlConnection::connect((config.database.driver.clone() + &"://".to_string()
        + &config.database.login.clone().unwrap_or(String::from(""))
        + &{ if config.database.password.is_none(){ "" } else { ":" } }.to_string()
        + &config.database.password.clone().unwrap_or(String::from(""))
        + &"@".to_string() + &config.database.host.clone()
        + &"/".to_string() + &config.database.name.clone()).as_str()).await.unwrap();
        // let db = AnyConnection::connect((config.database.driver.clone() + &"://".to_string()
        //     + &config.database.login.clone().unwrap_or(String::from(""))
        //     + &{ if config.database.password.is_none(){ "" } else { ":" } }.to_string()
        //     + &config.database.password.clone().unwrap_or(String::from(""))
        //     + &"@".to_string() + &config.database.host.clone()
        //     + &"/".to_string() + &config.database.name.clone()).as_str()).await.unwrap();
        App{ config: Box::new(config), handler: Processor::new(db) }
    }
    pub fn get_config() -> Config {
        toml::from_str::<Config>(std::fs::read_to_string("Application.toml")
        .expect("Unable to read application config file").as_str()).unwrap()
    }
}