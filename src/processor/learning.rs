use std::time::SystemTime;
use crate::processor::Processor;

pub trait Learner{
    fn learn(&self, data: Vec<i32>) -> Vec<usize>;
}

impl Learner for Processor {
    fn learn(&self, data: Vec<i32>) -> Vec<usize> {
        vec![]
    }
}

impl Processor{
    pub fn get_data(db: sqlx::any::AnyConnection)-> Vec<usize>{
        vec![]
    }
}