use serde::Serialize;


#[derive(Serialize, Debug, Default)]
pub struct Result{
    pub(crate) error: Option<String>,
    pub(crate) result: Option<Vec<(u32, f64)>>
}