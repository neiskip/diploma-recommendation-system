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
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self.user_id{
            0 => return Err(ValidationError::new("Not allowed user id").into()),
            _ => ()
        };
        match self.product_id{
            Some(0) => return Err(ValidationError::new("Not allowed product id").into()),
            _ => ()
        };
        match self.category_id{
            Some(0) => return Err(ValidationError::new("Not allowed category id").into()),
            _ => ()
        };
        match &self.word{
            Some(s) =>{
                if s.is_empty() {
                    return Err(ValidationError::new("Empty search word").into());
                }
            },
            _ => ()
        };
        Ok(())
    }
}

#[derive(Debug)]
struct ValidationError{ message: String }

impl ValidationError{
    fn new(msg: &str) -> ValidationError {
        ValidationError { message: msg.to_string() }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for ValidationError {
    fn description(&self) -> &str {
        &self.message
    }
}