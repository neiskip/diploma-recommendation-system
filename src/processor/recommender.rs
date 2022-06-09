use polars::export::chrono::format::StrftimeItems;
use sqlx::encode::IsNull::No;
use tokio::count;
use crate::processor::products;

pub struct Recommender {
    core: Option<discorec::Recommender<String, String>>
}

impl Recommender {
    pub fn new() -> Recommender {
        Recommender{ core: None }
    }
    pub fn fit_recommender(&mut self, data: &Vec<products::Product>, factors: u32, iterations: u32)
        -> Result<&mut Self, (i32, String)>{
        let mut dataset = discorec::Dataset::new();

        for product in data {
            dataset.push(
                product.user_id.to_string(),
                product.item_id.to_string(),
                product.rating
            );
        }
        let mut builder = discorec::RecommenderBuilder::new();
        builder
            .factors(factors)
            .iterations(iterations);
        self.core = Some(builder.fit_explicit(&dataset));
        Ok( self )
    }

    pub fn user_recs(&mut self, data: &Vec<products::Product>, count: usize, factors: u32, iterations: u32)
    ->Result<Vec<(String, String, f32)>, (i32, String)>{

        if self.core.is_none() {
            self.fit_recommender(data, factors, iterations)?;
        }
        let recommender = match self.core {
            Some(ref r) => r,
            None => return Err((-71, "Recommender core not found".to_string()))
        };
        let mut user_ids = recommender.user_ids().clone();
        user_ids.sort_unstable();

        let mut output = Vec::with_capacity(count);
        for user in user_ids.iter(){
            for (recommended_item, score) in recommender.user_recs(user, count){
                output.push((user.clone(), recommended_item.clone(), score.clone()));
            }
        }
        Ok(output)
    }

    pub fn item_recs(&mut self, data: &Vec<products::Product>, count: usize, factors: u32, iterations: u32)
                     ->Result<Vec<(String, String, f32)>, (i32, String)>{

        if self.core.is_none() {
            self.fit_recommender(data, factors, iterations)?;
        }
        let recommender = match self.core {
            Some(ref r) => r,
            None => return Err((-71, "Recommender core not found".to_string()))
        };
        let mut item_ids = recommender.item_ids().clone();
        item_ids.sort_unstable();

        let mut output = Vec::with_capacity(count);
        for item in item_ids.iter(){
            for (recommended_item, score) in recommender.item_recs(item, count){
                output.push((item.clone(), recommended_item.clone(), score.clone()));
            }
        }
        Ok(output)
    }

    pub fn target_user_recs(&mut self, data: &Vec<products::Product>, user_id: u32, count: usize, factors: u32, iterations: u32)
        ->Result<Vec<(String, f32)>, (i32, String)>{
        if self.core.is_none() {
            self.fit_recommender(data, factors, iterations)?;
        }
        let recommender = match self.core {
            Some(ref r) => r,
            None => return Err((-71, "Recommender core not found".to_string()))
        };
        let output = recommender.user_recs(&user_id.to_string(), count);

        Ok(output.iter().map(|i| (i.0.to_owned(), i.1)).collect())
    }

    pub fn target_item_recs(&mut self, data: &Vec<products::Product>, item_id: u32, count: usize, factors: u32, iterations: u32)
        ->Result<Vec<(String, f32)>, (i32, String)> {
        if self.core.is_none() {
            self.fit_recommender(data, factors, iterations)?;
        }
        let recommender = match self.core {
            Some(ref r) => r,
            None => return Err((-71, "Recommender core not found".to_string()))
        };
        let output = recommender.item_recs(&item_id.to_string(), count);
        Ok(output.iter().map(|i| (i.0.to_owned(), i.1)).collect())
    }

    pub fn complex_train(&mut self, data: &Vec<products::Product>, item_id: u32, user_id: u32) -> Result<(f32, f32, f32, ndarray::ArrayView1<f32>, ndarray::ArrayView1<f32> ), (i32, String)> {
        let mut dataset = discorec::Dataset::new();
            data.iter().for_each(|p|{
                dataset.push(p.user_id.to_owned().to_string(), p.item_id.to_owned().to_string(), p.rating.to_owned())
        });
        if self.core.is_none() {
            
            self.core = Some(discorec::RecommenderBuilder::new().iterations(100).factors(20).fit_explicit(&dataset));
        }
        let recommender = match self.core {
            Some(ref r) => r,
            None => return Err((-71, "Recommender core not found".to_string()))
        };
        Ok((
            recommender.predict(&user_id.to_string(), &item_id.to_string()),
            recommender.rmse(&dataset),
            recommender.global_mean(),
            recommender.item_factors(&item_id.to_string()).unwrap(),
            recommender.user_factors(&user_id.to_string()).unwrap()
        ))
    }
}