
#[derive(sqlx::FromRow, Debug)]
pub struct Product{
    pub(crate) item_id: u64,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) user_id: u64,
    pub(crate) rating: f32
}