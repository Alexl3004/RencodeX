use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "extract_sessions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub date: String,
    pub files: i32,
    pub tracks: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
