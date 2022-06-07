use polars::prelude::*;

pub struct Recommender {

}

impl Recommender {
    pub fn fuzzy_matching(map: &DataFrame, fav_movie: (i32, String)) -> Vec<(i32, String, f64)>{
        
        vec![]
    }
}