use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TypeHeader {
    pub tipus: String,
}

#[derive(Debug, Deserialize)]
pub struct ImgHeader {
    pub id: String,
}
