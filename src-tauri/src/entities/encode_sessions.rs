use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "encode_sessions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub date: String,
    pub files: i32,
    pub ratio_pct: f64,
    pub saved_mb: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::encoded_files::Entity")]
    EncodedFiles,
}

impl Related<super::encoded_files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EncodedFiles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
