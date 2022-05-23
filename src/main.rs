extern crate alloc;
use tokio;

mod cli;
mod app;
mod config;
mod processor;
mod db;
use std::sync::{Arc, Mutex};
use crate::cli::CLI;
use app::App;
#[tokio::main]
async fn main() {

    let app = App::new();

    let mut cli = CLI::default()
        .get_arguments()
        .run(Arc::new(Mutex::new(app.await)));

}
