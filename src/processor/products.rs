
#[derive(sqlx::FromRow, Debug)]
pub struct Product{
    pub(crate) item_id: i32,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) category_id: Option<i32>
}