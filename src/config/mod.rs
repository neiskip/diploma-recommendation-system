use serde_derive::Deserialize;

#[derive(Deserialize, Default)]
pub struct Config{
    pub(crate) data_definition: String,
    pub(crate) database: Database,
    pub(crate) data_storage: Option<TestData>,
}
#[derive(Deserialize, Default)]
pub struct Database{
    pub(crate) driver: String,
    pub(crate) host: String,
    pub(crate) name: String,
    pub(crate) login: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) rating_view: Option<String>,
    pub(crate) product_data_view: Option<String>,
    pub(crate) query: Option<String>
}

#[derive(Deserialize, Default)]
pub struct TestData{
    pub(crate) path: Option<String>
}