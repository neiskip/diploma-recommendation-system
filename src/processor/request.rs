use serde::Deserialize;

#[derive(Default, Deserialize, Clone)]
pub struct Request{
    pub(crate) user_id: Option<usize>,
    pub(crate) product_id: Option<usize>,
    pub(crate) method: String,
    pub(crate) limit: Option<u32>
}

impl Request{
    pub fn validate(&self) -> Result<(), (i32, String)> {
        match self.user_id{
            Some(0) => return Err((-51, "Not allowed user id".to_string())),
            _ => ()
        };
        match self.product_id{
            Some(0) => return Err((-52, "Not allowed product id".to_string())),
            _ => ()
        };
        Ok(())
    }
}