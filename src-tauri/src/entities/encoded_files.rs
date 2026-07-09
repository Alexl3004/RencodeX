use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "encoded_files")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub session_id: Option<i32>,
    pub name: String,
    pub original_mb: f64,
    pub encoded_mb: f64,
    pub ratio_pct: f64,
    pub duration_s: f64,
    pub encoded_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::encode_sessions::Entity",
        from = "Column::SessionId",
        to = "super::encode_sessions::Column::Id",
        on_delete = "Cascade"
    )]
    EncodeSession,
}

impl Related<super::encode_sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EncodeSession.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
