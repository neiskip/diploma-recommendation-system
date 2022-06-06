use serde::Deserialize;

#[derive(Default, Deserialize, Clone)]
pub struct Request{
    pub(crate) user_id: usize,
    pub(crate) product_id: Option<usize>,
    pub(crate) category_id: Option<usize>,
    pub(crate) word: Option<String>,
    pub(crate) method: String
}

impl Request{
    pub fn validate(&self) -> Result<(), (i32, String)> {
        match self.user_id{
            0 => return Err((-51, "Not allowed user id".to_string())),
            _ => ()
        };
        match self.product_id{
            Some(0) => return Err((-52, "Not allowed product id".to_string())),
            _ => ()
        };
        match self.category_id{
            Some(0) => return Err((-53, "Not allowed category id".to_string())),
            _ => ()
        };
        match &self.word{
            Some(s) =>{
                if s.is_empty() {
                    return Err((-54, "Empty search word".to_string()));
                }
            },
            _ => ()
        };
        Ok(())
    }
}