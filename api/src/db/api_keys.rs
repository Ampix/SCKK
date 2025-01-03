//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "api_keys")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "custom(\"LONGTEXT\")")]
    pub key: String,
    #[sea_orm(column_type = "Text")]
    pub perms: String,
    pub owner: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
