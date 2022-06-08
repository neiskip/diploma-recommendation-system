
#[derive(sqlx::FromRow, Debug)]
pub struct Product{
    pub(crate) user_id: u64,
    pub(crate) item_id: u64,
    pub(crate) rating: f32,
}